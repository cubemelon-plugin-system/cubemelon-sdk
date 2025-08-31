use crate::error::CubeMelonPluginErrorCode;
use crate::types::CubeMelonExecutionStatus;
use crate::instance::CubeMelonPlugin;

/// Resident Interface - Background service execution
/// 
/// Runs in memory as a resident service and automatically executes tasks according to configuration.
/// Configuration is done in user-defined JSON format.
/// 
/// # State Transitions
/// - `start()`: IDLE → RUNNING (with initial data)
/// - `suspend()`: RUNNING → SUSPENDED (pause)
/// - `resume()`: SUSPENDED → RUNNING (resume)
/// - `stop()`: RUNNING/SUSPENDED → COMPLETED (terminate)
/// - `reset()`: COMPLETED/ERROR/CANCELLED → IDLE (reusable)
/// 
/// # Implementation Example
/// ```rust
/// use cubemelon_sdk::prelude::*;
/// 
/// struct MyPlugin {
///     status: CubeMelonExecutionStatus,
///     config: String,
/// }
/// 
/// impl CubeMelonResidentInterface for MyPlugin {
///     fn get_status(&self) -> CubeMelonExecutionStatus {
///         self.status
///     }
///     
///     fn start(&mut self, config_json: *const u8) -> CubeMelonPluginErrorCode {
///         if self.status != CubeMelonExecutionStatus::Idle {
///             return CubeMelonPluginErrorCode::InvalidState;
///         }
///         self.status = CubeMelonExecutionStatus::Running;
///         CubeMelonPluginErrorCode::Success
///     }
///     
///     // ... other methods
/// }
/// ```
pub trait CubeMelonResidentInterface {
    /// Get service status
    /// 
    /// # Returns
    /// * `CubeMelonExecutionStatus` - Current execution status
    fn get_status(&self) -> CubeMelonExecutionStatus;

    /// Get current configuration
    /// 
    /// # Returns
    /// * Configuration JSON string (static, should not be freed by caller)
    /// * Returns null pointer if no configuration is set
    fn get_configuration(&self) -> *const u8;

    /// Update configuration during execution
    /// 
    /// # Arguments
    /// * `config_json` - New configuration in JSON format (UTF-8, null-terminated)
    /// 
    /// # Returns
    /// * `CubeMelonPluginErrorCode` - Error code for operation result
    /// 
    /// # Notes
    /// - Can be called while service is running
    /// - Configuration changes take effect immediately or at next cycle
    fn update_configuration(&mut self, config_json: *const u8) -> CubeMelonPluginErrorCode;

    /// Start service
    /// 
    /// # Arguments
    /// * `config_json` - Initial configuration in JSON format (UTF-8, null-terminated)
    /// 
    /// # Returns
    /// * `CubeMelonPluginErrorCode` - Error code for operation result
    /// 
    /// # State Transition
    /// - Success: IDLE → RUNNING
    /// - Error if not in IDLE state
    fn start(&mut self, config_json: *const u8) -> CubeMelonPluginErrorCode;

    /// Suspend service
    /// 
    /// # Returns
    /// * `CubeMelonPluginErrorCode` - Error code for operation result
    /// 
    /// # State Transition
    /// - Success: RUNNING → SUSPENDED
    /// - Error if not in RUNNING state
    fn suspend(&mut self) -> CubeMelonPluginErrorCode;

    /// Resume service
    /// 
    /// # Returns
    /// * `CubeMelonPluginErrorCode` - Error code for operation result
    /// 
    /// # State Transition
    /// - Success: SUSPENDED → RUNNING
    /// - Error if not in SUSPENDED state
    fn resume(&mut self) -> CubeMelonPluginErrorCode;

    /// Stop service
    /// 
    /// # Returns
    /// * `CubeMelonPluginErrorCode` - Error code for operation result
    /// 
    /// # State Transition
    /// - Success: RUNNING/SUSPENDED → COMPLETED
    /// - Can be called from RUNNING or SUSPENDED state
    fn stop(&mut self) -> CubeMelonPluginErrorCode;

    /// Reset to initial state
    /// 
    /// # Returns
    /// * `CubeMelonPluginErrorCode` - Error code for operation result
    /// 
    /// # State Transition
    /// - Success: COMPLETED/ERROR/CANCELLED → IDLE
    /// - Error if not in terminal state
    fn reset(&mut self) -> CubeMelonPluginErrorCode;
}

/// C ABI ResidentInterface structure
/// 
/// The actual interface structure returned by `get_interface()` from plugins
#[repr(C)]
pub struct CubeMelonResidentInterfaceImpl {
    /// Get service status
    pub get_status: extern "C" fn(
        plugin: *const CubeMelonPlugin,
    ) -> CubeMelonExecutionStatus,

    /// Get current configuration
    /// Returns static string, caller should not free
    pub get_configuration: extern "C" fn(
        plugin: *const CubeMelonPlugin,
    ) -> *const u8,

    /// Update configuration during execution
    pub update_configuration: extern "C" fn(
        plugin: *mut CubeMelonPlugin,
        config_json: *const u8,
    ) -> CubeMelonPluginErrorCode,

    /// Start service
    /// Success: IDLE → RUNNING, Error if not IDLE
    pub start: extern "C" fn(
        plugin: *mut CubeMelonPlugin,
        config_json: *const u8,
    ) -> CubeMelonPluginErrorCode,

    /// Suspend service
    /// Success: RUNNING → SUSPENDED, Error if not RUNNING
    pub suspend: extern "C" fn(
        plugin: *mut CubeMelonPlugin,
    ) -> CubeMelonPluginErrorCode,

    /// Resume service
    /// Success: SUSPENDED → RUNNING, Error if not SUSPENDED
    pub resume: extern "C" fn(
        plugin: *mut CubeMelonPlugin,
    ) -> CubeMelonPluginErrorCode,

    /// Stop service
    /// Success: RUNNING/SUSPENDED → COMPLETED
    pub stop: extern "C" fn(
        plugin: *mut CubeMelonPlugin,
    ) -> CubeMelonPluginErrorCode,

    /// Reset to initial state
    /// Success: COMPLETED/ERROR/CANCELLED → IDLE
    pub reset: extern "C" fn(
        plugin: *mut CubeMelonPlugin,
    ) -> CubeMelonPluginErrorCode,
}

/// Helper function to generate C ABI interface from CubeMelonResidentInterface trait implementation
/// 
/// # Usage Example
/// ```rust
/// use cubemelon_sdk::prelude::*;
///
/// #[plugin]
/// struct MyPlugin;
///
/// let interface = create_resident_interface::<MyPlugin>();
/// ```
pub fn create_resident_interface<T>() -> CubeMelonResidentInterfaceImpl
where
    T: CubeMelonResidentInterface + 'static,
{
    extern "C" fn get_status_wrapper<T: CubeMelonResidentInterface + 'static>(
        plugin: *const CubeMelonPlugin,
    ) -> CubeMelonExecutionStatus {
        // NULL pointer check
        if plugin.is_null() {
            return CubeMelonExecutionStatus::Error;
        }

        // Access plugin instance safely through type-safe wrapper
        match crate::instance::with_plugin::<T, _, _>(plugin, |plugin_instance| {
            plugin_instance.get_status()
        }) {
            Some(status) => status,
            None => CubeMelonExecutionStatus::Error, // Plugin not found
        }
    }

    extern "C" fn get_configuration_wrapper<T: CubeMelonResidentInterface + 'static>(
        plugin: *const CubeMelonPlugin,
    ) -> *const u8 {
        // NULL pointer check
        if plugin.is_null() {
            return std::ptr::null();
        }

        // Access plugin instance safely through type-safe wrapper
        match crate::instance::with_plugin::<T, _, _>(plugin, |plugin_instance| {
            plugin_instance.get_configuration()
        }) {
            Some(config_ptr) => config_ptr,
            None => std::ptr::null(), // Plugin not found
        }
    }

    extern "C" fn update_configuration_wrapper<T: CubeMelonResidentInterface + 'static>(
        plugin: *mut CubeMelonPlugin,
        config_json: *const u8,
    ) -> CubeMelonPluginErrorCode {
        // NULL pointer check
        if plugin.is_null() {
            return CubeMelonPluginErrorCode::NullPointer;
        }

        // Access plugin instance safely through type-safe wrapper
        match crate::instance::with_plugin_mut::<T, _, _>(plugin, |plugin_instance| {
            plugin_instance.update_configuration(config_json)
        }) {
            Some(error_code) => error_code,
            None => CubeMelonPluginErrorCode::PluginNotFound,
        }
    }

    extern "C" fn start_wrapper<T: CubeMelonResidentInterface + 'static>(
        plugin: *mut CubeMelonPlugin,
        config_json: *const u8,
    ) -> CubeMelonPluginErrorCode {
        // NULL pointer check
        if plugin.is_null() {
            return CubeMelonPluginErrorCode::NullPointer;
        }

        // Access plugin instance safely through type-safe wrapper
        match crate::instance::with_plugin_mut::<T, _, _>(plugin, |plugin_instance| {
            plugin_instance.start(config_json)
        }) {
            Some(error_code) => error_code,
            None => CubeMelonPluginErrorCode::PluginNotFound,
        }
    }

    extern "C" fn suspend_wrapper<T: CubeMelonResidentInterface + 'static>(
        plugin: *mut CubeMelonPlugin,
    ) -> CubeMelonPluginErrorCode {
        // NULL pointer check
        if plugin.is_null() {
            return CubeMelonPluginErrorCode::NullPointer;
        }

        // Access plugin instance safely through type-safe wrapper
        match crate::instance::with_plugin_mut::<T, _, _>(plugin, |plugin_instance| {
            plugin_instance.suspend()
        }) {
            Some(error_code) => error_code,
            None => CubeMelonPluginErrorCode::PluginNotFound,
        }
    }

    extern "C" fn resume_wrapper<T: CubeMelonResidentInterface + 'static>(
        plugin: *mut CubeMelonPlugin,
    ) -> CubeMelonPluginErrorCode {
        // NULL pointer check
        if plugin.is_null() {
            return CubeMelonPluginErrorCode::NullPointer;
        }

        // Access plugin instance safely through type-safe wrapper
        match crate::instance::with_plugin_mut::<T, _, _>(plugin, |plugin_instance| {
            plugin_instance.resume()
        }) {
            Some(error_code) => error_code,
            None => CubeMelonPluginErrorCode::PluginNotFound,
        }
    }

    extern "C" fn stop_wrapper<T: CubeMelonResidentInterface + 'static>(
        plugin: *mut CubeMelonPlugin,
    ) -> CubeMelonPluginErrorCode {
        // NULL pointer check
        if plugin.is_null() {
            return CubeMelonPluginErrorCode::NullPointer;
        }

        // Access plugin instance safely through type-safe wrapper
        match crate::instance::with_plugin_mut::<T, _, _>(plugin, |plugin_instance| {
            plugin_instance.stop()
        }) {
            Some(error_code) => error_code,
            None => CubeMelonPluginErrorCode::PluginNotFound,
        }
    }

    extern "C" fn reset_wrapper<T: CubeMelonResidentInterface + 'static>(
        plugin: *mut CubeMelonPlugin,
    ) -> CubeMelonPluginErrorCode {
        // NULL pointer check
        if plugin.is_null() {
            return CubeMelonPluginErrorCode::NullPointer;
        }

        // Access plugin instance safely through type-safe wrapper
        match crate::instance::with_plugin_mut::<T, _, _>(plugin, |plugin_instance| {
            plugin_instance.reset()
        }) {
            Some(error_code) => error_code,
            None => CubeMelonPluginErrorCode::PluginNotFound,
        }
    }

    CubeMelonResidentInterfaceImpl {
        get_status: get_status_wrapper::<T>,
        get_configuration: get_configuration_wrapper::<T>,
        update_configuration: update_configuration_wrapper::<T>,
        start: start_wrapper::<T>,
        suspend: suspend_wrapper::<T>,
        resume: resume_wrapper::<T>,
        stop: stop_wrapper::<T>,
        reset: reset_wrapper::<T>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    struct TestResidentPlugin {
        status: CubeMelonExecutionStatus,
        config: String,
    }

    impl CubeMelonResidentInterface for TestResidentPlugin {
        fn get_status(&self) -> CubeMelonExecutionStatus {
            self.status
        }

        fn get_configuration(&self) -> *const u8 {
            if self.config.is_empty() {
                std::ptr::null()
            } else {
                self.config.as_ptr()
            }
        }

        fn update_configuration(&mut self, config_json: *const u8) -> CubeMelonPluginErrorCode {
            if config_json.is_null() {
                return CubeMelonPluginErrorCode::NullPointer;
            }
            
            // In real implementation, would parse the JSON
            // For testing, just mark as updated
            CubeMelonPluginErrorCode::Success
        }

        fn start(&mut self, config_json: *const u8) -> CubeMelonPluginErrorCode {
            if self.status != CubeMelonExecutionStatus::Idle {
                return CubeMelonPluginErrorCode::InvalidState;
            }

            if config_json.is_null() {
                return CubeMelonPluginErrorCode::NullPointer;
            }

            self.status = CubeMelonExecutionStatus::Running;
            // In real implementation, would parse and store config
            CubeMelonPluginErrorCode::Success
        }

        fn suspend(&mut self) -> CubeMelonPluginErrorCode {
            if self.status != CubeMelonExecutionStatus::Running {
                return CubeMelonPluginErrorCode::InvalidState;
            }

            self.status = CubeMelonExecutionStatus::Suspended;
            CubeMelonPluginErrorCode::Success
        }

        fn resume(&mut self) -> CubeMelonPluginErrorCode {
            if self.status != CubeMelonExecutionStatus::Suspended {
                return CubeMelonPluginErrorCode::InvalidState;
            }

            self.status = CubeMelonExecutionStatus::Running;
            CubeMelonPluginErrorCode::Success
        }

        fn stop(&mut self) -> CubeMelonPluginErrorCode {
            match self.status {
                CubeMelonExecutionStatus::Running | CubeMelonExecutionStatus::Suspended => {
                    self.status = CubeMelonExecutionStatus::Completed;
                    CubeMelonPluginErrorCode::Success
                }
                _ => CubeMelonPluginErrorCode::InvalidState,
            }
        }

        fn reset(&mut self) -> CubeMelonPluginErrorCode {
            match self.status {
                CubeMelonExecutionStatus::Completed 
                | CubeMelonExecutionStatus::Error 
                | CubeMelonExecutionStatus::Cancelled => {
                    self.status = CubeMelonExecutionStatus::Idle;
                    self.config.clear();
                    CubeMelonPluginErrorCode::Success
                }
                _ => CubeMelonPluginErrorCode::InvalidState,
            }
        }
    }

    #[test]
    fn test_resident_interface_creation() {
        let interface = create_resident_interface::<TestResidentPlugin>();
        
        // Verify function pointers are set (not null)
        let get_status_fn = interface.get_status;
        let get_config_fn = interface.get_configuration;
        let update_config_fn = interface.update_configuration;
        let start_fn = interface.start;
        let suspend_fn = interface.suspend;
        let resume_fn = interface.resume;
        let stop_fn = interface.stop;
        let reset_fn = interface.reset;
        
        assert!(get_status_fn as usize != 0);
        assert!(get_config_fn as usize != 0);
        assert!(update_config_fn as usize != 0);
        assert!(start_fn as usize != 0);
        assert!(suspend_fn as usize != 0);
        assert!(resume_fn as usize != 0);
        assert!(stop_fn as usize != 0);
        assert!(reset_fn as usize != 0);
    }

    #[test]
    fn test_resident_plugin_state_transitions() {
        let mut plugin = TestResidentPlugin {
            status: CubeMelonExecutionStatus::Idle,
            config: String::new(),
        };

        // Initial state should be Idle
        assert_eq!(plugin.get_status(), CubeMelonExecutionStatus::Idle);

        // Start should succeed from Idle
        let test_config = "{}".as_ptr();
        assert_eq!(plugin.start(test_config), CubeMelonPluginErrorCode::Success);
        assert_eq!(plugin.get_status(), CubeMelonExecutionStatus::Running);

        // Suspend should succeed from Running
        assert_eq!(plugin.suspend(), CubeMelonPluginErrorCode::Success);
        assert_eq!(plugin.get_status(), CubeMelonExecutionStatus::Suspended);

        // Resume should succeed from Suspended
        assert_eq!(plugin.resume(), CubeMelonPluginErrorCode::Success);
        assert_eq!(plugin.get_status(), CubeMelonExecutionStatus::Running);

        // Stop should succeed from Running
        assert_eq!(plugin.stop(), CubeMelonPluginErrorCode::Success);
        assert_eq!(plugin.get_status(), CubeMelonExecutionStatus::Completed);

        // Reset should succeed from Completed
        assert_eq!(plugin.reset(), CubeMelonPluginErrorCode::Success);
        assert_eq!(plugin.get_status(), CubeMelonExecutionStatus::Idle);
    }

    #[test]
    fn test_invalid_state_transitions() {
        let mut plugin = TestResidentPlugin {
            status: CubeMelonExecutionStatus::Running,
            config: String::new(),
        };

        // Start should fail from Running state
        let test_config = "{}".as_ptr();
        assert_eq!(plugin.start(test_config), CubeMelonPluginErrorCode::InvalidState);

        // Resume should fail from Running state
        assert_eq!(plugin.resume(), CubeMelonPluginErrorCode::InvalidState);

        // Reset should fail from Running state
        assert_eq!(plugin.reset(), CubeMelonPluginErrorCode::InvalidState);
    }
}