//! Hello World Plugin - A minimal CubeMelon plugin example
//! 
//! This plugin demonstrates the simplest possible plugin implementation
//! without using procedural macros. It provides:
//! - Basic plugin information (PLUGIN_TYPE_BASIC)
//! - Plugin lifecycle management (initialize/uninitialize)
//! - Manual C ABI implementation
//! - Logging "Hello, World!" during initialization

use cubemelon_sdk::prelude::*;

/// Hello World Plugin Structure
/// 
/// This is the simplest possible plugin - it only provides basic information
/// and logs messages during initialization/uninitialization.
pub struct Plugin {
    initialized: bool,
    host_services: Option<CubeMelonHostServices>,
}

impl Plugin {
    pub fn new() -> Self {
        Self {
            initialized: false,
            host_services: None,
        }
    }

    fn log_message(&self, level: CubeMelonLogLevel, message: &str) {
        if let Some(ref services) = self.host_services {
            services.log_message(level, "HelloWorldPlugin", message);
        }
    }
}

/// Implement the base plugin trait
impl PluginBase for Plugin {
    fn get_uuid() -> CubeMelonUUID {
        uuid!("550e8400-e29b-41d4-a716-446655440001")
    }

    fn get_version() -> CubeMelonVersion {
        version!(1, 0, 0)
    }

    fn get_supported_types() -> u64 {
        // PLUGIN_TYPE_BASIC - This plugin only provides basic interface
        CubeMelonPluginType::Basic as u64
    }

    fn is_thread_safe() -> bool {
        true
    }

    fn get_thread_requirements() -> u32 {
        // No special requirements
        CubeMelonThreadRequirements::NoRequirements as u32
    }

    fn get_name(&self, language: CubeMelonLanguage) -> *const u8 {
        multilang_map!(language, "Hello World Plugin", {
            "ja-JP" => "ハローワールドプラグイン",
            "zh-CN" => "你好世界插件", 
            "ko-KR" => "헬로 월드 플러그인",
            "ar-SA" => "مكون إضافة مرحبا بالعالم",
            "ru-RU" => "Плагин Hello World",
            "fr-FR" => "Plugin Hello World",
            "de-DE" => "Hallo-Welt-Plugin",
        })
    }

    fn get_description(&self, language: CubeMelonLanguage) -> *const u8 {
        multilang_map!(language, "The simplest possible plugin example. Provides only the basic interface.", {
            "ja-JP" => "最もシンプルなプラグインの例です。基本インターフェイスのみを提供します。",
            "zh-CN" => "最简单的插件示例。仅提供基本接口。",
            "ko-KR" => "가장 간단한 플러그인 예제입니다. 기본 인터페이스만 제공합니다.",
            "ar-SA" => "أبسط مثال ممكن للمكونات الإضافية. يوفر الواجهة الأساسية فقط.",
            "ru-RU" => "Простейший пример плагина. Предоставляет только базовый интерфейс.",
            "fr-FR" => "L'exemple de plugin le plus simple possible. Fournit seulement l'interface de base.",
            "de-DE" => "Das einfachste mögliche Plugin-Beispiel. Stellt nur die grundlegende Schnittstelle bereit.",
        })
    }
    
    fn initialize(
        &mut self,
        _host_plugin: Option<&CubeMelonPlugin>,
        _host_interface: Option<&CubeMelonInterface>,
        host_services: Option<&CubeMelonHostServices>,
    ) -> Result<(), CubeMelonPluginErrorCode> {
        if self.initialized {
            return Err(CubeMelonPluginErrorCode::AlreadyInitialized);
        }

        if let Some(services) = host_services {
            self.host_services = Some(*services);
        }

        self.initialized = true;
        
        // This is where the "Hello, World!" magic happens!
        self.log_message(CubeMelonLogLevel::Info, "Hello, World!");
        self.log_message(CubeMelonLogLevel::Info, "Hello World Plugin initialized successfully");
        
        Ok(())
    }

    fn uninitialize(&mut self) -> Result<(), CubeMelonPluginErrorCode> {
        if !self.initialized {
            return Err(CubeMelonPluginErrorCode::NotInitialized);
        }

        self.log_message(CubeMelonLogLevel::Info, "Goodbye, World!");
        self.log_message(CubeMelonLogLevel::Info, "Hello World Plugin uninitialized");
        
        self.initialized = false;
        self.host_services = None;
        Ok(())
    }
}

// === C ABI Implementation ===

/// C ABI: Get plugin UUID
#[no_mangle]
pub extern "C" fn get_plugin_uuid() -> CubeMelonUUID {
    Plugin::get_uuid()
}

/// C ABI: Get plugin SDK version
#[no_mangle]
pub extern "C" fn get_plugin_sdk_version() -> CubeMelonVersion {
    cubemelon_sdk::SDK_VERSION
}

/// C ABI: Get plugin version
#[no_mangle]
pub extern "C" fn get_plugin_version() -> CubeMelonVersion {
    Plugin::get_version()
}

/// C ABI: Get supported plugin types
#[no_mangle]
pub extern "C" fn get_plugin_supported_types() -> u64 {
    Plugin::get_supported_types()
}

/// C ABI: Create plugin instance
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut CubeMelonPlugin {
    let plugin = Plugin::new();
    create_plugin_instance(plugin)
}

/// C ABI: Get plugin interface
#[no_mangle]
pub extern "C" fn get_plugin_interface(plugin: *const CubeMelonPlugin) -> *const CubeMelonInterface {
    if plugin.is_null() {
        return std::ptr::null();
    }

    // Create interface with our implementations
    static INTERFACE: CubeMelonInterface = CubeMelonInterface {
        get_uuid: get_plugin_uuid,
        get_version: get_plugin_version,
        get_supported_types: c_get_supported_types,
        is_thread_safe: c_is_thread_safe,
        get_thread_requirements: c_get_thread_requirements,
        get_name: c_get_name,
        get_description: c_get_description,
        get_interface: c_get_interface,
        initialize: c_initialize,
        uninitialize: c_uninitialize,
    };

    &INTERFACE
}

/// C ABI: Destroy plugin instance
#[no_mangle]
pub extern "C" fn destroy_plugin(plugin: *mut CubeMelonPlugin) {
    destroy_plugin_instance(plugin);
}

/// C ABI: Destroy plugin instance
#[no_mangle]
pub extern "C" fn can_unload_now() -> bool {
    get_plugin_ref_count() == 0
}

// === C ABI Interface Implementation ===

extern "C" fn c_get_supported_types() -> u64 {
    Plugin::get_supported_types()
}

extern "C" fn c_is_thread_safe() -> bool {
    Plugin::is_thread_safe()
}

extern "C" fn c_get_thread_requirements() -> u32 {
    Plugin::get_thread_requirements()
}

extern "C" fn c_get_name(
    plugin: *const CubeMelonPlugin,
    language: CubeMelonLanguage,
) -> *const u8 {
    with_plugin::<Plugin, _, _>(plugin, |p| {
        p.get_name(language)
    }).unwrap_or(std::ptr::null())
}

extern "C" fn c_get_description(
    plugin: *const CubeMelonPlugin,
    language: CubeMelonLanguage,
) -> *const u8 {
    with_plugin::<Plugin, _, _>(plugin, |p| {
        p.get_description(language)
    }).unwrap_or(std::ptr::null())
}

extern "C" fn c_get_interface(
    plugin: *mut CubeMelonPlugin,
    plugin_types: u64,
    plugin_version: u32,
    interface: *mut *const std::ffi::c_void,
) -> CubeMelonPluginErrorCode {
    with_plugin::<Plugin, _, _>(plugin, |p| {
        match p.get_interface(plugin_types, plugin_version) {
            Ok(interface_ptr) => {
                unsafe {
                    *interface = interface_ptr;
                }
                CubeMelonPluginErrorCode::Success
            }
            Err(err) => err,
        }
    }).unwrap_or(CubeMelonPluginErrorCode::InterfaceNotSupported)
}

extern "C" fn c_initialize(
    plugin: *mut CubeMelonPlugin,
    host_plugin: *const CubeMelonPlugin,
    host_interface: *const CubeMelonInterface,
    host_services: *const CubeMelonHostServices,
) -> CubeMelonPluginErrorCode {
    if plugin.is_null() {
        return CubeMelonPluginErrorCode::NullPointer;
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

    with_plugin_mut::<Plugin, _, _>(plugin, |p| {
        match p.initialize(host_plugin_opt, host_interface_opt, host_services_opt) {
            Ok(()) => CubeMelonPluginErrorCode::Success,
            Err(err) => err,
        }
    }).unwrap_or(CubeMelonPluginErrorCode::InterfaceNotSupported)
}

extern "C" fn c_uninitialize(plugin: *mut CubeMelonPlugin) -> CubeMelonPluginErrorCode {
    with_plugin_mut::<Plugin, _, _>(plugin, |p| {
        match p.uninitialize() {
            Ok(()) => CubeMelonPluginErrorCode::Success,
            Err(err) => err,
        }
    }).unwrap_or(CubeMelonPluginErrorCode::Unknown)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = Plugin::new();
        assert!(!plugin.initialized);
        assert!(plugin.host_services.is_none());
    }

    #[test]
    fn test_plugin_metadata() {
        assert_eq!(Plugin::get_version(), CubeMelonVersion::new(1, 0, 0));
        assert_eq!(Plugin::get_supported_types(), 0); // PLUGIN_TYPE_BASIC
        assert!(Plugin::is_thread_safe());
    }

    #[test]
    fn test_c_abi_functions() {
        // Test C ABI entry points
        let uuid = get_plugin_uuid();
        assert_ne!(uuid.bytes, [0; 16]);

        let version = get_plugin_version();
        assert_eq!(version.major, 1);

        let types = get_plugin_supported_types();
        assert_eq!(types, 0); // PLUGIN_TYPE_BASIC
    }

    #[test]
    fn test_plugin_lifecycle() {
        let plugin_ptr = create_plugin();
        assert!(!plugin_ptr.is_null());

        let interface_ptr = get_plugin_interface(plugin_ptr);
        assert!(!interface_ptr.is_null());

        destroy_plugin(plugin_ptr);
    }
}