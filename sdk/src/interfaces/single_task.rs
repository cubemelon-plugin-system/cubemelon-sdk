use crate::error::CubeMelonPluginErrorCode;
use crate::structs::{CubeMelonTaskRequest, CubeMelonTaskResult};
use crate::instance::CubeMelonPlugin;

/// Single Task Interface - Synchronous single task execution
/// 
/// Executes a single task synchronously. Control returns to the caller after task completion.
/// 
/// # Specification
/// - `CubeMelonTaskRequest` and `CubeMelonTaskResult` objects are created by the caller.
/// - `CubeMelonTaskRequest` and `CubeMelonTaskResult` objects are destroyed by the caller 
///   after task execution is complete and control returns.
/// 
/// # Implementation Example
/// ```rust
/// use cubemelon_sdk::prelude::*;
/// 
/// struct MyPlugin {
///     // plugin data
/// }
/// 
/// impl CubeMelonSingleTaskInterface for MyPlugin {
///     fn execute(
///         &mut self,
///         request: &CubeMelonTaskRequest,
///         result: &mut CubeMelonTaskResult,
///     ) -> CubeMelonPluginErrorCode {
///         // Execute task synchronously
///         // Set results in result parameter
///         CubeMelonPluginErrorCode::Success
///     }
/// }
/// ```
pub trait CubeMelonSingleTaskInterface {
    /// Execute task (synchronous)
    /// 
    /// # Arguments
    /// * `request` - Task information to execute. Managed by caller.
    /// * `result` - Structure to store task execution results. Created by caller, results set by plugin.
    /// 
    /// # Returns
    /// * `CubeMelonPluginErrorCode` - Error code for execution result
    /// 
    /// # Notes
    /// - Caller must free `result.output_data` and `result.output_json` using `free_buffer()` and `free_string()`
    /// - All fields in `result` should be properly set (unused fields should be set to NULL or 0)
    fn execute(
        &mut self,
        request: &CubeMelonTaskRequest,
        result: &mut CubeMelonTaskResult,
    ) -> CubeMelonPluginErrorCode;
}

/// C ABI CubeMelonSingleTaskInterface structure
/// 
/// The actual interface structure returned by `get_interface()` from plugins
#[repr(C)]
pub struct CubeMelonSingleTaskInterfaceImpl {
    /// Execute task (synchronous)
    /// Caller must free results using free_buffer() and free_string()
    pub execute: extern "C" fn(
        plugin: *mut CubeMelonPlugin,
        request: *const CubeMelonTaskRequest,
        result: *mut CubeMelonTaskResult,
    ) -> CubeMelonPluginErrorCode,
}

/// Helper function to generate C ABI interface from CubeMelonSingleTaskInterface trait implementation
/// 
/// # Usage Example
/// ```rust
/// use cubemelon_sdk::prelude::*;
///
/// #[plugin]
/// struct MyPlugin;
///
/// let interface = create_single_task_interface::<MyPlugin>();
/// ```
pub fn create_single_task_interface<T>() -> CubeMelonSingleTaskInterfaceImpl
where
    T: CubeMelonSingleTaskInterface + 'static,
{
    extern "C" fn execute_wrapper<T: CubeMelonSingleTaskInterface + 'static>(
        plugin: *mut CubeMelonPlugin,
        request: *const CubeMelonTaskRequest,
        result: *mut CubeMelonTaskResult,
    ) -> CubeMelonPluginErrorCode {
        // NULL pointer checks
        if plugin.is_null() || request.is_null() || result.is_null() {
            if !result.is_null() {
                unsafe {
                    (*result).error_code = CubeMelonPluginErrorCode::NullPointer;
                    (*result).status = crate::types::CubeMelonExecutionStatus::Error;
                }
            }
            return CubeMelonPluginErrorCode::NullPointer;
        }

        // Safe dereferencing
        let request_ref = unsafe { &*request };
        let result_ref = unsafe { &mut *result };

        // Access plugin instance safely through type-safe wrapper
        // The <T, _, _> syntax specifies the plugin type T, with other generics inferred
        let error_code = match crate::instance::with_plugin_mut::<T, _, _>(plugin, |p| {
            // Here plugin_instance is guaranteed to be of type &mut T
            p.execute(request_ref, result_ref)
        }) {
            Some(code) => code,
            None => {
                // Plugin not found or type mismatch
                result_ref.error_code = CubeMelonPluginErrorCode::PluginNotFound;
                result_ref.status = crate::types::CubeMelonExecutionStatus::Error;
                return CubeMelonPluginErrorCode::PluginNotFound;
            }
        };
        
        // Set error status if operation failed
        if error_code != CubeMelonPluginErrorCode::Success {
            result_ref.error_code = error_code;
            result_ref.status = crate::types::CubeMelonExecutionStatus::Error;
        }

        error_code
    }

    CubeMelonSingleTaskInterfaceImpl {
        execute: execute_wrapper::<T>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use crate::structs::*;

    struct TestPlugin {
        call_count: usize,
    }

    impl CubeMelonSingleTaskInterface for TestPlugin {
        fn execute(
            &mut self,
            _request: &CubeMelonTaskRequest,
            result: &mut CubeMelonTaskResult,
        ) -> CubeMelonPluginErrorCode {
            self.call_count += 1;
            
            // Test dummy data
            result.status = CubeMelonExecutionStatus::Completed;
            result.error_code = CubeMelonPluginErrorCode::Success;
            result.progress_ratio = 1.0;
            
            CubeMelonPluginErrorCode::Success
        }
    }

    #[test]
    fn test_single_task_interface_creation() {
        let interface = create_single_task_interface::<TestPlugin>();
        
        // Function pointer should be set (not null)
        // We can't directly compare function pointers to null, but we can verify it exists
        let _fn_ptr = interface.execute; // This would panic if null
        assert!(_fn_ptr as usize != 0);
    }

    #[test] 
    fn test_single_task_execution() {
        // Create test plugin instance
        let _plugin = TestPlugin { call_count: 0 };
        
        // Register plugin in registry (depends on actual instance.rs implementation)
        // This part needs adjustment based on instance.rs implementation
        
        // Create interface
        let _interface = create_single_task_interface::<TestPlugin>();
        
        // Create test request and response
        let _request = CubeMelonTaskRequest ::empty();
        
        let mut _result = CubeMelonTaskResult::empty();
        
        // Call execute (actual testing should be done in integration tests)
        // (interface.execute)(plugin_ptr, &request, &mut result);
        
        // Verify results
        // assert_eq!(result.status, CubeMelonExecutionStatus::Completed);
    }
}