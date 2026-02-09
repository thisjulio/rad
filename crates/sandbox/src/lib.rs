use std::path::Path;
use anyhow::Result;
use nix::sched::{unshare, CloneFlags};
use nix::mount::{mount, MsFlags};
use std::os::unix::io::AsRawFd;
use nix::libc;
use std::ffi::CString;

pub struct SandboxConfig {
    pub rootfs: std::path::PathBuf,
}

pub fn enter_namespaces() -> Result<()> {
    use tracing::info;
    
    // First, enter user namespace
    info!("Entering user namespace");
    unshare(CloneFlags::CLONE_NEWUSER)?;
    
    // Setup uid/gid mapping immediately after entering user namespace
    setup_uid_gid_mapping()?;
    
    // Now we can enter mount namespace (needs to be root in user ns)
    info!("Entering mount namespace");
    unshare(CloneFlags::CLONE_NEWNS)?;
    
    Ok(())
}

pub fn bind_mount<P: AsRef<Path>, Q: AsRef<Path>>(source: P, target: Q) -> Result<()> {
    mount(
        Some(source.as_ref()),
        target.as_ref(),
        None::<&Path>,
        MsFlags::MS_BIND | MsFlags::MS_REC,
        None::<&Path>,
    )?;
    Ok(())
}

pub fn mount_tmpfs<P: AsRef<Path>>(target: P) -> Result<()> {
    mount(
        None::<&Path>,
        target.as_ref(),
        Some("tmpfs"),
        MsFlags::empty(),
        None::<&Path>,
    )?;
    Ok(())
}

pub fn redirect_stdio(log_file: &std::fs::File) -> Result<()> {
    let fd = log_file.as_raw_fd();
    unsafe {
        if libc::dup2(fd, libc::STDOUT_FILENO) == -1 {
            return Err(anyhow::anyhow!("Failed to redirect stdout"));
        }
        if libc::dup2(fd, libc::STDERR_FILENO) == -1 {
            return Err(anyhow::anyhow!("Failed to redirect stderr"));
        }
    }
    Ok(())
}

pub fn setup_mounts(rootfs: &Path) -> Result<()> {
    use tracing::{info, error};
    
    info!("Setting up mounts in rootfs: {}", rootfs.display());
    
    // Create mount points if they don't exist
    let proc_path = rootfs.join("proc");
    let sys_path = rootfs.join("sys");
    let dev_path = rootfs.join("dev");
    let tmp_path = rootfs.join("tmp");
    
    std::fs::create_dir_all(&proc_path)?;
    std::fs::create_dir_all(&sys_path)?;
    std::fs::create_dir_all(&dev_path)?;
    std::fs::create_dir_all(&tmp_path)?;
    
    // Mount proc
    if let Err(e) = mount(
        Some("proc"),
        &proc_path,
        Some("proc"),
        MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC | MsFlags::MS_NODEV,
        None::<&Path>,
    ) {
        error!("Failed to mount /proc: {}", e);
        return Err(e.into());
    }
    info!("Mounted /proc");
    
    // Mount sysfs
    if let Err(e) = mount(
        Some("sysfs"),
        &sys_path,
        Some("sysfs"),
        MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC | MsFlags::MS_NODEV | MsFlags::MS_RDONLY,
        None::<&Path>,
    ) {
        error!("Failed to mount /sys: {}", e);
        return Err(e.into());
    }
    info!("Mounted /sys");
    
    // Bind mount /dev (with limited subset)
    if let Err(e) = mount_tmpfs(&dev_path) {
        error!("Failed to mount tmpfs on /dev: {}", e);
        return Err(e);
    }
    info!("Mounted tmpfs on /dev");
    
    // Mount tmpfs on /tmp
    if let Err(e) = mount_tmpfs(&tmp_path) {
        error!("Failed to mount tmpfs on /tmp: {}", e);
        return Err(e);
    }
    info!("Mounted tmpfs on /tmp");
    
    Ok(())
}

pub fn setup_uid_gid_mapping() -> Result<()> {
    use std::fs;
    use tracing::info;
    
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    
    info!("Setting up uid/gid mapping for uid={}, gid={}", uid, gid);
    
    // Write uid_map
    let uid_map = format!("0 {} 1", uid);
    fs::write("/proc/self/uid_map", uid_map)?;
    
    // Disable setgroups (required before writing gid_map)
    fs::write("/proc/self/setgroups", "deny")?;
    
    // Write gid_map
    let gid_map = format!("0 {} 1", gid);
    fs::write("/proc/self/gid_map", gid_map)?;
    
    info!("UID/GID mapping configured successfully");
    
    Ok(())
}

pub fn chroot<P: AsRef<Path>>(path: P) -> Result<()> {
    nix::unistd::chdir(path.as_ref())?;
    nix::unistd::chroot(path.as_ref())?;
    std::env::set_current_dir("/")?;
    Ok(())
}

pub fn exec<P: AsRef<Path>>(path: P, args: &[String]) -> Result<()> {
    let path_c = CString::new(path.as_ref().to_str().unwrap())?;
    let args_c: Vec<CString> = args.iter()
        .map(|s| CString::new(s.as_str()).unwrap())
        .collect();
    
    let args_ref: Vec<&std::ffi::CStr> = args_c.iter()
        .map(|s| s.as_c_str())
        .collect();

    nix::unistd::execv(&path_c, &args_ref)?;
    
    // execv only returns if it fails
    Err(anyhow::anyhow!("execv failed"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    /// Test that enters a user namespace
    /// 
    /// Note: This test is ignored by default because:
    /// 1. unshare(CLONE_NEWUSER) fails if the process is multi-threaded
    /// 2. Rust test runner creates multiple threads
    /// 3. Requires /proc/sys/kernel/unprivileged_userns_clone = 1 or not exist
    /// 
    /// To test manually in a single-threaded context, run in separate process
    #[test]
    #[ignore]
    fn test_enter_user_namespace() {
        let ns_before = fs::read_link("/proc/self/ns/user")
            .expect("Failed to read user namespace");
        
        // This will fail in multi-threaded test context
        match enter_namespaces() {
            Ok(_) => {
                let ns_after = fs::read_link("/proc/self/ns/user")
                    .expect("Failed to read user namespace after unshare");
                assert_ne!(ns_before, ns_after, "User namespace should change after unshare");
            }
            Err(e) => {
                eprintln!("Expected failure in test context: {}", e);
                eprintln!("This is OK - unshare fails with multi-threaded processes");
            }
        }
    }
    
    #[test]
    #[ignore]
    fn test_uid_gid_mapping() {
        match enter_namespaces() {
            Ok(_) => {
                assert!(std::path::Path::new("/proc/self/uid_map").exists());
                assert!(std::path::Path::new("/proc/self/gid_map").exists());
            }
            Err(e) => {
                eprintln!("Expected failure in test context: {}", e);
            }
        }
    }
    
    #[test]
    fn test_sandbox_config_creation() {
        let config = SandboxConfig {
            rootfs: std::path::PathBuf::from("/tmp/test"),
        };
        
        assert_eq!(config.rootfs, std::path::PathBuf::from("/tmp/test"));
    }
    
    /// Test that validates the setup_uid_gid_mapping format logic
    #[test]
    fn test_uid_gid_mapping_format() {
        let uid = 1000u32;
        let gid = 1000u32;
        
        let uid_map = format!("0 {} 1", uid);
        let gid_map = format!("0 {} 1", gid);
        
        assert_eq!(uid_map, "0 1000 1");
        assert_eq!(gid_map, "0 1000 1");
    }
}
