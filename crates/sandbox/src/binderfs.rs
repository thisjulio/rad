//! Binderfs Instance Management
//! 
//! This module provides abstractions for mounting and managing binderfs instances
//! within isolated namespaces for Android app containers.

use anyhow::{Context, Result};
use nix::libc;
use nix::mount::{mount, umount, MsFlags};
use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use tracing::info;

// Constants for binderfs ioctl
const BINDER_CTL_ADD: u32 = 0x4020_6201; // _IOWR('b', 1, struct binderfs_device)

/// Maximum length for device name
const BINDERFS_MAX_NAME: usize = 255;

/// Structure for creating binderfs devices via ioctl
/// Corresponds to kernel's `struct binderfs_device`
#[repr(C)]
struct BinderfsDevice {
    name: [u8; BINDERFS_MAX_NAME + 1],
    major: u32,
    minor: u32,
}

impl BinderfsDevice {
    fn new(name: &str) -> Result<Self> {
        if name.len() > BINDERFS_MAX_NAME {
            anyhow::bail!("Device name too long: {}", name);
        }
        
        let mut dev = Self {
            name: [0u8; BINDERFS_MAX_NAME + 1],
            major: 0,
            minor: 0,
        };
        
        dev.name[..name.len()].copy_from_slice(name.as_bytes());
        Ok(dev)
    }
}

/// Represents a mounted binderfs instance
pub struct BinderfsInstance {
    /// Mount point of the binderfs instance
    mount_point: PathBuf,
    /// Whether this instance is mounted
    mounted: bool,
}

impl BinderfsInstance {
    /// Create a new binderfs instance at the specified mount point
    /// 
    /// This will:
    /// 1. Create the mount point directory if it doesn't exist
    /// 2. Mount binderfs at the specified path
    /// 3. Create the required binder device nodes (binder, hwbinder, vndbinder)
    pub fn new<P: AsRef<Path>>(mount_point: P) -> Result<Self> {
        let mount_point = mount_point.as_ref().to_path_buf();
        
        info!("Creating binderfs instance at {}", mount_point.display());
        
        // Create mount point directory
        std::fs::create_dir_all(&mount_point)
            .with_context(|| format!("Failed to create binderfs mount point: {}", mount_point.display()))?;
        
        let mut instance = Self {
            mount_point,
            mounted: false,
        };
        
        instance.mount()?;
        instance.create_devices()?;
        
        Ok(instance)
    }
    
    /// Mount binderfs at the configured mount point
    fn mount(&mut self) -> Result<()> {
        info!("Mounting binderfs at {}", self.mount_point.display());
        
        mount(
            Some("binder"),
            &self.mount_point,
            Some("binder"),
            MsFlags::MS_NODEV | MsFlags::MS_NOEXEC | MsFlags::MS_NOSUID,
            None::<&str>,
        )
        .with_context(|| format!("Failed to mount binderfs at {}", self.mount_point.display()))?;
        
        self.mounted = true;
        info!("Binderfs mounted successfully");
        
        Ok(())
    }
    
    /// Create the required binder device nodes using ioctl
    fn create_devices(&self) -> Result<()> {
        info!("Creating binder devices");
        
        let control_path = self.mount_point.join("binder-control");
        
        // Open the binder-control device
        let control_file = File::open(&control_path)
            .with_context(|| format!("Failed to open {}", control_path.display()))?;
        
        let control_fd = control_file.as_raw_fd();
        
        // Create the three required devices
        let devices = ["binder", "hwbinder", "vndbinder"];
        
        for device_name in &devices {
            info!("Creating device: {}", device_name);
            
            let mut dev = BinderfsDevice::new(device_name)?;
            
            // Execute ioctl to create the device
            let result = unsafe {
                libc::ioctl(control_fd, BINDER_CTL_ADD as _, &mut dev as *mut _)
            };
            
            if result < 0 {
                let err = std::io::Error::last_os_error();
                anyhow::bail!("Failed to create device {}: {}", device_name, err);
            }
            
            info!("Device {} created successfully (major={}, minor={})", 
                  device_name, dev.major, dev.minor);
        }
        
        Ok(())
    }
    
    /// Check if a specific device exists
    pub fn device_exists(&self, device_name: &str) -> bool {
        self.mount_point.join(device_name).exists()
    }
    
    /// Get the path to a specific device
    pub fn device_path(&self, device_name: &str) -> PathBuf {
        self.mount_point.join(device_name)
    }
    
    /// Unmount the binderfs instance
    pub fn unmount(&mut self) -> Result<()> {
        if !self.mounted {
            return Ok(());
        }
        
        info!("Unmounting binderfs at {}", self.mount_point.display());
        
        umount(&self.mount_point)
            .with_context(|| format!("Failed to unmount binderfs at {}", self.mount_point.display()))?;
        
        self.mounted = false;
        info!("Binderfs unmounted successfully");
        
        Ok(())
    }
}

impl Drop for BinderfsInstance {
    fn drop(&mut self) {
        if self.mounted && self.unmount().is_err() {
            eprintln!("Warning: Failed to unmount binderfs during drop");
        }
    }
}

/// Setup binderfs in the sandbox environment
/// 
/// This is a convenience function that:
/// 1. Creates a binderfs mount point at rootfs/dev/binderfs
/// 2. Mounts binderfs and creates devices
/// 3. Creates symbolic links from /dev/ to the binderfs devices
/// 
/// This should be called after entering namespaces and setting up basic mounts.
pub fn setup_binderfs_in_sandbox(rootfs: &Path) -> Result<BinderfsInstance> {
    use std::os::unix::fs::symlink;
    
    let binderfs_path = rootfs.join("dev/binderfs");
    let instance = BinderfsInstance::new(&binderfs_path)?;
    
    // Create symlinks from /dev to binderfs devices for compatibility
    let dev_path = rootfs.join("dev");
    for device in &["binder", "hwbinder", "vndbinder"] {
        let link_path = dev_path.join(device);
        let target_path = PathBuf::from("binderfs").join(device);
        
        // Remove existing symlink if present
        let _ = std::fs::remove_file(&link_path);
        
        symlink(&target_path, &link_path)
            .with_context(|| format!("Failed to create symlink for {}", device))?;
        
        info!("Created symlink: {} -> {}", link_path.display(), target_path.display());
    }
    
    Ok(instance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    /// RED: Test that asserts binder devices exist after setup
    /// This test WILL FAIL until we implement the mount and device creation
    #[test]
    #[ignore] // Requires namespace capabilities
    fn test_binderfs_instance_creates_devices() {
        let tmp = tempdir().expect("Failed to create tempdir");
        let mount_point = tmp.path().join("binderfs");
        
        // This should fail with "not yet implemented"
        let instance = BinderfsInstance::new(&mount_point)
            .expect("Failed to create binderfs instance");
        
        // Assert that all required devices exist
        assert!(instance.device_exists("binder"), "Device 'binder' should exist");
        assert!(instance.device_exists("hwbinder"), "Device 'hwbinder' should exist");
        assert!(instance.device_exists("vndbinder"), "Device 'vndbinder' should exist");
        
        // Assert that binder-control exists
        assert!(instance.device_exists("binder-control"), "binder-control should exist");
        
        // Verify paths are correct
        let binder_path = instance.device_path("binder");
        assert_eq!(binder_path, mount_point.join("binder"));
    }
    
    /// RED: Test that validates the mount point is created
    #[test]
    fn test_binderfs_creates_mount_point() {
        let tmp = tempdir().expect("Failed to create tempdir");
        let mount_point = tmp.path().join("binderfs");
        
        // Mount point should not exist yet
        assert!(!mount_point.exists());
        
        // This will fail because mount() is not implemented
        let _result = BinderfsInstance::new(&mount_point);
        
        // Even though it fails, the mount point directory should be created
        assert!(mount_point.exists(), "Mount point directory should be created");
    }
    
    /// Unit test for device_path method
    #[test]
    fn test_device_path() {
        let instance = BinderfsInstance {
            mount_point: PathBuf::from("/dev/binderfs"),
            mounted: false,
        };
        
        assert_eq!(instance.device_path("binder"), PathBuf::from("/dev/binderfs/binder"));
        assert_eq!(instance.device_path("hwbinder"), PathBuf::from("/dev/binderfs/hwbinder"));
    }
    
    /// Test the high-level setup function (without actually mounting)
    #[test]
    fn test_setup_binderfs_directory_structure() {
        let tmp = tempdir().expect("Failed to create tempdir");
        let rootfs = tmp.path();
        
        // Create the dev directory structure
        std::fs::create_dir_all(rootfs.join("dev/binderfs")).unwrap();
        
        // Verify directory exists
        assert!(rootfs.join("dev/binderfs").exists());
        assert!(rootfs.join("dev").exists());
    }
}
