use apk::{ApkInspector};

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
