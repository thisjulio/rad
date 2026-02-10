use std::fs;
use nix::sched::{unshare, CloneFlags};
use nix::unistd::{fork, ForkResult};
use nix::sys::wait::{waitpid, WaitStatus};
use thiserror::Error;

/// Result of a system capability check
#[derive(Debug, Clone, PartialEq)]
pub enum CheckResult {
    /// Capability is available and working
    Available,
    /// Capability is disabled in kernel config
    Disabled,
    /// Capability check failed with an error
    Error(String),
}

impl CheckResult {
    pub fn is_available(&self) -> bool {
        matches!(self, CheckResult::Available)
    }
}


#[derive(Debug, Error)]
pub enum UserNamespaceError {
    #[error("Failed to read /proc/sys/kernel/unprivileged_userns_clone: {0}")]
    ProcReadError(#[from] std::io::Error),
    
    #[error("User namespaces are disabled (unprivileged_userns_clone = 0)")]
    Disabled,
    
    #[error("Failed to unshare user namespace: {0}")]
    UnshareError(#[from] nix::Error),
    
    #[error("Fork failed during user namespace test: {0}")]
    ForkError(String),
    
    #[error("Child process failed to unshare user namespace")]
    ChildUnshareFailure,
}

/// Check if user namespaces are supported on this system
/// 
/// This function performs two checks:
/// 1. Reads /proc/sys/kernel/unprivileged_userns_clone (if exists)
/// 2. Attempts to actually unshare(CLONE_NEWUSER) to validate
/// 
/// Returns Ok(()) if user namespaces are available, Err otherwise
pub fn check_user_namespaces_support() -> Result<(), UserNamespaceError> {
    // Check /proc/sys/kernel/unprivileged_userns_clone
    // Note: This file may not exist on all systems (e.g., Fedora allows by default)
    match fs::read_to_string("/proc/sys/kernel/unprivileged_userns_clone") {
        Ok(content) => {
            let value = content.trim();
            if value == "0" {
                return Err(UserNamespaceError::Disabled);
            }
        }
        Err(_) => {
            // File doesn't exist - this is OK on some systems
            // We'll rely on the practical test below
        }
    }
    
    // Perform practical test: try to unshare(CLONE_NEWUSER) in a child process
    // We use fork() to avoid affecting the current process
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            // Parent process: wait for child to exit
            match waitpid(child, None) {
                Ok(WaitStatus::Exited(_, exit_code)) => {
                    if exit_code == 0 {
                        Ok(())
                    } else {
                        Err(UserNamespaceError::ChildUnshareFailure)
                    }
                }
                Ok(_) => Err(UserNamespaceError::ForkError(
                    "Child terminated abnormally".to_string()
                )),
                Err(e) => Err(UserNamespaceError::ForkError(
                    format!("waitpid failed: {}", e)
                )),
            }
        }
        Ok(ForkResult::Child) => {
            // Child process: try to unshare user namespace
            match unshare(CloneFlags::CLONE_NEWUSER) {
                Ok(_) => {
                    // Success! Exit with code 0
                    std::process::exit(0);
                }
                Err(_) => {
                    // Failed to unshare - exit with code 1
                    std::process::exit(1);
                }
            }
        }
        Err(e) => Err(UserNamespaceError::ForkError(
            format!("fork() failed: {}", e)
        )),
    }
}

/// Public API for checking user namespace support
/// Returns a CheckResult for easy consumption by the doctor command
pub fn check_user_namespaces() -> CheckResult {
    match check_user_namespaces_support() {
        Ok(_) => CheckResult::Available,
        Err(UserNamespaceError::Disabled) => CheckResult::Disabled,
        Err(e) => CheckResult::Error(e.to_string()),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_user_namespaces_support() {
        // This test should pass on systems with user namespace support
        // and fail appropriately on systems without it
        let result = check_user_namespaces_support();
        
        match result {
            Ok(_) => {
                // User namespaces are supported
                println!("User namespaces are supported");
            }
            Err(UserNamespaceError::Disabled) => {
                // Expected on systems where unprivileged_userns_clone = 0
                println!("User namespaces are disabled");
            }
            Err(e) => {
                panic!("Unexpected error checking user namespaces: {}", e);
            }
        }
    }
    
    #[test]
    fn test_check_user_namespaces_public_api() {
        // Test the public API function
        let result = check_user_namespaces();
        
        // Should return one of the valid CheckResult variants
        match &result {
            CheckResult::Available => {
                println!("User namespaces: Available");
                assert!(result.is_available());
            }
            CheckResult::Disabled => {
                println!("User namespaces: Disabled");
                assert!(!result.is_available());
            }
            CheckResult::Error(msg) => {
                println!("User namespaces: Error - {}", msg);
                assert!(!result.is_available());
            }
        }
    }
    
    #[test]
    fn test_proc_file_parsing() {
        // Test that we correctly parse the proc file if it exists
        if let Ok(content) = fs::read_to_string("/proc/sys/kernel/unprivileged_userns_clone") {
            let value = content.trim();
            assert!(value == "0" || value == "1", "Expected 0 or 1, got: {}", value);
        }
    }
    
    #[test]
    fn test_check_result_variants() {
        // Test CheckResult equality and is_available
        assert!(CheckResult::Available.is_available());
        assert!(!CheckResult::Disabled.is_available());
        assert!(!CheckResult::Error("test".to_string()).is_available());
        
        assert_eq!(CheckResult::Available, CheckResult::Available);
        assert_eq!(CheckResult::Disabled, CheckResult::Disabled);
        assert_ne!(CheckResult::Available, CheckResult::Disabled);
    }
}
