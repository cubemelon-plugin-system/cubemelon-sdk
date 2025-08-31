//! Minimal plugin instance management for the CubeMelon Plugin System
//! 
//! This module provides a thin wrapper around plugin instances.
//! Thread safety and lifecycle management are the host's responsibility.

use std::any::Any;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Reference counter for tracking plugin file usage
/// 
/// This is incremented when a plugin instance is created and
/// decremented when destroyed. The host can use this to determine
/// when it's safe to unload the plugin DLL/SO.
static PLUGIN_REF_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Get the current plugin reference count
pub fn get_plugin_ref_count() -> usize {
    PLUGIN_REF_COUNT.load(Ordering::Relaxed)
}

/// Plugin instance - opaque type for C ABI
#[repr(C)]
pub struct CubeMelonPlugin {
    // Private field to make it opaque
    _private: [u8; 0],
}

/// Internal plugin data (not exposed via C ABI)
pub struct PluginBox {
    /// The actual plugin object
    plugin: Box<dyn Any + Send + Sync>,
}

impl PluginBox {
    /// Create a new plugin box
    pub fn new<T: Any + Send + Sync>(
        plugin: T,
    ) -> Box<Self> {
        // Increment reference count
        PLUGIN_REF_COUNT.fetch_add(1, Ordering::Relaxed);
        
        Box::new(Self {
            plugin: Box::new(plugin),
        })
    }

    /// Convert to raw pointer for C ABI
    pub fn into_raw(self: Box<Self>) -> *mut CubeMelonPlugin {
        Box::into_raw(self) as *mut CubeMelonPlugin
    }

    /// Create from raw pointer
    /// 
    /// # Safety
    /// The pointer must be valid and created by `into_raw`
    pub unsafe fn from_raw(ptr: *mut CubeMelonPlugin) -> Box<Self> {
        Box::from_raw(ptr as *mut Self)
    }

    /// Get reference from pointer
    /// 
    /// # Safety
    /// The pointer must be valid and not null
    pub unsafe fn from_ptr<'a>(ptr: *const CubeMelonPlugin) -> &'a Self {
        &*(ptr as *const Self)
    }

    /// Get mutable reference from pointer
    /// 
    /// # Safety
    /// The pointer must be valid and not null
    pub unsafe fn from_mut_ptr<'a>(ptr: *mut CubeMelonPlugin) -> &'a mut Self {
        &mut *(ptr as *mut Self)
    }

    /// Downcast to concrete type
    pub fn downcast<T: Any>(&self) -> Option<&T> {
        self.plugin.downcast_ref()
    }

    /// Downcast to concrete type (mutable)
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.plugin.downcast_mut()
    }
}

impl Drop for PluginBox {
    fn drop(&mut self) {
        // Decrement reference count
        PLUGIN_REF_COUNT.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Create a new plugin instance
/// 
/// This is a helper for implementing the C ABI `create_plugin` function.
/// 
/// # Example
/// ```rust
/// use cubemelon_sdk::prelude::*;
///
/// #[no_mangle]
/// pub extern "C" fn create_plugin() -> *mut CubeMelonPlugin {
///     let plugin = MyPlugin::new();
///     create_plugin_instance(plugin)
/// }
/// ```
pub fn create_plugin_instance<T: Any + Send + Sync>(
    plugin: T,
) -> *mut CubeMelonPlugin {
    PluginBox::new(plugin).into_raw()
}

/// Destroy a plugin instance
/// 
/// This is a helper for implementing the C ABI `destroy_plugin` function.
/// 
/// # Example
/// ```rust
/// use cubemelon_sdk::prelude::*;
///
/// #[no_mangle]
/// pub extern "C" fn destroy_plugin(plugin: *mut CubeMelonPlugin) {
///     destroy_plugin_instance(plugin);
/// }
/// ```
pub fn destroy_plugin_instance(plugin: *mut CubeMelonPlugin) {
    if !plugin.is_null() {
        unsafe {
            // Box automatically drops and deallocates
            let _ = PluginBox::from_raw(plugin);
        }
    }
}

/// Access plugin data immutably
pub fn with_plugin<T, R, F>(
    plugin: *const CubeMelonPlugin,
    f: F,
) -> Option<R>
where
    T: Any,
    F: FnOnce(&T) -> R,
{
    if plugin.is_null() {
        return None;
    }
    unsafe {
        let boxed = PluginBox::from_ptr(plugin);
        boxed.downcast::<T>().map(f)
    }
}

/// Access plugin data mutably
pub fn with_plugin_mut<T, R, F>(
    plugin: *mut CubeMelonPlugin,
    f: F,
) -> Option<R>
where
    T: Any,
    F: FnOnce(&mut T) -> R,
{
    if plugin.is_null() {
        return None;
    }
    unsafe {
        let boxed = PluginBox::from_mut_ptr(plugin);
        boxed.downcast_mut::<T>().map(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestPlugin {
        name: String,
        value: i32,
    }

    impl TestPlugin {
        fn new(name: String, value: i32) -> Self {
            Self { name, value }
        }
    }

    #[test]
    fn test_plugin_creation_and_destruction() {
        let initial_count = get_plugin_ref_count();
        
        let plugin = TestPlugin::new("Test".to_string(), 42);
        let name = plugin.name.clone();
        let ptr = create_plugin_instance(plugin);
        assert!(!ptr.is_null());
        assert_eq!(name, "Test");
        assert_eq!(get_plugin_ref_count(), initial_count + 1);
        
        // Destroy plugin
        destroy_plugin_instance(ptr);
        assert_eq!(get_plugin_ref_count(), initial_count);
    }

    #[test]
    fn test_plugin_initialization() {
        let plugin = TestPlugin::new("Test".to_string(), 65536);
        let value = plugin.value;
        let ptr = create_plugin_instance(plugin);
        assert!(!ptr.is_null());
        assert_eq!(value, 65536);

        // Destroy plugin
        destroy_plugin_instance(ptr);
    }

    #[test]
    fn test_null_safety() {
        let null_ptr: *mut CubeMelonPlugin = std::ptr::null_mut();
        
        // destroy should handle null safely
        destroy_plugin_instance(null_ptr); // Should not crash
    }

    #[test]
    fn test_multiple_instances() {
        let initial_count = get_plugin_ref_count();
        
        let mut ptrs = vec![];
        
        // Create multiple instances
        for i in 0..5 {
            let plugin = TestPlugin::new(format!("Plugin {}", i), 0);
            
            let ptr = create_plugin_instance(plugin);
            ptrs.push(ptr);
        }
        
        assert_eq!(get_plugin_ref_count(), initial_count + 5);
        
        // Destroy all instances
        for ptr in ptrs {
            destroy_plugin_instance(ptr);
        }
        
        assert_eq!(get_plugin_ref_count(), initial_count);
    }
}