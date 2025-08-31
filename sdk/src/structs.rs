//! Complex data structures for the CubeMelon Plugin System
//! 
//! This module contains more complex structures that are built from basic types
//! and may reference other modules. This helps avoid circular dependencies.

use crate::types::{
    CubeMelonUUID, CubeMelonVersion, CubeMelonLanguage, CubeMelonTaskType, CubeMelonExecutionStatus
};
use crate::error::CubeMelonPluginErrorCode;
use crate::memory::{CubeMelonString, CubeMelonValue,};
use crate::instance::CubeMelonPlugin;

/// Plugin basic information structure
#[repr(C)]
#[derive(Debug, Clone)]
pub struct CubeMelonPluginBasicInfo {
    /// Plugin UUID
    pub uuid: CubeMelonUUID,
    /// Plugin version
    pub version: CubeMelonVersion,
    /// Plugin name in specified language
    pub name: CubeMelonString,
    /// Plugin description in specified language
    pub description: CubeMelonString,
    /// Supported functionality types (raw u64 value of combined CubeMelonPluginType flags)
    pub supported_types: u64,
}

impl CubeMelonPluginBasicInfo {
    /// Create a new plugin basic info structure
    pub fn new(
        uuid: CubeMelonUUID,
        version: CubeMelonVersion,
        name: CubeMelonString,
        description: CubeMelonString,
        supported_types: u64,
    ) -> Self {
        Self {
            uuid,
            version,
            name,
            description,
            supported_types,
        }
    }
}

/// Task request structure containing all information needed to execute a task
#[repr(C)]
#[derive(Debug)]
pub struct CubeMelonTaskRequest {
    /// Calling plugin (managed by caller)
    pub caller: *const CubeMelonPlugin,
    /// Input data (managed by caller)
    pub input_data: *mut CubeMelonValue,
    /// Additional information in JSON format
    pub input_json: CubeMelonString,
    /// Task type
    pub task_type: CubeMelonTaskType,
    /// Task language
    pub language: CubeMelonLanguage,
    /// Task execution start time (microseconds)
    pub request_time_us: i64,
    /// Timeout for asynchronous execution (microseconds)
    pub timeout_us: i64,
    /// Application-specific data
    pub user_data: *mut std::ffi::c_void,
    /// Reserved for future expansion
    pub reserved: [*mut std::ffi::c_void; 2],
}

impl CubeMelonTaskRequest {
    /// Create a new task request
    pub fn new(
        caller: *const CubeMelonPlugin,
        input_data: *mut CubeMelonValue,
        input_json: CubeMelonString,
        task_type: CubeMelonTaskType,
        language: CubeMelonLanguage,
        request_time_us: i64,
        timeout_us: i64,
    ) -> Self {
        Self {
            caller,
            input_data,
            input_json,
            task_type,
            language,
            request_time_us,
            timeout_us,
            user_data: std::ptr::null_mut(),
            reserved: [std::ptr::null_mut(); 2],
        }
    }

    /// Create an empty task request
    pub fn empty() -> Self {
        Self {
            caller: std::ptr::null(),
            input_data: std::ptr::null_mut(),
            input_json: CubeMelonString::empty(),
            task_type: CubeMelonTaskType::None,
            language: CubeMelonLanguage::EN_US,
            request_time_us: 0,
            timeout_us: 0,
            user_data: std::ptr::null_mut(),
            reserved: [std::ptr::null_mut(); 2],
        }
    }
}

unsafe impl Send for CubeMelonTaskRequest {}
unsafe impl Sync for CubeMelonTaskRequest {}

/// Task result structure containing execution results and progress information
#[repr(C)]
#[derive(Debug)]
pub struct CubeMelonTaskResult {
    /// Executing plugin (managed by plugin)
    pub callee: *const CubeMelonPlugin,
    /// Output data (allocated by plugin)
    pub output_data: *mut CubeMelonValue,
    /// Additional information in JSON format
    pub output_json: CubeMelonString,
    /// Execution status
    pub status: CubeMelonExecutionStatus,
    /// Error code
    pub error_code: CubeMelonPluginErrorCode,
    /// Task execution completion time (microseconds)
    pub completion_time_us: i64,
    /// Progress ratio [0.0, 1.0] (< 0.0 means unknown)
    pub progress_ratio: f64,
    /// Progress information ("Processing file 3/10", etc.)
    pub progress_message: CubeMelonString,
    /// Progress stage ("downloading", "processing", "uploading", etc.)
    pub progress_stage: CubeMelonString,
    /// Estimated remaining time in milliseconds (UINT64_MAX( == -1) means unknown)
    pub estimated_remaining_us: u64,
    /// Reserved for future expansion
    pub reserved: [*mut std::ffi::c_void; 2],
}

impl CubeMelonTaskResult {
    /// Create a new task result
    pub fn new(
        callee: *const CubeMelonPlugin,
        output_data: *mut CubeMelonValue,
        output_json: CubeMelonString,
        status: CubeMelonExecutionStatus,
        error_code: CubeMelonPluginErrorCode,
        completion_time_us: i64,
    ) -> Self {
        Self {
            callee,
            output_data,
            output_json,
            status,
            error_code,
            completion_time_us,
            progress_ratio: -1.0, // Unknown progress
            progress_message: CubeMelonString::empty(),
            progress_stage: CubeMelonString::empty(),
            estimated_remaining_us: u64::MAX, // Unknown time
            reserved: [std::ptr::null_mut(); 2],
        }
    }

    /// Create a successful result
    pub fn success(
        callee: *const CubeMelonPlugin,
        output_data: *mut CubeMelonValue,
        output_json: CubeMelonString,
        completion_time_us: i64,
    ) -> Self {
        Self::new(
            callee,
            output_data,
            output_json,
            CubeMelonExecutionStatus::Completed,
            CubeMelonPluginErrorCode::Success,
            completion_time_us,
        )
    }

    /// Create an error result
    pub fn error(
        callee: *const CubeMelonPlugin,
        error_code: CubeMelonPluginErrorCode,
        error_message: CubeMelonString,
    ) -> Self {
        Self::new(
            callee,
            std::ptr::null_mut(),
            error_message,
            CubeMelonExecutionStatus::Error,
            error_code,
            0,
        )
    }

    /// Create an empty result
    pub fn empty() -> Self {
        Self {
            callee: std::ptr::null(),
            output_data: std::ptr::null_mut(),
            output_json: CubeMelonString::empty(),
            status: CubeMelonExecutionStatus::Idle,
            error_code: CubeMelonPluginErrorCode::Success,
            completion_time_us: 0,
            progress_ratio: -1.0,
            progress_message: CubeMelonString::empty(),
            progress_stage: CubeMelonString::empty(),
            estimated_remaining_us: u64::MAX,
            reserved: [std::ptr::null_mut(); 2],
        }
    }

    /// Update progress information
    pub fn set_progress(
        &mut self,
        ratio: f64,
        message: CubeMelonString,
        stage: CubeMelonString,
        estimated_remaining_us: u64,
    ) {
        self.progress_ratio = ratio;
        self.progress_message = message;
        self.progress_stage = stage;
        self.estimated_remaining_us = estimated_remaining_us;
    }

    /// Check if the result represents success
    pub fn is_success(&self) -> bool {
        self.error_code == CubeMelonPluginErrorCode::Success &&
        (self.status == CubeMelonExecutionStatus::Completed || 
         self.status == CubeMelonExecutionStatus::Running)
    }

    /// Check if the result represents an error
    pub fn is_error(&self) -> bool {
        self.status == CubeMelonExecutionStatus::Error ||
        self.error_code != CubeMelonPluginErrorCode::Success
    }

    /// Check if progress is known
    pub fn has_progress(&self) -> bool {
        self.progress_ratio >= 0.0
    }

    /// Check if remaining time is known
    pub fn has_estimated_time(&self) -> bool {
        self.estimated_remaining_us != u64::MAX
    }
}

unsafe impl Send for CubeMelonTaskResult {}
unsafe impl Sync for CubeMelonTaskResult {}

/// Task callback function type for asynchronous operations
pub type CubeMelonTaskCallback = unsafe extern "C" fn(
    request: *mut CubeMelonTaskRequest,
    result: *const CubeMelonTaskResult,
);

/// Host services structure provided by the host application
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CubeMelonHostServices {
    /// Log output function
    pub log: Option<unsafe extern "C" fn(
        level: crate::types::CubeMelonLogLevel,
        plugin_name: *const u8,
        message: *const u8,
    )>,
    /// System language function
    pub get_system_language: Option<unsafe extern "C" fn() -> CubeMelonLanguage>,

    /// Reserved for future host services
    /// Future services might include:
    /// - get_system_time
    /// - get_app_data_directory
    /// - etc.
    pub reserved: [*mut std::ffi::c_void; 8],
}

impl CubeMelonHostServices {
    /// Create a new host services structure
    pub fn new(
        log_fn: Option<unsafe extern "C" fn(
            crate::types::CubeMelonLogLevel,
            *const u8,
            *const u8,
        )>,
        get_system_language_fn: Option<unsafe extern "C" fn() -> CubeMelonLanguage>
    ) -> Self {
        Self {
            log: log_fn,
            get_system_language: get_system_language_fn,
            reserved: [std::ptr::null_mut(); 8],
        }
    }

    /// Create an empty host services structure
    pub const fn empty() -> Self {
        Self {
            log: None,
            get_system_language: None,
            reserved: [std::ptr::null_mut(); 8],
        }
    }

    /// Log a message using the host's logging system
    /// 
    /// # Safety
    /// 
    /// The plugin_name and message must be valid NULL-terminated UTF-8 strings
    pub fn log_message(
        &self,
        level: crate::types::CubeMelonLogLevel,
        plugin_name: &str,
        message: &str,
    ) {
        if let Some(log_fn) = self.log {
            if let (Ok(plugin_name_cstr), Ok(message_cstr)) = 
                (std::ffi::CString::new(plugin_name), std::ffi::CString::new(message)) {
                unsafe {
                    log_fn(
                        level,
                        plugin_name_cstr.as_ptr() as *const u8,
                        message_cstr.as_ptr() as *const u8,
                    );
                    // CStringは自動的に解放される
                }
            }
        }
    }

    pub unsafe fn get_system_language(&self) -> CubeMelonLanguage {
        if let Some(get_lang_fn) = self.get_system_language {
            get_lang_fn()
        } else {
            CubeMelonLanguage::EN_US
        }
    }
}

unsafe impl Send for CubeMelonHostServices {}
unsafe impl Sync for CubeMelonHostServices {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[test]
    fn test_plugin_basic_info_creation() {
        let uuid = CubeMelonUUID::from_bytes([1; 16]);
        let version = CubeMelonVersion::new(1, 0, 0);
        let name = CubeMelonString::from_static_str("Test Plugin");
        let description = CubeMelonString::from_static_str("A test plugin");
        let types = CubeMelonPluginType::SingleTask as u64;

        let info = CubeMelonPluginBasicInfo::new(uuid, version, name, description, types);
        
        assert_eq!(info.uuid, uuid);
        assert_eq!(info.version, version);
        assert_eq!(info.supported_types, types);
    }

    #[test]
    fn test_task_request_creation() {
        let caller = std::ptr::null();
        let input_data = std::ptr::null_mut();
        let input_json = CubeMelonString::from_static_str("{}");
        let task_type = CubeMelonTaskType::Generic;
        let task_language = CubeMelonLanguage::EN_US;
        let timeout = 5000;

        let request = CubeMelonTaskRequest::new(
            caller,
            input_data,
            input_json,
            task_type,
            task_language,
            0,
            timeout,
        );

        assert_eq!(request.task_type, task_type);
        assert_eq!(request.timeout_us, timeout);
        assert!(request.user_data.is_null());
    }

    #[test]
    fn test_task_result_creation() {
        let callee = std::ptr::null();
        let output_data = std::ptr::null_mut();
        let output_json = CubeMelonString::from_static_str("{}");

        let success_result = CubeMelonTaskResult::success(callee, output_data, output_json, 0);
        assert!(success_result.is_success());
        assert!(!success_result.is_error());

        let error_result = CubeMelonTaskResult::error(
            callee,
            CubeMelonPluginErrorCode::Network,
            CubeMelonString::from_static_str("Network error"),
        );
        assert!(!error_result.is_success());
        assert!(error_result.is_error());
    }

    #[test]
    fn test_task_result_progress() {
        let mut result = CubeMelonTaskResult::empty();
        
        assert!(!result.has_progress());
        assert!(!result.has_estimated_time());

        result.set_progress(
            0.5,
            CubeMelonString::from_static_str("Processing..."),
            CubeMelonString::from_static_str("processing"),
            3_000_000,
        );

        assert!(result.has_progress());
        assert!(result.has_estimated_time());
        assert_eq!(result.progress_ratio, 0.5);
        assert_eq!(result.estimated_remaining_us, 3 * 1000 * 1000);
    }

    #[test]
    fn test_host_services() {
        let services = CubeMelonHostServices::empty();
        assert!(services.log.is_none());

        // Test logging (this won't actually log anything since log function is None)
        services.log_message(
            CubeMelonLogLevel::Info,
            "TestPlugin",
            "Test message",
        );
    }
}
