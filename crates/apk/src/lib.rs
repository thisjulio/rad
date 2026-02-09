use std::fs::{self, File};
use std::path::Path;
use zip::ZipArchive;
use anyhow::Result;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Abi {
    Arm64V8a,
    ArmV7a,
    X86_64,
    X86,
}

impl Abi {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "arm64-v8a" => Some(Abi::Arm64V8a),
            "armeabi-v7a" => Some(Abi::ArmV7a),
            "x86_64" => Some(Abi::X86_64),
            "x86" => Some(Abi::X86),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Abi::Arm64V8a => "arm64-v8a",
            Abi::ArmV7a => "armeabi-v7a",
            Abi::X86_64 => "x86_64",
            Abi::X86 => "x86",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApkInfo {
    pub package_name: String,
    pub supported_abis: Vec<Abi>,
}

pub struct ApkInspector {
    path: std::path::PathBuf,
}

impl ApkInspector {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn inspect(&self) -> Result<ApkInfo> {
        let file = File::open(&self.path)?;
        let mut archive = ZipArchive::new(file)?;

        let mut abis = std::collections::HashSet::new();
        
        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let name = file.name();
            
            if name.starts_with("lib/") {
                let parts: Vec<&str> = name.split('/').collect();
                if parts.len() >= 2 {
                    if let Some(abi) = Abi::from_str(parts[1]) {
                        abis.insert(abi);
                    }
                }
            }
        }

        // Placeholder for package name extraction (AXML parsing)
        let package_name = "com.example.placeholder".to_string();

        Ok(ApkInfo {
            package_name,
            supported_abis: abis.into_iter().collect(),
        })
    }

    pub fn extract_libs(&self, target_dir: &Path, abi: &Abi) -> Result<()> {
        let file = File::open(&self.path)?;
        let mut archive = ZipArchive::new(file)?;
        let abi_prefix = format!("lib/{}/", abi.as_str());

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();
            
            if name.starts_with(&abi_prefix) && name.ends_with(".so") {
                let rel_path = name.strip_prefix(&abi_prefix).unwrap();
                let out_path = target_dir.join(rel_path);
                
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                
                let mut outfile = File::create(&out_path)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        Ok(())
    }
}
