//! Resident Interface procedural macro implementation
//! 
//! This module implements the #[resident_plugin_impl] procedural macro
//! that generates C ABI code for the ResidentInterface.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{ImplItem, ItemImpl, Ident};

/// Process the #[resident_plugin_impl] attribute
/// 
/// This generates C ABI code for plugins that implement resident/background service functionality.
pub fn process_resident_impl_attribute(
    input: ItemImpl,
) -> Result<TokenStream2, syn::Error> {
    // Extract the struct name from the impl block
    let struct_name = match &*input.self_ty {
        syn::Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                &segment.ident
            } else {
                return Err(syn::Error::new_spanned(
                    &input.self_ty,
                    "Could not determine struct name from impl block",
                ));
            }
        }
        _ => {
            return Err(syn::Error::new_spanned(
                &input.self_ty,
                "#[resident_plugin_impl] can only be applied to impl blocks for named structs",
            ));
        }
    };

    // Parse the methods in the impl block
    let resident_methods = parse_resident_methods(&input)?;

    // Generate all the code components
    let original_impl = quote! { #input };
    let resident_interface_impl = generate_resident_interface_impl(struct_name, &resident_methods)?;
    let interface_implementation = generate_interface_implementation(struct_name);

    Ok(quote! {
        // Include the original impl block
        #original_impl
        
        // Generate CubeMelonResidentInterface trait implementation
        #resident_interface_impl
        
        // Generate methods to support get_interface in the main PluginBase implementation
        #interface_implementation
    })
}

/// Parsed resident methods from the impl block
struct ResidentMethods {
    get_status_method: Option<syn::ImplItemFn>,
    get_configuration_method: Option<syn::ImplItemFn>,
    update_configuration_method: Option<syn::ImplItemFn>,
    start_method: Option<syn::ImplItemFn>,
    suspend_method: Option<syn::ImplItemFn>,
    resume_method: Option<syn::ImplItemFn>,
    stop_method: Option<syn::ImplItemFn>,
    reset_method: Option<syn::ImplItemFn>,
    other_methods: Vec<syn::ImplItem>,
}

/// Parse methods from the resident impl block
fn parse_resident_methods(input: &ItemImpl) -> Result<ResidentMethods, syn::Error> {
    let mut methods = ResidentMethods {
        get_status_method: None,
        get_configuration_method: None,
        update_configuration_method: None,
        start_method: None,
        suspend_method: None,
        resume_method: None,
        stop_method: None,
        reset_method: None,
        other_methods: Vec::new(),
    };

    for item in &input.items {
        if let ImplItem::Fn(method) = item {
            let method_name = method.sig.ident.to_string();
            
            match method_name.as_str() {
                "get_status" => methods.get_status_method = Some(method.clone()),
                "get_configuration" => methods.get_configuration_method = Some(method.clone()),
                "update_configuration" => methods.update_configuration_method = Some(method.clone()),
                "start" => methods.start_method = Some(method.clone()),
                "suspend" => methods.suspend_method = Some(method.clone()),
                "resume" => methods.resume_method = Some(method.clone()),
                "stop" => methods.stop_method = Some(method.clone()),
                "reset" => methods.reset_method = Some(method.clone()),
                _ => methods.other_methods.push(item.clone()),
            }
        } else {
            // Non-method items (associated types, constants, etc.)
            methods.other_methods.push(item.clone());
        }
    }

    // Validate required methods are present
    validate_required_resident_methods(&methods, input)?;

    Ok(methods)
}

/// Validate that all required resident methods are present
fn validate_required_resident_methods(
    methods: &ResidentMethods,
    input: &ItemImpl,
) -> Result<(), syn::Error> {
    let required_methods = [
        ("get_status", &methods.get_status_method),
        ("get_configuration", &methods.get_configuration_method),
        ("update_configuration", &methods.update_configuration_method),
        ("start", &methods.start_method),
        ("suspend", &methods.suspend_method),
        ("resume", &methods.resume_method),
        ("stop", &methods.stop_method),
        ("reset", &methods.reset_method),
    ];

    for (method_name, method_option) in required_methods.iter() {
        if method_option.is_none() {
            return Err(syn::Error::new_spanned(
                input,
                format!(
                    "ResidentInterface implementation must include a '{}' method. See specification for required signature.",
                    method_name
                ),
            ));
        }
    }

    Ok(())
}

/// Generate CubeMelonResidentInterface trait implementation
fn generate_resident_interface_impl(
    struct_name: &Ident,
    methods: &ResidentMethods,
) -> Result<TokenStream2, syn::Error> {
    let get_status_method = &methods.get_status_method.as_ref().unwrap().sig.ident;
    let get_configuration_method = &methods.get_configuration_method.as_ref().unwrap().sig.ident;
    let update_configuration_method = &methods.update_configuration_method.as_ref().unwrap().sig.ident;
    let start_method = &methods.start_method.as_ref().unwrap().sig.ident;
    let suspend_method = &methods.suspend_method.as_ref().unwrap().sig.ident;
    let resume_method = &methods.resume_method.as_ref().unwrap().sig.ident;
    let stop_method = &methods.stop_method.as_ref().unwrap().sig.ident;
    let reset_method = &methods.reset_method.as_ref().unwrap().sig.ident;

    Ok(quote! {
        impl ::cubemelon_sdk::interfaces::resident::CubeMelonResidentInterface for #struct_name {
            fn get_status(&self) -> ::cubemelon_sdk::types::CubeMelonExecutionStatus {
                self.#get_status_method()
            }

            fn get_configuration(&self) -> *const u8 {
                self.#get_configuration_method()
            }

            fn update_configuration(
                &mut self,
                config_json: &str,
            ) -> ::cubemelon_sdk::error::CubeMelonPluginErrorCode {
                self.#update_configuration_method(config_json)
            }

            fn start(
                &mut self,
                config_json: &str,
            ) -> ::cubemelon_sdk::error::CubeMelonPluginErrorCode {
                self.#start_method(config_json)
            }

            fn suspend(&mut self) -> ::cubemelon_sdk::error::CubeMelonPluginErrorCode {
                self.#suspend_method()
            }

            fn resume(&mut self) -> ::cubemelon_sdk::error::CubeMelonPluginErrorCode {
                self.#resume_method()
            }

            fn stop(&mut self) -> ::cubemelon_sdk::error::CubeMelonPluginErrorCode {
                self.#stop_method()
            }

            fn reset(&mut self) -> ::cubemelon_sdk::error::CubeMelonPluginErrorCode {
                self.#reset_method()
            }
        }
    })
}

/// Generate interface implementation methods for integration with main plugin
fn generate_interface_implementation(struct_name: &Ident) -> TokenStream2 {
    quote! {
        // Add methods to the main plugin implementation to support ResidentInterface
        impl #struct_name {
            /// Get interface implementation for Resident
            /// 
            /// This method should be called from the main get_interface implementation
            /// when PLUGIN_TYPE_RESIDENT is requested.
            /// 
            /// # Usage in main get_interface method:
            /// ```rust
            /// fn get_interface(
            ///     &self,
            ///     plugin_types: u64,
            ///     interface_version: u32,
            /// ) -> Result<*const std::ffi::c_void, CubeMelonPluginErrorCode> {
            ///     use cubemelon_sdk::types::CubeMelonPluginType;
            ///     
            ///     if (plugin_types & (CubeMelonPluginType::Resident as u64)) != 0 {
            ///         return self.resident_interface(interface_version);
            ///     }
            ///     
            ///     // Handle other interfaces...
            ///     
            ///     Err(CubeMelonPluginErrorCode::InterfaceNotSupported)
            /// }
            /// ```
            pub fn resident_interface(
                &self,
                interface_version: u32,
            ) -> Result<*const std::ffi::c_void, ::cubemelon_sdk::error::CubeMelonPluginErrorCode> {
                // Check interface version (version 1 for now)
                if interface_version != 1 {
                    return Err(::cubemelon_sdk::error::CubeMelonPluginErrorCode::VersionMismatch);
                }

                // Create the interface using the existing helper
                let interface = ::cubemelon_sdk::interfaces::resident::create_resident_interface::<#struct_name>();
                
                // Box the interface and return as pointer
                // Note: The caller is responsible for freeing this memory
                let boxed_interface = Box::new(interface);
                Ok(Box::into_raw(boxed_interface) as *const std::ffi::c_void)
            }

            /// Check if this plugin supports Resident interface
            /// 
            /// This is a helper method for use in supported_types() implementations.
            /// Add CubeMelonPluginType::Resident to your supported types.
            pub const fn __cubemelon_supports_resident() -> bool {
                true
            }
        }

        // Provide a compile-time check that Resident should be included in supported_types
        const _: () = {
            // This will generate a compile warning if Resident is not included in supported_types
            // (This is a development aid - in production, developers should ensure correct supported_types)
            
            // We can't easily check the supported_types() method at compile time,
            // so we provide this constant for documentation purposes
            const RESIDENT_TYPE_VALUE: u64 = ::cubemelon_sdk::types::CubeMelonPluginType::Resident as u64;
            let _ = RESIDENT_TYPE_VALUE; // Use the constant to avoid unused warnings
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_parse_resident_methods_complete() {
        let input: ItemImpl = parse_quote! {
            impl TestPlugin {
                pub fn get_status(&self) -> CubeMelonExecutionStatus {
                    CubeMelonExecutionStatus::Idle
                }

                pub fn get_configuration(&self) -> *const u8 {
                    b"{}\0".as_ptr()
                }

                pub fn update_configuration(
                    &mut self,
                    config_json: &str,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }

                pub fn start(
                    &mut self,
                    config_json: &str,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }

                pub fn suspend(&mut self) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }

                pub fn resume(&mut self) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }

                pub fn stop(&mut self) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }

                pub fn reset(&mut self) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }
            }
        };
        
        let methods = parse_resident_methods(&input);
        assert!(methods.is_ok());
        
        let methods = methods.unwrap();
        assert!(methods.get_status_method.is_some());
        assert!(methods.get_configuration_method.is_some());
        assert!(methods.update_configuration_method.is_some());
        assert!(methods.start_method.is_some());
        assert!(methods.suspend_method.is_some());
        assert!(methods.resume_method.is_some());
        assert!(methods.stop_method.is_some());
        assert!(methods.reset_method.is_some());
    }

    #[test]
    fn test_parse_resident_methods_missing_required() {
        let input: ItemImpl = parse_quote! {
            impl TestPlugin {
                pub fn get_status(&self) -> CubeMelonExecutionStatus {
                    CubeMelonExecutionStatus::Idle
                }
                // Missing other required methods
            }
        };
        
        let methods = parse_resident_methods(&input);
        assert!(methods.is_err());
    }

    #[test]
    fn test_generate_resident_interface_impl() {
        let struct_name = syn::parse_str::<Ident>("TestPlugin").unwrap();
        let methods = ResidentMethods {
            get_status_method: Some(syn::parse_quote! {
                pub fn get_status(&self) -> CubeMelonExecutionStatus {
                    CubeMelonExecutionStatus::Idle
                }
            }),
            get_configuration_method: Some(syn::parse_quote! {
                pub fn get_configuration(&self) -> *const u8 {
                    b"{}\0".as_ptr()
                }
            }),
            update_configuration_method: Some(syn::parse_quote! {
                pub fn update_configuration(
                    &mut self,
                    config_json: &str,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }
            }),
            start_method: Some(syn::parse_quote! {
                pub fn start(
                    &mut self,
                    config_json: &str,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }
            }),
            suspend_method: Some(syn::parse_quote! {
                pub fn suspend(&mut self) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }
            }),
            resume_method: Some(syn::parse_quote! {
                pub fn resume(&mut self) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }
            }),
            stop_method: Some(syn::parse_quote! {
                pub fn stop(&mut self) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }
            }),
            reset_method: Some(syn::parse_quote! {
                pub fn reset(&mut self) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }
            }),
            other_methods: Vec::new(),
        };

        let result = generate_resident_interface_impl(&struct_name, &methods);
        assert!(result.is_ok());
    }
}