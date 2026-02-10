use std::path::{Path, PathBuf};
use anyhow::Result;
use std::fs;
use sandbox;
use apk::{ApkInfo, ApkInspector, Abi};
use tracing::info;
use nix::unistd::{fork, ForkResult};
use nix::sys::wait::{waitpid, WaitStatus};

pub struct Prefix {
    pub root: PathBuf,
}

impl Prefix {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    pub fn initialize(&self) -> Result<()> {
        let dirs = [
            "system",
            "data",
            "data/app",
            "data/data",
            "dev",
            "proc",
            "sys",
            "tmp",
            "apex",
            "logs",
        ];

        for dir in dirs {
            let path = self.root.join(dir);
            if !path.exists() {
                fs::create_dir_all(&path)?;
            }
        }

        Ok(())
    }

    pub fn reset(&self) -> Result<()> {
        let data_dir = self.root.join("data");
        if data_dir.exists() {
            fs::remove_dir_all(&data_dir)?;
        }
        let logs_dir = self.root.join("logs");
        if logs_dir.exists() {
            fs::remove_dir_all(&logs_dir)?;
        }
        self.initialize()?;
        Ok(())
    }

    pub fn install_apk(&self, apk_path: &Path, info: &ApkInfo) -> Result<()> {
        let pkg_name = &info.package_name;
        
        let app_dir = self.root.join("data/app").join(pkg_name);
        fs::create_dir_all(&app_dir)?;
        let target_apk = app_dir.join("base.apk");
        fs::copy(apk_path, &target_apk)?;
        info!("Copied APK to {}", target_apk.display());

        let abi = info.supported_abis.iter()
            .find(|a| matches!(a, Abi::X86_64))
            .or_else(|| info.supported_abis.first())
            .cloned();

        if let Some(abi) = abi {
            let lib_dir = app_dir.join("lib").join(abi.as_str());
            fs::create_dir_all(&lib_dir)?;
            let inspector = ApkInspector::new(apk_path);
            inspector.extract_libs(&lib_dir, &abi)?;
            info!("Extracted libs for {} to {}", abi.as_str(), lib_dir.display());
        }

        let data_dir = self.root.join("data/data").join(pkg_name);
        fs::create_dir_all(&data_dir)?;

        Ok(())
    }

    pub fn enter_shell(&self, payload_path: &Path) -> Result<()> {
        let (command, args) = self.resolve_shell_command()?;
        self.run_in_sandbox(payload_path, &command, &args, false)
    }

    pub fn run_in_sandbox(&self, payload_path: &Path, command: &str, args: &[String], redirect: bool) -> Result<()> {
        use tracing::error;
        
        // We must fork before entering namespaces because unshare(CLONE_NEWUSER) 
        // fails in multi-threaded processes (and cargo creates threads)
        match unsafe { fork()? } {
            ForkResult::Parent { child } => {
                // Parent process: wait for child
                match waitpid(child, None)? {
                    WaitStatus::Exited(_, code) => {
                        if code != 0 {
                            return Err(anyhow::anyhow!("Child process exited with code {}", code));
                        }
                    }
                    WaitStatus::Signaled(_, signal, _) => {
                        return Err(anyhow::anyhow!("Child process killed by signal {:?}", signal));
                    }
                    _ => {}
                }
                Ok(())
            }
            ForkResult::Child => {
                // Child process: setup sandbox and exec
                let result = self.run_in_sandbox_child(payload_path, command, args, redirect);
                
                match result {
                    Ok(_) => {
                        // exec should never return
                        std::process::exit(1);
                    }
                    Err(e) => {
                        error!("Sandbox setup failed: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
    }
    
    fn run_in_sandbox_child(&self, payload_path: &Path, command: &str, args: &[String], redirect: bool) -> Result<()> {
        let log_path = self.root.join("logs/app.log");
        
        // Enter namespaces (safe in child process)
        sandbox::enter_namespaces()?;

        // Setup mounts inside the new mount namespace
        self.setup_sandbox_mounts(payload_path)?;

        if redirect {
            let log_file = fs::File::create(&log_path)?;
            sandbox::redirect_stdio(&log_file)?;
        }

        // Chroot into the prefix root
        sandbox::chroot(&self.root)?;

        // Exec the command (never returns if successful)
        let mut exec_args = Vec::with_capacity(args.len() + 1);
        exec_args.push(command.to_string());
        exec_args.extend_from_slice(args);
        sandbox::exec(command, &exec_args)?;
        
        Ok(())
    }

    fn setup_sandbox_mounts(&self, payload_path: &Path) -> Result<()> {
        // 1. Mount system from payload
        let system_source = payload_path.join("system");
        let system_target = self.root.join("system");
        if system_source.exists() {
            sandbox::bind_mount(&system_source, &system_target)?;
        }

        let host_bin = Path::new("/bin");
        let root_bin = self.root.join("bin");
        if host_bin.exists() {
            fs::create_dir_all(&root_bin)?;
            sandbox::bind_mount(host_bin, &root_bin)?;
        }

        // 2. Setup proc, sys, dev
        sandbox::setup_mounts(&self.root)?;

        Ok(())
    }

    fn resolve_shell_command(&self) -> Result<(String, Vec<String>)> {
        if self.root.join("system/bin/sh").exists() {
            return Ok(("/system/bin/sh".to_string(), Vec::new()));
        }

        if self.root.join("system/bin/busybox").exists() {
            return Ok(("/system/bin/busybox".to_string(), vec!["sh".to_string()]));
        }

        if Path::new("/bin/sh").exists() {
            return Ok(("/bin/sh".to_string(), Vec::new()));
        }

        if let Some(shell) = std::env::var_os("SHELL") {
            let shell = PathBuf::from(shell);
            if shell.exists() {
                return Ok((shell.to_string_lossy().into_owned(), Vec::new()));
            }
        }

        Err(anyhow::anyhow!(
            "no shell executable found in payload runtime or host"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::Prefix;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn shell_prefers_runtime_sh_when_available() {
        let root = make_temp_prefix_root("runtime-sh");
        fs::create_dir_all(root.join("system/bin")).unwrap();
        fs::write(root.join("system/bin/sh"), b"#!/bin/sh\n").unwrap();

        let prefix = Prefix::new(&root);
        let (command, args) = prefix.resolve_shell_command().unwrap();

        assert_eq!(command, "/system/bin/sh");
        assert!(args.is_empty());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn shell_uses_busybox_sh_when_system_shell_is_missing() {
        let root = make_temp_prefix_root("busybox-shell");
        fs::create_dir_all(root.join("system/bin")).unwrap();
        fs::write(root.join("system/bin/busybox"), b"#!/bin/sh\n").unwrap();

        let prefix = Prefix::new(&root);
        let (command, args) = prefix.resolve_shell_command().unwrap();

        assert_eq!(command, "/system/bin/busybox");
        assert_eq!(args, vec!["sh".to_string()]);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn shell_falls_back_to_host_bin_sh() {
        let root = make_temp_prefix_root("host-shell");
        let prefix = Prefix::new(&root);
        let (command, args) = prefix.resolve_shell_command().unwrap();

        assert_eq!(command, "/bin/sh");
        assert!(args.is_empty());

        let _ = fs::remove_dir_all(root);
    }

    fn make_temp_prefix_root(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("rad-prefix-{label}-{nanos}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
