//! Error handling for the CubeMelon Plugin System
//! 
//! This module provides comprehensive error handling with C ABI compatible error codes
//! and Rust-native error types for better development experience.

use std::fmt;
use std::error::Error;
use crate::types::CubeMelonLanguage;

/// Plugin error codes compatible with C ABI
/// 
/// Success: 0, Failure: negative values, Information: positive values
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubeMelonPluginErrorCode {
    // Success
    Success = 0,
    
    // General errors (-1 ~ -19)
    Unknown = -1,                    // Unknown error
    InvalidParameter = -2,           // Invalid parameter
    NotSupported = -3,               // Unsupported operation
    MemoryAllocation = -4,           // Memory allocation failure
    NullPointer = -5,                // NULL pointer error
    OutOfBounds = -6,                // Out of bounds access
    InvalidState = -7,               // Invalid state
    PermissionDenied = -8,           // Access permission denied
    ResourceBusy = -9,               // Resource is busy
    ResourceExhausted = -10,         // Resource exhausted
    
    // Initialization related (-20 ~ -29)
    InitializationFailed = -20,      // Initialization failed
    AlreadyInitialized = -21,        // Already initialized
    NotInitialized = -22,            // Not initialized
    VersionMismatch = -23,           // Version mismatch
    Incompatible = -24,              // Incompatible
    
    // Plugin/Interface related (-30 ~ -39)
    PluginNotFound = -30,            // Plugin not found
    InterfaceNotSupported = -31,     // Interface not supported
    NotImplemented = -32,            // Not implemented
    PluginLoadFailed = -33,          // Plugin load failed
    PluginUnloadFailed = -34,        // Plugin unload failed
    
    // Communication/I/O related (-40 ~ -49)
    ConnectionFailed = -40,          // Connection failed
    Timeout = -41,                   // Timeout
    IO = -42,                        // I/O error
    Network = -43,                   // Network error
    Cancelled = -44,                 // Operation cancelled
    
    // Data/Parsing related (-50 ~ -59)
    Parse = -50,                     // Parse error
    Validation = -51,                // Validation error
    Encoding = -52,                  // Encoding error
    DataCorrupted = -53,             // Data corrupted
    FormatUnsupported = -54,         // Unsupported format
    
    // Synchronization/Concurrency related (-60 ~ -69)
    LockFailed = -60,                // Lock acquisition failed
    Deadlock = -61,                  // Deadlock detected
    State = -62,                     // State management error (RwLock, etc.)
    ThreadPanic = -63,               // Thread panic
    
    // File system related (-70 ~ -79)
    FileNotFound = -70,              // File not found
    FileExists = -71,                // File already exists
    DirectoryNotEmpty = -72,         // Directory not empty
    DiskFull = -73,                  // Disk full
    
    // Reserved for future expansion
    ReservedStart = -100,            // Data in this range
    ReservedEnd = -999,              // should be ignored
}

impl CubeMelonPluginErrorCode {
    /// Check if the error code represents success
    pub fn is_success(self) -> bool {
        self == CubeMelonPluginErrorCode::Success
    }

    /// Check if the error code represents an error
    pub fn is_error(self) -> bool {
        (self as i32) < 0
    }

    /// Check if the error code represents informational status
    pub fn is_info(self) -> bool {
        (self as i32) > 0
    }

    /// Convert error code to human-readable string
    pub fn to_message(self, _language: CubeMelonLanguage) -> &'static str {
        // For now, we only support English. In the future, this could
        // be expanded to support multiple languages based on the language parameter.
        match self {
            CubeMelonPluginErrorCode::Success => "Success",
            
            // General errors
            CubeMelonPluginErrorCode::Unknown => "Unknown error",
            CubeMelonPluginErrorCode::InvalidParameter => "Invalid parameter",
            CubeMelonPluginErrorCode::NotSupported => "Unsupported operation",
            CubeMelonPluginErrorCode::MemoryAllocation => "Memory allocation failure",
            CubeMelonPluginErrorCode::NullPointer => "NULL pointer error",
            CubeMelonPluginErrorCode::OutOfBounds => "Out of bounds access",
            CubeMelonPluginErrorCode::InvalidState => "Invalid state",
            CubeMelonPluginErrorCode::PermissionDenied => "Access permission denied",
            CubeMelonPluginErrorCode::ResourceBusy => "Resource is busy",
            CubeMelonPluginErrorCode::ResourceExhausted => "Resource exhausted",
            
            // Initialization related
            CubeMelonPluginErrorCode::InitializationFailed => "Initialization failed",
            CubeMelonPluginErrorCode::AlreadyInitialized => "Already initialized",
            CubeMelonPluginErrorCode::NotInitialized => "Not initialized",
            CubeMelonPluginErrorCode::VersionMismatch => "Version mismatch",
            CubeMelonPluginErrorCode::Incompatible => "Incompatible",
            
            // Plugin/Interface related
            CubeMelonPluginErrorCode::PluginNotFound => "Plugin not found",
            CubeMelonPluginErrorCode::InterfaceNotSupported => "Interface not supported",
            CubeMelonPluginErrorCode::NotImplemented => "Not implemented",
            CubeMelonPluginErrorCode::PluginLoadFailed => "Plugin load failed",
            CubeMelonPluginErrorCode::PluginUnloadFailed => "Plugin unload failed",
            
            // Communication/I/O related
            CubeMelonPluginErrorCode::ConnectionFailed => "Connection failed",
            CubeMelonPluginErrorCode::Timeout => "Timeout",
            CubeMelonPluginErrorCode::IO => "I/O error",
            CubeMelonPluginErrorCode::Network => "Network error",
            CubeMelonPluginErrorCode::Cancelled => "Operation cancelled",
            
            // Data/Parsing related
            CubeMelonPluginErrorCode::Parse => "Parse error",
            CubeMelonPluginErrorCode::Validation => "Validation error",
            CubeMelonPluginErrorCode::Encoding => "Encoding error",
            CubeMelonPluginErrorCode::DataCorrupted => "Data corrupted",
            CubeMelonPluginErrorCode::FormatUnsupported => "Unsupported format",
            
            // Synchronization/Concurrency related
            CubeMelonPluginErrorCode::LockFailed => "Lock acquisition failed",
            CubeMelonPluginErrorCode::Deadlock => "Deadlock detected",
            CubeMelonPluginErrorCode::State => "State management error",
            CubeMelonPluginErrorCode::ThreadPanic => "Thread panic",
            
            // File system related
            CubeMelonPluginErrorCode::FileNotFound => "File not found",
            CubeMelonPluginErrorCode::FileExists => "File already exists",
            CubeMelonPluginErrorCode::DirectoryNotEmpty => "Directory not empty",
            CubeMelonPluginErrorCode::DiskFull => "Disk full",
            
            // Reserved
            CubeMelonPluginErrorCode::ReservedStart |
            CubeMelonPluginErrorCode::ReservedEnd => "Reserved error code",
        }
    }
}

impl fmt::Display for CubeMelonPluginErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_message(CubeMelonLanguage::EN_US))
    }
}

impl Error for CubeMelonPluginErrorCode {}

/// Rust-native error type for better error handling in Rust code
#[derive(Debug)]
pub enum CubeMelonError {
    /// Plugin error with error code
    Plugin {
        code: CubeMelonPluginErrorCode,
        message: String,
    },
    /// I/O error
    Io(std::io::Error),
    /// UTF-8 conversion error
    Utf8(std::str::Utf8Error),
    /// NULL pointer dereference
    NullPointer,
    /// Custom error with message
    Custom(String),
}

impl fmt::Display for CubeMelonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CubeMelonError::Plugin { code, message } => {
                write!(f, "Plugin error {}: {}", *code as i32, message)
            }
            CubeMelonError::Io(err) => write!(f, "I/O error: {}", err),
            CubeMelonError::Utf8(err) => write!(f, "UTF-8 error: {}", err),
            CubeMelonError::NullPointer => write!(f, "NULL pointer dereference"),
            CubeMelonError::Custom(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl Error for CubeMelonError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CubeMelonError::Io(err) => Some(err),
            CubeMelonError::Utf8(err) => Some(err),
            CubeMelonError::Plugin { code, .. } => Some(code),
            _ => None,
        }
    }
}

/// Convenient Result type alias
pub type CubeMelonResult<T> = Result<T, CubeMelonError>;

/// Convenient Result type alias for C ABI functions
pub type CubeMelonPluginResult<T> = Result<T, CubeMelonPluginErrorCode>;

// Conversion implementations
impl From<CubeMelonPluginErrorCode> for CubeMelonError {
    fn from(code: CubeMelonPluginErrorCode) -> Self {
        CubeMelonError::Plugin {
            code,
            message: code.to_message(CubeMelonLanguage::EN_US).to_string(),
        }
    }
}

impl From<std::io::Error> for CubeMelonError {
    fn from(err: std::io::Error) -> Self {
        CubeMelonError::Io(err)
    }
}

impl From<std::str::Utf8Error> for CubeMelonError {
    fn from(err: std::str::Utf8Error) -> Self {
        CubeMelonError::Utf8(err)
    }
}

impl From<CubeMelonError> for CubeMelonPluginErrorCode {
    fn from(err: CubeMelonError) -> Self {
        match err {
            CubeMelonError::Plugin { code, .. } => code,
            CubeMelonError::Io(_) => CubeMelonPluginErrorCode::IO,
            CubeMelonError::Utf8(_) => CubeMelonPluginErrorCode::Encoding,
            CubeMelonError::NullPointer => CubeMelonPluginErrorCode::NullPointer,
            CubeMelonError::Custom(_) => CubeMelonPluginErrorCode::Unknown,
        }
    }
}

/// Helper macro for creating plugin errors
#[macro_export]
macro_rules! plugin_error {
    ($code:expr) => {
        CubeMelonError::Plugin {
            code: $code,
            message: $code.to_message(CubeMelonLanguage::EN_US).to_string(),
        }
    };
    ($code:expr, $msg:expr) => {
        CubeMelonError::Plugin {
            code: $code,
            message: $msg.to_string(),
        }
    };
}

/// Helper macro for early return on error
#[macro_export]
macro_rules! try_plugin {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => return Err(err.into()),
        }
    };
}

/// SDK-provided helper function for converting error codes to strings
/// 
/// This is the function that the specification mentions should be provided by the SDK.
pub fn plugin_error_code_to_string(
    code: CubeMelonPluginErrorCode,
    language: CubeMelonLanguage,
) -> &'static str {
    code.to_message(language)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_categorization() {
        assert!(CubeMelonPluginErrorCode::Success.is_success());
        assert!(!CubeMelonPluginErrorCode::Success.is_error());
        assert!(!CubeMelonPluginErrorCode::Success.is_info());

        assert!(!CubeMelonPluginErrorCode::Unknown.is_success());
        assert!(CubeMelonPluginErrorCode::Unknown.is_error());
        assert!(!CubeMelonPluginErrorCode::Unknown.is_info());
    }

    #[test]
    fn test_error_code_to_string() {
        let error_msg = CubeMelonPluginErrorCode::InvalidParameter
            .to_message(CubeMelonLanguage::EN_US);
        assert_eq!(error_msg, "Invalid parameter");

        let success_msg = CubeMelonPluginErrorCode::Success
            .to_message(CubeMelonLanguage::EN_US);
        assert_eq!(success_msg, "Success");
    }

    #[test]
    fn test_error_conversion() {
        let plugin_err = CubeMelonError::from(CubeMelonPluginErrorCode::Network);
        match plugin_err {
            CubeMelonError::Plugin { code, .. } => {
                assert_eq!(code, CubeMelonPluginErrorCode::Network);
            }
            _ => panic!("Expected Plugin error variant"),
        }

        let code = CubeMelonPluginErrorCode::from(plugin_err);
        assert_eq!(code, CubeMelonPluginErrorCode::Network);
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let plugin_err = CubeMelonError::from(io_err);
        
        match plugin_err {
            CubeMelonError::Io(_) => (),
            _ => panic!("Expected Io error variant"),
        }

        let code = CubeMelonPluginErrorCode::from(plugin_err);
        assert_eq!(code, CubeMelonPluginErrorCode::IO);
    }

    #[test]
    fn test_plugin_error_macro() {
        let err = plugin_error!(CubeMelonPluginErrorCode::Network);
        match err {
            CubeMelonError::Plugin { code, message } => {
                assert_eq!(code, CubeMelonPluginErrorCode::Network);
                assert_eq!(message, "Network error");
            }
            _ => panic!("Expected Plugin error variant"),
        }

        let err = plugin_error!(CubeMelonPluginErrorCode::Network, "Custom message");
        match err {
            CubeMelonError::Plugin { code, message } => {
                assert_eq!(code, CubeMelonPluginErrorCode::Network);
                assert_eq!(message, "Custom message");
            }
            _ => panic!("Expected Plugin error variant"),
        }
    }

    #[test]
    fn test_helper_function() {
        let msg = plugin_error_code_to_string(
            CubeMelonPluginErrorCode::Timeout,
            CubeMelonLanguage::EN_US,
        );
        assert_eq!(msg, "Timeout");
    }
}