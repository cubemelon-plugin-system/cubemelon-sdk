//! Plugin Manager Interface procedural macro implementation
//! 
//! This module implements the #[manager_plugin_impl] procedural macro
//! that generates C ABI code for the PluginManagerInterface.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{ImplItem, ItemImpl, Ident};

/// Process the #[manager_plugin_impl] attribute
/// 
/// This generates C ABI code for plugins that implement plugin management functionality.
pub fn process_manager_impl_attribute(
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
                "#[manager_plugin_impl] can only be applied to impl blocks for named structs",
            ));
        }
    };

    // Parse the methods in the impl block
    let manager_methods = parse_manager_methods(&input)?;

    // Generate all the code components
    let original_impl = quote! { #input };
    let manager_interface_impl = generate_manager_interface_impl(struct_name, &manager_methods)?;
    let interface_implementation = generate_interface_implementation(struct_name);

    Ok(quote! {
        // Include the original impl block
        #original_impl
        
        // Generate CubeMelonPluginManagerInterface trait implementation
        #manager_interface_impl
        
        // Generate interface implementation methods for Manager interface
        #interface_implementation
    })
}

/// Parsed manager methods from the impl block
struct ManagerMethods {
    get_all_plugins_basic_info_method: Option<syn::ImplItemFn>,
    get_plugin_detailed_info_method: Option<syn::ImplItemFn>,
    find_plugins_for_task_method: Option<syn::ImplItemFn>,
    is_plugin_alive_method: Option<syn::ImplItemFn>,
    execute_task_method: Option<syn::ImplItemFn>,
    execute_async_task_method: Option<syn::ImplItemFn>,
    cancel_async_task_method: Option<syn::ImplItemFn>,
    other_methods: Vec<syn::ImplItem>,
}

/// Parse methods from the manager impl block
fn parse_manager_methods(input: &ItemImpl) -> Result<ManagerMethods, syn::Error> {
    let mut methods = ManagerMethods {
        get_all_plugins_basic_info_method: None,
        get_plugin_detailed_info_method: None,
        find_plugins_for_task_method: None,
        is_plugin_alive_method: None,
        execute_task_method: None,
        execute_async_task_method: None,
        cancel_async_task_method: None,
        other_methods: Vec::new(),
    };

    for item in &input.items {
        if let ImplItem::Fn(method) = item {
            let method_name = method.sig.ident.to_string();
            
            match method_name.as_str() {
                "get_all_plugins_basic_info" => methods.get_all_plugins_basic_info_method = Some(method.clone()),
                "get_plugin_detailed_info" => methods.get_plugin_detailed_info_method = Some(method.clone()),
                "find_plugins_for_task" => methods.find_plugins_for_task_method = Some(method.clone()),
                "is_plugin_alive" => methods.is_plugin_alive_method = Some(method.clone()),
                "execute_task" => methods.execute_task_method = Some(method.clone()),
                "execute_async_task" => methods.execute_async_task_method = Some(method.clone()),
                "cancel_async_task" => methods.cancel_async_task_method = Some(method.clone()),
                _ => methods.other_methods.push(item.clone()),
            }
        } else {
            // Non-method items (associated types, constants, etc.)
            methods.other_methods.push(item.clone());
        }
    }

    // Validate required methods are present
    validate_required_manager_methods(&methods, input)?;

    Ok(methods)
}

/// Validate that all required manager methods are present
fn validate_required_manager_methods(
    methods: &ManagerMethods,
    input: &ItemImpl,
) -> Result<(), syn::Error> {
    let required_methods = [
        ("get_all_plugins_basic_info", &methods.get_all_plugins_basic_info_method),
        ("get_plugin_detailed_info", &methods.get_plugin_detailed_info_method),
        ("find_plugins_for_task", &methods.find_plugins_for_task_method),
        ("is_plugin_alive", &methods.is_plugin_alive_method),
        ("execute_task", &methods.execute_task_method),
        ("execute_async_task", &methods.execute_async_task_method),
        ("cancel_async_task", &methods.cancel_async_task_method),
    ];

    for (method_name, method_option) in required_methods.iter() {
        if method_option.is_none() {
            return Err(syn::Error::new_spanned(
                input,
                format!(
                    "PluginManagerInterface implementation must include a '{}' method. See specification for required signature.",
                    method_name
                ),
            ));
        }
    }

    Ok(())
}

/// Generate CubeMelonPluginManagerInterface trait implementation
fn generate_manager_interface_impl(
    struct_name: &Ident,
    methods: &ManagerMethods,
) -> Result<TokenStream2, syn::Error> {
    let get_all_plugins_basic_info_method = &methods.get_all_plugins_basic_info_method.as_ref().unwrap().sig.ident;
    let get_plugin_detailed_info_method = &methods.get_plugin_detailed_info_method.as_ref().unwrap().sig.ident;
    let find_plugins_for_task_method = &methods.find_plugins_for_task_method.as_ref().unwrap().sig.ident;
    let is_plugin_alive_method = &methods.is_plugin_alive_method.as_ref().unwrap().sig.ident;
    let execute_task_method = &methods.execute_task_method.as_ref().unwrap().sig.ident;
    let execute_async_task_method = &methods.execute_async_task_method.as_ref().unwrap().sig.ident;
    let cancel_async_task_method = &methods.cancel_async_task_method.as_ref().unwrap().sig.ident;

    Ok(quote! {
        impl ::cubemelon_sdk::interfaces::manager::CubeMelonPluginManagerInterface for #struct_name {
            fn get_all_plugins_basic_info(
                &self,
                language: &::cubemelon_sdk::types::CubeMelonLanguage,
                out_infos: &mut ::cubemelon_sdk::memory::CubeMelonPluginBasicInfoArray,
            ) -> ::cubemelon_sdk::error::CubeMelonPluginErrorCode {
                self.#get_all_plugins_basic_info_method(language, out_infos)
            }

            fn get_plugin_detailed_info(
                &self,
                target_uuid: ::cubemelon_sdk::types::CubeMelonUUID,
                language: &::cubemelon_sdk::types::CubeMelonLanguage,
                out_detailed_json: &mut ::cubemelon_sdk::memory::CubeMelonString,
            ) -> ::cubemelon_sdk::error::CubeMelonPluginErrorCode {
                self.#get_plugin_detailed_info_method(target_uuid, language, out_detailed_json)
            }

            fn find_plugins_for_task(
                &self,
                task_json: &str,
                out_uuids: &mut ::cubemelon_sdk::memory::CubeMelonUUIDArray,
            ) -> ::cubemelon_sdk::error::CubeMelonPluginErrorCode {
                self.#find_plugins_for_task_method(task_json, out_uuids)
            }

            fn is_plugin_alive(
                &self,
                target_uuid: ::cubemelon_sdk::types::CubeMelonUUID,
            ) -> bool {
                self.#is_plugin_alive_method(target_uuid)
            }

            fn execute_task(
                &mut self,
                target_uuid: ::cubemelon_sdk::types::CubeMelonUUID,
                request: &::cubemelon_sdk::structs::CubeMelonTaskRequest,
                result: &mut ::cubemelon_sdk::structs::CubeMelonTaskResult,
            ) -> ::cubemelon_sdk::error::CubeMelonPluginErrorCode {
                self.#execute_task_method(target_uuid, request, result)
            }

            fn execute_async_task(
                &mut self,
                target_uuid: ::cubemelon_sdk::types::CubeMelonUUID,
                request: &::cubemelon_sdk::structs::CubeMelonTaskRequest,
                callback: ::cubemelon_sdk::structs::CubeMelonTaskCallback,
            ) -> ::cubemelon_sdk::error::CubeMelonPluginErrorCode {
                self.#execute_async_task_method(target_uuid, request, callback)
            }

            fn cancel_async_task(
                &mut self,
                request: &mut ::cubemelon_sdk::structs::CubeMelonTaskRequest,
            ) -> ::cubemelon_sdk::error::CubeMelonPluginErrorCode {
                self.#cancel_async_task_method(request)
            }
        }
    })
}

/// Generate compile-time markers and helper methods
fn generate_interface_implementation(struct_name: &Ident) -> TokenStream2 {
    quote! {
        // Add helper methods to the main plugin implementation
        impl #struct_name {
            /// Check if this plugin supports PluginManager interface
            /// 
            /// This is a helper method for use in supported_types() implementations.
            /// Add CubeMelonPluginType::Manager to your supported types.
            pub const fn __cubemelon_supports_manager() -> bool {
                true
            }
        }

        // Provide a compile-time check that Manager should be included in supported_types
        const _: () = {
            // This will generate a compile warning if Manager is not included in supported_types
            // (This is a development aid - in production, developers should ensure correct supported_types)
            
            // We can't easily check the supported_types() method at compile time,
            // so we provide this constant for documentation purposes
            const MANAGER_TYPE_VALUE: u64 = ::cubemelon_sdk::types::CubeMelonPluginType::Manager as u64;
            let _ = MANAGER_TYPE_VALUE; // Use the constant to avoid unused warnings
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_parse_manager_methods_complete() {
        let input: ItemImpl = parse_quote! {
            impl TestPlugin {
                pub fn get_all_plugins_basic_info(
                    &self,
                    language: &CubeMelonLanguage,
                    out_infos: &mut CubeMelonPluginBasicInfoArray,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }

                pub fn get_plugin_detailed_info(
                    &self,
                    target_uuid: CubeMelonUUID,
                    language: &CubeMelonLanguage,
                    out_detailed_json: &mut CubeMelonString,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }

                pub fn find_plugins_for_task(
                    &self,
                    task_json: &str,
                    out_uuids: &mut CubeMelonUUIDArray,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }

                pub fn is_plugin_alive(
                    &self,
                    target_uuid: CubeMelonUUID,
                ) -> bool {
                    true
                }

                pub fn execute_task(
                    &mut self,
                    target_uuid: CubeMelonUUID,
                    request: &CubeMelonTaskRequest,
                    result: &mut CubeMelonTaskResult,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }

                pub fn execute_async_task(
                    &mut self,
                    target_uuid: CubeMelonUUID,
                    request: &CubeMelonTaskRequest,
                    callback: CubeMelonTaskCallback,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }

                pub fn cancel_async_task(
                    &mut self,
                    request: &mut CubeMelonTaskRequest,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }
            }
        };
        
        let methods = parse_manager_methods(&input);
        assert!(methods.is_ok());
        
        let methods = methods.unwrap();
        assert!(methods.get_all_plugins_basic_info_method.is_some());
        assert!(methods.get_plugin_detailed_info_method.is_some());
        assert!(methods.find_plugins_for_task_method.is_some());
        assert!(methods.is_plugin_alive_method.is_some());
        assert!(methods.execute_task_method.is_some());
        assert!(methods.execute_async_task_method.is_some());
        assert!(methods.cancel_async_task_method.is_some());
    }

    #[test]
    fn test_parse_manager_methods_missing_required() {
        let input: ItemImpl = parse_quote! {
            impl TestPlugin {
                pub fn get_all_plugins_basic_info(
                    &self,
                    language: &CubeMelonLanguage,
                    out_infos: &mut CubeMelonPluginBasicInfoArray,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }
                // Missing other required methods
            }
        };
        
        let methods = parse_manager_methods(&input);
        assert!(methods.is_err());
    }

    #[test]
    fn test_generate_manager_interface_impl() {
        let struct_name = syn::parse_str::<Ident>("TestPlugin").unwrap();
        let methods = ManagerMethods {
            get_all_plugins_basic_info_method: Some(syn::parse_quote! {
                pub fn get_all_plugins_basic_info(
                    &self,
                    language: &CubeMelonLanguage,
                    out_infos: &mut CubeMelonPluginBasicInfoArray,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }
            }),
            get_plugin_detailed_info_method: Some(syn::parse_quote! {
                pub fn get_plugin_detailed_info(
                    &self,
                    target_uuid: CubeMelonUUID,
                    language: &CubeMelonLanguage,
                    out_detailed_json: &mut CubeMelonString,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }
            }),
            find_plugins_for_task_method: Some(syn::parse_quote! {
                pub fn find_plugins_for_task(
                    &self,
                    task_json: &str,
                    out_uuids: &mut CubeMelonUUIDArray,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }
            }),
            is_plugin_alive_method: Some(syn::parse_quote! {
                pub fn is_plugin_alive(
                    &self,
                    target_uuid: CubeMelonUUID,
                ) -> bool {
                    true
                }
            }),
            execute_task_method: Some(syn::parse_quote! {
                pub fn execute_task(
                    &mut self,
                    target_uuid: CubeMelonUUID,
                    request: &CubeMelonTaskRequest,
                    result: &mut CubeMelonTaskResult,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }
            }),
            execute_async_task_method: Some(syn::parse_quote! {
                pub fn execute_async_task(
                    &mut self,
                    target_uuid: CubeMelonUUID,
                    request: &CubeMelonTaskRequest,
                    callback: CubeMelonTaskCallback,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }
            }),
            cancel_async_task_method: Some(syn::parse_quote! {
                pub fn cancel_async_task(
                    &mut self,
                    request: &mut CubeMelonTaskRequest,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }
            }),
            other_methods: Vec::new(),
        };

        let result = generate_manager_interface_impl(&struct_name, &methods);
        assert!(result.is_ok());
    }
}