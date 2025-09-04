//! State Interface procedural macro implementation
//! 
//! This module implements the #[state_plugin_impl] procedural macro
//! that generates C ABI code for the StateInterface.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{ImplItem, ItemImpl, Ident};

/// Process the #[state_plugin_impl] attribute
/// 
/// This generates C ABI code for plugins that implement state management functionality.
pub fn process_state_impl_attribute(
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
                "#[state_plugin_impl] can only be applied to impl blocks for named structs",
            ));
        }
    };

    // Parse the methods in the impl block
    let state_methods = parse_state_methods(&input)?;

    // Generate all the code components
    let original_impl = quote! { #input };
    let state_interface_impl = generate_state_interface_impl(struct_name, &state_methods)?;
    let interface_implementation = generate_interface_implementation(struct_name);

    Ok(quote! {
        // Include the original impl block
        #original_impl
        
        // Generate CubeMelonPluginStateInterface trait implementation
        #state_interface_impl
        
        // Generate interface implementation methods for State interface
        #interface_implementation
    })
}

/// Parsed state methods from the impl block
struct StateMethods {
    load_state_method: Option<syn::ImplItemFn>,
    save_state_method: Option<syn::ImplItemFn>,
    get_format_name_method: Option<syn::ImplItemFn>,
    get_state_value_method: Option<syn::ImplItemFn>,
    set_state_value_method: Option<syn::ImplItemFn>,
    list_state_keys_method: Option<syn::ImplItemFn>,
    clear_state_value_method: Option<syn::ImplItemFn>,
    other_methods: Vec<syn::ImplItem>,
}

/// Parse methods from the state impl block
fn parse_state_methods(input: &ItemImpl) -> Result<StateMethods, syn::Error> {
    let mut methods = StateMethods {
        load_state_method: None,
        save_state_method: None,
        get_format_name_method: None,
        get_state_value_method: None,
        set_state_value_method: None,
        list_state_keys_method: None,
        clear_state_value_method: None,
        other_methods: Vec::new(),
    };

    for item in &input.items {
        if let ImplItem::Fn(method) = item {
            let method_name = method.sig.ident.to_string();
            
            match method_name.as_str() {
                "load_state" => methods.load_state_method = Some(method.clone()),
                "save_state" => methods.save_state_method = Some(method.clone()),
                "get_format_name" => methods.get_format_name_method = Some(method.clone()),
                "get_state_value" => methods.get_state_value_method = Some(method.clone()),
                "set_state_value" => methods.set_state_value_method = Some(method.clone()),
                "list_state_keys" => methods.list_state_keys_method = Some(method.clone()),
                "clear_state_value" => methods.clear_state_value_method = Some(method.clone()),
                _ => methods.other_methods.push(item.clone()),
            }
        } else {
            // Non-method items (associated types, constants, etc.)
            methods.other_methods.push(item.clone());
        }
    }

    // Validate required methods are present
    validate_required_state_methods(&methods, input)?;

    Ok(methods)
}

/// Validate that all required state methods are present
fn validate_required_state_methods(
    methods: &StateMethods,
    input: &ItemImpl,
) -> Result<(), syn::Error> {
    let required_methods = [
        ("load_state", &methods.load_state_method),
        ("save_state", &methods.save_state_method),
        ("get_format_name", &methods.get_format_name_method),
        ("get_state_value", &methods.get_state_value_method),
        ("set_state_value", &methods.set_state_value_method),
        ("list_state_keys", &methods.list_state_keys_method),
        ("clear_state_value", &methods.clear_state_value_method),
    ];

    for (method_name, method_option) in required_methods.iter() {
        if method_option.is_none() {
            return Err(syn::Error::new_spanned(
                input,
                format!(
                    "StateInterface implementation must include a '{}' method. See specification for required signature.",
                    method_name
                ),
            ));
        }
    }

    Ok(())
}

/// Generate CubeMelonPluginStateInterface trait implementation
fn generate_state_interface_impl(
    struct_name: &Ident,
    methods: &StateMethods,
) -> Result<TokenStream2, syn::Error> {
    let load_state_method = &methods.load_state_method.as_ref().unwrap().sig.ident;
    let save_state_method = &methods.save_state_method.as_ref().unwrap().sig.ident;
    let get_format_name_method = &methods.get_format_name_method.as_ref().unwrap().sig.ident;
    let get_state_value_method = &methods.get_state_value_method.as_ref().unwrap().sig.ident;
    let set_state_value_method = &methods.set_state_value_method.as_ref().unwrap().sig.ident;
    let list_state_keys_method = &methods.list_state_keys_method.as_ref().unwrap().sig.ident;
    let clear_state_value_method = &methods.clear_state_value_method.as_ref().unwrap().sig.ident;

    Ok(quote! {
        impl ::cubemelon_sdk::interfaces::state::CubeMelonPluginStateInterface for #struct_name {
            fn load_state(
                &self,
                scope: ::cubemelon_sdk::types::CubeMelonPluginStateScope,
                data: &mut ::cubemelon_sdk::memory::CubeMelonValue,
            ) -> ::cubemelon_sdk::error::CubeMelonPluginErrorCode {
                self.#load_state_method(scope, data)
            }

            fn save_state(
                &mut self,
                scope: ::cubemelon_sdk::types::CubeMelonPluginStateScope,
                data: *const u8,
                size: usize,
            ) -> ::cubemelon_sdk::error::CubeMelonPluginErrorCode {
                self.#save_state_method(scope, data, size)
            }

            fn get_format_name(
                &self,
                scope: ::cubemelon_sdk::types::CubeMelonPluginStateScope,
            ) -> *const u8 {
                self.#get_format_name_method(scope)
            }

            fn get_state_value(
                &self,
                scope: ::cubemelon_sdk::types::CubeMelonPluginStateScope,
                key: *const u8,
                value: &mut ::cubemelon_sdk::memory::CubeMelonValue,
            ) -> ::cubemelon_sdk::error::CubeMelonPluginErrorCode {
                self.#get_state_value_method(scope, key, value)
            }

            fn set_state_value(
                &mut self,
                scope: ::cubemelon_sdk::types::CubeMelonPluginStateScope,
                key: *const u8,
                data: *const u8,
                size: usize,
            ) -> ::cubemelon_sdk::error::CubeMelonPluginErrorCode {
                self.#set_state_value_method(scope, key, data, size)
            }

            fn list_state_keys(
                &self,
                scope: ::cubemelon_sdk::types::CubeMelonPluginStateScope,
                keys: &mut ::cubemelon_sdk::memory::CubeMelonValue,
            ) -> ::cubemelon_sdk::error::CubeMelonPluginErrorCode {
                self.#list_state_keys_method(scope, keys)
            }

            fn clear_state_value(
                &mut self,
                scope: ::cubemelon_sdk::types::CubeMelonPluginStateScope,
                key: *const u8,
            ) -> ::cubemelon_sdk::error::CubeMelonPluginErrorCode {
                self.#clear_state_value_method(scope, key)
            }
        }
    })
}

/// Generate compile-time markers and helper methods
fn generate_interface_implementation(struct_name: &Ident) -> TokenStream2 {
    quote! {
        // Add helper methods to the main plugin implementation
        impl #struct_name {
            /// Check if this plugin supports State interface
            /// 
            /// This is a helper method for use in supported_types() implementations.
            /// Add CubeMelonPluginType::State to your supported types.
            pub const fn __cubemelon_supports_state() -> bool {
                true
            }
        }

        // Provide a compile-time check that State should be included in supported_types
        const _: () = {
            // This will generate a compile warning if State is not included in supported_types
            // (This is a development aid - in production, developers should ensure correct supported_types)
            
            // We can't easily check the supported_types() method at compile time,
            // so we provide this constant for documentation purposes
            const STATE_TYPE_VALUE: u64 = ::cubemelon_sdk::types::CubeMelonPluginType::State as u64;
            let _ = STATE_TYPE_VALUE; // Use the constant to avoid unused warnings
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_parse_state_methods_complete() {
        let input: ItemImpl = parse_quote! {
            impl TestPlugin {
                pub fn load_state(
                    &self,
                    scope: CubeMelonPluginStateScope,
                    data: &mut CubeMelonValue,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }

                pub fn save_state(
                    &mut self,
                    scope: CubeMelonPluginStateScope,
                    data: *const u8,
                    size: usize,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }

                pub fn get_format_name(
                    &self,
                    scope: CubeMelonPluginStateScope,
                ) -> *const u8 {
                    b"json\0".as_ptr()
                }

                pub fn get_state_value(
                    &self,
                    scope: CubeMelonPluginStateScope,
                    key: *const u8,
                    value: &mut CubeMelonValue,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }

                pub fn set_state_value(
                    &mut self,
                    scope: CubeMelonPluginStateScope,
                    key: *const u8,
                    data: *const u8,
                    size: usize,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }

                pub fn list_state_keys(
                    &self,
                    scope: CubeMelonPluginStateScope,
                    keys: &mut CubeMelonValue,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }

                pub fn clear_state_value(
                    &mut self,
                    scope: CubeMelonPluginStateScope,
                    key: *const u8,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }
            }
        };
        
        let methods = parse_state_methods(&input);
        assert!(methods.is_ok());
        
        let methods = methods.unwrap();
        assert!(methods.load_state_method.is_some());
        assert!(methods.save_state_method.is_some());
        assert!(methods.get_format_name_method.is_some());
        assert!(methods.get_state_value_method.is_some());
        assert!(methods.set_state_value_method.is_some());
        assert!(methods.list_state_keys_method.is_some());
        assert!(methods.clear_state_value_method.is_some());
    }

    #[test]
    fn test_parse_state_methods_missing_required() {
        let input: ItemImpl = parse_quote! {
            impl TestPlugin {
                pub fn load_state(
                    &self,
                    scope: CubeMelonPluginStateScope,
                    data: &mut CubeMelonValue,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }
                // Missing other required methods
            }
        };
        
        let methods = parse_state_methods(&input);
        assert!(methods.is_err());
    }

    #[test]
    fn test_generate_state_interface_impl() {
        let struct_name = syn::parse_str::<Ident>("TestPlugin").unwrap();
        let methods = StateMethods {
            load_state_method: Some(syn::parse_quote! {
                pub fn load_state(
                    &self,
                    scope: CubeMelonPluginStateScope,
                    data: &mut CubeMelonValue,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }
            }),
            save_state_method: Some(syn::parse_quote! {
                pub fn save_state(
                    &mut self,
                    scope: CubeMelonPluginStateScope,
                    data: *const u8,
                    size: usize,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }
            }),
            get_format_name_method: Some(syn::parse_quote! {
                pub fn get_format_name(
                    &self,
                    scope: CubeMelonPluginStateScope,
                ) -> *const u8 {
                    b"json\0".as_ptr()
                }
            }),
            get_state_value_method: Some(syn::parse_quote! {
                pub fn get_state_value(
                    &self,
                    scope: CubeMelonPluginStateScope,
                    key: *const u8,
                    value: &mut CubeMelonValue,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }
            }),
            set_state_value_method: Some(syn::parse_quote! {
                pub fn set_state_value(
                    &mut self,
                    scope: CubeMelonPluginStateScope,
                    key: *const u8,
                    data: *const u8,
                    size: usize,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }
            }),
            list_state_keys_method: Some(syn::parse_quote! {
                pub fn list_state_keys(
                    &self,
                    scope: CubeMelonPluginStateScope,
                    keys: &mut CubeMelonValue,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }
            }),
            clear_state_value_method: Some(syn::parse_quote! {
                pub fn clear_state_value(
                    &mut self,
                    scope: CubeMelonPluginStateScope,
                    key: *const u8,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }
            }),
            other_methods: Vec::new(),
        };

        let result = generate_state_interface_impl(&struct_name, &methods);
        assert!(result.is_ok());
    }
}