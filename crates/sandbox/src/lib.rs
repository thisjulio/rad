use std::path::Path;
use anyhow::Result;
use nix::sched::{unshare, CloneFlags};
use nix::mount::{mount, MsFlags};
use std::os::unix::io::AsRawFd;
use nix::libc;

pub struct SandboxConfig {
    pub rootfs: std::path::PathBuf,
}

pub fn enter_namespaces() -> Result<()> {
    // Attempt to unshare User and Mount namespaces first
    let flags = CloneFlags::CLONE_NEWUSER | CloneFlags::CLONE_NEWNS;
    unshare(flags)?;
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
