use std::path::Path;
use anyhow::Result;
use nix::sched::{unshare, CloneFlags};
use nix::mount::{mount, MsFlags};

pub struct SandboxConfig {
    pub rootfs: std::path::PathBuf,
}

pub fn enter_namespaces() -> Result<()> {
    // Attempt to unshare User and Mount namespaces first, as they are most critical for mounting
    let flags = CloneFlags::CLONE_NEWUSER | CloneFlags::CLONE_NEWNS;
    
    // We can also try NEWPID, but it might require more setup (forking)
    // flags |= CloneFlags::CLONE_NEWPID;

    unshare(flags)?;
    
    // Note: In a real "rootless" setup, we must map UID/GID here.
    // Without mapping, we are "nobody" in the new namespace, which might limit mount capabilities.
    
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
