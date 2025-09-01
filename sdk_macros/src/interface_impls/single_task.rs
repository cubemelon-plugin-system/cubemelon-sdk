//! Single Task Interface procedural macro implementation
//! 
//! This module implements the #[single_task_plugin_impl] procedural macro
//! that generates C ABI code for the SingleTaskInterface.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{ImplItem, ItemImpl, Ident};

/// Process the #[single_task_plugin_impl] attribute
/// 
/// This generates C ABI code for plugins that implement synchronous task execution.
pub fn process_single_task_impl_attribute(
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
                "#[single_task_plugin_impl] can only be applied to impl blocks for named structs",
            ));
        }
    };

    // Parse the methods in the impl block
    let single_task_methods = parse_single_task_methods(&input)?;

    // Generate all the code components
    let original_impl = quote! { #input };
    let single_task_interface_impl = generate_single_task_interface_impl(struct_name, &single_task_methods)?;
    let interface_implementation = generate_interface_implementation(struct_name);

    Ok(quote! {
        // Include the original impl block
        #original_impl
        
        // Generate CubeMelonSingleTaskInterface trait implementation
        #single_task_interface_impl
        
        // Generate interface implementation methods for SingleTask interface
        #interface_implementation
        
    })
}

/// Parsed single task methods from the impl block
struct SingleTaskMethods {
    execute_method: Option<syn::ImplItemFn>,
    other_methods: Vec<syn::ImplItem>,
}

/// Parse methods from the single task impl block
fn parse_single_task_methods(input: &ItemImpl) -> Result<SingleTaskMethods, syn::Error> {
    let mut methods = SingleTaskMethods {
        execute_method: None,
        other_methods: Vec::new(),
    };

    for item in &input.items {
        if let ImplItem::Fn(method) = item {
            let method_name = method.sig.ident.to_string();
            
            match method_name.as_str() {
                "execute" => methods.execute_method = Some(method.clone()),
                _ => methods.other_methods.push(item.clone()),
            }
        } else {
            // Non-method items (associated types, constants, etc.)
            methods.other_methods.push(item.clone());
        }
    }

    // Validate required methods are present
    if methods.execute_method.is_none() {
        return Err(syn::Error::new_spanned(
            input,
            "SingleTaskInterface implementation must include an 'execute' method with signature: execute(&mut self, request: &CubeMelonTaskRequest, result: &mut CubeMelonTaskResult) -> CubeMelonPluginErrorCode",
        ));
    }

    // Validate execute method signature
    if let Some(execute_method) = &methods.execute_method {
        validate_execute_method_signature(execute_method)?;
    }

    Ok(methods)
}

/// Validate the execute method has the correct signature
fn validate_execute_method_signature(method: &syn::ImplItemFn) -> Result<(), syn::Error> {
    let sig = &method.sig;
    
    // Should have 3 parameters: &mut self, &CubeMelonTaskRequest, &mut CubeMelonTaskResult
    if sig.inputs.len() != 3 {
        return Err(syn::Error::new_spanned(
            sig,
            "execute method must have exactly 3 parameters: (&mut self, &CubeMelonTaskRequest, &mut CubeMelonTaskResult)",
        ));
    }

    // Check return type is CubeMelonPluginErrorCode
    if let syn::ReturnType::Type(_, return_type) = &sig.output {
        if let syn::Type::Path(type_path) = return_type.as_ref() {
            if let Some(segment) = type_path.path.segments.last() {
                if segment.ident != "CubeMelonPluginErrorCode" {
                    return Err(syn::Error::new_spanned(
                        return_type,
                        "execute method must return CubeMelonPluginErrorCode",
                    ));
                }
            }
        }
    } else {
        return Err(syn::Error::new_spanned(
            sig,
            "execute method must return CubeMelonPluginErrorCode",
        ));
    }

    Ok(())
}

/// Generate CubeMelonSingleTaskInterface trait implementation
fn generate_single_task_interface_impl(
    struct_name: &Ident,
    methods: &SingleTaskMethods,
) -> Result<TokenStream2, syn::Error> {
    let execute_method = &methods.execute_method.as_ref().unwrap().sig.ident;

    Ok(quote! {
        impl ::cubemelon_sdk::interfaces::single_task::CubeMelonSingleTaskInterface for #struct_name {
            fn execute(
                &mut self,
                request: &::cubemelon_sdk::structs::CubeMelonTaskRequest,
                result: &mut ::cubemelon_sdk::structs::CubeMelonTaskResult,
            ) -> ::cubemelon_sdk::error::CubeMelonPluginErrorCode {
                self.#execute_method(request, result)
            }
        }
    })
}

/// Generate compile-time markers and helper methods
fn generate_interface_implementation(struct_name: &Ident) -> TokenStream2 {
    quote! {
        // Add helper methods to the main plugin implementation
        impl #struct_name {
            /// Check if this plugin supports SingleTask interface
            /// 
            /// This is a helper method for use in supported_types() implementations.
            /// Add CubeMelonPluginType::SingleTask to your supported types.
            pub const fn __cubemelon_supports_single_task() -> bool {
                true
            }
        }

        // Provide a compile-time check that SingleTask should be included in supported_types
        const _: () = {
            // This will generate a compile warning if SingleTask is not included in supported_types
            // (This is a development aid - in production, developers should ensure correct supported_types)
            
            // We can't easily check the supported_types() method at compile time,
            // so we provide this constant for documentation purposes
            const SINGLE_TASK_TYPE_VALUE: u64 = ::cubemelon_sdk::types::CubeMelonPluginType::SingleTask as u64;
            let _ = SINGLE_TASK_TYPE_VALUE; // Use the constant to avoid unused warnings
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_parse_single_task_methods_with_execute() {
        let input: ItemImpl = parse_quote! {
            impl TestPlugin {
                pub fn execute(
                    &mut self,
                    request: &CubeMelonTaskRequest,
                    result: &mut CubeMelonTaskResult,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }
            }
        };
        
        let methods = parse_single_task_methods(&input);
        assert!(methods.is_ok());
        
        let methods = methods.unwrap();
        assert!(methods.execute_method.is_some());
    }

    #[test]
    fn test_parse_single_task_methods_missing_execute() {
        let input: ItemImpl = parse_quote! {
            impl TestPlugin {
                pub fn some_other_method(&self) {
                    // Not the required execute method
                }
            }
        };
        
        let methods = parse_single_task_methods(&input);
        assert!(methods.is_err());
    }

    #[test]
    fn test_validate_execute_method_signature_correct() {
        let method: syn::ImplItemFn = parse_quote! {
            pub fn execute(
                &mut self,
                request: &CubeMelonTaskRequest,
                result: &mut CubeMelonTaskResult,
            ) -> CubeMelonPluginErrorCode {
                CubeMelonPluginErrorCode::Success
            }
        };
        
        let result = validate_execute_method_signature(&method);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_execute_method_signature_wrong_return_type() {
        let method: syn::ImplItemFn = parse_quote! {
            pub fn execute(
                &mut self,
                request: &CubeMelonTaskRequest,
                result: &mut CubeMelonTaskResult,
            ) -> bool {
                true
            }
        };
        
        let result = validate_execute_method_signature(&method);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_execute_method_signature_wrong_param_count() {
        let method: syn::ImplItemFn = parse_quote! {
            pub fn execute(&mut self) -> CubeMelonPluginErrorCode {
                CubeMelonPluginErrorCode::Success
            }
        };
        
        let result = validate_execute_method_signature(&method);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_single_task_interface_impl() {
        let struct_name = syn::parse_str::<Ident>("TestPlugin").unwrap();
        let methods = SingleTaskMethods {
            execute_method: Some(syn::parse_quote! {
                pub fn execute(
                    &mut self,
                    request: &CubeMelonTaskRequest,
                    result: &mut CubeMelonTaskResult,
                ) -> CubeMelonPluginErrorCode {
                    CubeMelonPluginErrorCode::Success
                }
            }),
            other_methods: Vec::new(),
        };

        let result = generate_single_task_interface_impl(&struct_name, &methods);
        assert!(result.is_ok());
    }
}