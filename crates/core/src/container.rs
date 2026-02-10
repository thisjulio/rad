//! Rootless Android container lifecycle management.
//!
//! This module manages the full lifecycle of an Android container WITHOUT sudo:
//! - Mount system.img and vendor.img via fuse2fs (userspace FUSE)
//! - Fork child process
//! - Enter user + mount + PID + IPC + UTS namespaces (via nix crate)
//! - Set up overlayfs rootfs inside the namespace
//! - Boot Android init as PID 1 inside the container
//! - Execute commands inside running container (nsenter via setns)
//! - Stop the container and clean up FUSE mounts

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use nix::sys::signal::Signal;
use nix::sys::wait::{waitpid, WaitPidFlag};
use nix::unistd::Pid;
use tracing::{error, info, warn};

use crate::image::{ImagePaths, MountPoints};

/// State of a running container (fully rootless)
#[derive(Debug)]
pub struct Container {
    /// PID of the container init process (PID 1 inside container)
    pub init_pid: Option<u32>,
    /// Mount points used by this container
    pub mounts: MountPoints,
    /// Image paths
    pub images: ImagePaths,
    /// Whether system.img is currently FUSE-mounted
    system_mounted: bool,
    /// Whether vendor.img is currently FUSE-mounted
    vendor_mounted: bool,
    /// Whether overlayfs is mounted (inside the namespace)
    overlay_mounted: bool,
    /// PID file path for persisting container PID
    pid_file: Option<PathBuf>,
}

impl Container {
    /// Create a new container (not yet started)
    pub fn new(images: ImagePaths, mounts: MountPoints) -> Self {
        Self {
            init_pid: None,
            mounts,
            images,
            system_mounted: false,
            vendor_mounted: false,
            overlay_mounted: false,
            pid_file: None,
        }
    }

    /// Set a PID file path for persisting the init PID
    pub fn with_pid_file(mut self, path: PathBuf) -> Self {
        self.pid_file = Some(path);
        self
    }

    /// Start the container: FUSE-mount images, fork, enter namespaces, boot init
    ///
    /// This is fully rootless - no sudo required.
    /// Requires: fuse2fs, user namespaces enabled, overlayfs support.
    pub fn start(&mut self) -> Result<()> {
        info!("Starting rootless Android container...");

        // Validate images exist
        self.images.validate()?;

        // Create mount point directories
        self.mounts.ensure_dirs()?;

        // Prepare overlay upper/work directories
        self.prepare_prefix_dirs()?;

        // Step 1: FUSE-mount system.img and vendor.img (userspace, no root)
        self.fuse_mount_images()?;

        // Step 2: Pre-create APEX dirs in overlay upper layer.
        // The system image's /apex/ dir is owned by root:root with 0755 perms,
        // which maps to nobody inside the user namespace via fuse2fs. This means
        // mkdir inside the overlayfs merged /apex/ would fail with Permission denied.
        // By pre-creating these dirs in the upper layer (which we own), overlayfs
        // will show our writable dirs instead of the unwritable lower ones.
        self.prepare_apex_dirs()?;

        // Step 3: Generate linker config to suppress Android linker warnings
        self.generate_linkerconfig()?;

        // Step 4: Fork + enter namespaces + overlayfs + chroot + exec init
        self.launch_init()?;

        info!(
            "Container started successfully (init PID: {:?})",
            self.init_pid
        );

        // Persist PID
        if let (Some(pid), Some(pid_file)) = (self.init_pid, &self.pid_file)
            && let Err(e) = std::fs::write(pid_file, pid.to_string())
        {
            warn!("Failed to write PID file: {}", e);
        }

        Ok(())
    }

    /// Stop the container: kill init, unmount FUSE
    pub fn stop(&mut self) -> Result<()> {
        info!("Stopping Android container...");

        // Kill init process
        if let Some(pid) = self.init_pid.take() {
            info!("Killing init process (PID {})", pid);
            let pid = Pid::from_raw(pid as i32);
            // Send SIGTERM first
            if nix::sys::signal::kill(pid, Signal::SIGTERM).is_ok() {
                // Wait a bit for graceful shutdown
                std::thread::sleep(std::time::Duration::from_secs(2));
                // Force kill if still alive
                let _ = nix::sys::signal::kill(pid, Signal::SIGKILL);
                let _ = waitpid(pid, Some(WaitPidFlag::WNOHANG));
            }
        }

        // Unmount FUSE mounts (no sudo needed - fusermount -u)
        self.fuse_unmount_all()?;

        // Clean up PID file
        if let Some(pid_file) = &self.pid_file {
            let _ = std::fs::remove_file(pid_file);
        }

        info!("Container stopped");
        Ok(())
    }

    /// Execute a command inside the running container using nsenter
    ///
    /// Note: nsenter into your own user namespace doesn't require root.
    /// We use the external `nsenter` binary for simplicity.
    pub fn exec_command(&self, command: &str, args: &[&str]) -> Result<std::process::Output> {
        let init_pid = self
            .init_pid
            .context("Container is not running (no init PID)")?;

        info!("Executing in container: {} {:?}", command, args);

        let output = Command::new("nsenter")
            .arg("-t")
            .arg(init_pid.to_string())
            .arg("--user")
            .arg("--mount")
            .arg("--uts")
            .arg("--ipc")
            .arg("--pid")
            .arg("--")
            .arg(command)
            .args(args)
            .output()
            .context("Failed to execute nsenter")?;

        Ok(output)
    }

    /// Install an APK into the running container
    pub fn install_apk(&self, apk_path: &Path) -> Result<()> {
        let _init_pid = self.init_pid.context("Container is not running")?;

        info!("Installing APK: {}", apk_path.display());

        // Copy APK into the container's /data directory via the overlay upper layer
        let container_apk_dir = self.mounts.overlay_upper.join("data/local/tmp");
        std::fs::create_dir_all(&container_apk_dir)?;
        let container_apk = container_apk_dir.join("install.apk");
        std::fs::copy(apk_path, &container_apk)
            .context("Failed to copy APK into container overlay")?;

        // Use pm install inside the container
        let output =
            self.exec_command("pm", &["install", "-r", "/data/local/tmp/install.apk"])?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            warn!("pm install output: {}{}", stdout, stderr);
            anyhow::bail!("pm install failed: {}{}", stdout, stderr);
        }

        info!("APK installed successfully");
        Ok(())
    }

    /// Launch an Android app by package name
    pub fn launch_app(&self, package: &str, activity: &str) -> Result<()> {
        info!("Launching {}/{}", package, activity);

        let component = format!("{}/{}", package, activity);
        let output = self.exec_command("am", &["start", "-n", &component])?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            anyhow::bail!("am start failed: {}{}", stdout, stderr);
        }

        info!("App launched");
        Ok(())
    }

    /// Check if the container init process is still running
    pub fn is_running(&self) -> bool {
        if let Some(pid) = self.init_pid {
            let pid = Pid::from_raw(pid as i32);
            nix::sys::signal::kill(pid, None).is_ok()
        } else {
            false
        }
    }

    /// Wait for the Android system to boot (poll for sys.boot_completed)
    pub fn wait_for_boot(&self, timeout_secs: u64) -> Result<()> {
        info!(
            "Waiting for Android system to boot (timeout: {}s)...",
            timeout_secs
        );

        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);

        loop {
            if start.elapsed() > timeout {
                anyhow::bail!(
                    "Timeout waiting for Android system to boot after {}s",
                    timeout_secs
                );
            }

            if !self.is_running() {
                anyhow::bail!("Container init process died during boot");
            }

            // Check if system has booted
            match self.exec_command("getprop", &["sys.boot_completed"]) {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if stdout.trim() == "1" {
                        info!("Android system boot completed!");
                        return Ok(());
                    }
                }
                Err(_) => {
                    // getprop may not be available yet, keep waiting
                }
            }

            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }

    // --- Private methods ---

    fn prepare_prefix_dirs(&self) -> Result<()> {
        let data_dir = self.mounts.overlay_upper.join("data");
        std::fs::create_dir_all(&data_dir)?;
        std::fs::create_dir_all(data_dir.join("app"))?;
        std::fs::create_dir_all(data_dir.join("data"))?;
        std::fs::create_dir_all(data_dir.join("local/tmp"))?;
        Ok(())
    }

    /// Pre-create APEX module directories in the overlay upper layer.
    ///
    /// Android binaries (sh, linker64, app_process64, etc.) are symlinked to
    /// /apex/<module>/... but the /apex/ directory in the system image is empty.
    /// The actual APEX module contents live at /system/apex/<module>/.
    ///
    /// Inside the user namespace, the fuse2fs-mounted /apex/ dir is owned by
    /// nobody (unmapped root), making mkdir fail. We pre-create the dirs in the
    /// overlay upper layer (which we own) so they appear writable in the merged view.
    fn prepare_apex_dirs(&self) -> Result<()> {
        let system_apex_dir = self.mounts.system_mount.join("system/apex");
        if !system_apex_dir.exists() {
            info!("No /system/apex directory found in system image, skipping APEX prep");
            return Ok(());
        }

        let upper_apex_dir = self.mounts.overlay_upper.join("apex");
        std::fs::create_dir_all(&upper_apex_dir)?;

        let mut count = 0;
        for entry in std::fs::read_dir(&system_apex_dir)
            .context("Failed to read /system/apex directory")?
        {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let name = entry.file_name();
                let target = upper_apex_dir.join(&name);
                std::fs::create_dir_all(&target)?;
                count += 1;
            }
        }

        info!(
            "Pre-created {} APEX directories in overlay upper layer",
            count
        );
        Ok(())
    }

    /// Generate a minimal linker configuration to suppress Android linker warnings.
    ///
    /// The Android dynamic linker (linker64) looks for /linkerconfig/ld.config.txt
    /// on boot. Without it, every binary invocation prints a warning.
    fn generate_linkerconfig(&self) -> Result<()> {
        let linkerconfig_dir = self.mounts.overlay_upper.join("linkerconfig");
        std::fs::create_dir_all(&linkerconfig_dir)?;

        let config = "\
dir.system = /system/bin/
dir.system = /system/xbin/
dir.vendor = /vendor/bin/

[system]
namespace.default.search.paths = /system/lib64:/system/lib:/apex/com.android.art/lib64:/apex/com.android.runtime/lib64
namespace.default.permitted.paths = /system:/apex:/vendor:/data

[vendor]
namespace.default.search.paths = /vendor/lib64:/vendor/lib:/system/lib64:/system/lib
namespace.default.permitted.paths = /system:/apex:/vendor:/data
";

        std::fs::write(linkerconfig_dir.join("ld.config.txt"), config)
            .context("Failed to write linker config")?;

        info!("Generated /linkerconfig/ld.config.txt");
        Ok(())
    }

    /// Mount images using fuse2fs (userspace, no root needed)
    fn fuse_mount_images(&mut self) -> Result<()> {
        // Mount system.img via fuse2fs
        info!(
            "FUSE-mounting system.img at {}...",
            self.mounts.system_mount.display()
        );
        fuse2fs_mount(&self.images.system_img, &self.mounts.system_mount, true)?;
        self.system_mounted = true;
        info!("system.img FUSE-mounted (read-only)");

        // Mount vendor.img via fuse2fs
        info!(
            "FUSE-mounting vendor.img at {}...",
            self.mounts.vendor_mount.display()
        );
        match fuse2fs_mount(&self.images.vendor_img, &self.mounts.vendor_mount, true) {
            Ok(()) => {
                self.vendor_mounted = true;
                info!("vendor.img FUSE-mounted (read-only)");
            }
            Err(e) => {
                // Clean up system mount on failure
                let _ = fusermount_unmount(&self.mounts.system_mount);
                self.system_mounted = false;
                return Err(e).context("Failed to FUSE-mount vendor.img");
            }
        }

        Ok(())
    }

    /// Launch init inside namespaces using unshare (no sudo)
    fn launch_init(&mut self) -> Result<()> {
        info!("Launching Android init inside rootless namespaces...");

        // Check for init binary in the FUSE-mounted system
        let init_path = if self.mounts.system_mount.join("init").exists() {
            "/init"
        } else if self.mounts.system_mount.join("system/bin/init").exists() {
            "/system/bin/init"
        } else if self.mounts.system_mount.join("bin/init").exists() {
            "/bin/init"
        } else {
            anyhow::bail!(
                "No init binary found in system image. Checked:\n  \
                 {}/init\n  {}/system/bin/init\n  {}/bin/init",
                self.mounts.system_mount.display(),
                self.mounts.system_mount.display(),
                self.mounts.system_mount.display()
            );
        };

        info!("Using init: {}", init_path);

        // Use unprivileged unshare to create namespaces and run init
        // unshare --user --map-root-user --pid --fork --mount-proc --uts --ipc --mount
        //   -- sh -c "set up overlayfs + chroot + exec init"
        //
        // We use a shell wrapper inside the namespace to:
        // 1. Mount overlayfs (system as lower, overlay upper/work)
        // 2. Bind-mount vendor into rootfs/vendor
        // 3. Create /data directories
        // 4. Mount /proc, /dev, /tmp
        // 5. chroot into rootfs
        // 6. exec init

        let rootfs = &self.mounts.rootfs;
        let system_mount = &self.mounts.system_mount;
        let vendor_mount = &self.mounts.vendor_mount;
        let overlay_upper = &self.mounts.overlay_upper;
        let overlay_work = &self.mounts.overlay_work;

        // Build the shell script that runs inside the namespace
        let setup_script = format!(
            r#"
set -e

# Mount overlayfs: system as lower, prefix overlay as upper
# APEX dirs are pre-created in the upper layer (see prepare_apex_dirs)
mount -t overlay overlay \
    -o lowerdir={system},upperdir={upper},workdir={work} \
    {rootfs}

# Bind-mount vendor into rootfs
mkdir -p {rootfs}/vendor
mount --bind {vendor} {rootfs}/vendor

# APEX bind mounts: Android binaries (sh, linker64, etc.) are symlinked
# to /apex/<module>/... but the /apex/ directory in the image is empty.
# The actual APEX module contents live at /system/apex/<module>/.
# The target dirs in /apex/ were pre-created in the overlay upper layer.
if [ -d "{rootfs}/system/apex" ]; then
    for apex_dir in {rootfs}/system/apex/*/; do
        apex_name=$(basename "$apex_dir")
        if [ -d "$apex_dir" ] && [ -d "{rootfs}/apex/$apex_name" ]; then
            mount --bind "$apex_dir" "{rootfs}/apex/$apex_name"
        fi
    done
fi

# Create essential directories in rootfs
mkdir -p {rootfs}/data/app {rootfs}/data/data {rootfs}/data/local/tmp \
         {rootfs}/data/system {rootfs}/data/misc {rootfs}/data/dalvik-cache \
         {rootfs}/proc {rootfs}/sys {rootfs}/dev {rootfs}/tmp

# Mount proc/dev/tmp inside rootfs
mount -t proc proc {rootfs}/proc || true
mount -t tmpfs tmpfs {rootfs}/dev || true
mount -t tmpfs tmpfs {rootfs}/tmp || true

# Create basic /dev nodes (mknod works as "root" in user ns)
mknod -m 666 {rootfs}/dev/null c 1 3 2>/dev/null || true
mknod -m 666 {rootfs}/dev/zero c 1 5 2>/dev/null || true
mknod -m 666 {rootfs}/dev/random c 1 8 2>/dev/null || true
mknod -m 666 {rootfs}/dev/urandom c 1 9 2>/dev/null || true

# Pivot root and exec init
cd {rootfs}
exec chroot {rootfs} {init} \
    </dev/null >/dev/null 2>&1
"#,
            system = system_mount.display(),
            vendor = vendor_mount.display(),
            upper = overlay_upper.display(),
            work = overlay_work.display(),
            rootfs = rootfs.display(),
            init = init_path,
        );

        let child = Command::new("unshare")
            .arg("--user")
            .arg("--map-root-user")
            .arg("--pid")
            .arg("--fork")
            .arg("--mount-proc")
            .arg("--uts")
            .arg("--ipc")
            .arg("--mount")
            .arg("--")
            .arg("sh")
            .arg("-c")
            .arg(&setup_script)
            .env("ANDROID_ROOT", "/system")
            .env("ANDROID_DATA", "/data")
            .env("PATH", "/system/bin:/system/xbin:/vendor/bin:/bin:/usr/bin")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .context("Failed to spawn container via unshare (is unshare available?)")?;

        let pid = child.id();
        self.init_pid = Some(pid);
        self.overlay_mounted = true;
        info!("Init process spawned (outer PID: {})", pid);

        // Give init a moment to start
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Check it's still alive
        if !self.is_running() {
            let output = child.wait_with_output()?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!(
                "Init process died immediately.\nstdout: {}\nstderr: {}",
                stdout,
                stderr
            );
        }

        Ok(())
    }

    /// Unmount all FUSE mounts (no sudo needed)
    fn fuse_unmount_all(&mut self) -> Result<()> {
        let mut errors = Vec::new();

        // Note: overlayfs and bind mounts inside the namespace are cleaned up
        // automatically when the namespace (init process) dies.

        // Unmount vendor.img FUSE
        if self.vendor_mounted {
            if let Err(e) = fusermount_unmount(&self.mounts.vendor_mount) {
                errors.push(format!("vendor.img FUSE: {}", e));
            } else {
                self.vendor_mounted = false;
            }
        }

        // Unmount system.img FUSE
        if self.system_mounted {
            if let Err(e) = fusermount_unmount(&self.mounts.system_mount) {
                errors.push(format!("system.img FUSE: {}", e));
            } else {
                self.system_mounted = false;
            }
        }

        self.overlay_mounted = false;

        if !errors.is_empty() {
            warn!("Some unmount operations failed: {:?}", errors);
        }

        Ok(())
    }
}

impl Drop for Container {
    fn drop(&mut self) {
        if (self.init_pid.is_some() || self.system_mounted || self.vendor_mounted)
            && let Err(e) = self.stop()
        {
            error!("Failed to stop container during drop: {}", e);
        }
    }
}

// --- Helper functions ---

/// Mount an ext4 image using fuse2fs (no root required)
fn fuse2fs_mount(image: &Path, mount_point: &Path, read_only: bool) -> Result<()> {
    let mut cmd = Command::new("fuse2fs");
    cmd.arg(image);
    cmd.arg(mount_point);

    if read_only {
        cmd.arg("-o").arg("ro,fakeroot");
    } else {
        cmd.arg("-o").arg("fakeroot");
    }

    let output = cmd
        .output()
        .context("Failed to execute fuse2fs. Is it installed? (pacman -S fuse2fs)")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "fuse2fs failed for {}: {}",
            image.display(),
            stderr.trim()
        );
    }

    Ok(())
}

/// Unmount a FUSE mount using fusermount (no root required)
fn fusermount_unmount(mount_point: &Path) -> Result<()> {
    // Try fusermount3 first (newer), fall back to fusermount
    let result = Command::new("fusermount3")
        .arg("-u")
        .arg(mount_point)
        .output();

    let output = match result {
        Ok(o) if o.status.success() => return Ok(()),
        _ => {
            // Fall back to fusermount
            Command::new("fusermount")
                .arg("-u")
                .arg(mount_point)
                .output()
                .with_context(|| {
                    format!(
                        "Failed to execute fusermount -u {}",
                        mount_point.display()
                    )
                })?
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Try lazy unmount as last resort
        let lazy_result = Command::new("fusermount")
            .arg("-uz")
            .arg(mount_point)
            .output();

        if let Ok(lazy_out) = lazy_result
            && lazy_out.status.success()
        {
            warn!(
                "Used lazy unmount for {} (busy mount)",
                mount_point.display()
            );
            return Ok(());
        }

        anyhow::bail!(
            "fusermount -u failed for {}: {}",
            mount_point.display(),
            stderr.trim()
        );
    }

    Ok(())
}

/// Check if fuse2fs is available on the system
pub fn check_fuse2fs() -> bool {
    Command::new("fuse2fs")
        .arg("--help")
        .output()
        .map(|o| o.status.success() || !o.stderr.is_empty())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::{ImagePaths, MountPoints};

    #[test]
    fn container_new_not_running() {
        let images = ImagePaths {
            system_img: PathBuf::from("/tmp/nonexistent/system.img"),
            vendor_img: PathBuf::from("/tmp/nonexistent/vendor.img"),
        };
        let mounts = MountPoints::for_prefix(Path::new("/tmp/nonexistent-prefix"));
        let container = Container::new(images, mounts);

        assert!(!container.is_running());
        assert!(container.init_pid.is_none());
    }

    #[test]
    fn container_start_fails_without_images() {
        let images = ImagePaths {
            system_img: PathBuf::from("/tmp/nonexistent/system.img"),
            vendor_img: PathBuf::from("/tmp/nonexistent/vendor.img"),
        };
        let mounts = MountPoints::for_prefix(Path::new("/tmp/nonexistent-prefix"));
        let mut container = Container::new(images, mounts);

        let result = container.start();
        assert!(result.is_err());
    }

    #[test]
    fn check_fuse2fs_returns_bool() {
        // Just verify it doesn't panic
        let _ = check_fuse2fs();
    }
}
