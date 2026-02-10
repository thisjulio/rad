use std::fs::{self, File};
use std::path::Path;
use zip::ZipArchive;
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use std::io::Read;
use axmldecoder::Node;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Abi {
    Arm64V8a,
    ArmV7a,
    X86_64,
    X86,
}

impl Abi {
    pub fn from_str_opt(s: &str) -> Option<Self> {
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

/// Parsed data from AndroidManifest.xml
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppManifest {
    pub package_name: String,
    pub version_code: Option<i32>,
    pub version_name: Option<String>,
    pub main_activity: Option<String>,
}

pub struct ApkInspector {
    pub path: std::path::PathBuf,
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
                if let Some(abi) = parts.get(1).and_then(|s| Abi::from_str_opt(s)) {
                    abis.insert(abi);
                }
            }
        }

        let package_name = self.extract_package_name(&mut archive)?;

        Ok(ApkInfo {
            package_name,
            supported_abis: abis.into_iter().collect(),
        })
    }

    fn extract_package_name(&self, archive: &mut ZipArchive<File>) -> Result<String> {
        let mut manifest_file = archive.by_name("AndroidManifest.xml")
            .map_err(|_| anyhow!("AndroidManifest.xml not found in APK"))?;
        
        let mut buffer = Vec::new();
        manifest_file.read_to_end(&mut buffer)?;

        let doc = axmldecoder::parse(&buffer)
            .map_err(|e| anyhow!("Failed to decode AXML: {:?}", e))?;

        match doc.get_root() {
            Some(Node::Element(root)) if root.get_tag() == "manifest" => {
                if let Some(package) = root.get_attributes().get("package") {
                    return Ok(package.to_string());
                }
            }
            _ => {}
        }

        Err(anyhow!("Could not find package attribute in AndroidManifest.xml"))
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

    /// Parse AndroidManifest.xml and extract package name, version, and main activity
    pub fn parse_manifest(&self) -> Result<AppManifest> {
        let file = File::open(&self.path)?;
        let mut archive = ZipArchive::new(file)?;
        
        let mut manifest_file = archive.by_name("AndroidManifest.xml")
            .map_err(|_| anyhow!("AndroidManifest.xml not found in APK"))?;
        
        let mut buffer = Vec::new();
        manifest_file.read_to_end(&mut buffer)?;

        let doc = axmldecoder::parse(&buffer)
            .map_err(|e| anyhow!("Failed to decode AXML: {:?}", e))?;

        let mut manifest = AppManifest {
            package_name: String::new(),
            version_code: None,
            version_name: None,
            main_activity: None,
        };

        // Parse manifest tag attributes
        if let Some(Node::Element(root)) = doc.get_root() {
            if root.get_tag() != "manifest" {
                return Err(anyhow!("Root element is not <manifest>"));
            }
            
            let attrs = root.get_attributes();
            
            // Extract package name
            if let Some(package) = attrs.get("package") {
                manifest.package_name = package.to_string();
            } else {
                return Err(anyhow!("Missing package attribute in manifest"));
            }
            
            // Extract version code (android:versionCode)
            if let Some(version_code) = attrs.get("android:versionCode") {
                manifest.version_code = version_code.parse().ok();
            }
            
            // Extract version name (android:versionName)
            if let Some(version_name) = attrs.get("android:versionName") {
                manifest.version_name = Some(version_name.to_string());
            }
            
            // Find main activity
            manifest.main_activity = find_main_activity(root);
        }

        Ok(manifest)
    }
}

/// Recursively search for the activity with MAIN action intent filter
fn find_main_activity(element: &axmldecoder::Element) -> Option<String> {
    // Search for <application> tag
    for child in element.get_children() {
        if let Node::Element(app) = child
            && app.get_tag() == "application" {
                // Search for <activity> tags
                for activity_node in app.get_children() {
                    if let Node::Element(activity) = activity_node
                        && activity.get_tag() == "activity" {
                            // Check if this activity has MAIN intent filter
                            if has_main_intent_filter(activity) {
                                // Return the android:name attribute
                                if let Some(name) = activity.get_attributes().get("android:name") {
                                    return Some(name.to_string());
                                }
                            }
                        }
                }
            }
    }
    None
}

/// Check if an activity element has an intent-filter with action.MAIN
fn has_main_intent_filter(activity: &axmldecoder::Element) -> bool {
    for child in activity.get_children() {
        if let Node::Element(intent_filter) = child
            && intent_filter.get_tag() == "intent-filter" {
                // Look for <action android:name="android.intent.action.MAIN" />
                for action_node in intent_filter.get_children() {
                    if let Node::Element(action) = action_node
                        && action.get_tag() == "action"
                            && let Some(name) = action.get_attributes().get("android:name")
                                && name == "android.intent.action.MAIN" {
                                    return true;
                                }
                }
            }
    }
    false
}
