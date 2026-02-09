use std::path::{Path, PathBuf};
use anyhow::Result;
use std::fs;
use sandbox;

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
        ];

        for dir in dirs {
            let path = self.root.join(dir);
            if !path.exists() {
                fs::create_dir_all(&path)?;
            }
        }

        Ok(())
    }

    /// Orchestrates the execution of a command within the prefix sandbox
    pub fn run_in_sandbox(&self, payload_path: &Path, _command: &str) -> Result<()> {
        println!("Entering sandbox namespaces...");
        sandbox::enter_namespaces()?;

        println!("Setting up mounts...");
        self.mount_runtime(payload_path)?;

        println!("Sandbox ready. (Execution placeholder)");
        // In a real implementation, we would pivot_root and exec the command
        
        Ok(())
    }

    fn mount_runtime(&self, payload_path: &Path) -> Result<()> {
        // Bind mount the system partition from payload to prefix/system
        let system_source = payload_path.join("system");
        let system_target = self.root.join("system");
        
        if system_source.exists() {
            // Note: This will fail if not run with proper privileges or inside user ns
            sandbox::bind_mount(&system_source, &system_target)?;
        }

        Ok(())
    }
}
