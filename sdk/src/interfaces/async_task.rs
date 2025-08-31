use crate::error::CubeMelonPluginErrorCode;
use crate::structs::{CubeMelonTaskRequest, CubeMelonTaskCallback};
use crate::instance::CubeMelonPlugin;

/// Async Task Interface - Asynchronous single task execution
/// 
/// Executes a single task asynchronously. Control returns to the caller immediately.
/// Results are received through a callback function.
/// 
/// # Specification
/// - The plugin must check if the callback function is NULL. If NULL, results are not notified.
/// - `CubeMelonTaskRequest` objects are created by the caller and destroyed within the callback function.
/// - `CubeMelonTaskResult` objects are created by the plugin and destroyed by the plugin 
///   after the callback function returns control to the plugin.
/// - For `cancel()` calls, the caller destroys the `CubeMelonTaskRequest` object after the operation completes,
///   but care must be taken to avoid double-freeing the same object within the callback function.
/// - If the `CubeMelonTaskRequest` object has already been destroyed before calling `cancel()`, 
///   the operation is ignored.
/// 
/// # Implementation Example
/// ```rust
/// use cubemelon_sdk::prelude::*;
/// 
/// struct MyPlugin {
///     // plugin data
/// }
/// 
/// impl CubeMelonAsyncTaskInterface for MyPlugin {
///     fn execute(
///         &mut self,
///         request: &CubeMelonTaskRequest,
///         callback: Option<CubeMelonTaskCallback>,
///     ) -> CubeMelonPluginErrorCode {
///         // Start asynchronous task execution
///         // Call callback when complete
///         CubeMelonPluginErrorCode::Success
///     }
///     
///     fn cancel(
///         &mut self,
///         request: &mut CubeMelonTaskRequest,
///     ) -> CubeMelonPluginErrorCode {
///         // Cancel the asynchronous task
///         CubeMelonPluginErrorCode::Success
///     }
/// }
/// ```
pub trait CubeMelonAsyncTaskInterface {
    /// Execute task (asynchronous)
    /// 
    /// # Arguments
    /// * `request` - Task information to execute. Managed by caller.
    /// * `callback` - Callback function to receive results. May be None.
    /// 
    /// # Returns
    /// * `CubeMelonPluginErrorCode` - Error code for execution start result
    /// 
    /// # Notes
    /// - Returns immediately after starting the task
    /// - Results are delivered through the callback function
    /// - Plugin must check if callback is None before calling
    fn execute(
        &mut self,
        request: &CubeMelonTaskRequest,
        callback: Option<CubeMelonTaskCallback>,
    ) -> CubeMelonPluginErrorCode;

    /// Cancel asynchronous task execution
    /// 
    /// # Arguments
    /// * `request` - Task request to cancel
    /// 
    /// # Returns
    /// * `CubeMelonPluginErrorCode` - Error code for cancellation result
    /// 
    /// # Notes
    /// - Caller destroys `CubeMelonTaskRequest` object after operation completes
    /// - Avoid double-freeing within callback functions
    /// - If request is already destroyed, operation is ignored
    fn cancel(
        &mut self,
        request: &mut CubeMelonTaskRequest,
    ) -> CubeMelonPluginErrorCode;
}

/// C ABI CubeMelonAsyncTaskInterface structure
/// 
/// The actual interface structure returned by `get_interface()` from plugins
#[repr(C)]
pub struct CubeMelonAsyncTaskInterfaceImpl {
    /// Execute task (asynchronous)
    pub execute: extern "C" fn(
        plugin: *mut CubeMelonPlugin,
        request: *const CubeMelonTaskRequest,
        callback: CubeMelonTaskCallback,
    ) -> CubeMelonPluginErrorCode,

    /// Cancel asynchronous task execution
    /// Caller must free buffer() and free_string() after completion
    pub cancel: extern "C" fn(
        plugin: *mut CubeMelonPlugin,
        request: *mut CubeMelonTaskRequest,
    ) -> CubeMelonPluginErrorCode,
}

/// Helper function to generate C ABI interface from CubeMelonAsyncTaskInterface trait implementation
/// 
/// # Usage Example
/// ```rust
/// use cubemelon_sdk::prelude::*;
///
/// #[plugin]
/// struct MyPlugin;
///
/// let interface = create_async_task_interface::<MyPlugin>();
/// ```
pub fn create_async_task_interface<T>() -> CubeMelonAsyncTaskInterfaceImpl
where
    T: CubeMelonAsyncTaskInterface + 'static,
{
    extern "C" fn execute_wrapper<T: CubeMelonAsyncTaskInterface + 'static>(
        plugin: *mut CubeMelonPlugin,
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
        // CubeMelonTaskCallback is a function pointer, so we check if it's null
        let callback_opt = if callback as usize != 0 {
            Some(callback)
        } else {
            None
        };

        // Access plugin instance safely through type-safe wrapper
        // The <T, _, _> syntax specifies the plugin type T, with other generics inferred
        match crate::instance::with_plugin_mut::<T, _, _>(plugin, |plugin_instance| {
            // Here plugin_instance is guaranteed to be of type &mut T
            plugin_instance.execute(request_ref, callback_opt)
        }) {
            Some(error_code) => error_code,
            None => {
                // Plugin not found or type mismatch
                CubeMelonPluginErrorCode::PluginNotFound
            }
        }
    }

    extern "C" fn cancel_wrapper<T: CubeMelonAsyncTaskInterface + 'static>(
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
            plugin_instance.cancel(request_ref)
        }) {
            Some(error_code) => error_code,
            None => {
                // Plugin not found or type mismatch
                CubeMelonPluginErrorCode::PluginNotFound
            }
        }
    }

    CubeMelonAsyncTaskInterfaceImpl {
        execute: execute_wrapper::<T>,
        cancel: cancel_wrapper::<T>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structs::*;

    struct TestAsyncPlugin {
        call_count: usize,
        cancelled_requests: Vec<usize>,
    }

    impl CubeMelonAsyncTaskInterface for TestAsyncPlugin {
        fn execute(
            &mut self,
            _request: &CubeMelonTaskRequest,
            callback: Option<CubeMelonTaskCallback>,
        ) -> CubeMelonPluginErrorCode {
            self.call_count += 1;
            
            // Simulate async execution
            if let Some(cb) = callback {
                // In real implementation, this would be called from another thread
                // For testing, we just verify the callback is not null
                assert!(cb as usize != 0);
            }
            
            CubeMelonPluginErrorCode::Success
        }

        fn cancel(
            &mut self,
            request: &mut CubeMelonTaskRequest,
        ) -> CubeMelonPluginErrorCode {
            // Track cancelled requests for testing
            self.cancelled_requests.push(request as *const _ as usize);
            CubeMelonPluginErrorCode::Success
        }
    }

    #[test]
    fn test_async_task_interface_creation() {
        let interface = create_async_task_interface::<TestAsyncPlugin>();
        
        // Verify function pointers are set (not null)
        let execute_fn = interface.execute;
        let cancel_fn = interface.cancel;
        assert!(execute_fn as usize != 0);
        assert!(cancel_fn as usize != 0);
    }

    #[test]
    fn test_async_task_execution() {
        // Create test plugin instance
        let _plugin = TestAsyncPlugin { 
            call_count: 0, 
            cancelled_requests: Vec::new(),
        };
        
        // Register plugin in registry (depends on actual instance.rs implementation)
        // This part needs adjustment based on instance.rs implementation
        
        // Create interface
        let _interface = create_async_task_interface::<TestAsyncPlugin>();
        
        // Create test request
        let _request = CubeMelonTaskRequest::empty();

        // Test callback function
        extern "C" fn _test_callback(
            request: *mut CubeMelonTaskRequest,
            result: *const CubeMelonTaskResult,
        ) {
            // Verify pointers are not null
            assert!(!request.is_null());
            assert!(!result.is_null());
        }
        
        // Call execute (actual testing should be done in integration tests)
        // let result = (interface.execute)(plugin_ptr, &request, test_callback);
        // assert_eq!(result, CubeMelonPluginErrorCode::Success);
    }

    #[test]
    fn test_null_callback_handling() {
        let _interface = create_async_task_interface::<TestAsyncPlugin>();
        
        // Test with null callback - should not crash
        let _request = CubeMelonTaskRequest::empty();
        
        // This should handle null callback gracefully
        // let result = (interface.execute)(plugin_ptr, &request, None);
    }
}