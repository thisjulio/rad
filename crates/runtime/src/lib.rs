//! Runtime - Android System Services Stubs
//!
//! This module provides stub implementations of Android system services
//! that respond to Binder calls from Android applications.
//!
//! # Architecture
//!
//! The runtime provides minimal stub services that Android apps expect:
//! - ActivityManager: App lifecycle and permission checks
//! - PackageManager: Package and component queries
//!
//! These stubs prevent apps from crashing when they try to access
//! system services via Binder IPC.

use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn, debug};

/// Result type for runtime operations
pub type Result<T> = std::result::Result<T, RuntimeError>;

/// Errors that can occur in the runtime
#[derive(Debug, Clone)]
pub enum RuntimeError {
    ServiceNotFound(String),
    ServiceAlreadyRegistered(String),
    InvalidRequest(String),
    NotImplemented(String),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::ServiceNotFound(name) => write!(f, "Service not found: {}", name),
            RuntimeError::ServiceAlreadyRegistered(name) => write!(f, "Service already registered: {}", name),
            RuntimeError::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
            RuntimeError::NotImplemented(feature) => write!(f, "Not implemented: {}", feature),
        }
    }
}

impl std::error::Error for RuntimeError {}

/// Trait representing a stub system service
///
/// Implementors provide minimal responses to prevent app crashes
pub trait StubService: Send + Sync {
    /// Get the service name (e.g., "activity", "package")
    fn service_name(&self) -> &str;
    
    /// Get the service interface descriptor
    fn interface_descriptor(&self) -> &str;
    
    /// Handle a method call
    /// 
    /// # Arguments
    /// * `method` - The method name being called
    /// * `args` - Serialized arguments (format TBD based on Binder protocol)
    /// 
    /// # Returns
    /// Serialized response data
    fn handle_call(&self, method: &str, _args: &[u8]) -> Result<Vec<u8>> {
        debug!("Stub service '{}' received call to '{}'", self.service_name(), method);
        // Default implementation returns empty success
        Ok(Vec::new())
    }
}

/// Stub implementation of ActivityManager service
///
/// Provides minimal responses for:
/// - Permission checks (always allow)
/// - App ops queries (always allow)
/// - Process management (no-op)
pub struct ActivityManagerStub {
    name: String,
}

impl ActivityManagerStub {
    pub fn new() -> Self {
        Self {
            name: "activity".to_string(),
        }
    }
}

impl Default for ActivityManagerStub {
    fn default() -> Self {
        Self::new()
    }
}

impl StubService for ActivityManagerStub {
    fn service_name(&self) -> &str {
        &self.name
    }
    
    fn interface_descriptor(&self) -> &str {
        "android.app.IActivityManager"
    }
    
    fn handle_call(&self, method: &str, _args: &[u8]) -> Result<Vec<u8>> {
        match method {
            "checkPermission" => {
                // Return permission granted (1 = PERMISSION_GRANTED)
                Ok(vec![1, 0, 0, 0]) // i32 as little-endian bytes
            }
            "getAppOpsService" => {
                // Return empty (null service reference)
                Ok(Vec::new())
            }
            _ => {
                warn!("ActivityManager method '{}' not implemented in stub", method);
                Ok(Vec::new())
            }
        }
    }
}

/// Stub implementation of PackageManager service
///
/// Provides minimal responses for package queries
pub struct PackageManagerStub {
    name: String,
}

impl PackageManagerStub {
    pub fn new() -> Self {
        Self {
            name: "package".to_string(),
        }
    }
}

impl Default for PackageManagerStub {
    fn default() -> Self {
        Self::new()
    }
}

impl StubService for PackageManagerStub {
    fn service_name(&self) -> &str {
        &self.name
    }
    
    fn interface_descriptor(&self) -> &str {
        "android.content.pm.IPackageManager"
    }
}

/// Service registry for managing stub services
///
/// Maintains a registry of all available stub services and provides
/// lookup by service name.
pub struct ServiceRegistry {
    services: HashMap<String, Arc<dyn StubService>>,
}

impl ServiceRegistry {
    /// Create a new empty service registry
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }
    
    /// Register a stub service
    ///
    /// # Errors
    /// Returns error if a service with the same name is already registered
    pub fn register(&mut self, service: Arc<dyn StubService>) -> Result<()> {
        let name = service.service_name().to_string();
        
        if self.services.contains_key(&name) {
            return Err(RuntimeError::ServiceAlreadyRegistered(name));
        }
        
        info!("Registering stub service: {}", name);
        self.services.insert(name, service);
        Ok(())
    }
    
    /// Lookup a service by name
    pub fn get_service(&self, name: &str) -> Option<Arc<dyn StubService>> {
        self.services.get(name).cloned()
    }
    
    /// Get the number of registered services
    pub fn service_count(&self) -> usize {
        self.services.len()
    }
    
    /// Check if a service is registered
    pub fn has_service(&self, name: &str) -> bool {
        self.services.contains_key(name)
    }
    
    /// Get list of all registered service names
    pub fn list_services(&self) -> Vec<String> {
        self.services.keys().cloned().collect()
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize the minimal system services
///
/// This function creates and registers the basic stub services
/// that Android apps expect to find via Binder.
///
/// # Services Registered
/// - activity: ActivityManagerStub
/// - package: PackageManagerStub
///
/// # Example
/// ```
/// use runtime::init_minimal_services;
///
/// let registry = init_minimal_services().expect("Failed to init services");
/// assert!(registry.has_service("activity"));
/// ```
pub fn init_minimal_services() -> Result<ServiceRegistry> {
    info!("Initializing minimal Android system services");
    
    let mut registry = ServiceRegistry::new();
    
    // Register ActivityManager stub
    let activity_manager = Arc::new(ActivityManagerStub::new());
    registry.register(activity_manager)?;
    
    // Register PackageManager stub
    let package_manager = Arc::new(PackageManagerStub::new());
    registry.register(package_manager)?;
    
    info!("Registered {} minimal services", registry.service_count());
    info!("Services: {:?}", registry.list_services());
    
    Ok(registry)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // RED: Test that service registry starts empty
    #[test]
    fn test_service_registry_creation() {
        let registry = ServiceRegistry::new();
        assert_eq!(registry.service_count(), 0);
        assert!(registry.list_services().is_empty());
    }
    
    // RED: Test that ActivityManager stub has correct metadata
    #[test]
    fn test_activity_manager_stub_creation() {
        let am = ActivityManagerStub::new();
        assert_eq!(am.service_name(), "activity");
        assert_eq!(am.interface_descriptor(), "android.app.IActivityManager");
    }
    
    // RED: Test that we can register services
    #[test]
    fn test_register_service() {
        let mut registry = ServiceRegistry::new();
        let service = Arc::new(ActivityManagerStub::new());
        
        registry.register(service).expect("Failed to register service");
        
        assert_eq!(registry.service_count(), 1);
        assert!(registry.has_service("activity"));
        assert!(registry.get_service("activity").is_some());
    }
    
    // RED: Test that duplicate registration fails
    #[test]
    fn test_duplicate_registration_fails() {
        let mut registry = ServiceRegistry::new();
        let service1 = Arc::new(ActivityManagerStub::new());
        let service2 = Arc::new(ActivityManagerStub::new());
        
        registry.register(service1).expect("First registration should succeed");
        let result = registry.register(service2);
        
        assert!(result.is_err());
        match result.unwrap_err() {
            RuntimeError::ServiceAlreadyRegistered(name) => {
                assert_eq!(name, "activity");
            }
            _ => panic!("Expected ServiceAlreadyRegistered error"),
        }
    }
    
    // RED: Test service lookup returns None for non-existent service
    #[test]
    fn test_service_not_found() {
        let registry = ServiceRegistry::new();
        assert!(!registry.has_service("nonexistent"));
        assert!(registry.get_service("nonexistent").is_none());
    }
    
    // RED: Test initialization creates expected services
    #[test]
    fn test_init_minimal_services() {
        let registry = init_minimal_services().expect("Failed to init services");
        
        // Should have ActivityManager and PackageManager
        assert_eq!(registry.service_count(), 2);
        assert!(registry.has_service("activity"));
        assert!(registry.has_service("package"));
    }
    
    // RED: Test ActivityManager handle_call for checkPermission
    #[test]
    fn test_activity_manager_check_permission() {
        let am = ActivityManagerStub::new();
        
        // checkPermission should return granted (non-empty response)
        let result = am.handle_call("checkPermission", &[])
            .expect("checkPermission should succeed");
        
        assert!(!result.is_empty(), "checkPermission should return a response");
    }
    
    // RED: Test that unimplemented methods return empty response (no crash)
    #[test]
    fn test_unimplemented_method_returns_empty() {
        let am = ActivityManagerStub::new();
        
        let result = am.handle_call("unknownMethod", &[])
            .expect("Unknown method should not fail");
        
        assert!(result.is_empty());
    }
    
    // RED: Test PackageManager stub
    #[test]
    fn test_package_manager_stub() {
        let pm = PackageManagerStub::new();
        assert_eq!(pm.service_name(), "package");
        assert_eq!(pm.interface_descriptor(), "android.content.pm.IPackageManager");
    }
    
    // RED: Test service list
    #[test]
    fn test_list_services() {
        let registry = init_minimal_services().expect("Failed to init");
        let services = registry.list_services();
        
        assert_eq!(services.len(), 2);
        assert!(services.contains(&"activity".to_string()));
        assert!(services.contains(&"package".to_string()));
    }
}
