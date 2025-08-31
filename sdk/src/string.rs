//! String processing utilities
//! 
//! Provides string processing functionality for the CubeMelon plugin system.
//! Includes multilingual support, UTF-8/UTF-16 conversion, and C string interoperability.

use std::ffi::{CString};
use std::os::raw::c_char;

use crate::types::CubeMelonLanguage;

// =============================================================================
// Basic macro collection
// =============================================================================

/// Create C string literal (static memory, NULL-terminated)
#[macro_export]
macro_rules! c_str_literal {
    ($s:expr) => {{
        static STORAGE: std::sync::OnceLock<std::ffi::CString> = std::sync::OnceLock::new();
        let cstr = STORAGE.get_or_init(|| std::ffi::CString::new($s).unwrap());
        cstr.as_ptr() as *const u8
    }};
}

/// Create static CubeMelonString (no deallocation needed)
#[macro_export]
macro_rules! static_cubemelon_string {
    ($s:expr) => {
        $crate::memory::CubeMelonString::from_static_str($s)
    };
}

/// Multilingual string map
#[macro_export]
macro_rules! multilang_map {
    ($lang:expr, $default:expr, { $($code:expr => $text:expr),* $(,)? }) => {{
        let lang_code = $lang.as_str();
        match lang_code {
            $($code => c_str_literal!($text),)*
            _ => c_str_literal!($default),
        }
    }};
}

/// Multilingual error message macro
#[macro_export]
macro_rules! error_message {
    ($lang:expr, $code:expr, { $($error_code:literal => { ja => $ja:expr, en => $en:expr }),* $(,)? }) => {{
        let lang_code = unsafe {
            std::ffi::CStr::from_ptr($lang.code as *const std::os::raw::c_char)
                .to_str()
                .unwrap_or("en")
        };
        
        match $code {
            $(
                $error_code => match lang_code {
                    "ja" | "ja-JP" => c_str_literal!($ja),
                    _ => c_str_literal!($en),
                },
            )*
            _ => match lang_code {
                "ja" | "ja-JP" => c_str_literal!("不明なエラー"),
                _ => c_str_literal!("Unknown error"),
            },
        }
    }};
}

// =============================================================================
// UTF-8 validation and conversion utilities
// =============================================================================

/// Convert C string pointer to Rust string slice safely
pub fn c_str_to_str<'a>(ptr: *const u8) -> Result<&'a str, std::str::Utf8Error> {
    if ptr.is_null() {
        return Ok("");
    }
    
    unsafe {
        std::ffi::CStr::from_ptr(ptr as *const i8).to_str()
    }
}

/// Convert C string pointer to owned Rust String safely
pub fn c_str_to_string(ptr: *const u8) -> Option<String> {
    c_str_to_str(ptr).ok().map(|s| s.to_owned())
}

/// Free dynamically allocated C string
pub unsafe fn free_c_string(ptr: *mut u8) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr as *mut c_char);
    }
}

// =============================================================================
// Windows UTF-16 conversion (Windows-specific)
// =============================================================================

#[cfg(windows)]
pub mod utf16 {
    use std::ffi::OsString;
    use std::os::windows::ffi::{OsStringExt, OsStrExt};
    
    /// Convert UTF-8 string to UTF-16
    pub fn utf8_to_utf16(utf8_str: &str) -> Vec<u16> {
        let os_string = OsString::from(utf8_str);
        let mut utf16: Vec<u16> = os_string.encode_wide().collect();
        utf16.push(0); // NULL termination
        utf16
    }
    
    /// Convert UTF-16 string to UTF-8
    pub fn utf16_to_utf8(utf16_ptr: *const u16) -> Option<String> {
        if utf16_ptr.is_null() {
            return None;
        }
        
        unsafe {
            // Calculate length up to NULL termination
            let mut len = 0;
            while *utf16_ptr.add(len) != 0 {
                len += 1;
            }
            
            let slice = std::slice::from_raw_parts(utf16_ptr, len);
            let os_string = OsString::from_wide(slice);
            os_string.into_string().ok()
        }
    }
    
    /// Convert UTF-8 string to wide string for Windows API
    pub fn to_wide_cstring(s: &str) -> Vec<u16> {
        utf8_to_utf16(s)
    }
}

// Empty implementation for non-Windows environments
#[cfg(not(windows))]
pub mod utf16 {
    pub fn utf8_to_utf16(_utf8_str: &str) -> Vec<u16> {
        vec![0] // NULL termination only
    }
    
    pub fn utf16_to_utf8(_utf16_ptr: *const u16) -> Option<String> {
        None
    }
    
    pub fn to_wide_cstring(_s: &str) -> Vec<u16> {
        vec![0]
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    #[test]
    fn test_static_string_macro() {
        let s = static_cubemelon_string!("Test string\0");
        assert!(!s.is_empty());
        assert_eq!(s.as_str().unwrap(), "Test string");
    }

}

// =============================================================================
// Usage examples (for documentation)
// =============================================================================

#[allow(dead_code)]
fn usage_examples() {
    // Basic usage examples
    let _name = c_str_literal!("ハローワールドプラグイン");
    let _string = static_cubemelon_string!("静的文字列\0");
    
    // Multilingual support example
    let lang = CubeMelonLanguage {
        code: b"ja-JP\0".as_ptr(),
    };

    let _description = 
    multilang_map!(lang, "This is a sample plugin", {
        "ja-JP" => "これはサンプルプラグインです",
        "zh-CN" => "这是一个示例插件",
    });

    // Error message example
    let _error_msg = error_message!(lang, 404, {
        404 => { ja => "ファイルが見つかりません", en => "File not found" },
        500 => { ja => "内部エラー", en => "Internal error" },
    });
}