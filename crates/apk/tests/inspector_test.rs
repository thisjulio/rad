use apk::{ApkInspector};

/// Test parsing AndroidManifest.xml from a real APK
/// Uses F-Droid APK which has proper AXML format
#[test]
fn test_parse_manifest_package_name() {
    let inspector = ApkInspector::new("test_data/real.apk");
    let manifest = inspector.parse_manifest().expect("Failed to parse manifest");
    
    // F-Droid package name
    assert_eq!(manifest.package_name, "org.fdroid.fdroid");
}

/// Test parsing version code and version name from manifest
#[test]
fn test_parse_manifest_version() {
    let inspector = ApkInspector::new("test_data/real.apk");
    let manifest = inspector.parse_manifest().expect("Failed to parse manifest");
    
    // F-Droid has version info
    assert_eq!(manifest.version_code, Some(1016050));
    assert_eq!(manifest.version_name.as_deref(), Some("1.16"));
}

/// Test parsing main activity from manifest
#[test]
fn test_parse_manifest_main_activity() {
    let inspector = ApkInspector::new("test_data/real.apk");
    let manifest = inspector.parse_manifest().expect("Failed to parse manifest");
    
    // F-Droid has a main activity (panic mode calculator)
    assert_eq!(
        manifest.main_activity.as_deref(),
        Some("org.fdroid.fdroid.panic.CalculatorActivity")
    );
}

/// This test requires a real APK with valid binary AXML.
/// To test manually, download any APK from F-Droid and place it at test_data/real.apk
/// 
/// Example:
/// ```bash
/// wget https://f-droid.org/repo/org.fdroid.fdroid_1016050.apk -O crates/apk/test_data/real.apk
/// cargo test -p apk test_inspect_real_apk_structure -- --ignored
/// ```
#[test]
#[ignore = "requires real APK - manually crafted binary AXML is too complex"]
fn test_inspect_real_apk_structure() {
    let inspector = ApkInspector::new("test_data/real.apk");
    let result = inspector.inspect().expect("Failed to inspect real.apk");
    
    // F-Droid APK example
    assert!(result.package_name.starts_with("org.") || result.package_name.starts_with("com."));
    println!("Package: {}", result.package_name);
    println!("ABIs: {:?}", result.supported_abis);
}
