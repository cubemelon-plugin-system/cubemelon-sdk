//! Plugin macro implementations
//! 
//! This module implements the #[plugin] and #[plugin_impl] procedural macros
//! that generate the necessary C ABI boilerplate for CubeMelon plugins.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse_macro_input, DeriveInput, ImplItem, ItemImpl, Lit, Meta,
    parse::Parse, parse::ParseStream, Token, punctuated::Punctuated, Expr, ExprLit
};

/// Custom NestedMeta replacement for syn 2.0
#[derive(Debug, Clone)]
#[allow(dead_code)] // Lit variant is for future extensibility
enum NestedMeta {
    Meta(Meta),
    Lit(Lit),
}

impl Parse for NestedMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Lit) {
            Ok(NestedMeta::Lit(input.parse()?))
        } else {
            Ok(NestedMeta::Meta(input.parse()?))
        }
    }
}

/// Custom attribute args parser (replaces deprecated AttributeArgs)
#[derive(Debug)]
struct AttributeArgs {
    args: Punctuated<NestedMeta, Token![,]>,
}

impl Parse for AttributeArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(AttributeArgs {
            args: input.parse_terminated(NestedMeta::parse, Token![,])?,
        })
    }
}

impl IntoIterator for AttributeArgs {
    type Item = NestedMeta;
    type IntoIter = syn::punctuated::IntoIter<NestedMeta>;

    fn into_iter(self) -> Self::IntoIter {
        self.args.into_iter()
    }
}

/// Implementation of the #[plugin] attribute macro
/// 
/// This macro is applied to struct definitions to mark them as plugins.
/// It performs validation and adds metadata for the plugin system.
pub fn plugin_attribute(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input_struct = parse_macro_input!(input as DeriveInput);
    
    match process_plugin_attribute(args, input_struct) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Implementation of the #[plugin_impl] attribute macro
/// 
/// This macro is applied to impl blocks to generate all C ABI code.
/// It's the heavy-lifting macro that creates the plugin infrastructure.
pub fn plugin_impl_attribute(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input_impl = parse_macro_input!(input as ItemImpl);
    
    match process_plugin_impl_attribute(args, input_impl) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Process the #[plugin] attribute
/// 
/// Currently this is a simple passthrough that validates the input
/// and adds some compile-time checks.
fn process_plugin_attribute(
    args: AttributeArgs,
    input: DeriveInput,
) -> Result<TokenStream2, syn::Error> {
    // Validate that it's applied to a struct
    match input.data {
        syn::Data::Struct(_) => {
            // Good - this is a struct
        }
        syn::Data::Enum(ref data_enum) => {
            return Err(syn::Error::new_spanned(
                data_enum.enum_token,
                "#[plugin] can only be applied to structs, not enums",
            ));
        }
        syn::Data::Union(ref data_union) => {
            return Err(syn::Error::new_spanned(
                data_union.union_token,
                "#[plugin] can only be applied to structs, not unions",
            ));
        }
    }

    // Parse any arguments (for future extensibility)
    let _plugin_config = parse_plugin_args(args)?;

    // For now, just pass through the original struct
    // In the future, we might add derive macros or additional fields
    let struct_name = &input.ident;
    let attrs = &input.attrs;
    let vis = &input.vis;
    let generics = &input.generics;

    // Properly handle the struct data
    let struct_body = match &input.data {
        syn::Data::Struct(data_struct) => {
            let fields = &data_struct.fields;
            match fields {
                syn::Fields::Named(_) => quote! { #fields },
                syn::Fields::Unnamed(_) => quote! { #fields },
                syn::Fields::Unit => quote! {},
            }
        }
        _ => unreachable!(), // We already validated this is a struct
    };

    Ok(quote! {
        #(#attrs)*
        #vis struct #struct_name #generics #struct_body
        
        // Add a compile-time marker to verify this struct was marked with #[plugin]
        const _: () = {
            // This will be used by #[plugin_impl] to verify the struct is properly marked
            impl #struct_name #generics {
                #[doc(hidden)]
                const __CUBEMELON_PLUGIN_MARKER: bool = true;
            }
        };
    })
}

/// Process the #[plugin_impl] attribute
/// 
/// This generates all the C ABI boilerplate code.
fn process_plugin_impl_attribute(
    args: AttributeArgs,
    input: ItemImpl,
) -> Result<TokenStream2, syn::Error> {
    // Parse any arguments (for future extensibility)
    let _impl_config = parse_plugin_impl_args(args)?;

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
                "#[plugin_impl] can only be applied to impl blocks for named structs",
            ));
        }
    };

    // Verify the struct was marked with #[plugin]
    // (This will be a compile-time check using the marker we added)
    
    // Parse the methods in the impl block
    let plugin_methods = parse_plugin_methods(&input)?;

    // Generate all the code components
    let original_impl = generate_original_impl(&input);
    let plugin_base_impl = generate_plugin_base_impl(struct_name, &plugin_methods)?;
    let c_abi_exports = generate_c_abi_exports(struct_name, &plugin_methods);
    let c_abi_interface = generate_c_abi_interface(struct_name, &plugin_methods);
    let c_abi_wrappers = generate_c_abi_wrappers(struct_name, &plugin_methods);

    Ok(quote! {
        // Include the original impl block
        #original_impl
        
        // Generate PluginBase trait implementation
        #plugin_base_impl
        
        // Generate C ABI export functions
        #c_abi_exports
        
        // Generate static CubeMelonInterface
        #c_abi_interface
        
        // Generate C ABI wrapper functions
        #c_abi_wrappers
    })
}

/// Configuration for #[plugin] attribute
#[derive(Default)]
struct PluginConfig {
    // Future: plugin-specific configuration options
}

/// Configuration for #[plugin_impl] attribute  
#[derive(Default)]
struct PluginImplConfig {
    // Future: implementation-specific configuration options
}

/// Parsed plugin methods from the impl block
struct PluginMethods {
    // Required methods
    get_uuid_method: Option<syn::ImplItemFn>,
    get_version_method: Option<syn::ImplItemFn>,
    get_supported_types_method: Option<syn::ImplItemFn>,
    
    // Optional methods
    is_thread_safe_method: Option<syn::ImplItemFn>,
    get_thread_requirements_method: Option<syn::ImplItemFn>,
    get_name_method: Option<syn::ImplItemFn>,
    get_description_method: Option<syn::ImplItemFn>,
    initialize_method: Option<syn::ImplItemFn>,
    uninitialize_method: Option<syn::ImplItemFn>,
    
    // Constructor method (new)
    new_method: Option<syn::ImplItemFn>,
    
    // Other methods (passed through)
    other_methods: Vec<syn::ImplItem>,
}

/// Parse plugin attribute arguments
fn parse_plugin_args(args: AttributeArgs) -> Result<PluginConfig, syn::Error> {
    let config = PluginConfig::default();
    
    for arg in args {
        match arg {
            NestedMeta::Meta(Meta::NameValue(meta)) if meta.path.is_ident("name") => {
                if let Expr::Lit(ExprLit { lit: Lit::Str(_), .. }) = meta.value {
                    // Future: handle plugin name configuration
                }
            }
            _ => {
                // For now, just ignore unknown arguments instead of erroring
                // This makes the macro more flexible for future extensions
            }
        }
    }
    
    Ok(config)
}

/// Parse plugin impl attribute arguments
fn parse_plugin_impl_args(args: AttributeArgs) -> Result<PluginImplConfig, syn::Error> {
    let config = PluginImplConfig::default();
    
    for arg in args {
        match arg {
            // Future: handle implementation-specific configuration
            _ => {
                // For now, just ignore unknown arguments
            }
        }
    }
    
    Ok(config)
}

/// Parse methods from the plugin impl block
fn parse_plugin_methods(input: &ItemImpl) -> Result<PluginMethods, syn::Error> {
    let mut methods = PluginMethods {
        get_uuid_method: None,
        get_version_method: None,
        get_supported_types_method: None,
        is_thread_safe_method: None,
        get_thread_requirements_method: None,
        get_name_method: None,
        get_description_method: None,
        initialize_method: None,
        uninitialize_method: None,
        new_method: None,
        other_methods: Vec::new(),
    };

    for item in &input.items {
        if let ImplItem::Fn(method) = item {
            let method_name = method.sig.ident.to_string();
            
            match method_name.as_str() {
                "get_uuid" => methods.get_uuid_method = Some(method.clone()),
                "get_version" => methods.get_version_method = Some(method.clone()),
                "get_supported_types" => methods.get_supported_types_method = Some(method.clone()),
                "is_thread_safe" => methods.is_thread_safe_method = Some(method.clone()),
                "get_thread_requirements" => methods.get_thread_requirements_method = Some(method.clone()),
                "get_name" => methods.get_name_method = Some(method.clone()),
                "get_description" => methods.get_description_method = Some(method.clone()),
                "initialize" => methods.initialize_method = Some(method.clone()),
                "uninitialize" => methods.uninitialize_method = Some(method.clone()),
                "new" => methods.new_method = Some(method.clone()),
                _ => methods.other_methods.push(item.clone()),
            }
        } else {
            // Non-method items (associated types, constants, etc.)
            methods.other_methods.push(item.clone());
        }
    }

    // Validate required methods are present
    if methods.get_uuid_method.is_none() {
        return Err(syn::Error::new_spanned(
            input,
            "Plugin implementation must include a 'uuid() -> CubeMelonUUID' method",
        ));
    }
    
    if methods.get_version_method.is_none() {
        return Err(syn::Error::new_spanned(
            input,
            "Plugin implementation must include a 'get_version() -> CubeMelonVersion' method",
        ));
    }
    
    if methods.get_supported_types_method.is_none() {
        return Err(syn::Error::new_spanned(
            input,
            "Plugin implementation must include a 'get_supported_types() -> u64' method",
        ));
    }

    Ok(methods)
}

/// Generate the original impl block (passthrough)
fn generate_original_impl(input: &ItemImpl) -> TokenStream2 {
    quote! { #input }
}

/// Generate PluginBase trait implementation
fn generate_plugin_base_impl(
    struct_name: &syn::Ident,
    methods: &PluginMethods,
) -> Result<TokenStream2, syn::Error> {
    // Generate calls to user-provided methods or defaults
    let uuid_method = &methods.get_uuid_method.as_ref().unwrap().sig.ident; // We validated this exists
    let uuid_call = quote! { Self::#uuid_method() };

    let version_method = &methods.get_version_method.as_ref().unwrap().sig.ident; // We validated this exists
    let version_call = quote! { Self::#version_method() };

    let supported_types_method = &methods.get_supported_types_method.as_ref().unwrap().sig.ident; // We validated this exists
    let supported_types_call = quote! { Self::#supported_types_method() };

    // Optional methods with defaults
    let is_thread_safe_impl = if let Some(method) = &methods.is_thread_safe_method {
        let method_name = &method.sig.ident;
        quote! { Self::#method_name() }
    } else {
        quote! { true } // Default: thread-safe
    };

    let get_thread_requirements_impl = if let Some(method) = &methods.get_thread_requirements_method {
        let method_name = &method.sig.ident;
        quote! { Self::#method_name() }
    } else {
        quote! { 0 } // Default: no special requirements
    };

    let get_name_impl = if let Some(method) = &methods.get_name_method {
        let method_name = &method.sig.ident;
        quote! { self.#method_name(language) }
    } else {
        quote! { b"Unnamed Plugin\0".as_ptr() } // Default name
    };

    let get_description_impl = if let Some(method) = &methods.get_description_method {
        let method_name = &method.sig.ident;
        quote! { self.#method_name(language) }
    } else {
        quote! { b"No description\0".as_ptr() } // Default description
    };

    let initialize_impl = if let Some(method) = &methods.initialize_method {
        let method_name = &method.sig.ident;
        quote! { self.#method_name(host_plugin, host_interface, host_services) }
    } else {
        quote! { Ok(()) } // Default: no-op initialization
    };

    let uninitialize_impl = if let Some(method) = &methods.uninitialize_method {
        let method_name = &method.sig.ident;
        quote! { self.#method_name() }
    } else {
        quote! { Ok(()) } // Default: no-op uninitialization
    };

    Ok(quote! {
        impl ::cubemelon_sdk::macros::PluginBase for #struct_name {
            fn get_uuid() -> ::cubemelon_sdk::types::CubeMelonUUID {
                #uuid_call
            }

            fn get_version() -> ::cubemelon_sdk::types::CubeMelonVersion {
                #version_call
            }

            fn get_supported_types() -> u64 {
                #supported_types_call
            }

            fn is_thread_safe() -> bool {
                #is_thread_safe_impl
            }

            fn get_thread_requirements() -> u32 {
                #get_thread_requirements_impl
            }

            fn get_name(&self, language: ::cubemelon_sdk::types::CubeMelonLanguage) -> *const u8 {
                #get_name_impl
            }

            fn get_description(&self, language: ::cubemelon_sdk::types::CubeMelonLanguage) -> *const u8 {
                #get_description_impl
            }

            fn initialize(
                &mut self,
                host_plugin: Option<&::cubemelon_sdk::instance::CubeMelonPlugin>,
                host_interface: Option<&::cubemelon_sdk::interfaces::CubeMelonInterface>,
                host_services: Option<&::cubemelon_sdk::structs::CubeMelonHostServices>,
            ) -> Result<(), ::cubemelon_sdk::error::CubeMelonPluginErrorCode> {
                #initialize_impl
            }

            fn uninitialize(&mut self) -> Result<(), ::cubemelon_sdk::error::CubeMelonPluginErrorCode> {
                #uninitialize_impl
            }
        }
    })
}

/// Generate C ABI export functions
fn generate_c_abi_exports(
    struct_name: &syn::Ident,
    methods: &PluginMethods,
) -> TokenStream2 {
    // Extract method names for calling user implementations
    let uuid_method = &methods.get_uuid_method.as_ref().unwrap().sig.ident;
    let version_method = &methods.get_version_method.as_ref().unwrap().sig.ident;
    let supported_types_method = &methods.get_supported_types_method.as_ref().unwrap().sig.ident;

    // Check if constructor exists
    let constructor_call = if methods.new_method.is_some() {
        quote! { #struct_name::new() }
    } else {
        quote! { #struct_name::default() }
    };

    quote! {
        /// C ABI: Get plugin UUID
        #[no_mangle]
        pub extern "C" fn get_plugin_uuid() -> ::cubemelon_sdk::types::CubeMelonUUID {
            #struct_name::#uuid_method()
        }

        /// C ABI: Get plugin SDK version
        #[no_mangle]
        pub extern "C" fn get_plugin_sdk_version() -> ::cubemelon_sdk::types::CubeMelonVersion {
            ::cubemelon_sdk::SDK_VERSION
        }

        /// C ABI: Get plugin version
        #[no_mangle]
        pub extern "C" fn get_plugin_version() -> ::cubemelon_sdk::types::CubeMelonVersion {
            #struct_name::#version_method()
        }

        /// C ABI: Get supported plugin types
        #[no_mangle]
        pub extern "C" fn get_plugin_supported_types() -> u64 {
            #struct_name::#supported_types_method()
        }

        /// C ABI: Create plugin instance
        #[no_mangle]
        pub extern "C" fn create_plugin() -> *mut ::cubemelon_sdk::instance::CubeMelonPlugin {
            let plugin = #constructor_call;
            ::cubemelon_sdk::instance::create_plugin_instance(plugin)
        }

        /// C ABI: Destroy plugin instance
        #[no_mangle]
        pub extern "C" fn destroy_plugin(plugin: *mut ::cubemelon_sdk::instance::CubeMelonPlugin) {
            ::cubemelon_sdk::instance::destroy_plugin_instance(plugin);
        }

        /// C ABI: Check if plugin can be unloaded
        #[no_mangle]
        pub extern "C" fn can_unload_now() -> bool {
            ::cubemelon_sdk::instance::get_plugin_ref_count() == 0
        }
    }
}

/// Generate static CubeMelonInterface
fn generate_c_abi_interface(
    struct_name: &syn::Ident,
    methods: &PluginMethods,
) -> TokenStream2 {
    // Extract method names for the wrapper functions
    let _uuid_method = &methods.get_uuid_method.as_ref().unwrap().sig.ident;
    let _version_method = &methods.get_version_method.as_ref().unwrap().sig.ident;
    let supported_types_method = &methods.get_supported_types_method.as_ref().unwrap().sig.ident;

    // Generate wrapper function names (these will be implemented in generate_c_abi_wrappers)
    let is_thread_safe_wrapper = if methods.is_thread_safe_method.is_some() {
        quote! { __cubemelon_c_is_thread_safe }
    } else {
        quote! { __cubemelon_c_is_thread_safe_default }
    };

    let get_thread_requirements_wrapper = if methods.get_thread_requirements_method.is_some() {
        quote! { __cubemelon_c_get_thread_requirements }
    } else {
        quote! { __cubemelon_c_get_thread_requirements_default }
    };

    quote! {
        /// Generated static CubeMelonInterface structure
        pub const __CUBEMELON_GENERATED_INTERFACE: ::cubemelon_sdk::interfaces::CubeMelonInterface = ::cubemelon_sdk::interfaces::CubeMelonInterface {
            // Static methods - can call directly
            get_uuid: get_plugin_uuid,
            get_version: get_plugin_version,
            get_supported_types: __cubemelon_c_get_supported_types,
            
            // Thread safety methods
            is_thread_safe: #is_thread_safe_wrapper,
            get_thread_requirements: #get_thread_requirements_wrapper,
            
            // Instance methods - need wrapper functions
            get_name: __cubemelon_c_get_name,
            get_description: __cubemelon_c_get_description,
            initialize: __cubemelon_c_initialize,
            uninitialize: __cubemelon_c_uninitialize,
        };

        /// C ABI wrapper: Get supported types
        extern "C" fn __cubemelon_c_get_supported_types() -> u64 {
            #struct_name::#supported_types_method()
        }

        /// C ABI wrapper: Is thread safe (default implementation)
        extern "C" fn __cubemelon_c_is_thread_safe_default() -> bool {
            true
        }

        /// C ABI wrapper: Get thread requirements (default implementation)  
        extern "C" fn __cubemelon_c_get_thread_requirements_default() -> u32 {
            0
        }
    }
}

/// Generate C ABI wrapper functions
fn generate_c_abi_wrappers(
    struct_name: &syn::Ident,
    methods: &PluginMethods,
) -> TokenStream2 {
    // Generate conditional wrappers for optional methods
    let is_thread_safe_wrapper = if let Some(method) = &methods.is_thread_safe_method {
        let method_name = &method.sig.ident;
        quote! {
            /// C ABI wrapper: Is thread safe (user implementation)
            extern "C" fn __cubemelon_c_is_thread_safe() -> bool {
                #struct_name::#method_name()
            }
        }
    } else {
        quote! {} // Default implementation already generated in generate_c_abi_interface
    };

    let get_thread_requirements_wrapper = if let Some(method) = &methods.get_thread_requirements_method {
        let method_name = &method.sig.ident;
        quote! {
            /// C ABI wrapper: Get thread requirements (user implementation)
            extern "C" fn __cubemelon_c_get_thread_requirements() -> u32 {
                #struct_name::#method_name()
            }
        }
    } else {
        quote! {} // Default implementation already generated in generate_c_abi_interface
    };

    // Generate name method wrapper
    let name_wrapper = if let Some(method) = &methods.get_name_method {
        let method_name = &method.sig.ident;
        quote! {
            /// C ABI wrapper: Get plugin name (user implementation)
            extern "C" fn __cubemelon_c_get_name(
                plugin: *const ::cubemelon_sdk::instance::CubeMelonPlugin,
                language: ::cubemelon_sdk::types::CubeMelonLanguage,
            ) -> *const u8 {
                ::cubemelon_sdk::instance::with_plugin::<#struct_name, _, _>(plugin, |p| {
                    p.#method_name(language)
                }).unwrap_or(std::ptr::null())
            }
        }
    } else {
        quote! {
            /// C ABI wrapper: Get plugin name (default implementation)
            extern "C" fn __cubemelon_c_get_name(
                _plugin: *const ::cubemelon_sdk::instance::CubeMelonPlugin,
                _language: ::cubemelon_sdk::types::CubeMelonLanguage,
            ) -> *const u8 {
                b"Unnamed Plugin\0".as_ptr()
            }
        }
    };

    // Generate description method wrapper
    let description_wrapper = if let Some(method) = &methods.get_description_method {
        let method_name = &method.sig.ident;
        quote! {
            /// C ABI wrapper: Get plugin description (user implementation)
            extern "C" fn __cubemelon_c_get_description(
                plugin: *const ::cubemelon_sdk::instance::CubeMelonPlugin,
                language: ::cubemelon_sdk::types::CubeMelonLanguage,
            ) -> *const u8 {
                ::cubemelon_sdk::instance::with_plugin::<#struct_name, _, _>(plugin, |p| {
                    p.#method_name(language)
                }).unwrap_or(std::ptr::null())
            }
        }
    } else {
        quote! {
            /// C ABI wrapper: Get plugin description (default implementation)
            extern "C" fn __cubemelon_c_get_description(
                _plugin: *const ::cubemelon_sdk::instance::CubeMelonPlugin,
                _language: ::cubemelon_sdk::types::CubeMelonLanguage,
            ) -> *const u8 {
                b"No description\0".as_ptr()
            }
        }
    };


    // Generate initialize method wrapper
    let initialize_wrapper = if let Some(method) = &methods.initialize_method {
        let method_name = &method.sig.ident;
        quote! {
            /// C ABI wrapper: Initialize plugin (user implementation)
            extern "C" fn __cubemelon_c_initialize(
                plugin: *mut ::cubemelon_sdk::instance::CubeMelonPlugin,
                host_plugin: *const ::cubemelon_sdk::instance::CubeMelonPlugin,
                host_interface: *const ::cubemelon_sdk::interfaces::CubeMelonInterface,
                host_services: *const ::cubemelon_sdk::structs::CubeMelonHostServices,
            ) -> ::cubemelon_sdk::error::CubeMelonPluginErrorCode {
                if plugin.is_null() {
                    return ::cubemelon_sdk::error::CubeMelonPluginErrorCode::NullPointer;
                }

                let host_plugin_opt = if host_plugin.is_null() {
                    None
                } else {
                    unsafe { Some(&*host_plugin) }
                };

                let host_interface_opt = if host_interface.is_null() {
                    None
                } else {
                    unsafe { Some(&*host_interface) }
                };

                let host_services_opt = if host_services.is_null() {
                    None
                } else {
                    unsafe { Some(&*host_services) }
                };

                ::cubemelon_sdk::instance::with_plugin_mut::<#struct_name, _, _>(plugin, |p| {
                    match p.#method_name(host_plugin_opt, host_interface_opt, host_services_opt) {
                        Ok(()) => ::cubemelon_sdk::error::CubeMelonPluginErrorCode::Success,
                        Err(err) => err,
                    }
                }).unwrap_or(::cubemelon_sdk::error::CubeMelonPluginErrorCode::PluginNotFound)
            }
        }
    } else {
        quote! {
            /// C ABI wrapper: Initialize plugin (default implementation)
            extern "C" fn __cubemelon_c_initialize(
                _plugin: *mut ::cubemelon_sdk::instance::CubeMelonPlugin,
                _host_plugin: *const ::cubemelon_sdk::instance::CubeMelonPlugin,
                _host_interface: *const ::cubemelon_sdk::interfaces::CubeMelonInterface,
                _host_services: *const ::cubemelon_sdk::structs::CubeMelonHostServices,
            ) -> ::cubemelon_sdk::error::CubeMelonPluginErrorCode {
                ::cubemelon_sdk::error::CubeMelonPluginErrorCode::Success
            }
        }
    };

    // Generate uninitialize method wrapper
    let uninitialize_wrapper = if let Some(method) = &methods.uninitialize_method {
        let method_name = &method.sig.ident;
        quote! {
            /// C ABI wrapper: Uninitialize plugin (user implementation)
            extern "C" fn __cubemelon_c_uninitialize(
                plugin: *mut ::cubemelon_sdk::instance::CubeMelonPlugin,
            ) -> ::cubemelon_sdk::error::CubeMelonPluginErrorCode {
                if plugin.is_null() {
                    return ::cubemelon_sdk::error::CubeMelonPluginErrorCode::NullPointer;
                }

                ::cubemelon_sdk::instance::with_plugin_mut::<#struct_name, _, _>(plugin, |p| {
                    match p.#method_name() {
                        Ok(()) => ::cubemelon_sdk::error::CubeMelonPluginErrorCode::Success,
                        Err(err) => err,
                    }
                }).unwrap_or(::cubemelon_sdk::error::CubeMelonPluginErrorCode::PluginNotFound)
            }
        }
    } else {
        quote! {
            /// C ABI wrapper: Uninitialize plugin (default implementation)
            extern "C" fn __cubemelon_c_uninitialize(
                _plugin: *mut ::cubemelon_sdk::instance::CubeMelonPlugin,
            ) -> ::cubemelon_sdk::error::CubeMelonPluginErrorCode {
                ::cubemelon_sdk::error::CubeMelonPluginErrorCode::Success
            }
        }
    };

    quote! {
        #is_thread_safe_wrapper
        #get_thread_requirements_wrapper
        #name_wrapper
        #description_wrapper
        #initialize_wrapper
        #uninitialize_wrapper
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_plugin_attribute_on_struct() {
        let input: DeriveInput = parse_quote! {
            pub struct TestPlugin {
                initialized: bool,
            }
        };
        
        let result = process_plugin_attribute(AttributeArgs { args: Default::default() }, input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_plugin_attribute_on_enum_fails() {
        let input: DeriveInput = parse_quote! {
            pub enum TestPlugin {
                Variant,
            }
        };
        
        let result = process_plugin_attribute(AttributeArgs { args: Default::default() }, input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_required_methods() {
        let input: ItemImpl = parse_quote! {
            impl TestPlugin {
                pub fn get_uuid() -> CubeMelonUUID {
                    uuid!("12345678-1234-5678-9abc-123456789abc")
                }
                
                pub fn get_version() -> CubeMelonVersion {
                    version!(1, 0, 0)
                }
                
                pub fn get_supported_types() -> u64 {
                    0
                }
            }
        };
        
        let methods = parse_plugin_methods(&input);
        assert!(methods.is_ok());
        
        let methods = methods.unwrap();
        assert!(methods.get_uuid_method.is_some());
        assert!(methods.get_version_method.is_some());
        assert!(methods.get_supported_types_method.is_some());
        assert!(methods.get_name_method.is_none()); // Optional method not provided
    }

    #[test]
    fn test_missing_required_method_fails() {
        let input: ItemImpl = parse_quote! {
            impl TestPlugin {
                pub fn uuid() -> CubeMelonUUID {
                    uuid!("12345678-1234-5678-9abc-123456789abc")
                }
                // Missing version and supported_types methods
            }
        };
        
        let methods = parse_plugin_methods(&input);
        assert!(methods.is_err());
    }
}

/// Generate get_plugin_interface function for specified interfaces
pub fn plugin_interface_attribute(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input_item = parse_macro_input!(input as syn::Item);
    
    // Extract struct name from the input (should be an impl block)
    let struct_name = match &input_item {
        syn::Item::Impl(item_impl) => {
            match &*item_impl.self_ty {
                syn::Type::Path(type_path) => {
                    if let Some(segment) = type_path.path.segments.last() {
                        &segment.ident
                    } else {
                        return syn::Error::new_spanned(
                            &item_impl.self_ty,
                            "Could not determine struct name from impl block"
                        )
                        .to_compile_error()
                        .into();
                    }
                }
                _ => {
                    return syn::Error::new_spanned(
                        &item_impl.self_ty,
                        "#[plugin_interface] can only be applied to impl blocks for named structs"
                    )
                    .to_compile_error()
                    .into();
                }
            }
        }
        _ => {
            return syn::Error::new_spanned(
                &input_item,
                "#[plugin_interface] can only be applied to impl blocks"
            )
            .to_compile_error()
            .into();
        }
    };
    
    // Parse interface list from args
    let mut interfaces = Vec::new();
    for arg in args {
        match arg {
            NestedMeta::Meta(Meta::Path(path)) => {
                if let Some(ident) = path.get_ident() {
                    interfaces.push(ident.to_string());
                }
            }
            _ => {
                return syn::Error::new_spanned(
                    &input_item,
                    "Expected interface names like: #[plugin_interface(single_task, async_task)]"
                )
                .to_compile_error()
                .into();
            }
        }
    }
    
    // Generate get_plugin_interface function
    let interface_handlers = generate_interface_handlers(struct_name, &interfaces);
    
    let result = quote! {
        #input_item
        
        #interface_handlers
    };
    
    result.into()
}

/// Generate interface handler code for get_plugin_interface
fn generate_interface_handlers(struct_name: &syn::Ident, interfaces: &[String]) -> TokenStream2 {
    let mut interface_checks = Vec::new();
    
    // Generate handlers for each specified interface
    for interface in interfaces {
        match interface.as_str() {
            "single_task" => {
                interface_checks.push(quote! {
                    // Handle SingleTask interface
                    if (plugin_types & (CubeMelonPluginType::SingleTask as u64)) != 0 {
                        let single_task_interface = ::cubemelon_sdk::interfaces::single_task::create_single_task_interface::<#struct_name>();
                        let boxed_interface = Box::new(single_task_interface);
                        unsafe {
                            *interface = Box::into_raw(boxed_interface) as *const std::ffi::c_void;
                        }
                        return ::cubemelon_sdk::error::CubeMelonPluginErrorCode::Success;
                    }
                });
            }
            "async_task" => {
                interface_checks.push(quote! {
                    // Handle AsyncTask interface
                    if (plugin_types & (CubeMelonPluginType::AsyncTask as u64)) != 0 {
                        let async_task_interface = ::cubemelon_sdk::interfaces::async_task::create_async_task_interface::<#struct_name>();
                        let boxed_interface = Box::new(async_task_interface);
                        unsafe {
                            *interface = Box::into_raw(boxed_interface) as *const std::ffi::c_void;
                        }
                        return ::cubemelon_sdk::error::CubeMelonPluginErrorCode::Success;
                    }
                });
            }
            "resident" => {
                interface_checks.push(quote! {
                    // Handle Resident interface
                    if (plugin_types & (CubeMelonPluginType::Resident as u64)) != 0 {
                        let resident_interface = ::cubemelon_sdk::interfaces::resident::create_resident_interface::<#struct_name>();
                        let boxed_interface = Box::new(resident_interface);
                        unsafe {
                            *interface = Box::into_raw(boxed_interface) as *const std::ffi::c_void;
                        }
                        return ::cubemelon_sdk::error::CubeMelonPluginErrorCode::Success;
                    }
                });
            }
            "state" => {
                interface_checks.push(quote! {
                    // Handle State interface
                    if (plugin_types & (CubeMelonPluginType::State as u64)) != 0 {
                        let state_interface = ::cubemelon_sdk::interfaces::state::create_state_interface::<#struct_name>();
                        let boxed_interface = Box::new(state_interface);
                        unsafe {
                            *interface = Box::into_raw(boxed_interface) as *const std::ffi::c_void;
                        }
                        return ::cubemelon_sdk::error::CubeMelonPluginErrorCode::Success;
                    }
                });
            }
            "manager" => {
                interface_checks.push(quote! {
                    // Handle Manager interface
                    if (plugin_types & (CubeMelonPluginType::Manager as u64)) != 0 {
                        let manager_interface = ::cubemelon_sdk::interfaces::manager::create_manager_interface::<#struct_name>();
                        let boxed_interface = Box::new(manager_interface);
                        unsafe {
                            *interface = Box::into_raw(boxed_interface) as *const std::ffi::c_void;
                        }
                        return ::cubemelon_sdk::error::CubeMelonPluginErrorCode::Success;
                    }
                });
            }
            _ => {
                // Ignore unknown interfaces for now (could add compile warning)
            }
        }
    }
    
    quote! {
        /// C ABI: Get plugin interface (generated by plugin_interface macro)
        #[no_mangle]
        pub extern "C" fn get_plugin_interface(
            plugin_types: u64,
            interface_version: u32,
            interface: *mut *const std::ffi::c_void,
        ) -> ::cubemelon_sdk::error::CubeMelonPluginErrorCode {
            if interface.is_null() {
                return ::cubemelon_sdk::error::CubeMelonPluginErrorCode::NullPointer;
            }

            // Check interface version (currently only version 1 is supported)
            if interface_version != 1 {
                unsafe { *interface = std::ptr::null(); }
                return ::cubemelon_sdk::error::CubeMelonPluginErrorCode::VersionMismatch;
            }

            use ::cubemelon_sdk::types::CubeMelonPluginType;

            #(#interface_checks)*

            // Default: return the basic interface
            if (plugin_types & (CubeMelonPluginType::Basic as u64)) != 0 || plugin_types == 0 {
                // Use the generated const interface from the plugin_impl macro
                unsafe {
                    *interface = &__CUBEMELON_GENERATED_INTERFACE as *const _ as *const std::ffi::c_void;
                }
                return ::cubemelon_sdk::error::CubeMelonPluginErrorCode::Success;
            }

            // If we reach here, no supported interface was found
            unsafe { *interface = std::ptr::null(); }
            ::cubemelon_sdk::error::CubeMelonPluginErrorCode::InterfaceNotSupported
        }
    }
}