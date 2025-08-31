use crate::error::CubeMelonPluginErrorCode;
use crate::structs::{
    CubeMelonTaskRequest, CubeMelonTaskResult, CubeMelonTaskCallback
};
use crate::memory::{
    CubeMelonPluginBasicInfoArray, CubeMelonUUIDArray, CubeMelonString
};
use crate::types::{CubeMelonUUID, CubeMelonLanguage};
use crate::instance::CubeMelonPlugin;

/// Plugin Manager Interface - Plugin management and inter-plugin communication
/// 
/// Manages loaded plugins and mediates task execution across plugins.
/// 
/// # Specification Notes
/// - Plugin must check if callback function is NULL. If NULL, results are not notified.
/// - For `execute_task()`, both `CubeMelonTaskRequest` and `CubeMelonTaskResult` objects 
///   are created and destroyed by the caller.
/// - For `execute_async_task()`, `CubeMelonTaskRequest` objects are created by the caller 
///   and destroyed within the callback function. `CubeMelonTaskResult` objects are created 
///   by the plugin and destroyed by the plugin after the callback function returns.
/// - For `cancel_async_task()`, the caller destroys the `CubeMelonTaskRequest` object after 
///   the operation completes, but care must be taken to avoid double-freeing within callback functions.
/// - If the `CubeMelonTaskRequest` object has already been destroyed before calling 
///   `cancel_async_task()`, the operation is ignored.
/// 
/// # Implementation Example
/// ```rust
/// use cubemelon_sdk::prelude::*;
/// 
/// struct MyPlugin {
///     plugins: Vec<PluginInfo>,
/// }
/// 
/// impl CubeMelonPluginManagerInterface for MyPlugin {
///     fn get_all_plugins_basic_info(
///         &self,
///         language: CubeMelonLanguage,
///         out_infos: &mut CubeMelonPluginBasicInfoArray,
///     ) -> CubeMelonPluginErrorCode {
///         // Return info for all loaded plugins
///         CubeMelonPluginErrorCode::Success
///     }
///     
///     // ... other methods
/// }
/// ```
pub trait CubeMelonPluginManagerInterface {
    /// Get basic information for all plugins (for UI)
    /// 
    /// # Arguments
    /// * `language` - Language for localized strings
    /// * `out_infos` - Array to store plugin info. Caller must free using out_infos.free_array()
    /// 
    /// # Returns
    /// * `CubeMelonPluginErrorCode` - Error code for operation result
    fn get_all_plugins_basic_info(
        &self,
        language: CubeMelonLanguage,
        out_infos: &mut CubeMelonPluginBasicInfoArray,
    ) -> CubeMelonPluginErrorCode;

    /// Get detailed information for a single plugin (JSON format, includes extended info)
    /// 
    /// # Arguments
    /// * `target_uuid` - UUID of target plugin
    /// * `language` - Language for localized strings
    /// * `out_detailed_json` - String to store JSON data. Caller must free using out_detailed_json.free_string()
    /// 
    /// # Returns
    /// * `CubeMelonPluginErrorCode` - Error code for operation result
    fn get_plugin_detailed_info(
        &self,
        target_uuid: CubeMelonUUID,
        language: CubeMelonLanguage,
        out_detailed_json: &mut CubeMelonString,
    ) -> CubeMelonPluginErrorCode;

    /// Find plugins for task (search across hierarchy)
    /// 
    /// # Arguments
    /// * `task_json` - Task description in JSON format
    /// * `out_uuids` - Array to store matching plugin UUIDs. Caller must free using out_uuids.free_array()
    /// 
    /// # Returns
    /// * `CubeMelonPluginErrorCode` - Error code for operation result
    fn find_plugins_for_task(
        &self,
        task_json: *const u8,
        out_uuids: &mut CubeMelonUUIDArray,
    ) -> CubeMelonPluginErrorCode;

    /// Check plugin liveness
    /// 
    /// # Arguments
    /// * `target_uuid` - UUID of plugin to check
    /// 
    /// # Returns
    /// * `bool` - true if plugin is alive, false otherwise
    fn is_plugin_alive(&self, target_uuid: CubeMelonUUID) -> bool;

    /// Execute synchronous task
    /// 
    /// # Arguments
    /// * `target_uuid` - UUID of target plugin
    /// * `request` - Task request data
    /// * `result` - Task result data (caller managed)
    /// 
    /// # Returns
    /// * `CubeMelonPluginErrorCode` - Error code for operation result
    fn execute_task(
        &mut self,
        target_uuid: CubeMelonUUID,
        request: &CubeMelonTaskRequest,
        result: &mut CubeMelonTaskResult,
    ) -> CubeMelonPluginErrorCode;

    /// Execute asynchronous task
    /// 
    /// # Arguments
    /// * `target_uuid` - UUID of target plugin
    /// * `request` - Task request data
    /// * `callback` - Callback function for results (may be null)
    /// 
    /// # Returns
    /// * `CubeMelonPluginErrorCode` - Error code for execution start result
    fn execute_async_task(
        &mut self,
        target_uuid: CubeMelonUUID,
        request: &CubeMelonTaskRequest,
        callback: Option<CubeMelonTaskCallback>,
    ) -> CubeMelonPluginErrorCode;

    /// Cancel asynchronous task
    /// 
    /// # Arguments
    /// * `request` - Task request to cancel
    /// 
    /// # Returns
    /// * `CubeMelonPluginErrorCode` - Error code for cancellation result
    fn cancel_async_task(
        &mut self,
        request: &mut CubeMelonTaskRequest,
    ) -> CubeMelonPluginErrorCode;
}

/// C ABI PluginManagerInterface structure
/// 
/// The actual interface structure returned by `get_interface()` from plugins
#[repr(C)]
pub struct CubeMelonPluginManagerInterfaceImpl {
    /// Get basic information for all plugins (for UI)
    /// Caller must free out_infos using out_infos.free_array()
    pub get_all_plugins_basic_info: extern "C" fn(
        plugin: *const CubeMelonPlugin,
        language: CubeMelonLanguage,
        out_infos: *mut CubeMelonPluginBasicInfoArray,
    ) -> CubeMelonPluginErrorCode,

    /// Get detailed information for single plugin (JSON format, includes extended info)
    /// Caller must free out_detailed_json using out_detailed_json.free_string()
    pub get_plugin_detailed_info: extern "C" fn(
        plugin: *const CubeMelonPlugin,
        target_uuid: CubeMelonUUID,
        language: CubeMelonLanguage,
        out_detailed_json: *mut CubeMelonString,
    ) -> CubeMelonPluginErrorCode,

    /// Find plugins for task (search across hierarchy)
    /// Caller must free out_uuids using out_uuids.free_array()
    pub find_plugins_for_task: extern "C" fn(
        plugin: *const CubeMelonPlugin,
        task_json: *const u8,
        out_uuids: *mut CubeMelonUUIDArray,
    ) -> CubeMelonPluginErrorCode,

    /// Check plugin liveness
    pub is_plugin_alive: extern "C" fn(
        plugin: *const CubeMelonPlugin,
        target_uuid: CubeMelonUUID,
    ) -> bool,

    /// Execute synchronous task
    pub execute_task: extern "C" fn(
        plugin: *mut CubeMelonPlugin,
        target_uuid: CubeMelonUUID,
        request: *const CubeMelonTaskRequest,
        result: *mut CubeMelonTaskResult,
    ) -> CubeMelonPluginErrorCode,

    /// Execute asynchronous task
    pub execute_async_task: extern "C" fn(
        plugin: *mut CubeMelonPlugin,
        target_uuid: CubeMelonUUID,
        request: *const CubeMelonTaskRequest,
        callback: CubeMelonTaskCallback,
    ) -> CubeMelonPluginErrorCode,

    /// Cancel asynchronous task
    pub cancel_async_task: extern "C" fn(
        plugin: *mut CubeMelonPlugin,
        request: *mut CubeMelonTaskRequest,
    ) -> CubeMelonPluginErrorCode,
}

/// Helper function to generate C ABI interface from CubeMelonPluginManagerInterface trait implementation
/// 
/// # Usage Example
/// ```rust
/// use cubemelon_sdk::prelude::*;
///
/// #[plugin]
/// struct MyPlugin;
///
/// let interface = create_plugin_manager_interface::<MyPlugin>();
/// ```
pub fn create_plugin_manager_interface<T>() -> CubeMelonPluginManagerInterfaceImpl
where
    T: CubeMelonPluginManagerInterface + 'static,
{
    extern "C" fn get_all_plugins_basic_info_wrapper<T: CubeMelonPluginManagerInterface + 'static>(
        plugin: *const CubeMelonPlugin,
        language: CubeMelonLanguage,
        out_infos: *mut CubeMelonPluginBasicInfoArray,
    ) -> CubeMelonPluginErrorCode {
        // NULL pointer checks
        if plugin.is_null() || out_infos.is_null() {
            return CubeMelonPluginErrorCode::NullPointer;
        }

        // Safe dereferencing
        let out_infos_ref = unsafe { &mut *out_infos };

        // Access plugin instance safely through type-safe wrapper
        match crate::instance::with_plugin::<T, _, _>(plugin, |plugin_instance| {
            plugin_instance.get_all_plugins_basic_info(language, out_infos_ref)
        }) {
            Some(error_code) => error_code,
            None => CubeMelonPluginErrorCode::PluginNotFound,
        }
    }

    extern "C" fn get_plugin_detailed_info_wrapper<T: CubeMelonPluginManagerInterface + 'static>(
        plugin: *const CubeMelonPlugin,
        target_uuid: CubeMelonUUID,
        language: CubeMelonLanguage,
        out_detailed_json: *mut CubeMelonString,
    ) -> CubeMelonPluginErrorCode {
        // NULL pointer checks
        if plugin.is_null() || out_detailed_json.is_null() {
            return CubeMelonPluginErrorCode::NullPointer;
        }

        // Safe dereferencing
        let out_detailed_json_ref = unsafe { &mut *out_detailed_json };

        // Access plugin instance safely through type-safe wrapper
        match crate::instance::with_plugin::<T, _, _>(plugin, |plugin_instance| {
            plugin_instance.get_plugin_detailed_info(target_uuid, language, out_detailed_json_ref)
        }) {
            Some(error_code) => error_code,
            None => CubeMelonPluginErrorCode::PluginNotFound,
        }
    }

    extern "C" fn find_plugins_for_task_wrapper<T: CubeMelonPluginManagerInterface + 'static>(
        plugin: *const CubeMelonPlugin,
        task_json: *const u8,
        out_uuids: *mut CubeMelonUUIDArray,
    ) -> CubeMelonPluginErrorCode {
        // NULL pointer checks
        if plugin.is_null() || out_uuids.is_null() {
            return CubeMelonPluginErrorCode::NullPointer;
        }

        // Safe dereferencing
        let out_uuids_ref = unsafe { &mut *out_uuids };

        // Access plugin instance safely through type-safe wrapper
        match crate::instance::with_plugin::<T, _, _>(plugin, |plugin_instance| {
            plugin_instance.find_plugins_for_task(task_json, out_uuids_ref)
        }) {
            Some(error_code) => error_code,
            None => CubeMelonPluginErrorCode::PluginNotFound,
        }
    }

    extern "C" fn is_plugin_alive_wrapper<T: CubeMelonPluginManagerInterface + 'static>(
        plugin: *const CubeMelonPlugin,
        target_uuid: CubeMelonUUID,
    ) -> bool {
        // NULL pointer check
        if plugin.is_null() {
            return false;
        }

        // Access plugin instance safely through type-safe wrapper
        match crate::instance::with_plugin::<T, _, _>(plugin, |plugin_instance| {
            plugin_instance.is_plugin_alive(target_uuid)
        }) {
            Some(is_alive) => is_alive,
            None => false, // Plugin not found = not alive
        }
    }

    extern "C" fn execute_task_wrapper<T: CubeMelonPluginManagerInterface + 'static>(
        plugin: *mut CubeMelonPlugin,
        target_uuid: CubeMelonUUID,
        request: *const CubeMelonTaskRequest,
        result: *mut CubeMelonTaskResult,
    ) -> CubeMelonPluginErrorCode {
        // NULL pointer checks
        if plugin.is_null() || request.is_null() || result.is_null() {
            return CubeMelonPluginErrorCode::NullPointer;
        }

        // Safe dereferencing
        let request_ref = unsafe { &*request };
        let result_ref = unsafe { &mut *result };

        // Access plugin instance safely through type-safe wrapper
        match crate::instance::with_plugin_mut::<T, _, _>(plugin, |plugin_instance| {
            plugin_instance.execute_task(target_uuid, request_ref, result_ref)
        }) {
            Some(error_code) => error_code,
            None => CubeMelonPluginErrorCode::PluginNotFound,
        }
    }

    extern "C" fn execute_async_task_wrapper<T: CubeMelonPluginManagerInterface + 'static>(
        plugin: *mut CubeMelonPlugin,
        target_uuid: CubeMelonUUID,
        request: *const CubeMelonTaskRequest,
        callback: CubeMelonTaskCallback,
    ) -> CubeMelonPluginErrorCode {
        // NULL pointer checks
        if plugin.is_null() || request.is_null() {
            return CubeMelonPluginErrorCode::NullPointer;
        }

        // Safe dereferencing
        let request_ref = unsafe { &*request };
        
        // Convert callback function pointer to Option
        let callback_opt = if callback as usize != 0 {
            Some(callback)
        } else {
            None
        };

        // Access plugin instance safely through type-safe wrapper
        match crate::instance::with_plugin_mut::<T, _, _>(plugin, |plugin_instance| {
            plugin_instance.execute_async_task(target_uuid, request_ref, callback_opt)
        }) {
            Some(error_code) => error_code,
            None => CubeMelonPluginErrorCode::PluginNotFound,
        }
    }

    extern "C" fn cancel_async_task_wrapper<T: CubeMelonPluginManagerInterface + 'static>(
        plugin: *mut CubeMelonPlugin,
        request: *mut CubeMelonTaskRequest,
    ) -> CubeMelonPluginErrorCode {
        // NULL pointer checks
        if plugin.is_null() || request.is_null() {
            return CubeMelonPluginErrorCode::NullPointer;
        }

        // Safe dereferencing
        let request_ref = unsafe { &mut *request };

        // Access plugin instance safely through type-safe wrapper
        match crate::instance::with_plugin_mut::<T, _, _>(plugin, |plugin_instance| {
            plugin_instance.cancel_async_task(request_ref)
        }) {
            Some(error_code) => error_code,
            None => CubeMelonPluginErrorCode::PluginNotFound,
        }
    }

    CubeMelonPluginManagerInterfaceImpl {
        get_all_plugins_basic_info: get_all_plugins_basic_info_wrapper::<T>,
        get_plugin_detailed_info: get_plugin_detailed_info_wrapper::<T>,
        find_plugins_for_task: find_plugins_for_task_wrapper::<T>,
        is_plugin_alive: is_plugin_alive_wrapper::<T>,
        execute_task: execute_task_wrapper::<T>,
        execute_async_task: execute_async_task_wrapper::<T>,
        cancel_async_task: cancel_async_task_wrapper::<T>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use crate::structs::*;
    use std::collections::HashMap;

    struct TestManagerPlugin {
        plugins: HashMap<CubeMelonUUID, String>,
        task_count: usize,
    }

    impl CubeMelonPluginManagerInterface for TestManagerPlugin {
        fn get_all_plugins_basic_info(
            &self,
            _language: CubeMelonLanguage,
            _out_infos: &mut CubeMelonPluginBasicInfoArray,
        ) -> CubeMelonPluginErrorCode {
            // Mock implementation for testing
            CubeMelonPluginErrorCode::Success
        }

        fn get_plugin_detailed_info(
            &self,
            _target_uuid: CubeMelonUUID,
            _language: CubeMelonLanguage,
            _out_detailed_json: &mut CubeMelonString,
        ) -> CubeMelonPluginErrorCode {
            CubeMelonPluginErrorCode::Success
        }

        fn find_plugins_for_task(
            &self,
            _task_json: *const u8,
            _out_uuids: &mut CubeMelonUUIDArray,
        ) -> CubeMelonPluginErrorCode {
            CubeMelonPluginErrorCode::Success
        }

        fn is_plugin_alive(&self, target_uuid: CubeMelonUUID) -> bool {
            self.plugins.contains_key(&target_uuid)
        }

        fn execute_task(
            &mut self,
            _target_uuid: CubeMelonUUID,
            _request: &CubeMelonTaskRequest,
            _result: &mut CubeMelonTaskResult,
        ) -> CubeMelonPluginErrorCode {
            self.task_count += 1;
            CubeMelonPluginErrorCode::Success
        }

        fn execute_async_task(
            &mut self,
            _target_uuid: CubeMelonUUID,
            _request: &CubeMelonTaskRequest,
            _callback: Option<CubeMelonTaskCallback>,
        ) -> CubeMelonPluginErrorCode {
            self.task_count += 1;
            CubeMelonPluginErrorCode::Success
        }

        fn cancel_async_task(
            &mut self,
            _request: &mut CubeMelonTaskRequest,
        ) -> CubeMelonPluginErrorCode {
            CubeMelonPluginErrorCode::Success
        }
    }

    #[test]
    fn test_plugin_manager_interface_creation() {
        let interface = create_plugin_manager_interface::<TestManagerPlugin>();
        
        // Verify function pointers are set (not null)
        let get_all_plugins_fn = interface.get_all_plugins_basic_info;
        let get_detailed_info_fn = interface.get_plugin_detailed_info;
        let find_plugins_fn = interface.find_plugins_for_task;
        let is_alive_fn = interface.is_plugin_alive;
        let execute_task_fn = interface.execute_task;
        let execute_async_fn = interface.execute_async_task;
        let cancel_async_fn = interface.cancel_async_task;
        
        assert!(get_all_plugins_fn as usize != 0);
        assert!(get_detailed_info_fn as usize != 0);
        assert!(find_plugins_fn as usize != 0);
        assert!(is_alive_fn as usize != 0);
        assert!(execute_task_fn as usize != 0);
        assert!(execute_async_fn as usize != 0);
        assert!(cancel_async_fn as usize != 0);
    }

    #[test]
    fn test_manager_plugin_creation() {
        // Create test plugin instance
        let plugin = TestManagerPlugin {
            plugins: HashMap::new(),
            task_count: 0,
        };
        
        // Basic validation
        assert!(plugin.plugins.is_empty());
        assert_eq!(plugin.task_count, 0);
    }

    #[test]
    fn test_plugin_liveness_check() {
        let mut plugin = TestManagerPlugin {
            plugins: HashMap::new(),
            task_count: 0,
        };

        // Create a test UUID
        let test_uuid = CubeMelonUUID {
            bytes: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        };

        // Plugin should not be alive initially
        assert!(!plugin.is_plugin_alive(test_uuid));

        // Add plugin to registry
        plugin.plugins.insert(test_uuid, "TestPlugin".to_string());

        // Plugin should now be alive
        assert!(plugin.is_plugin_alive(test_uuid));
    }
}