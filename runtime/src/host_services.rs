//! Host Services Implementation
//! 
//! This module implements all host services provided to plugins,
//! including logging, system language detection, and utility functions.

use chrono::Local;

use cubemelon_sdk::{
    CubeMelonLanguage, CubeMelonLogLevel, CubeMelonPluginErrorCode, CubeMelonPluginType,
    CubeMelonPlugin, CubeMelonPluginManagerInterfaceImpl, CubeMelonPluginStateInterfaceImpl,
    create_plugin_instance, create_plugin_manager_interface, create_plugin_state_interface,
};
use std::ffi::c_void;
use std::sync::OnceLock;

use crate::RuntimeData;

/// Custom logging function for runtime to match plugin log format
pub fn runtime_log(level: CubeMelonLogLevel, message: &str) {
    let now = Local::now();
    let timestamp = now.format("%Y-%m-%d %H:%M:%S:%.3f");
    println!("{}> [{}] Runtime: {}", timestamp, level, message);
}

/// Helper function to convert CubeMelonLanguage to readable string
pub fn language_to_string(lang: &CubeMelonLanguage) -> String {
    if lang.code.is_null() {
        return "Unknown".to_string();
    }
    
    match unsafe { std::ffi::CStr::from_ptr(lang.code as *const i8) }.to_str() {
        Ok(code) => code.trim_end_matches('\0').to_string(),
        Err(_) => "Invalid".to_string(),
    }
}

/// System language callback function
/// This function returns the system language setting
pub unsafe extern "C" fn get_system_language_callback() -> CubeMelonLanguage {
    // Try to get system locale on Windows
    #[cfg(windows)]
    {
        use windows::Win32::Globalization::GetUserDefaultLCID;
        
        // Get user default locale
        let lcid = unsafe { GetUserDefaultLCID() };
        let lang_id = lcid & 0x3FF; // Extract primary language ID
        
        match lang_id {
            0x11 => CubeMelonLanguage::JA_JP, // Japanese
            0x09 => CubeMelonLanguage::EN_US, // English
            0x04 => CubeMelonLanguage::ZH_CN, // Chinese (Simplified)
            0x0C => CubeMelonLanguage::FR_FR, // French
            0x07 => CubeMelonLanguage::DE_DE, // German
            0x0A => CubeMelonLanguage::ES_ES, // Spanish
            _ => CubeMelonLanguage::EN_US,    // Default to English
        }
    }
    
    // For non-Windows systems, check environment variables
    #[cfg(not(windows))]
    {
        if let Ok(lang) = std::env::var("LANG") {
            if lang.starts_with("ja") {
                return CubeMelonLanguage::JA_JP;
            } else if lang.starts_with("zh") {
                return CubeMelonLanguage::ZH_CN;
            } else if lang.starts_with("fr") {
                return CubeMelonLanguage::FR_FR;
            } else if lang.starts_with("de") {
                return CubeMelonLanguage::DE_DE;
            } else if lang.starts_with("es") {
                return CubeMelonLanguage::ES_ES;
            }
        }
        CubeMelonLanguage::EN_US // Default to English
    }
}

/// Plugin log callback function
/// This function receives log messages from plugins and outputs them to standard output
pub unsafe extern "C" fn plugin_log_callback(
    level: CubeMelonLogLevel,
    plugin_name: *const u8,
    message: *const u8,
) {
    // Safely convert C strings to Rust strings
    let plugin_name_str = if plugin_name.is_null() {
        "Unknown Plugin"
    } else {
        match std::ffi::CStr::from_ptr(plugin_name as *const i8).to_str() {
            Ok(s) => s,
            Err(_) => "Unknown Plugin"
        }
    };
    
    let message_str = if message.is_null() {
        "Empty message"
    } else {
        match std::ffi::CStr::from_ptr(message as *const i8).to_str() {
            Ok(s) => s,
            Err(_) => "Invalid message encoding"
        }
    };
    
    // Format timestamp as YYYY-MM-DD hh:mm:ss:xxx.xxx> [CubeMelonLogLevel]
    let now = Local::now();
    let timestamp = now.format("%Y-%m-%d %H:%M:%S:%.3f");
    
    // Output log message with timestamp and level formatting in requested format
    match level {
        CubeMelonLogLevel::Error => println!("{}> [ERROR] {}: {}", timestamp, plugin_name_str, message_str),
        CubeMelonLogLevel::Warn => println!("{}> [WARN] {}: {}", timestamp, plugin_name_str, message_str),
        CubeMelonLogLevel::Info => println!("{}> [INFO] {}: {}", timestamp, plugin_name_str, message_str),
        CubeMelonLogLevel::Debug => println!("{}> [DEBUG] {}: {}", timestamp, plugin_name_str, message_str),
        CubeMelonLogLevel::Trace => println!("{}> [TRACE] {}: {}", timestamp, plugin_name_str, message_str),
    }
}

/// Host proxy type used to expose manager/state interfaces via SDK wrappers.
#[derive(Debug)]
pub struct HostRuntimeProxy;

unsafe impl Send for HostRuntimeProxy {}
unsafe impl Sync for HostRuntimeProxy {}

// Global pointer to active RuntimeData. Set on startup from main.
static mut RUNTIME_SINGLETON: *mut RuntimeData = std::ptr::null_mut();

/// Set the global runtime pointer. Called from main after creating runtime.
pub fn set_runtime_singleton(runtime: *mut RuntimeData) {
    unsafe { RUNTIME_SINGLETON = runtime; }
}

/// Execute closure with immutable runtime reference, if available.
pub(crate) fn with_runtime<R>(f: impl FnOnce(&RuntimeData) -> R) -> Option<R> {
    let ptr = unsafe { RUNTIME_SINGLETON };
    if ptr.is_null() { return None; }
    Some(unsafe { f(&*ptr) })
}

/// Execute closure with mutable runtime reference, if available.
pub(crate) fn with_runtime_mut<R>(f: impl FnOnce(&mut RuntimeData) -> R) -> Option<R> {
    let ptr = unsafe { RUNTIME_SINGLETON };
    if ptr.is_null() { return None; }
    Some(unsafe { f(&mut *ptr) })
}

// Lazily created proxy plugin instance and interface tables
// Store pointer address as usize to satisfy Send + Sync bounds for OnceLock
static HOST_PROXY_PLUGIN: OnceLock<usize> = OnceLock::new();
static MANAGER_VTABLE: OnceLock<CubeMelonPluginManagerInterfaceImpl> = OnceLock::new();
static STATE_VTABLE: OnceLock<CubeMelonPluginStateInterfaceImpl> = OnceLock::new();

fn ensure_proxy_plugin() -> *const CubeMelonPlugin {
    let addr = HOST_PROXY_PLUGIN
        .get_or_init(|| create_plugin_instance(HostRuntimeProxy) as usize);
    *addr as *const CubeMelonPlugin
}

/// Host callback to provide interfaces to plugins.
pub unsafe extern "C" fn get_host_interface_callback(
    interface_type: CubeMelonPluginType,
    interface_version: u32,
    plugin_out: *mut *const CubeMelonPlugin,
    interface_out: *mut *const c_void,
) -> CubeMelonPluginErrorCode {
    if plugin_out.is_null() || interface_out.is_null() {
        return CubeMelonPluginErrorCode::NullPointer;
    }
    if interface_version != 1 {
        return CubeMelonPluginErrorCode::VersionMismatch;
    }
    if RUNTIME_SINGLETON.is_null() {
        return CubeMelonPluginErrorCode::NotInitialized;
    }

    match interface_type {
        CubeMelonPluginType::Manager => {
            let vtbl = MANAGER_VTABLE
                .get_or_init(|| create_plugin_manager_interface::<HostRuntimeProxy>());
            *plugin_out = ensure_proxy_plugin();
            *interface_out = (vtbl as *const _) as *const c_void;
            CubeMelonPluginErrorCode::Success
        }
        CubeMelonPluginType::State => {
            let vtbl = STATE_VTABLE
                .get_or_init(|| create_plugin_state_interface::<HostRuntimeProxy>());
            *plugin_out = ensure_proxy_plugin();
            *interface_out = (vtbl as *const _) as *const c_void;
            CubeMelonPluginErrorCode::Success
        }
        _ => CubeMelonPluginErrorCode::InterfaceNotSupported,
    }
}

/// Parse language in strict BCP 47 canonical form.
/// - Case-sensitive: expects canonical tags like "en-US", "ja-JP".
/// - Unknown or non-canonical values fall back to "en-US".
pub fn parse_language(code: &str) -> CubeMelonLanguage {
    match code {
        // English (United States)
        "en-US" => CubeMelonLanguage::EN_US,
        // Japanese (Japan)
        "ja-JP" => CubeMelonLanguage::JA_JP,
        // Chinese (Simplified, China)
        "zh-CN" => CubeMelonLanguage::ZH_CN,
        // Chinese (Traditional, Taiwan)
        "zh-TW" => CubeMelonLanguage::ZH_TW,
        // Korean (Korea)
        "ko-KR" => CubeMelonLanguage::KO_KR,
        // French (France)
        "fr-FR" => CubeMelonLanguage::FR_FR,
        // German (Germany)
        "de-DE" => CubeMelonLanguage::DE_DE,
        // Spanish (Spain)
        "es-ES" => CubeMelonLanguage::ES_ES,
        // Italian (Italy)
        "it-IT" => CubeMelonLanguage::IT_IT,
        // Russian (Russia)
        "ru-RU" => CubeMelonLanguage::RU_RU,
        // Portuguese (Brazil)
        "pt-BR" => CubeMelonLanguage::PT_BR,
        // Arabic (Saudi Arabia)
        "ar-SA" => CubeMelonLanguage::AR_SA,
        // Turkish (Turkey)
        "tr-TR" => CubeMelonLanguage::TR_TR,
        // Persian (Iran)
        "fa-IR" => CubeMelonLanguage::FA_IR,
        // Greek (Greece)
        "el-GR" => CubeMelonLanguage::EL_GR,
        // Indonesian (Indonesia)
        "id-ID" => CubeMelonLanguage::ID_ID,
        // Vietnamese (Vietnam)
        "vi-VN" => CubeMelonLanguage::VI_VN,
        // Thai (Thailand)
        "th-TH" => CubeMelonLanguage::TH_TH,
        // Polish (Poland)
        "pl-PL" => CubeMelonLanguage::PL_PL,
        // Dutch (Netherlands)
        "nl-NL" => CubeMelonLanguage::NL_NL,
        // Swedish (Sweden)
        "sv-SE" => CubeMelonLanguage::SV_SE,
        // Danish (Denmark)
        "da-DK" => CubeMelonLanguage::DA_DK,
        // Norwegian (Norway)
        "no-NO" => CubeMelonLanguage::NO_NO,
        // Finnish (Finland)
        "fi-FI" => CubeMelonLanguage::FI_FI,
        // Ukrainian (Ukraine)
        "uk-UA" => CubeMelonLanguage::UK_UA,
        _ => CubeMelonLanguage::EN_US,
    }
}
