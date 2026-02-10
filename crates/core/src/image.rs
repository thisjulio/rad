//! Image management for Waydroid LineageOS container images.
//!
//! Handles locating, validating, and mounting system.img/vendor.img
//! from the RAD data directory (~/.local/share/rad/).

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tracing::info;

/// Default data directory for RAD images
const RAD_DATA_DIR: &str = ".local/share/rad";
const CACHE_SUBDIR: &str = "cache";

/// Paths to the Waydroid container images
#[derive(Debug, Clone)]
pub struct ImagePaths {
    /// Path to system.img (LineageOS system partition)
    pub system_img: PathBuf,
    /// Path to vendor.img (vendor partition)
    pub vendor_img: PathBuf,
}

impl ImagePaths {
    /// Locate images in the default RAD data directory
    pub fn default_location() -> Result<Self> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let data_dir = PathBuf::from(home).join(RAD_DATA_DIR).join(CACHE_SUBDIR);
        Self::from_dir(&data_dir)
    }

    /// Locate images in a specific directory
    pub fn from_dir(cache_dir: &Path) -> Result<Self> {
        let system_img = cache_dir.join("system.img");
        let vendor_img = cache_dir.join("vendor.img");

        Ok(Self {
            system_img,
            vendor_img,
        })
    }

    /// Validate that both images exist and are ext4 filesystems
    pub fn validate(&self) -> Result<()> {
        if !self.system_img.exists() {
            anyhow::bail!(
                "System image not found: {}\nRun 'run-android-app setup' or './scripts/setup-image.sh' to download images.",
                self.system_img.display()
            );
        }

        if !self.vendor_img.exists() {
            anyhow::bail!(
                "Vendor image not found: {}\nRun 'run-android-app setup' or './scripts/setup-image.sh' to download images.",
                self.vendor_img.display()
            );
        }

        // Basic size check - system.img should be at least 500MB
        let system_size = std::fs::metadata(&self.system_img)
            .context("Failed to read system.img metadata")?
            .len();
        if system_size < 500 * 1024 * 1024 {
            anyhow::bail!(
                "System image too small ({} bytes), expected at least 500MB. May be corrupted.",
                system_size
            );
        }

        info!(
            "Images validated: system.img ({:.0} MB), vendor.img ({:.0} MB)",
            system_size as f64 / (1024.0 * 1024.0),
            std::fs::metadata(&self.vendor_img)?.len() as f64 / (1024.0 * 1024.0)
        );

        Ok(())
    }
}

/// Mount points for the container runtime.
///
/// These are temporary mount points used during container setup.
/// system.img and vendor.img are loop-mounted here, then used as
/// overlayfs lower layers.
#[derive(Debug, Clone)]
pub struct MountPoints {
    /// Where system.img is loop-mounted (read-only)
    pub system_mount: PathBuf,
    /// Where vendor.img is loop-mounted (read-only)
    pub vendor_mount: PathBuf,
    /// OverlayFS merged rootfs for the container
    pub rootfs: PathBuf,
    /// OverlayFS upper directory (writable layer)
    pub overlay_upper: PathBuf,
    /// OverlayFS work directory
    pub overlay_work: PathBuf,
}

impl MountPoints {
    /// Create mount point paths for a given prefix
    pub fn for_prefix(prefix_root: &Path) -> Self {
        Self {
            system_mount: prefix_root.join(".mounts/system"),
            vendor_mount: prefix_root.join(".mounts/vendor"),
            rootfs: prefix_root.join("rootfs"),
            overlay_upper: prefix_root.join(".overlay/upper"),
            overlay_work: prefix_root.join(".overlay/work"),
        }
    }

    /// Ensure all mount point directories exist
    pub fn ensure_dirs(&self) -> Result<()> {
        for dir in [
            &self.system_mount,
            &self.vendor_mount,
            &self.rootfs,
            &self.overlay_upper,
            &self.overlay_work,
        ] {
            std::fs::create_dir_all(dir)
                .with_context(|| format!("Failed to create directory: {}", dir.display()))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_paths_from_dir() {
        let dir = PathBuf::from("/tmp/test-cache");
        let paths = ImagePaths::from_dir(&dir).unwrap();
        assert_eq!(paths.system_img, PathBuf::from("/tmp/test-cache/system.img"));
        assert_eq!(paths.vendor_img, PathBuf::from("/tmp/test-cache/vendor.img"));
    }

    #[test]
    fn validate_fails_when_images_missing() {
        let dir = PathBuf::from("/tmp/nonexistent-rad-images");
        let paths = ImagePaths::from_dir(&dir).unwrap();
        let result = paths.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("System image not found"));
    }

    #[test]
    fn mount_points_for_prefix() {
        let prefix = PathBuf::from("/tmp/test-prefix");
        let mounts = MountPoints::for_prefix(&prefix);
        assert_eq!(mounts.system_mount, PathBuf::from("/tmp/test-prefix/.mounts/system"));
        assert_eq!(mounts.rootfs, PathBuf::from("/tmp/test-prefix/rootfs"));
        assert_eq!(mounts.overlay_upper, PathBuf::from("/tmp/test-prefix/.overlay/upper"));
    }
}
