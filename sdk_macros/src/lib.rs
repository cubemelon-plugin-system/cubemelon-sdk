//! CubeMelon Plugin System - Procedural Macros
//! 
//! This crate provides procedural macros to simplify plugin development.
//! It automatically generates the necessary C ABI boilerplate code.

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemImpl};

mod plugin;
mod interface_impls;

/// Mark a struct as a CubeMelon plugin
/// 
/// This macro prepares a struct to be used as a plugin by adding necessary
/// metadata and ensuring it can be properly managed by the plugin system.
/// 
/// # Example
/// ```rust
/// use cubemelon_sdk_macros::plugin;
/// 
/// #[plugin]
/// pub struct MyPlugin {
///     initialized: bool,
/// }
/// ```
/// 
/// # What it does
/// - Validates the struct definition
/// - Adds compile-time metadata for the plugin system
/// - Ensures the struct can be used with #[plugin_impl]
/// 
/// # Requirements
/// - Must be applied to a struct (not enum or union)
/// - The struct must be public if you want to export it
/// - Should be used together with #[plugin_impl]
#[proc_macro_attribute]
pub fn plugin(args: TokenStream, input: TokenStream) -> TokenStream {
    plugin::plugin_attribute(args, input)
}

/// Implement plugin functionality for a marked struct
/// 
/// This macro generates all the necessary C ABI code for a plugin implementation.
/// It must be applied to an impl block for a struct that has been marked with #[plugin].
/// 
/// # Example
/// ```rust
/// use cubemelon_sdk_macros::{plugin, plugin_impl};
/// use cubemelon_sdk::prelude::*;
/// 
/// #[plugin]
/// pub struct MyPlugin {
///     initialized: bool,
/// }
/// 
/// #[plugin_impl]
/// impl MyPlugin {
///     pub fn new() -> Self {
///         Self { initialized: false }
///     }
///     
///     // Required methods:
///     pub fn get_uuid() -> CubeMelonUUID {
///         uuid!("12345678-1234-5678-9abc-123456789abc")
///     }
///     
///     pub fn get_version() -> CubeMelonVersion {
///         version!(1, 0, 0)
///     }
///     
///     pub fn get_supported_types() -> u64 {
///         CubeMelonPluginType::Basic as u64
///     }
///     
///     // Optional methods with defaults:
///     pub fn is_thread_safe() -> bool { true }
///     pub fn get_thread_requirements() -> u32 { 0 }
///     
///     pub fn get_name(&self, language: CubeMelonLanguage) -> *const u8 {
///         multilang_map!(language, "My Plugin", {})
///     }
///     
///     pub fn get_description(&self, language: CubeMelonLanguage) -> *const u8 {
///         multilang_map!(language, "A sample plugin", {})
///     }
///     
///     pub fn initialize(
///         &mut self,
///         _host_plugin: Option<&CubeMelonPlugin>,
///         _host_interface: Option<&CubeMelonInterface>,
///         _host_services: Option<&CubeMelonHostServices>,
///     ) -> Result<(), CubeMelonPluginErrorCode> {
///         self.initialized = true;
///         Ok(())
///     }
///     
///     pub fn uninitialize(&mut self) -> Result<(), CubeMelonPluginErrorCode> {
///         self.initialized = false;
///         Ok(())
///     }
/// }
/// ```
/// 
/// # What it generates
/// - All C ABI export functions (get_plugin_uuid, create_plugin, etc.)
/// - PluginBase trait implementation
/// - Static CubeMelonInterface structure
/// - C ABI wrapper functions
/// - Plugin instance management code
/// 
/// # Required methods
/// These methods must be implemented in the impl block:
/// - `get_uuid() -> CubeMelonUUID` - Plugin's unique identifier
/// - `get_version() -> CubeMelonVersion` - Plugin version
/// - `get_supported_types() -> u64` - Supported plugin type flags
/// 
/// # Optional methods with defaults
/// - `is_thread_safe() -> bool` - Defaults to `true`
/// - `get_thread_requirements() -> u32` - Defaults to `0` (no requirements)
/// - `get_name(&self, CubeMelonLanguage) -> *const u8` - Defaults to "Unnamed Plugin"
/// - `get_description(&self, CubeMelonLanguage) -> *const u8` - Defaults to "No description"
/// - `initialize(&mut self, ...) -> Result<(), CubeMelonPluginErrorCode>` - Defaults to `Ok(())`
/// - `uninitialize(&mut self) -> Result<(), CubeMelonPluginErrorCode>` - Defaults to `Ok(())`
/// 
/// # Constructor requirement
/// The plugin struct should have a `new() -> Self` method (or be Default),
/// as this will be used by the generated `create_plugin()` C ABI function.
#[proc_macro_attribute]
pub fn plugin_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    plugin::plugin_impl_attribute(args, input)
}

#[proc_macro_attribute]
pub fn single_task_plugin_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_impl = parse_macro_input!(input as ItemImpl);
    
    match crate::interface_impls::process_single_task_impl_attribute(input_impl) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute] 
pub fn async_task_plugin_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_impl = parse_macro_input!(input as ItemImpl);
    
    match crate::interface_impls::process_async_task_impl_attribute(input_impl) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn resident_plugin_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_impl = parse_macro_input!(input as ItemImpl);
    
    match crate::interface_impls::process_resident_impl_attribute(input_impl) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn state_plugin_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_impl = parse_macro_input!(input as ItemImpl);
    
    match crate::interface_impls::process_state_impl_attribute(input_impl) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn manager_plugin_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_impl = parse_macro_input!(input as ItemImpl);
    
    match crate::interface_impls::process_manager_impl_attribute(input_impl) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Generate get_plugin_interface function for specified plugin interfaces
/// 
/// This macro generates the C ABI get_plugin_interface function that handles
/// multiple plugin interface types. Use this after implementing the required
/// interfaces with their respective macros.
/// 
/// # Example
/// ```rust
/// use cubemelon_sdk_macros::{plugin, plugin_impl, single_task_plugin_impl, plugin_interface};
/// use cubemelon_sdk::prelude::*;
/// 
/// #[plugin]
/// pub struct MyPlugin;
/// 
/// #[plugin_impl]
/// impl MyPlugin {
///     pub fn new() -> Self { Self }
///     pub fn get_uuid() -> CubeMelonUUID { uuid!("...") }
///     pub fn get_version() -> CubeMelonVersion { version!(1, 0, 0) }
///     pub fn get_supported_types() -> u64 {
///         (CubeMelonPluginType::Basic as u64) | (CubeMelonPluginType::SingleTask as u64)
///     }
/// }
/// 
/// #[single_task_plugin_impl]
/// impl MyPlugin {
///     pub fn execute(
///         &mut self,
///         request: &CubeMelonTaskRequest,
///         result: &mut CubeMelonTaskResult,
///     ) -> CubeMelonPluginErrorCode {
///         CubeMelonPluginErrorCode::Success
///     }
/// }
/// 
/// // Generate get_plugin_interface that handles both Basic and SingleTask
/// #[plugin_interface(single_task)]
/// impl MyPlugin {}
/// ```
/// 
/// # Supported interface types
/// - `basic` - Basic plugin interface (always supported)
/// - `single_task` - Synchronous task execution
/// - `async_task` - Asynchronous task execution
/// - `resident` - Background service functionality
/// - `state` - State management
/// - `manager` - Plugin management
/// 
/// Multiple interfaces can be specified: `#[plugin_interface(single_task, async_task)]`
#[proc_macro_attribute]
pub fn plugin_interface(args: TokenStream, input: TokenStream) -> TokenStream {
    plugin::plugin_interface_attribute(args, input)
}

#[cfg(test)]
mod tests {
    // Note: Testing procedural macros requires special setup
    // These tests would typically be in integration tests or use trybuild
    
    #[test]
    fn test_lib_compiles() {
        // Basic compilation test
        // Real tests would use quote! and syn to test macro expansion
    }
}