use std::path::{Path, PathBuf};
use anyhow::Result;
use std::fs;
use sandbox;
use apk::{ApkInfo, ApkInspector, Abi};

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
        
        // 1. Copy APK to <prefix>/data/app/<pkg>/base.apk
        let app_dir = self.root.join("data/app").join(pkg_name);
        fs::create_dir_all(&app_dir)?;
        let target_apk = app_dir.join("base.apk");
        fs::copy(apk_path, &target_apk)?;
        println!("   📥 Copied APK to {}", target_apk.display());

        // 2. Extract libs
        let abi = info.supported_abis.iter()
            .find(|a| matches!(a, Abi::X86_64))
            .or_else(|| info.supported_abis.first())
            .cloned();

        if let Some(abi) = abi {
            let lib_dir = app_dir.join("lib").join(abi.as_str());
            fs::create_dir_all(&lib_dir)?;
            let inspector = ApkInspector::new(apk_path);
            inspector.extract_libs(&lib_dir, &abi)?;
            println!("   🏗️  Extracted libs for {} to {}", abi.as_str(), lib_dir.display());
        }

        // 3. Create data directory
        let data_dir = self.root.join("data/data").join(pkg_name);
        fs::create_dir_all(&data_dir)?;

        Ok(())
    }

    /// Orchestrates the execution of a command within the prefix sandbox
    pub fn run_in_sandbox(&self, payload_path: &Path, _command: &str) -> Result<()> {
        // We open the log file BEFORE entering namespaces to ensure we can write to it
        // Or we can do it after if the path is reachable in the sandbox
        let log_path = self.root.join("logs/app.log");
        let log_file = fs::File::create(&log_path)?;

        println!("Entering sandbox namespaces...");
        sandbox::enter_namespaces()?;

        println!("Redirecting output to {}...", log_path.display());
        sandbox::redirect_stdio(&log_file)?;

        println!("Setting up mounts...");
        self.mount_runtime(payload_path)?;

        println!("Sandbox ready. (Execution placeholder)");
        // Since we redirected stdout, the user won't see this in the terminal anymore
        // unless we use a supervisor or don't redirect yet.
        
        Ok(())
    }

    fn mount_runtime(&self, payload_path: &Path) -> Result<()> {
        let system_source = payload_path.join("system");
        let system_target = self.root.join("system");
        
        if system_source.exists() {
            sandbox::bind_mount(&system_source, &system_target)?;
        }

        Ok(())
    }
}
