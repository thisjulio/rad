use std::path::Path;

pub struct DoctorIssue {
    pub name: String,
    pub status: bool,
    pub description: String,
    pub fix: Option<String>,
}

pub fn run_doctor() -> Vec<DoctorIssue> {
    vec![
        // Check Binder
        check_binder(),
        // Check Namespaces
        check_namespaces(),
        // Check Cgroups v2
        check_cgroups_v2(),
        // Check OverlayFS
        check_overlayfs(),
    ]
}

fn check_overlayfs() -> DoctorIssue {
    let status = Path::new("/proc/filesystems").exists() && {
        let content = std::fs::read_to_string("/proc/filesystems").unwrap_or_default();
        content.contains("overlay")
    };

    DoctorIssue {
        name: "OverlayFS".to_string(),
        status,
        description: if status {
            "OverlayFS is supported by the kernel.".to_string()
        } else {
            "OverlayFS is NOT supported. Useful for efficient prefix management.".to_string()
        },
        fix: if !status {
            Some("Ensure overlay module is loaded (`modprobe overlay`).".to_string())
        } else {
            None
        },
    }
}

fn check_binder() -> DoctorIssue {
    let status = sandbox::check_binderfs();
    let binder_dev = Path::new("/dev/binder").exists();
    
    let ok = status.kernel_support || binder_dev;
    
    let mut description = if status.kernel_support {
        "Binderfs is supported by the kernel.".to_string()
    } else if binder_dev {
        "Legacy /dev/binder found.".to_string()
    } else {
        "Binder IPC is NOT supported by the kernel.".to_string()
    };

    if status.kernel_support && !status.control_exists {
        description.push_str(" However, /dev/binderfs/binder-control is missing.");
    }
    
    DoctorIssue {
        name: "Binder IPC".to_string(),
        status: ok,
        description,
        fix: if !ok {
            Some("Ensure CONFIG_ANDROID_BINDERFS=y or CONFIG_ANDROID_BINDER_IPC=y in kernel config.".to_string())
        } else if status.kernel_support && !status.control_exists {
            Some("Mount binderfs: `mkdir /dev/binderfs && mount -t binder binder /dev/binderfs`".to_string())
        } else {
            None
        },
    }
}

fn check_namespaces() -> DoctorIssue {
    let user_ns = Path::new("/proc/self/ns/user").exists();
    
    DoctorIssue {
        name: "User Namespaces".to_string(),
        status: user_ns,
        description: if user_ns {
            "User namespaces are supported by the kernel.".to_string()
        } else {
            "User namespaces are NOT supported or disabled. Required for rootless execution.".to_string()
        },
        fix: if !user_ns {
            Some("Enable user namespaces in your kernel config (CONFIG_USER_NS=y).".to_string())
        } else {
            None
        },
    }
}

fn check_cgroups_v2() -> DoctorIssue {
    check_cgroups_v2_with(
        |path| Path::new(path).exists(),
        |path| std::fs::read_to_string(path),
    )
}

fn check_cgroups_v2_with<ExistsFn, ReadFn>(exists: ExistsFn, read_to_string: ReadFn) -> DoctorIssue
where
    ExistsFn: Fn(&str) -> bool,
    ReadFn: Fn(&str) -> std::io::Result<String>,
{
    const CGROUP_CONTROLLERS_PATH: &str = "/sys/fs/cgroup/cgroup.controllers";

    let status = exists(CGROUP_CONTROLLERS_PATH);
    let controllers = if status {
        read_to_string(CGROUP_CONTROLLERS_PATH)
            .map(|content| parse_cgroup_controllers(&content))
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let description = if status {
        if controllers.is_empty() {
            "Cgroups v2 is enabled, but no controllers were reported.".to_string()
        } else {
            format!(
                "Cgroups v2 is enabled. Available controllers: {}.",
                controllers.join(", ")
            )
        }
    } else {
        "Cgroups v2 is NOT detected. Required for modern resource isolation.".to_string()
    };

    DoctorIssue {
        name: "Cgroups v2".to_string(),
        status,
        description,
        fix: if !status {
            Some(
                "Enable Cgroups v2 (unified hierarchy) and ensure /sys/fs/cgroup/cgroup.controllers exists."
                    .to_string(),
            )
        } else {
            None
        },
    }
}

fn parse_cgroup_controllers(content: &str) -> Vec<String> {
    content
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::{check_cgroups_v2_with, parse_cgroup_controllers, run_doctor};

    #[test]
    fn doctor_reports_cgroups_v2_check() {
        let issues = run_doctor();

        assert!(
            issues.iter().any(|issue| issue.name == "Cgroups v2"),
            "doctor output should include Cgroups v2 status"
        );
    }

    #[test]
    fn cgroups_v2_check_fails_when_controllers_file_is_missing() {
        let issue = check_cgroups_v2_with(|_| false, |_| Ok(String::new()));

        assert!(!issue.status);
        assert_eq!(issue.name, "Cgroups v2");
        assert!(issue.fix.is_some());
    }

    #[test]
    fn parse_cgroup_controllers_splits_whitespace() {
        let controllers = parse_cgroup_controllers("cpu memory\nio  pids\n");

        assert_eq!(controllers, vec!["cpu", "memory", "io", "pids"]);
    }
}
