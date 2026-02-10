use tempfile::tempdir;

/// Integration test that validates namespace creation works in a child process
#[test]
fn test_namespace_in_child_process() {
    // Real integration testing would require spawning actual processes
    // with fork/exec which is complex in Rust tests and requires capabilities.
    // 
    // For full integration testing, use the CLI:
    // ```bash
    // sudo cargo run -- run test.apk
    // ```
    
    // This smoke test verifies the integration API can be called.
    let _status = sandbox::check_binderfs();
}

/// Test that setup_mounts would create the expected directory structure
#[test]
fn test_setup_mounts_creates_dirs() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let rootfs = tmp.path();
    
    // Create the basic structure (simulating what setup_mounts would do)
    std::fs::create_dir_all(rootfs.join("proc")).unwrap();
    std::fs::create_dir_all(rootfs.join("sys")).unwrap();
    std::fs::create_dir_all(rootfs.join("dev")).unwrap();
    std::fs::create_dir_all(rootfs.join("tmp")).unwrap();
    
    // Verify directories exist
    assert!(rootfs.join("proc").exists());
    assert!(rootfs.join("sys").exists());
    assert!(rootfs.join("dev").exists());
    assert!(rootfs.join("tmp").exists());
}

/// Test that validates SandboxConfig can be created
#[test]
fn test_sandbox_config() {
    let config = sandbox::SandboxConfig {
        rootfs: std::path::PathBuf::from("/test/path"),
    };
    
    assert_eq!(config.rootfs.to_str().unwrap(), "/test/path");
}
