use std::env;
use wayland_client::protocol::wl_registry::WlRegistry;
use wayland_client::protocol::wl_compositor::WlCompositor;
use wayland_client::protocol::xdg_wm_base::XdgWmBase;
use wayland_client::protocol::wl_shm::WlShm;
use wayland_client::Display;
use wayland_client::globals::GlobalList;
use wayland_client::EventQueue;

pub struct WaylandConnection {
    display: Display,
    event_queue: EventQueue,
    compositor: Option<WlCompositor>,
    shm: Option<WlShm>,
    wm_base: Option<XdgWmBase>,
}

impl WaylandConnection {
    pub fn new() -> Result<Self, String> {
        // Check if WAYLAND_DISPLAY environment variable is set
        let display_name = env::var("WAYLAND_DISPLAY").map_err(|_| {
            "WAYLAND_DISPLAY environment variable not set".to_string()
        })?;
        
        if display_name.is_empty() {
            return Err("WAYLAND_DISPLAY environment variable is empty".to_string());
        }
        
        // Connect to Wayland display
        let display = Display::connect_to_name(&display_name).map_err(|e| {
            format!("Failed to connect to Wayland display '{}': {}", display_name, e)
        })?;
        
        // Create event queue
        let event_queue = display.create_event_queue();
        
        // Get registry
        let registry = display.get_registry();
        
        // Create globals manager
        let mut globals = GlobalList::new();
        
        // Roundtrip to get initial globals
        event_queue.roundtrip(&display).map_err(|e| {
            format!("Failed to roundtrip with Wayland display: {}", e)
        })?;
        
        // Get available globals
        globals.refresh(&registry);
        
        // Initialize connection with available globals
        let mut connection = WaylandConnection {
            display,
            event_queue,
            compositor: None,
            shm: None,
            wm_base: None,
        };
        
        // Setup available globals
        connection.setup_globals(&globals);
        
        Ok(connection)
    }
    
    fn setup_globals(&mut self, globals: &GlobalList) {
        // Check for compositor
        if let Some(global) = globals.get_global("wl_compositor") {
            self.compositor = Some(global.instantiate::<WlCompositor>(1).unwrap());
        }
        
        // Check for SHM
        if let Some(global) = globals.get_global("wl_shm") {
            self.shm = Some(global.instantiate::<WlShm>(1).unwrap());
        }
        
        // Check for xdg_wm_base
        if let Some(global) = globals.get_global("xdg_wm_base") {
            self.wm_base = Some(global.instantiate::<XdgWmBase>(1).unwrap());
        }
    }
    
    pub fn compositor(&self) -> Option<&WlCompositor> {
        self.compositor.as_ref()
    }
    
    pub fn shm(&self) -> Option<&WlShm> {
        self.shm.as_ref()
    }
    
    pub fn wm_base(&self) -> Option<&XdgWmBase> {
        self.wm_base.as_ref()
    }
    
    pub fn roundtrip(&mut self) -> Result<(), String> {
        self.event_queue.roundtrip(&self.display).map_err(|e| {
            format!("Failed to roundtrip with Wayland display: {}", e)
        })
    }
}

pub fn create_wayland_connection() -> Result<(), String> {
    WaylandConnection::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_wayland_connection_fails_without_display() {
        // Remove WAYLAND_DISPLAY environment variable if it exists
        let _ = env::remove_var("WAYLAND_DISPLAY");
        
        // Attempt to create Wayland connection
        let result = create_wayland_connection();
        
        // Should fail because WAYLAND_DISPLAY is not set
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "WAYLAND_DISPLAY environment variable not set");
    }
    
    #[test]
    fn test_wayland_connection_succeeds_with_display() {
        // Set WAYLAND_DISPLAY environment variable
        env::set_var("WAYLAND_DISPLAY", "wayland-0");
        
        // Attempt to create Wayland connection
        let result = create_wayland_connection();
        
        // Should succeed because WAYLAND_DISPLAY is set
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_wayland_connection_fails_with_empty_display() {
        // Set WAYLAND_DISPLAY environment variable to empty string
        env::set_var("WAYLAND_DISPLAY", "");
        
        // Attempt to create Wayland connection
        let result = create_wayland_connection();
        
        // Should fail because WAYLAND_DISPLAY is empty
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "WAYLAND_DISPLAY environment variable is empty");
    }
}