//! # CubeMelon Plugin System SDK
//! 
//! A flexible and extensible plugin system for dynamic loading and inter-plugin communication.
//! 
//! ## Features
//! 
//! - **C ABI Compatible**: Seamless integration with C, C++, Rust, and other languages
//! - **Type Safety**: Rust's type system ensures memory safety and correctness
//! - **Dynamic Loading**: Load and unload plugins at runtime
//! - **Inter-Plugin Communication**: Plugins can discover and communicate with each other
//! - **Automatic Memory Management**: SDK handles memory allocation/deallocation for C ABI
//! - **Extensible Interface System**: Support for various plugin types and custom interfaces
//! 
//! ## Quick Start
//! 
//! ```rust
//! use cubemelon_sdk::prelude::*;
//! 
//! #[plugin]
//! pub struct MyPlugin;
//! 
//! #[plugin_impl]
//! impl MyPlugin {
//!     pub fn new() -> Self { Self }
//!     
//!     pub fn get_uuid() -> CubeMelonUUID {
//!         uuid!("12345678-1234-5678-9abc-123456789abc")
//!     }
//!     
//!     pub fn get_version() -> CubeMelonVersion {
//!         CubeMelonVersion { major: 1, minor: 0, patch: 0 }
//!     }
//!     
//!     pub fn get_name(&self, _language: CubeMelonLanguage) -> *const u8 {
//!         multilang_map!!(language, "My Plugin", {})
//!     }
//!     
//!     pub fn get_description(&self, _language: CubeMelonLanguage) -> *const u8 {
//!         multilang_map!!(language, "A simple plugin example", {})
//!     }
//!     
//!     pub fn get_supported_types() -> u64 {
//!         CubeMelonPluginType::SINGLE_TASK as u64
//!     }
//! }
//! 
//! #[plugin_interface(basic)]
//! impl Plugin {}
//! ```

// SDK version information
pub const SDK_VERSION: CubeMelonVersion = CubeMelonVersion {
    major: 0,
    minor: 11,
    patch: 2,
};

pub const SDK_VERSION_STRING: &str = "0.11.2";

// Core modules
pub mod types;
pub mod string;
pub mod structs;
pub mod error;
pub mod memory;
pub mod macros;
pub mod instance;
pub mod interfaces;
//pub mod interface_ex;
//pub mod compat;

// Re-export procedural macros from sdk_macros crate
pub use cubemelon_sdk_macros::*;

// Re-export core types for convenience
pub use types::*;
pub use string::*;
pub use structs::*;
pub use error::*;
pub use memory::*;
pub use instance::*;

// Re-export all interfaces
pub use interfaces::*;
//pub use interface_ex::*;

/// Prelude module for convenient imports
pub mod prelude {
    //! Common imports for plugin development
    //! 
    //! ```rust
    //! use cubemelon_sdk::prelude::*;
    //! ```
    
    // Core types
    pub use crate::types::*;
    pub use crate::string::*;
    pub use crate::structs::*;
    pub use crate::error::*;
    pub use crate::memory::*;
    pub use crate::instance::*;

    pub use crate::uuid;
    pub use crate::version;
    pub use crate::plugin_types;
    pub use crate::thread_requirements;
    pub use crate::language;
    pub use crate::declare_plugin_base;
    pub use crate::create_vtable;
    pub use crate::generate_vtable_methods;
    pub use crate::c_str_literal;
    pub use crate::static_cubemelon_string;
    pub use crate::multilang_map;
    pub use crate::error_message;

    // All interfaces
    pub use crate::interfaces::*;
    //pub use crate::interface_ex::*;
    
    // Procedural macros
    pub use cubemelon_sdk_macros::*;
    
    // Declarative macros
    pub use crate::macros::*;
    
    // SDK information
    pub use crate::{SDK_VERSION, SDK_VERSION_STRING};
}

/// Compatibility check function
/// 
/// Checks if a plugin SDK version is compatible with the host SDK version.
/// 
/// # Compatibility Rules
/// 
/// - Major versions must match exactly
/// - Minor and patch versions are backward compatible within the same major version
/// 
/// # Examples
/// 
/// ```rust
/// use cubemelon_sdk::*;
/// 
/// let host_version = CubeMelonVersion { major: 1, minor: 2, patch: 3 };
/// let plugin_version = CubeMelonVersion { major: 1, minor: 0, patch: 0 };
/// 
/// assert!(check_plugin_compatibility(plugin_version, host_version)); // ✅ Compatible
/// 
/// let incompatible_version = CubeMelonVersion { major: 2, minor: 0, patch: 0 };
/// assert!(!check_plugin_compatibility(incompatible_version, host_version)); // ❌ Incompatible
/// ```
pub fn check_plugin_compatibility(
    plugin_sdk_version: CubeMelonVersion,
    host_sdk_version: CubeMelonVersion,
) -> bool {
    // Major versions must match exactly for ABI compatibility
    plugin_sdk_version.major == host_sdk_version.major
}

/// Initialize the SDK
/// 
/// This function should be called once when the SDK is first loaded.
/// It sets up internal state and registers necessary handlers.
/// 
/// # Safety
/// 
/// This function is automatically called by the plugin macros and should not
/// be called manually unless you know what you're doing.
pub fn initialize_sdk() {
    // Set up memory allocators if needed
    memory::initialize_memory_system();
    
    // Initialize compatibility layer
    //compat::initialize_compatibility_layer();
}

/// Cleanup the SDK
/// 
/// This function should be called when the SDK is being unloaded.
/// It cleans up internal state and releases resources.
/// 
/// # Safety
/// 
/// This function is automatically called by the plugin macros and should not
/// be called manually unless you know what you're doing.
pub fn cleanup_sdk() {
    // Cleanup memory system
    memory::cleanup_memory_system();
    
    // Cleanup compatibility layer
    //compat::cleanup_compatibility_layer();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdk_version() {
        assert_eq!(SDK_VERSION.major, 0);
        assert_eq!(SDK_VERSION.minor, 11);
        assert_eq!(SDK_VERSION.patch, 2);
        assert_eq!(SDK_VERSION_STRING, "0.11.2");
    }

    #[test]
    fn test_compatibility_check() {
        let host_v1 = CubeMelonVersion { major: 1, minor: 2, patch: 3 };
        let plugin_v1_old = CubeMelonVersion { major: 1, minor: 0, patch: 0 };
        let plugin_v1_new = CubeMelonVersion { major: 1, minor: 3, patch: 0 };
        let plugin_v2 = CubeMelonVersion { major: 2, minor: 0, patch: 0 };

        // Same major version should be compatible
        assert!(check_plugin_compatibility(plugin_v1_old, host_v1));
        assert!(check_plugin_compatibility(plugin_v1_new, host_v1));
        
        // Different major version should be incompatible
        assert!(!check_plugin_compatibility(plugin_v2, host_v1));
    }

    #[test]
    fn test_sdk_initialization() {
        // Test that SDK can be initialized and cleaned up without panicking
        initialize_sdk();
        cleanup_sdk();
    }
}