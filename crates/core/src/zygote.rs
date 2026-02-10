use std::path::Path;

use anyhow::Result;

pub const ANDROID_ROOT: &str = "ANDROID_ROOT";
pub const ANDROID_DATA: &str = "ANDROID_DATA";
pub const ANDROID_RUNTIME_ROOT: &str = "ANDROID_RUNTIME_ROOT";
pub const LD_LIBRARY_PATH: &str = "LD_LIBRARY_PATH";

pub const APP_PROCESS_PATH: &str = "/system/bin/app_process";
pub const APP_PROCESS_CLASS_PATH: &str = "/system/bin";

#[derive(Debug, Clone)]
pub struct ZygoteLaunchSpec {
    pub executable: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
}

pub fn build_launch_spec(prefix_root: &Path, main_class: &str, main_args: &[String]) -> Result<ZygoteLaunchSpec> {
    validate_runtime_layout(prefix_root)?;

    let mut args = vec![
        APP_PROCESS_CLASS_PATH.to_string(),
        main_class.to_string(),
    ];
    args.extend_from_slice(main_args);

    let env = build_android_env(prefix_root);

    Ok(ZygoteLaunchSpec {
        executable: APP_PROCESS_PATH.to_string(),
        args,
        env,
    })
}

pub fn build_android_env(prefix_root: &Path) -> Vec<(String, String)> {
    vec![
        (ANDROID_ROOT.to_string(), "/system".to_string()),
        (ANDROID_DATA.to_string(), "/data".to_string()),
        (
            ANDROID_RUNTIME_ROOT.to_string(),
            "/apex/com.android.runtime".to_string(),
        ),
        (
            LD_LIBRARY_PATH.to_string(),
            build_ld_library_path(prefix_root),
        ),
    ]
}

fn build_ld_library_path(_prefix_root: &Path) -> String {
    [
        "/system/lib64",
        "/system/lib",
        "/apex/com.android.runtime/lib64",
        "/apex/com.android.runtime/lib",
    ]
    .join(":")
}

fn validate_runtime_layout(prefix_root: &Path) -> Result<()> {
    let app_process = prefix_root.join("system/bin/app_process");
    if !app_process.exists() {
        return Err(anyhow::anyhow!(
            "missing runtime executable: {}",
            APP_PROCESS_PATH
        ));
    }

    let has_system_lib = prefix_root.join("system/lib").is_dir();
    let has_system_lib64 = prefix_root.join("system/lib64").is_dir();
    if !(has_system_lib || has_system_lib64) {
        return Err(anyhow::anyhow!(
            "missing runtime libraries under /system/lib or /system/lib64"
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{build_launch_spec, ANDROID_DATA, ANDROID_ROOT, APP_PROCESS_PATH};

    #[test]
    fn zygote_launch_fails_when_app_process_is_missing() {
        let root = make_temp_root("zygote-missing-app-process");
        fs::create_dir_all(root.join("system/bin")).unwrap();

        let result = build_launch_spec(&root, "com.android.internal.os.RuntimeInit", &[]);

        assert!(result.is_err());
        assert!(
            result
                .err()
                .unwrap()
                .to_string()
                .contains(APP_PROCESS_PATH)
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn zygote_launch_spec_contains_minimal_android_environment() {
        let root = make_temp_root("zygote-minimal-env");
        fs::create_dir_all(root.join("system/bin")).unwrap();
        fs::create_dir_all(root.join("system/lib64")).unwrap();
        fs::write(root.join("system/bin/app_process"), b"binary").unwrap();

        let spec = build_launch_spec(&root, "com.android.internal.os.RuntimeInit", &[]).unwrap();

        assert_eq!(spec.executable, APP_PROCESS_PATH);
        assert_eq!(
            spec.args,
            vec![
                "/system/bin".to_string(),
                "com.android.internal.os.RuntimeInit".to_string()
            ]
        );

        let env = spec.env.into_iter().collect::<std::collections::HashMap<_, _>>();
        assert_eq!(env.get(ANDROID_ROOT).unwrap(), "/system");
        assert_eq!(env.get(ANDROID_DATA).unwrap(), "/data");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn zygote_launch_fails_when_runtime_libraries_are_missing() {
        let root = make_temp_root("zygote-missing-libs");
        fs::create_dir_all(root.join("system/bin")).unwrap();
        fs::write(root.join("system/bin/app_process"), b"binary").unwrap();

        let result = build_launch_spec(&root, "com.android.internal.os.RuntimeInit", &[]);

        assert!(result.is_err());
        assert!(
            result
                .err()
                .unwrap()
                .to_string()
                .contains("missing runtime libraries")
        );

        let _ = fs::remove_dir_all(root);
    }

    fn make_temp_root(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("rad-zygote-{label}-{nanos}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
