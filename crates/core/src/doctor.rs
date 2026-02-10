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
    let binderfs = Path::new("/dev/binderfs").exists();
    let binder_dev = Path::new("/dev/binder").exists();
    
    let status = binderfs || binder_dev;
    
    DoctorIssue {
        name: "Binder Device".to_string(),
        status,
        description: if status {
            "Binder device or binderfs found.".to_string()
        } else {
            "Binder device not found. Android apps require Binder for IPC.".to_string()
        },
        fix: if !status {
            Some("Ensure binder is enabled in your kernel or mount binderfs.".to_string())
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
