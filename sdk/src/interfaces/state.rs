use crate::error::CubeMelonPluginErrorCode;
use crate::memory::{CubeMelonValue};
use crate::types::CubeMelonPluginStateScope;
use crate::instance::CubeMelonPlugin;

/// Plugin State Interface - State management
/// 
/// Manages plugin state and supports data persistence and sharing with other plugins.
/// 
/// # State Scopes
/// - `Local`: Plugin's internal state
/// - `Host`: Host environment (language settings, timezone, etc.)
/// - `Shared`: State shared with other plugins (images, history, etc.)
/// 
/// # Implementation Example
/// ```rust
/// use cubemelon_sdk::prelude::*;
/// 
/// struct MyPlugin {
///     local_data: std::collections::HashMap<String, Vec<u8>>,
/// }
/// 
/// impl CubeMelonPluginStateInterface for MyPlugin {
///     fn load_state(
///         &self,
///         scope: CubeMelonPluginStateScope,
///         data: &mut CubeMelonValue,
///     ) -> CubeMelonPluginErrorCode {
///         // Load state data based on scope
///         CubeMelonPluginErrorCode::Success
///     }
///     
///     // ... other methods
/// }
/// ```
pub trait CubeMelonPluginStateInterface {
    /// Load state data
    /// 
    /// # Arguments
    /// * `scope` - State scope (Local, Host, or Shared)
    /// * `data` - Buffer to store loaded data. Caller creates container, plugin sets contents, caller uses data.free_value() to free contents
    /// 
    /// # Returns
    /// * `CubeMelonPluginErrorCode` - Error code for operation result
    fn load_state(
        &self,
        scope: CubeMelonPluginStateScope,
        data: &mut CubeMelonValue,
    ) -> CubeMelonPluginErrorCode;

    /// Save state data
    /// 
    /// # Arguments
    /// * `scope` - State scope (Local, Host, or Shared)
    /// * `data` - Data to save
    /// * `size` - Size of data in bytes
    /// 
    /// # Returns
    /// * `CubeMelonPluginErrorCode` - Error code for operation result
    fn save_state(
        &mut self,
        scope: CubeMelonPluginStateScope,
        data: *const u8,
        size: usize,
    ) -> CubeMelonPluginErrorCode;

    /// Get format name of state data
    /// 
    /// # Arguments
    /// * `scope` - State scope to query
    /// 
    /// # Returns
    /// * Format name (e.g., "json", "toml", "msgpack")
    /// 
    /// # Notes
    /// - Returns a static string that remains valid until plugin unload
    /// - Should not be freed by caller
    fn get_format_name(
        &self,
        scope: CubeMelonPluginStateScope,
    ) -> *const u8;

    /// Get state value for specific key
    /// 
    /// # Arguments
    /// * `scope` - State scope to query
    /// * `key` - Key name (UTF-8, null-terminated)
    /// * `value` - Buffer to store value. Caller creates container, plugin sets contents, caller uses value.free_value() to free contents
    /// 
    /// # Returns
    /// * `CubeMelonPluginErrorCode` - Error code for operation result
    fn get_state_value(
        &self,
        scope: CubeMelonPluginStateScope,
        key: *const u8,
        value: &mut CubeMelonValue,
    ) -> CubeMelonPluginErrorCode;

    /// Set state value for specific key
    /// 
    /// # Arguments
    /// * `scope` - State scope to modify
    /// * `key` - Key name (UTF-8, null-terminated)
    /// * `data` - Data to store
    /// * `size` - Size of data in bytes
    /// 
    /// # Returns
    /// * `CubeMelonPluginErrorCode` - Error code for operation result
    fn set_state_value(
        &mut self,
        scope: CubeMelonPluginStateScope,
        key: *const u8,
        data: *const u8,
        size: usize,
    ) -> CubeMelonPluginErrorCode;

    /// List all state keys
    /// 
    /// # Arguments
    /// * `scope` - State scope to query
    /// * `keys` - String array to store key list. Caller creates container, plugin sets contents, caller uses keys.free_value() to free contents
    /// 
    /// # Returns
    /// * `CubeMelonPluginErrorCode` - Error code for operation result
    fn list_state_keys(
        &self,
        scope: CubeMelonPluginStateScope,
        keys: &mut CubeMelonValue,
    ) -> CubeMelonPluginErrorCode;

    /// Clear state value for specific key
    /// 
    /// # Arguments
    /// * `scope` - State scope to modify
    /// * `key` - Key name to clear (UTF-8, null-terminated)
    /// 
    /// # Returns
    /// * `CubeMelonPluginErrorCode` - Error code for operation result
    fn clear_state_value(
        &mut self,
        scope: CubeMelonPluginStateScope,
        key: *const u8,
    ) -> CubeMelonPluginErrorCode;
}

/// C ABI CubeMelonPluginStateInterface structure
/// 
/// The actual interface structure returned by `get_interface()` from plugins
#[repr(C)]
pub struct CubeMelonPluginStateInterfaceImpl {
    /// Load state data
    /// Caller creates container, plugin sets contents, caller uses data->free_value() to free contents
    pub load_state: extern "C" fn(
        plugin: *const CubeMelonPlugin,
        scope: CubeMelonPluginStateScope,
        data: *mut CubeMelonValue,
    ) -> CubeMelonPluginErrorCode,

    /// Save state data
    pub save_state: extern "C" fn(
        plugin: *mut CubeMelonPlugin,
        scope: CubeMelonPluginStateScope,
        data: *const u8,
        size: usize,
    ) -> CubeMelonPluginErrorCode,

    /// Get format name of state data
    /// Returns static string, caller should not free
    pub get_format_name: extern "C" fn(
        plugin: *const CubeMelonPlugin,
        scope: CubeMelonPluginStateScope,
    ) -> *const u8,

    /// Get state value for specific key
    /// Caller creates container, plugin sets contents, caller uses value->free_value() to free contents
    pub get_state_value: extern "C" fn(
        plugin: *const CubeMelonPlugin,
        scope: CubeMelonPluginStateScope,
        key: *const u8,
        value: *mut CubeMelonValue,
    ) -> CubeMelonPluginErrorCode,

    /// Set state value for specific key
    pub set_state_value: extern "C" fn(
        plugin: *mut CubeMelonPlugin,
        scope: CubeMelonPluginStateScope,
        key: *const u8,
        data: *const u8,
        size: usize,
    ) -> CubeMelonPluginErrorCode,

    /// List all state keys
    /// Caller creates container, plugin sets contents, caller uses keys->free_value() to free contents
    pub list_state_keys: extern "C" fn(
        plugin: *const CubeMelonPlugin,
        scope: CubeMelonPluginStateScope,
        keys: *mut CubeMelonValue,
    ) -> CubeMelonPluginErrorCode,

    /// Clear state value for specific key
    pub clear_state_value: extern "C" fn(
        plugin: *mut CubeMelonPlugin,
        scope: CubeMelonPluginStateScope,
        key: *const u8,
    ) -> CubeMelonPluginErrorCode,
}

/// Helper function to generate C ABI interface from CubeMelonPluginStateInterface trait implementation
/// 
/// # Usage Example
/// ```rust
/// use cubemelon_sdk::prelude::*;
///
/// #[plugin]
/// struct MyPlugin;
///
/// let interface = create_plugin_state_interface::<MyPlugin>();
/// ```
pub fn create_plugin_state_interface<T>() -> CubeMelonPluginStateInterfaceImpl
where
    T: CubeMelonPluginStateInterface + 'static,
{
    extern "C" fn load_state_wrapper<T: CubeMelonPluginStateInterface + 'static>(
        plugin: *const CubeMelonPlugin,
        scope: CubeMelonPluginStateScope,
        data: *mut CubeMelonValue,
    ) -> CubeMelonPluginErrorCode {
        // NULL pointer checks
        if plugin.is_null() || data.is_null() {
            return CubeMelonPluginErrorCode::NullPointer;
        }

        // Safe dereferencing
        let data_ref = unsafe { &mut *data };

        // Access plugin instance safely through type-safe wrapper
        match crate::instance::with_plugin::<T, _, _>(plugin, |plugin_instance| {
            plugin_instance.load_state(scope, data_ref)
        }) {
            Some(error_code) => error_code,
            None => CubeMelonPluginErrorCode::PluginNotFound,
        }
    }

    extern "C" fn save_state_wrapper<T: CubeMelonPluginStateInterface + 'static>(
        plugin: *mut CubeMelonPlugin,
        scope: CubeMelonPluginStateScope,
        data: *const u8,
        size: usize,
    ) -> CubeMelonPluginErrorCode {
        // NULL pointer checks
        if plugin.is_null() {
            return CubeMelonPluginErrorCode::NullPointer;
        }

        // Access plugin instance safely through type-safe wrapper
        match crate::instance::with_plugin_mut::<T, _, _>(plugin, |plugin_instance| {
            plugin_instance.save_state(scope, data, size)
        }) {
            Some(error_code) => error_code,
            None => CubeMelonPluginErrorCode::PluginNotFound,
        }
    }

    extern "C" fn get_format_name_wrapper<T: CubeMelonPluginStateInterface + 'static>(
        plugin: *const CubeMelonPlugin,
        scope: CubeMelonPluginStateScope,
    ) -> *const u8 {
        // NULL pointer check
        if plugin.is_null() {
            return std::ptr::null();
        }

        // Access plugin instance safely through type-safe wrapper
        match crate::instance::with_plugin::<T, _, _>(plugin, |plugin_instance| {
            plugin_instance.get_format_name(scope)
        }) {
            Some(format_ptr) => format_ptr,
            None => std::ptr::null(),
        }
    }

    extern "C" fn get_state_value_wrapper<T: CubeMelonPluginStateInterface + 'static>(
        plugin: *const CubeMelonPlugin,
        scope: CubeMelonPluginStateScope,
        key: *const u8,
        value: *mut CubeMelonValue,
    ) -> CubeMelonPluginErrorCode {
        // NULL pointer checks
        if plugin.is_null() || value.is_null() {
            return CubeMelonPluginErrorCode::NullPointer;
        }

        // Safe dereferencing
        let value_ref = unsafe { &mut *value };

        // Access plugin instance safely through type-safe wrapper
        match crate::instance::with_plugin::<T, _, _>(plugin, |plugin_instance| {
            plugin_instance.get_state_value(scope, key, value_ref)
        }) {
            Some(error_code) => error_code,
            None => CubeMelonPluginErrorCode::PluginNotFound,
        }
    }

    extern "C" fn set_state_value_wrapper<T: CubeMelonPluginStateInterface + 'static>(
        plugin: *mut CubeMelonPlugin,
        scope: CubeMelonPluginStateScope,
        key: *const u8,
        data: *const u8,
        size: usize,
    ) -> CubeMelonPluginErrorCode {
        // NULL pointer check
        if plugin.is_null() {
            return CubeMelonPluginErrorCode::NullPointer;
        }

        // Access plugin instance safely through type-safe wrapper
        match crate::instance::with_plugin_mut::<T, _, _>(plugin, |plugin_instance| {
            plugin_instance.set_state_value(scope, key, data, size)
        }) {
            Some(error_code) => error_code,
            None => CubeMelonPluginErrorCode::PluginNotFound,
        }
    }

    extern "C" fn list_state_keys_wrapper<T: CubeMelonPluginStateInterface + 'static>(
        plugin: *const CubeMelonPlugin,
        scope: CubeMelonPluginStateScope,
        keys: *mut CubeMelonValue,
    ) -> CubeMelonPluginErrorCode {
        // NULL pointer checks
        if plugin.is_null() || keys.is_null() {
            return CubeMelonPluginErrorCode::NullPointer;
        }

        // Safe dereferencing
        let keys_ref = unsafe { &mut *keys };

        // Access plugin instance safely through type-safe wrapper
        match crate::instance::with_plugin::<T, _, _>(plugin, |plugin_instance| {
            plugin_instance.list_state_keys(scope, keys_ref)
        }) {
            Some(error_code) => error_code,
            None => CubeMelonPluginErrorCode::PluginNotFound,
        }
    }

    extern "C" fn clear_state_value_wrapper<T: CubeMelonPluginStateInterface + 'static>(
        plugin: *mut CubeMelonPlugin,
        scope: CubeMelonPluginStateScope,
        key: *const u8,
    ) -> CubeMelonPluginErrorCode {
        // NULL pointer check
        if plugin.is_null() {
            return CubeMelonPluginErrorCode::NullPointer;
        }

        // Access plugin instance safely through type-safe wrapper
        match crate::instance::with_plugin_mut::<T, _, _>(plugin, |plugin_instance| {
            plugin_instance.clear_state_value(scope, key)
        }) {
            Some(error_code) => error_code,
            None => CubeMelonPluginErrorCode::PluginNotFound,
        }
    }

    CubeMelonPluginStateInterfaceImpl {
        load_state: load_state_wrapper::<T>,
        save_state: save_state_wrapper::<T>,
        get_format_name: get_format_name_wrapper::<T>,
        get_state_value: get_state_value_wrapper::<T>,
        set_state_value: set_state_value_wrapper::<T>,
        list_state_keys: list_state_keys_wrapper::<T>,
        clear_state_value: clear_state_value_wrapper::<T>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use std::collections::HashMap;

    struct TestStatePlugin {
        local_data: HashMap<String, Vec<u8>>,
        format: String,
    }

    impl CubeMelonPluginStateInterface for TestStatePlugin {
        fn load_state(
            &self,
            scope: CubeMelonPluginStateScope,
            _data: &mut CubeMelonValue,
        ) -> CubeMelonPluginErrorCode {
            // Mock implementation for testing
            match scope {
                CubeMelonPluginStateScope::Local => {
                    // Return some test data
                    CubeMelonPluginErrorCode::Success
                }
                _ => CubeMelonPluginErrorCode::NotSupported,
            }
        }

        fn save_state(
            &mut self,
            _scope: CubeMelonPluginStateScope,
            data: *const u8,
            size: usize,
        ) -> CubeMelonPluginErrorCode {
            if data.is_null() || size == 0 {
                return CubeMelonPluginErrorCode::InvalidParameter;
            }
            // Mock save operation
            CubeMelonPluginErrorCode::Success
        }

        fn get_format_name(&self, _scope: CubeMelonPluginStateScope) -> *const u8 {
            self.format.as_ptr()
        }

        fn get_state_value(
            &self,
            _scope: CubeMelonPluginStateScope,
            _key: *const u8,
            _value: &mut CubeMelonValue,
        ) -> CubeMelonPluginErrorCode {
            CubeMelonPluginErrorCode::Success
        }

        fn set_state_value(
            &mut self,
            _scope: CubeMelonPluginStateScope,
            _key: *const u8,
            _data: *const u8,
            _size: usize,
        ) -> CubeMelonPluginErrorCode {
            CubeMelonPluginErrorCode::Success
        }

        fn list_state_keys(
            &self,
            _scope: CubeMelonPluginStateScope,
            _keys: &mut CubeMelonValue,
        ) -> CubeMelonPluginErrorCode {
            CubeMelonPluginErrorCode::Success
        }

        fn clear_state_value(
            &mut self,
            _scope: CubeMelonPluginStateScope,
            _key: *const u8,
        ) -> CubeMelonPluginErrorCode {
            CubeMelonPluginErrorCode::Success
        }
    }

    #[test]
    fn test_plugin_state_interface_creation() {
        let interface = create_plugin_state_interface::<TestStatePlugin>();
        
        // Verify function pointers are set (not null)
        let load_state_fn = interface.load_state;
        let save_state_fn = interface.save_state;
        let get_format_name_fn = interface.get_format_name;
        let get_state_value_fn = interface.get_state_value;
        let set_state_value_fn = interface.set_state_value;
        let list_state_keys_fn = interface.list_state_keys;
        let clear_state_value_fn = interface.clear_state_value;
        
        assert!(load_state_fn as usize != 0);
        assert!(save_state_fn as usize != 0);
        assert!(get_format_name_fn as usize != 0);
        assert!(get_state_value_fn as usize != 0);
        assert!(set_state_value_fn as usize != 0);
        assert!(list_state_keys_fn as usize != 0);
        assert!(clear_state_value_fn as usize != 0);
    }

    #[test]
    fn test_state_plugin_creation() {
        // Create test plugin instance
        let plugin = TestStatePlugin {
            local_data: HashMap::new(),
            format: "json\0".to_string(), // Null-terminated for C compatibility
        };
        
        // Basic validation
        assert!(plugin.local_data.is_empty());
        assert_eq!(plugin.format, "json\0");
    }
}