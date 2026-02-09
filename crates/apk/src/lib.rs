use std::fs::File;
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
        // In a real scenario, we would parse AndroidManifest.xml
        let package_name = "com.example.placeholder".to_string();

        Ok(ApkInfo {
            package_name,
            supported_abis: abis.into_iter().collect(),
        })
    }
}
