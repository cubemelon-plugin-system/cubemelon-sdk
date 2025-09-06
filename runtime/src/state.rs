//! Plugin State Interface Implementation
//! 
//! This module implements CubeMelonPluginStateInterface for RuntimeData.

use cubemelon_sdk::{
    CubeMelonPluginErrorCode, CubeMelonPluginStateScope, CubeMelonValue, CubeMelonLogLevel,
    CubeMelonPluginStateInterface, CubeMelonPluginStateInterfaceImpl,
    create_plugin_state_interface,
};

use crate::{RuntimeData, RuntimeConfig, host_services::{runtime_log, HostRuntimeProxy, with_runtime}};

impl RuntimeData {
    /// Create the C ABI interface implementation for state management
    pub fn create_state_interface() -> CubeMelonPluginStateInterfaceImpl {
        create_plugin_state_interface::<Self>()
    }
}

impl CubeMelonPluginStateInterface for RuntimeData {
    /// Load state data
    fn load_state(
        &self,
        scope: CubeMelonPluginStateScope,
        _data: &mut CubeMelonValue,
    ) -> CubeMelonPluginErrorCode {
        runtime_log(CubeMelonLogLevel::Debug, &format!("load_state called with scope: {:?}", scope));
        
        match scope {
            CubeMelonPluginStateScope::Host => {
                // Return host configuration as TOML string
                match toml::to_string(&self.config) {
                    Ok(_toml_content) => {
                        // TODO: Set the TOML content to data
                        // For now, just log success
                        runtime_log(CubeMelonLogLevel::Info, "Host configuration loaded successfully");
                        CubeMelonPluginErrorCode::Success
                    }
                    Err(e) => {
                        runtime_log(CubeMelonLogLevel::Error, &format!("Failed to serialize config: {}", e));
                        CubeMelonPluginErrorCode::Parse
                    }
                }
            }
            CubeMelonPluginStateScope::Local | CubeMelonPluginStateScope::Shared => {
                runtime_log(CubeMelonLogLevel::Warn, "Local and Shared scopes not supported by runtime state manager");
                CubeMelonPluginErrorCode::NotSupported
            }
        }
    }

    /// Save state data
    fn save_state(
        &mut self,
        scope: CubeMelonPluginStateScope,
        data: *const u8,
        size: usize,
    ) -> CubeMelonPluginErrorCode {
        runtime_log(CubeMelonLogLevel::Debug, &format!("save_state called with scope: {:?}, size: {}", scope, size));
        
        if data.is_null() || size == 0 {
            runtime_log(CubeMelonLogLevel::Error, "Invalid data or size for save_state");
            return CubeMelonPluginErrorCode::InvalidParameter;
        }
        
        match scope {
            CubeMelonPluginStateScope::Host => {
                // Convert raw data to string
                let data_slice = unsafe { std::slice::from_raw_parts(data, size) };
                let config_str = match std::str::from_utf8(data_slice) {
                    Ok(s) => s,
                    Err(e) => {
                        runtime_log(CubeMelonLogLevel::Error, &format!("Invalid UTF-8 in config data: {}", e));
                        return CubeMelonPluginErrorCode::Encoding;
                    }
                };
                
                // Parse as TOML and update configuration
                match toml::from_str::<RuntimeConfig>(config_str) {
                    Ok(new_config) => {
                        self.config = new_config;
                        match Self::save_config(&self.config_path, &self.config) {
                            Ok(()) => {
                                runtime_log(CubeMelonLogLevel::Info, "Host configuration saved successfully");
                                CubeMelonPluginErrorCode::Success
                            }
                            Err(e) => {
                                runtime_log(CubeMelonLogLevel::Error, &format!("Failed to save config: {}", e));
                                CubeMelonPluginErrorCode::IO
                            }
                        }
                    }
                    Err(e) => {
                        runtime_log(CubeMelonLogLevel::Error, &format!("Failed to parse config TOML: {}", e));
                        CubeMelonPluginErrorCode::Parse
                    }
                }
            }
            CubeMelonPluginStateScope::Local | CubeMelonPluginStateScope::Shared => {
                runtime_log(CubeMelonLogLevel::Warn, "Local and Shared scopes not supported by runtime state manager");
                CubeMelonPluginErrorCode::NotSupported
            }
        }
    }

    /// Get format name of state data
    fn get_format_name(&self, scope: CubeMelonPluginStateScope) -> *const u8 {
        match scope {
            CubeMelonPluginStateScope::Host => {
                // Return "toml" as null-terminated string
                b"toml\0".as_ptr()
            }
            CubeMelonPluginStateScope::Local | CubeMelonPluginStateScope::Shared => {
                std::ptr::null()
            }
        }
    }

    /// Get state value for specific key
    fn get_state_value(
        &self,
        scope: CubeMelonPluginStateScope,
        key: *const u8,
        _value: &mut CubeMelonValue,
    ) -> CubeMelonPluginErrorCode {
        runtime_log(CubeMelonLogLevel::Debug, &format!("get_state_value called with scope: {:?}", scope));
        
        if key.is_null() {
            return CubeMelonPluginErrorCode::NullPointer;
        }
        
        let key_str = match unsafe { std::ffi::CStr::from_ptr(key as *const i8) }.to_str() {
            Ok(s) => s,
            Err(_) => {
                runtime_log(CubeMelonLogLevel::Error, "Invalid key encoding");
                return CubeMelonPluginErrorCode::Encoding;
            }
        };
        
        match scope {
            CubeMelonPluginStateScope::Host => {
                runtime_log(CubeMelonLogLevel::Info, &format!("Getting host state value for key: {}", key_str));
                
                match key_str {
                    "plugins_directory" => {
                        // TODO: Set the plugins_directory value to CubeMelonValue
                        runtime_log(CubeMelonLogLevel::Info, "Returned plugins_directory value");
                        CubeMelonPluginErrorCode::Success
                    }
                    "language" => {
                        // TODO: Set the language value to CubeMelonValue
                        runtime_log(CubeMelonLogLevel::Info, "Returned language value");
                        CubeMelonPluginErrorCode::Success
                    }
                    _ => {
                        runtime_log(CubeMelonLogLevel::Warn, &format!("Unknown host state key: {}", key_str));
                        CubeMelonPluginErrorCode::PluginNotFound
                    }
                }
            }
            CubeMelonPluginStateScope::Local | CubeMelonPluginStateScope::Shared => {
                CubeMelonPluginErrorCode::NotSupported
            }
        }
    }

    /// Set state value for specific key
    fn set_state_value(
        &mut self,
        scope: CubeMelonPluginStateScope,
        key: *const u8,
        data: *const u8,
        size: usize,
    ) -> CubeMelonPluginErrorCode {
        runtime_log(CubeMelonLogLevel::Debug, &format!("set_state_value called with scope: {:?}, size: {}", scope, size));
        
        if key.is_null() || data.is_null() || size == 0 {
            return CubeMelonPluginErrorCode::InvalidParameter;
        }
        
        let key_str = match unsafe { std::ffi::CStr::from_ptr(key as *const i8) }.to_str() {
            Ok(s) => s,
            Err(_) => {
                runtime_log(CubeMelonLogLevel::Error, "Invalid key encoding");
                return CubeMelonPluginErrorCode::Encoding;
            }
        };
        
        let data_slice = unsafe { std::slice::from_raw_parts(data, size) };
        let value_str = match std::str::from_utf8(data_slice) {
            Ok(s) => s,
            Err(_) => {
                runtime_log(CubeMelonLogLevel::Error, "Invalid value encoding");
                return CubeMelonPluginErrorCode::Encoding;
            }
        };
        
        match scope {
            CubeMelonPluginStateScope::Host => {
                runtime_log(CubeMelonLogLevel::Info, &format!("Setting host state value for key: {} = {}", key_str, value_str));
                
                let result = match key_str {
                    "plugins_directory" => {
                        self.set_plugins_directory(value_str.to_string())
                    }
                    "language" => {
                        self.set_language(value_str.to_string())
                    }
                    _ => {
                        runtime_log(CubeMelonLogLevel::Warn, &format!("Unknown host state key: {}", key_str));
                        return CubeMelonPluginErrorCode::PluginNotFound;
                    }
                };
                
                match result {
                    Ok(()) => CubeMelonPluginErrorCode::Success,
                    Err(e) => {
                        runtime_log(CubeMelonLogLevel::Error, &format!("Failed to set state value: {}", e));
                        CubeMelonPluginErrorCode::IO
                    }
                }
            }
            CubeMelonPluginStateScope::Local | CubeMelonPluginStateScope::Shared => {
                CubeMelonPluginErrorCode::NotSupported
            }
        }
    }

    /// List all state keys
    fn list_state_keys(
        &self,
        scope: CubeMelonPluginStateScope,
        _keys: &mut CubeMelonValue,
    ) -> CubeMelonPluginErrorCode {
        runtime_log(CubeMelonLogLevel::Debug, &format!("list_state_keys called with scope: {:?}", scope));
        
        match scope {
            CubeMelonPluginStateScope::Host => {
                // TODO: Set the list of available keys to CubeMelonValue
                // Available keys: "plugins_directory", "language"
                runtime_log(CubeMelonLogLevel::Info, "Listed host state keys");
                CubeMelonPluginErrorCode::Success
            }
            CubeMelonPluginStateScope::Local | CubeMelonPluginStateScope::Shared => {
                CubeMelonPluginErrorCode::NotSupported
            }
        }
    }

    /// Clear state value for specific key
    fn clear_state_value(
        &mut self,
        scope: CubeMelonPluginStateScope,
        key: *const u8,
    ) -> CubeMelonPluginErrorCode {
        runtime_log(CubeMelonLogLevel::Debug, &format!("clear_state_value called with scope: {:?}", scope));
        
        if key.is_null() {
            return CubeMelonPluginErrorCode::NullPointer;
        }
        
        let key_str = match unsafe { std::ffi::CStr::from_ptr(key as *const i8) }.to_str() {
            Ok(s) => s,
            Err(_) => {
                runtime_log(CubeMelonLogLevel::Error, "Invalid key encoding");
                return CubeMelonPluginErrorCode::Encoding;
            }
        };
        
        match scope {
            CubeMelonPluginStateScope::Host => {
                runtime_log(CubeMelonLogLevel::Info, &format!("Clearing host state value for key: {}", key_str));
                
                let result = match key_str {
                    "plugins_directory" => {
                        self.set_plugins_directory("plugins".to_string()) // Reset to default
                    }
                    "language" => {
                        self.set_language("auto".to_string()) // Reset to default
                    }
                    _ => {
                        runtime_log(CubeMelonLogLevel::Warn, &format!("Unknown host state key: {}", key_str));
                        return CubeMelonPluginErrorCode::PluginNotFound;
                    }
                };
                
                match result {
                    Ok(()) => CubeMelonPluginErrorCode::Success,
                    Err(e) => {
                        runtime_log(CubeMelonLogLevel::Error, &format!("Failed to clear state value: {}", e));
                        CubeMelonPluginErrorCode::IO
                    }
                }
            }
            CubeMelonPluginStateScope::Local | CubeMelonPluginStateScope::Shared => {
                CubeMelonPluginErrorCode::NotSupported
            }
        }
    }
}

impl CubeMelonPluginStateInterface for HostRuntimeProxy {
    fn load_state(
        &self,
        scope: cubemelon_sdk::CubeMelonPluginStateScope,
        _data: &mut cubemelon_sdk::CubeMelonValue,
    ) -> CubeMelonPluginErrorCode {
        if let Some(rt) = with_runtime(|r| r as *const RuntimeData) {
            let r = unsafe { &*rt };
            cubemelon_sdk::CubeMelonPluginStateInterface::load_state(r, scope, _data)
        } else {
            CubeMelonPluginErrorCode::NotInitialized
        }
    }

    fn save_state(
        &mut self,
        scope: cubemelon_sdk::CubeMelonPluginStateScope,
        data: *const u8,
        size: usize,
    ) -> CubeMelonPluginErrorCode {
        if let Some(code) = crate::host_services::with_runtime_mut(|r| {
            cubemelon_sdk::CubeMelonPluginStateInterface::save_state(r, scope, data, size)
        }) {
            code
        } else {
            CubeMelonPluginErrorCode::NotInitialized
        }
    }

    fn get_format_name(
        &self,
        scope: cubemelon_sdk::CubeMelonPluginStateScope,
    ) -> *const u8 {
        if let Some(rt) = with_runtime(|r| r as *const RuntimeData) {
            let r = unsafe { &*rt };
            cubemelon_sdk::CubeMelonPluginStateInterface::get_format_name(r, scope)
        } else {
            std::ptr::null()
        }
    }

    fn get_state_value(
        &self,
        scope: cubemelon_sdk::CubeMelonPluginStateScope,
        key: *const u8,
        _value: &mut cubemelon_sdk::CubeMelonValue,
    ) -> CubeMelonPluginErrorCode {
        if let Some(rt) = with_runtime(|r| r as *const RuntimeData) {
            let r = unsafe { &*rt };
            cubemelon_sdk::CubeMelonPluginStateInterface::get_state_value(r, scope, key, _value)
        } else {
            CubeMelonPluginErrorCode::NotInitialized
        }
    }

    fn set_state_value(
        &mut self,
        scope: cubemelon_sdk::CubeMelonPluginStateScope,
        key: *const u8,
        data: *const u8,
        size: usize,
    ) -> CubeMelonPluginErrorCode {
        if let Some(code) = crate::host_services::with_runtime_mut(|r| {
            cubemelon_sdk::CubeMelonPluginStateInterface::set_state_value(r, scope, key, data, size)
        }) {
            code
        } else {
            CubeMelonPluginErrorCode::NotInitialized
        }
    }

    fn list_state_keys(
        &self,
        scope: cubemelon_sdk::CubeMelonPluginStateScope,
        _keys: &mut cubemelon_sdk::CubeMelonValue,
    ) -> CubeMelonPluginErrorCode {
        if let Some(rt) = with_runtime(|r| r as *const RuntimeData) {
            let r = unsafe { &*rt };
            cubemelon_sdk::CubeMelonPluginStateInterface::list_state_keys(r, scope, _keys)
        } else {
            CubeMelonPluginErrorCode::NotInitialized
        }
    }

    fn clear_state_value(
        &mut self,
        scope: cubemelon_sdk::CubeMelonPluginStateScope,
        key: *const u8,
    ) -> CubeMelonPluginErrorCode {
        if let Some(code) = crate::host_services::with_runtime_mut(|r| {
            cubemelon_sdk::CubeMelonPluginStateInterface::clear_state_value(r, scope, key)
        }) {
            code
        } else {
            CubeMelonPluginErrorCode::NotInitialized
        }
    }
}
