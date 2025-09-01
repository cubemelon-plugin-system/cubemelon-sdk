//! Declarative macros for the CubeMelon Plugin System
//! 
//! This module provides convenient declarative macros for plugin development.
//! These macros help reduce boilerplate code and ensure correct plugin implementation.

/// Create a UUID from a string literal
/// 
/// # Example
/// 
/// ```rust
/// use cubemelon_sdk::uuid;
/// 
/// let plugin_uuid = uuid!("12345678-1234-5678-9abc-123456789abc");
/// ```
#[macro_export]
macro_rules! uuid {
    ($uuid_str:literal) => {{
        // Parse the UUID string at compile time
        const fn parse_uuid(s: &str) -> [u8; 16] {
            let bytes = s.as_bytes();
            let mut result = [0u8; 16];
            let mut result_idx = 0;
            let mut i = 0;
            
            while i < bytes.len() {
                let c = bytes[i];
                if c == b'-' {
                    i += 1;
                    continue;
                }
                
                let high = match c {
                    b'0'..=b'9' => c - b'0',
                    b'a'..=b'f' => c - b'a' + 10,
                    b'A'..=b'F' => c - b'A' + 10,
                    _ => panic!("Invalid UUID character"),
                };
                
                i += 1;
                if i >= bytes.len() {
                    panic!("Invalid UUID format");
                }
                
                let c = bytes[i];
                let low = match c {
                    b'0'..=b'9' => c - b'0',
                    b'a'..=b'f' => c - b'a' + 10,
                    b'A'..=b'F' => c - b'A' + 10,
                    _ => panic!("Invalid UUID character"),
                };
                
                if result_idx >= 16 {
                    panic!("UUID too long");
                }
                
                result[result_idx] = (high << 4) | low;
                result_idx += 1;
                i += 1;
            }
            
            if result_idx != 16 {
                panic!("UUID too short");
            }
            
            result
        }
        
        $crate::types::CubeMelonUUID::from_bytes(parse_uuid($uuid_str))
    }};
}

/// Create a version from major, minor, patch numbers
/// 
/// # Example
/// 
/// ```rust
/// use cubemelon_sdk::version;
/// 
/// let v = version!(1, 2, 3);  // Version 1.2.3
/// ```
#[macro_export]
macro_rules! version {
    ($major:expr, $minor:expr, $patch:expr) => {
        $crate::types::CubeMelonVersion::new($major as u16, $minor as u8, $patch as u8)
    };
}

/// Create plugin type combinations
/// 
/// # Example
/// 
/// ```rust
/// use cubemelon_sdk::{plugin_types, types::CubeMelonPluginType};
/// 
/// let types = plugin_types!(SingleTask | HttpClient | DataInput);
/// ```
#[macro_export]
macro_rules! plugin_types {
    ($($type:ident)|+) => {
        {
            use $crate::types::CubeMelonPluginType;
            $(CubeMelonPluginType::$type as u64)|+
        }
    };
    ($type:ident) => {
        $crate::types::CubeMelonPluginType::$type as u64
    };
}

/// Create thread requirements combinations
/// 
/// # Example
/// 
/// ```rust
/// use cubemelon_sdk::{thread_requirements, types::CubeMelonThreadRequirements};
/// 
/// let reqs = thread_requirements!(Background | HighPriority);
/// ```
#[macro_export]
macro_rules! thread_requirements {
    ($($req:ident)|+) => {
        {
            use $crate::types::CubeMelonThreadRequirements;
            $(CubeMelonThreadRequirements::$req as u32)|+
        }
    };
    ($req:ident) => {
        $crate::types::CubeMelonThreadRequirements::$req as u32
    };
}

/// Create a language constant
/// 
/// # Example
/// 
/// ```rust
/// use cubemelon_sdk::language;
/// 
/// let lang = language!("ja-JP");
/// ```
#[macro_export]
macro_rules! language {
    ($code:literal) => {{
        const CODE_WITH_NULL: &str = concat!($code, "\0");
        $crate::types::CubeMelonLanguage {
            code: CODE_WITH_NULL.as_ptr(),
        }
    }};
}

/// Declare a plugin base trait implementation
/// 
/// This macro helps implement the basic required methods for a plugin.
/// 
/// # Example
/// 
/// ```rust
/// use cubemelon_sdk::{thread_requirements, declare_plugin_base, uuid, version, plugin_types};
/// 
/// struct MyPlugin;
/// 
/// declare_plugin_base! {
///     MyPlugin,
///     uuid: uuid!("12345678-1234-5678-9abc-123456789abc"),
///     version: version!(1, 0, 0),
///     supported_types: plugin_types!(SingleTask | HttpClient),
///     thread_safe: true,
///     thread_requirements: thread_requirements!(Background)
/// }
/// ```
#[macro_export]
macro_rules! declare_plugin_base {
    (
        $plugin_type:ty,
        uuid: $uuid:expr,
        version: $version:expr,
        supported_types: $supported_types:expr,
        thread_safe: $thread_safe:expr,
        thread_requirements: $thread_requirements:expr
    ) => {
        impl PluginBase for $plugin_type {
            fn get_uuid() -> $crate::types::CubeMelonUUID {
                $uuid
            }

            fn get_version() -> $crate::types::CubeMelonVersion {
                $version
            }

            fn get_supported_types() -> u64 {
                $supported_types
            }

            fn is_thread_safe() -> bool {
                $thread_safe
            }

            fn get_thread_requirements() -> u32 {
                $thread_requirements
            }

            fn get_name(&self, _language: $crate::types::CubeMelonLanguage) -> *const u8 {
                // Default implementation - should be overridden
                b"Unnamed Plugin".as_ptr()
            }

            fn get_description(&self, _language: $crate::types::CubeMelonLanguage) -> *const u8 {
                // Default implementation - should be overridden
                "No description".as_ptr()
            }
        }
    };
}

/// Generate VTable methods for C ABI export
/// 
/// This macro generates the necessary C-compatible function pointers
/// for plugin interfaces.
#[macro_export]
macro_rules! create_vtable {
    (
        $interface_name:ident,
        $plugin_type:ty,
        {
            $(
                $method_name:ident: $method_type:ty
            ),* $(,)?
        }
    ) => {
        paste::paste! {
            pub struct [<$interface_name VTable>] {
                $(
                    pub $method_name: $method_type,
                )*
            }

            impl [<$interface_name VTable>] {
                pub fn new() -> Self {
                    Self {
                        $(
                            $method_name: [<$method_name _impl>],
                        )*
                    }
                }
            }

            $(
                extern "C" fn [<$method_name _impl>](
                    // Parameters will be generated based on method signature
                    // This is a simplified version - the procedural macro will handle this properly
                ) {
                    // Implementation will be generated
                    unimplemented!("VTable method implementation")
                }
            )*
        }
    };
}

/// Generate VTable method implementations
/// 
/// This is a helper macro for creating C-compatible method implementations.
#[macro_export]
macro_rules! generate_vtable_methods {
    (
        $plugin_type:ty,
        $($method_name:ident),* $(,)?
    ) => {
        $(
            paste::paste! {
                extern "C" fn [<$method_name _c_impl>](
                    plugin: *mut $crate::instance::PluginInstance,
                    // Other parameters would be added based on the specific method
                ) {
                    if plugin.is_null() {
                        return; // or appropriate error handling
                    }

                    // Safety: We assume the plugin pointer is valid
                    let plugin_ref = unsafe { &mut *plugin };
                    
                    // Call the actual Rust method
                    // This is a simplified version - the real implementation would
                    // handle parameter conversion and error handling
                }
            }
        )*
    };
}

/// Helper trait that all plugins must implement
/// 
/// This trait provides the basic interface that every plugin needs.
pub trait PluginBase {
    /// Get the plugin's UUID
    fn get_uuid() -> crate::types::CubeMelonUUID;

    /// Get the plugin's version
    fn get_version() -> crate::types::CubeMelonVersion;

    /// Get the supported plugin types as raw u64 (combination of CubeMelonPluginType flags)
    fn get_supported_types() -> u64;

    /// Check if the plugin is thread-safe
    fn is_thread_safe() -> bool;

    /// Get thread requirements as raw u32 (combination of CubeMelonThreadRequirements flags)
    fn get_thread_requirements() -> u32;

    /// Get the plugin name in the specified language
    fn get_name(&self, language: crate::types::CubeMelonLanguage) -> *const u8;

    /// Get the plugin description in the specified language
    fn get_description(&self, language: crate::types::CubeMelonLanguage) -> *const u8;

    /// Initialize the plugin
    fn initialize(
        &mut self,
        _host_plugin: Option<&crate::instance::CubeMelonPlugin>,
        _host_interface: Option<&crate::interfaces::CubeMelonInterface>,
        _host_services: Option<&crate::structs::CubeMelonHostServices>,
    ) -> Result<(), crate::error::CubeMelonPluginErrorCode> {
        // Default implementation, can be overridden
        Ok(())
    }

    /// Uninitialize the plugin
    fn uninitialize(&mut self) -> Result<(), crate::error::CubeMelonPluginErrorCode> {
        // Default implementation, can be overridden
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[test]
    fn test_uuid_macro() {
        let uuid1 = uuid!("12345678-1234-5678-9abc-123456789abc");
        let _ = uuid!("12345678123456789abc123456789abc"); // Without hyphens
        
        assert_eq!(uuid1.bytes[0], 0x12);
        assert_eq!(uuid1.bytes[1], 0x34);
        assert_eq!(uuid1.bytes[15], 0xbc);
    }

    #[test]
    fn test_version_macro() {
        let v = version!(1, 2, 3);
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_plugin_types_macro() {
        let single_type = plugin_types!(SingleTask);
        assert_eq!(single_type, CubeMelonPluginType::SingleTask as u64);

        let combined_types = plugin_types!(SingleTask | HttpClient);
        assert_eq!(
            combined_types,
            CubeMelonPluginType::SingleTask as u64 | CubeMelonPluginType::HttpClient as u64
        );
    }

    #[test]
    fn test_language_macro() {
        let lang = language!("en-US");
        assert_eq!(lang.as_str(), "en-US");
    }

    // Test the PluginBase trait implementation
    struct TestPlugin;

    declare_plugin_base! {
        TestPlugin,
        uuid: uuid!("12345678-1234-5678-9abc-123456789abc"),
        version: version!(1, 0, 0),
        supported_types: plugin_types!(SingleTask),
        thread_safe: true,
        thread_requirements: thread_requirements!(Background)
    }

    #[test]
    fn test_plugin_base_implementation() {
        let expected_uuid = uuid!("12345678-1234-5678-9abc-123456789abc");
        assert_eq!(TestPlugin::get_uuid(), expected_uuid);
        assert_eq!(TestPlugin::get_version(), version!(1, 0, 0));
        assert_eq!(TestPlugin::get_supported_types(), CubeMelonPluginType::SingleTask as u64);
        assert!(TestPlugin::is_thread_safe());
    }
}