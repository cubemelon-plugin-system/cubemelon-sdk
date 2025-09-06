//! Plugin Manager Interface Implementation
//! 
//! This module implements CubeMelonPluginManagerInterface for RuntimeData.

use cubemelon_sdk::{
    CubeMelonUUID, CubeMelonLanguage, CubeMelonPluginErrorCode, CubeMelonLogLevel,
    CubeMelonPluginBasicInfo, CubeMelonPluginBasicInfoArray, CubeMelonUUIDArray, CubeMelonString,
    CubeMelonTaskRequest, CubeMelonTaskResult, CubeMelonTaskCallback,
    CubeMelonPluginManagerInterface, CubeMelonPluginManagerInterfaceImpl,
    create_plugin_manager_interface,
};

use crate::{RuntimeData, host_services::{runtime_log, HostRuntimeProxy, with_runtime}};

impl RuntimeData {
    /// Create the C ABI interface implementation for plugin manager
    pub fn create_manager_interface() -> CubeMelonPluginManagerInterfaceImpl {
        create_plugin_manager_interface::<Self>()
    }
}

#[allow(unused_variables)]
impl CubeMelonPluginManagerInterface for RuntimeData {
    /// Get basic information for all plugins
    fn get_all_plugins_basic_info(
        &self,
        language: CubeMelonLanguage,
        out_infos: &mut CubeMelonPluginBasicInfoArray,
    ) -> CubeMelonPluginErrorCode {
        runtime_log(CubeMelonLogLevel::Debug, "get_all_plugins_basic_info called");
        // Build array from discovered plugins
        let mut infos: Vec<CubeMelonPluginBasicInfo> = Vec::with_capacity(self.discovered_plugins.len());
        for p in &self.discovered_plugins {
            // Note: name/description are already localized at discovery time.
            // If strict per-call localization is needed, we'd re-query via interface.
            let name = CubeMelonString::from_string(p.name.clone());
            let description = CubeMelonString::from_string(p.description.clone());
            let supported_types = p.supported_types;
            infos.push(CubeMelonPluginBasicInfo::new(
                p.uuid,
                p.version,
                supported_types,
                name,
                description,
            ));
        }
        *out_infos = CubeMelonPluginBasicInfoArray::from_vec(infos);

        runtime_log(
            CubeMelonLogLevel::Info,
            &format!("Returning info for {} plugins", self.discovered_plugins.len()),
        );
        CubeMelonPluginErrorCode::Success
    }

    /// Get detailed information for a single plugin (JSON format)
    fn get_plugin_detailed_info(
        &self,
        target_uuid: CubeMelonUUID,
        language: CubeMelonLanguage,
        out_detailed_json: &mut CubeMelonString,
    ) -> CubeMelonPluginErrorCode {
        runtime_log(CubeMelonLogLevel::Debug, &format!("get_plugin_detailed_info called for UUID: {}", target_uuid));
        
        // Find the plugin by UUID
        if let Some(plugin_info) = self.discovered_plugins.iter().find(|p| p.uuid == target_uuid) {
            // Simple JSON escaper for strings
            fn esc<S: AsRef<str>>(s: S) -> String {
                let mut out = String::with_capacity(s.as_ref().len() + 8);
                for ch in s.as_ref().chars() {
                    match ch {
                        '"' => out.push_str("\\\""),
                        '\\' => out.push_str("\\\\"),
                        '\n' => out.push_str("\\n"),
                        '\r' => out.push_str("\\r"),
                        '\t' => out.push_str("\\t"),
                        c if c.is_control() => out.push(' '),
                        c => out.push(c),
                    }
                }
                out
            }

            let loaded = self.loaded_libraries.contains_key(&plugin_info.uuid);
            let json = format!(
                "{{\n  \"uuid\": \"{}\",\n  \"version\": \"{}\",\n  \"supported_types\": {},\n  \"name\": \"{}\",\n  \"description\": \"{}\",\n  \"loaded\": {},\n  \"path\": \"{}\"\n}}",
                plugin_info.uuid,
                plugin_info.version,
                plugin_info.supported_types,
                esc(&plugin_info.name),
                esc(&plugin_info.description),
                if loaded { "true" } else { "false" },
                esc(&plugin_info.path.display().to_string()),
            );

            *out_detailed_json = CubeMelonString::from_string(json);
            runtime_log(CubeMelonLogLevel::Info, &format!("Found plugin details for: {}", plugin_info.name));
            CubeMelonPluginErrorCode::Success
        } else {
            runtime_log(CubeMelonLogLevel::Warn, &format!("Plugin not found for UUID: {}", target_uuid));
            *out_detailed_json = CubeMelonString::empty();
            CubeMelonPluginErrorCode::PluginNotFound
        }
    }

    /// Find plugins for task
    fn find_plugins_for_task(
        &self,
        task_json: *const u8,
        out_uuids: &mut CubeMelonUUIDArray,
    ) -> CubeMelonPluginErrorCode {
        runtime_log(CubeMelonLogLevel::Debug, "find_plugins_for_task called");
        
        // Convert task_json to string if valid
        if task_json.is_null() {
            runtime_log(CubeMelonLogLevel::Error, "task_json is null");
            return CubeMelonPluginErrorCode::NullPointer;
        }
        
        let task_json_str = unsafe {
            std::ffi::CStr::from_ptr(task_json as *const i8)
                .to_str()
        };
        
        match task_json_str {
            Ok(json_str) => {
                runtime_log(CubeMelonLogLevel::Info, &format!("Searching plugins for task: {}", json_str));
                
                // For now, return all loaded plugins as potential candidates
                // TODO: Implement actual task matching logic
                
                CubeMelonPluginErrorCode::Success
            }
            Err(_) => {
                runtime_log(CubeMelonLogLevel::Error, "Invalid task JSON encoding");
                CubeMelonPluginErrorCode::Encoding
            }
        }
    }

    /// Check plugin liveness
    fn is_plugin_alive(&self, target_uuid: CubeMelonUUID) -> bool {
        let is_alive = self.loaded_libraries.contains_key(&target_uuid);
        runtime_log(CubeMelonLogLevel::Debug, &format!("Plugin {} is_alive: {}", target_uuid, is_alive));
        is_alive
    }

    /// Execute synchronous task
    fn execute_task(
        &mut self,
        target_uuid: CubeMelonUUID,
        request: &CubeMelonTaskRequest,
        result: &mut CubeMelonTaskResult,
    ) -> CubeMelonPluginErrorCode {
        runtime_log(CubeMelonLogLevel::Info, &format!("execute_task called for plugin: {}", target_uuid));

        // Check if plugin is loaded
        let library = match self.loaded_libraries.get(&target_uuid) {
            Some(lib) => lib,
            None => {
                runtime_log(CubeMelonLogLevel::Error, &format!("Plugin not loaded: {}", target_uuid));
                return CubeMelonPluginErrorCode::PluginNotFound;
            }
        };

        // Resolve exported functions
        let get_plugin_interface: libloading::Symbol<unsafe extern "C" fn(u64, u32, *mut *const std::ffi::c_void) -> CubeMelonPluginErrorCode> = unsafe {
            match library.get(b"get_plugin_interface") {
                Ok(sym) => sym,
                Err(_) => return CubeMelonPluginErrorCode::InterfaceNotSupported,
            }
        };
        let create_plugin: libloading::Symbol<unsafe extern "C" fn() -> *mut cubemelon_sdk::instance::CubeMelonPlugin> = unsafe {
            match library.get(b"create_plugin") {
                Ok(sym) => sym,
                Err(_) => return CubeMelonPluginErrorCode::PluginLoadFailed,
            }
        };
        let destroy_plugin: libloading::Symbol<unsafe extern "C" fn(*mut cubemelon_sdk::instance::CubeMelonPlugin)> = unsafe {
            match library.get(b"destroy_plugin") {
                Ok(sym) => sym,
                Err(_) => return CubeMelonPluginErrorCode::PluginUnloadFailed,
            }
        };

        // Get basic interface (for initialize/uninitialize)
        let mut basic_ptr: *const std::ffi::c_void = std::ptr::null();
        let rc = unsafe {
            get_plugin_interface(cubemelon_sdk::types::CubeMelonPluginType::Basic as u64, 1, &mut basic_ptr as *mut *const std::ffi::c_void)
        };
        if rc != CubeMelonPluginErrorCode::Success || basic_ptr.is_null() {
            return rc;
        }
        let basic = unsafe { &*(basic_ptr as *const cubemelon_sdk::interfaces::CubeMelonInterface) };

        // Get single_task interface
        let mut st_ptr: *const std::ffi::c_void = std::ptr::null();
        let rc = unsafe {
            get_plugin_interface(cubemelon_sdk::types::CubeMelonPluginType::SingleTask as u64, 1, &mut st_ptr as *mut *const std::ffi::c_void)
        };
        if rc != CubeMelonPluginErrorCode::Success || st_ptr.is_null() {
            return rc;
        }
        let single_task = unsafe { &*(st_ptr as *const cubemelon_sdk::interfaces::CubeMelonSingleTaskInterfaceImpl) };

        // Create and initialize instance
        let instance = unsafe { create_plugin() };
        if instance.is_null() {
            return CubeMelonPluginErrorCode::PluginLoadFailed;
        }

        let init_rc = (basic.initialize)(instance, &self.host_services as *const _);
        if init_rc != CubeMelonPluginErrorCode::Success {
            unsafe { destroy_plugin(instance) };
            return init_rc;
        }

        // Execute the task
        let exec_rc = (single_task.execute)(instance, request as *const _, result as *mut _);

        // Uninitialize and destroy instance
        let _ = (basic.uninitialize)(instance);
        unsafe { destroy_plugin(instance) };

        exec_rc
    }

    /// Execute asynchronous task
    fn execute_async_task(
        &mut self,
        target_uuid: CubeMelonUUID,
        request: &CubeMelonTaskRequest,
        callback: Option<CubeMelonTaskCallback>,
    ) -> CubeMelonPluginErrorCode {
        runtime_log(CubeMelonLogLevel::Info, &format!("execute_async_task called for plugin: {}", target_uuid));
        
        // Check if plugin is loaded
        if !self.loaded_libraries.contains_key(&target_uuid) {
            runtime_log(CubeMelonLogLevel::Error, &format!("Plugin not loaded: {}", target_uuid));
            return CubeMelonPluginErrorCode::PluginNotFound;
        }
        
        // TODO: Implement actual async task execution
        runtime_log(CubeMelonLogLevel::Warn, "Async task execution not fully implemented yet");
        CubeMelonPluginErrorCode::NotSupported
    }

    /// Cancel asynchronous task
    fn cancel_async_task(
        &mut self,
        request: &mut CubeMelonTaskRequest,
    ) -> CubeMelonPluginErrorCode {
        runtime_log(CubeMelonLogLevel::Info, "cancel_async_task called");
        
        // TODO: Implement task cancellation logic
        runtime_log(CubeMelonLogLevel::Warn, "Task cancellation not fully implemented yet");
        CubeMelonPluginErrorCode::NotSupported
    }
}

// Host proxy delegates to the active RuntimeData instance
#[allow(unused_variables)]
impl CubeMelonPluginManagerInterface for HostRuntimeProxy {
    fn get_all_plugins_basic_info(
        &self,
        language: cubemelon_sdk::CubeMelonLanguage,
        out_infos: &mut cubemelon_sdk::CubeMelonPluginBasicInfoArray,
    ) -> CubeMelonPluginErrorCode {
        if let Some(rt) = with_runtime(|r| r as *const RuntimeData) {
            // SAFETY: rt came from &RuntimeData above
            let r = unsafe { &*rt };
            cubemelon_sdk::CubeMelonPluginManagerInterface::get_all_plugins_basic_info(
                r, language, out_infos,
            )
        } else {
            CubeMelonPluginErrorCode::NotInitialized
        }
    }

    fn get_plugin_detailed_info(
        &self,
        target_uuid: cubemelon_sdk::CubeMelonUUID,
        language: cubemelon_sdk::CubeMelonLanguage,
        out_detailed_json: &mut cubemelon_sdk::CubeMelonString,
    ) -> CubeMelonPluginErrorCode {
        if let Some(rt) = with_runtime(|r| r as *const RuntimeData) {
            let r = unsafe { &*rt };
            cubemelon_sdk::CubeMelonPluginManagerInterface::get_plugin_detailed_info(
                r, target_uuid, language, out_detailed_json,
            )
        } else {
            CubeMelonPluginErrorCode::NotInitialized
        }
    }

    fn find_plugins_for_task(
        &self,
        task_json: *const u8,
        out_uuids: &mut cubemelon_sdk::CubeMelonUUIDArray,
    ) -> CubeMelonPluginErrorCode {
        if let Some(rt) = with_runtime(|r| r as *const RuntimeData) {
            let r = unsafe { &*rt };
            cubemelon_sdk::CubeMelonPluginManagerInterface::find_plugins_for_task(
                r, task_json, out_uuids,
            )
        } else {
            CubeMelonPluginErrorCode::NotInitialized
        }
    }

    fn is_plugin_alive(&self, target_uuid: cubemelon_sdk::CubeMelonUUID) -> bool {
        if let Some(rt) = with_runtime(|r| r as *const RuntimeData) {
            let r = unsafe { &*rt };
            cubemelon_sdk::CubeMelonPluginManagerInterface::is_plugin_alive(r, target_uuid)
        } else {
            false
        }
    }

    fn execute_task(
        &mut self,
        target_uuid: cubemelon_sdk::CubeMelonUUID,
        request: &cubemelon_sdk::CubeMelonTaskRequest,
        result: &mut cubemelon_sdk::CubeMelonTaskResult,
    ) -> CubeMelonPluginErrorCode {
        if let Some(code) = crate::host_services::with_runtime_mut(|r| {
            cubemelon_sdk::CubeMelonPluginManagerInterface::execute_task(
                r, target_uuid, request, result,
            )
        }) {
            code
        } else {
            CubeMelonPluginErrorCode::NotInitialized
        }
    }

    fn execute_async_task(
        &mut self,
        target_uuid: cubemelon_sdk::CubeMelonUUID,
        request: &cubemelon_sdk::CubeMelonTaskRequest,
        callback: Option<cubemelon_sdk::CubeMelonTaskCallback>,
    ) -> CubeMelonPluginErrorCode {
        if let Some(code) = crate::host_services::with_runtime_mut(|r| {
            cubemelon_sdk::CubeMelonPluginManagerInterface::execute_async_task(
                r, target_uuid, request, callback,
            )
        }) {
            code
        } else {
            CubeMelonPluginErrorCode::NotInitialized
        }
    }

    fn cancel_async_task(
        &mut self,
        request: &mut cubemelon_sdk::CubeMelonTaskRequest,
    ) -> CubeMelonPluginErrorCode {
        if let Some(code) = crate::host_services::with_runtime_mut(|r| {
            cubemelon_sdk::CubeMelonPluginManagerInterface::cancel_async_task(r, request)
        }) {
            code
        } else {
            CubeMelonPluginErrorCode::NotInitialized
        }
    }
}
