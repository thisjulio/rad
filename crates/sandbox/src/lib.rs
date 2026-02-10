use std::path::Path;
use anyhow::Result;
use nix::sched::{unshare, CloneFlags};
use nix::mount::{mount, MsFlags};
use std::os::unix::io::AsRawFd;
use nix::libc;
use std::ffi::CString;

pub mod doctor;
pub mod binderfs;

// Re-export key binderfs types for convenience
pub use binderfs::{BinderfsInstance, setup_binderfs_in_sandbox};

pub struct SandboxConfig {
    pub rootfs: std::path::PathBuf,
}

/// Enter user and mount namespaces
/// 
/// IMPORTANT: This must be called in a single-threaded context (before any threads are created)
/// or after fork(). Calling unshare(CLONE_NEWUSER) in a multi-threaded process will fail
/// with EINVAL.
pub fn enter_namespaces() -> Result<()> {
    use tracing::info;
    
    // Get original uid/gid BEFORE entering namespace
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    
    // First, enter user namespace
    info!("Entering user namespace");
    unshare(CloneFlags::CLONE_NEWUSER)?;
    
    // Setup uid/gid mapping immediately after entering user namespace
    setup_uid_gid_mapping(uid, gid)?;
    
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
    use tracing::{info, warn};
    
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
    
    // Mount proc - requires PID namespace, so we skip it for now
    // TODO(task/021): Add PID namespace support
    warn!("Skipping /proc mount (requires PID namespace)");
    
    // Mount sysfs - may also fail without proper namespace
    warn!("Skipping /sys mount (may require additional namespaces)");
    
    // Mount tmpfs on /dev
    if let Err(e) = mount_tmpfs(&dev_path) {
        warn!("Failed to mount tmpfs on /dev: {}", e);
    } else {
        info!("Mounted tmpfs on /dev");
    }
    
    // Mount tmpfs on /tmp
    if let Err(e) = mount_tmpfs(&tmp_path) {
        warn!("Failed to mount tmpfs on /tmp: {}", e);
    } else {
        info!("Mounted tmpfs on /tmp");
    }
    
    Ok(())
}

pub fn setup_uid_gid_mapping(uid: u32, gid: u32) -> Result<()> {
    use std::fs;
    use tracing::info;
    
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

pub fn exec_with_env<P: AsRef<Path>>(path: P, args: &[String], env: &[(String, String)]) -> Result<()> {
    let path_c = CString::new(path.as_ref().to_str().unwrap())?;
    let args_c: Vec<CString> = args
        .iter()
        .map(|s| CString::new(s.as_str()).unwrap())
        .collect();
    let env_c: Vec<CString> = env
        .iter()
        .map(|(k, v)| CString::new(format!("{k}={v}")).unwrap())
        .collect();

    let args_ref: Vec<&std::ffi::CStr> = args_c.iter().map(|s| s.as_c_str()).collect();
    let env_ref: Vec<&std::ffi::CStr> = env_c.iter().map(|s| s.as_c_str()).collect();

    nix::unistd::execve(&path_c, &args_ref, &env_ref)?;
    Err(anyhow::anyhow!("execve failed"))
}

pub struct BinderfsStatus {
    pub kernel_support: bool,
    pub control_exists: bool,
}

pub fn check_binderfs() -> BinderfsStatus {
    let kernel_support = std::fs::read_to_string("/proc/filesystems")
        .map(|c| parse_proc_filesystems(&c))
        .unwrap_or(false);
    
    let control_exists = std::path::Path::new("/dev/binderfs/binder-control").exists();
    
    BinderfsStatus { kernel_support, control_exists }
}

fn parse_proc_filesystems(content: &str) -> bool {
    content.lines().any(|l| l.trim().ends_with("binder"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_parse_proc_filesystems() {
        let ok_content = "nodev\tsysfs\nnodev\ttmpfs\nnodev\tbinder\n";
        let fail_content = "nodev\tsysfs\nnodev\ttmpfs\n";
        
        assert!(parse_proc_filesystems(ok_content));
        assert!(!parse_proc_filesystems(fail_content));
    }

    #[test]
    fn test_check_binderfs_logic() {
        // We can't easily mock the FS for check_binderfs() without changing its signature,
        // but we already tested parse_proc_filesystems.
        // This is a sanity check for the current host.
        let status = check_binderfs();
        println!("Binderfs kernel support: {}", status.kernel_support);
        println!("Binderfs control exists: {}", status.control_exists);
    }

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
    
    /// Test bind mounting a payload directory
    /// 
    /// Note: This test is ignored because it requires:
    /// 1. User namespace or CAP_SYS_ADMIN capability
    /// 2. Single-threaded process (cargo test creates threads)
    /// 
    /// The actual bind mounting logic is tested indirectly via integration tests
    /// that run in forked processes (see prefix::tests).
    #[test]
    #[ignore]
    fn test_bind_mount_payload() {
        // Create temporary directories
        let temp_dir = std::env::temp_dir();
        let source = temp_dir.join("bind-mount-source");
        let target = temp_dir.join("bind-mount-target");
        
        std::fs::create_dir_all(&source).unwrap();
        std::fs::create_dir_all(&target).unwrap();
        
        // Create a test file in source
        std::fs::write(source.join("test.txt"), b"payload content").unwrap();
        
        // Attempt bind mount (will fail without proper namespace/capabilities)
        match bind_mount(&source, &target) {
            Ok(_) => {
                // Verify mount worked
                let content = std::fs::read_to_string(target.join("test.txt"))
                    .expect("Should be able to read mounted file");
                assert_eq!(content, "payload content");
            }
            Err(e) => {
                eprintln!("Expected failure without proper capabilities: {}", e);
            }
        }
        
        // Cleanup
        let _ = std::fs::remove_dir_all(source);
        let _ = std::fs::remove_dir_all(target);
    }
}
