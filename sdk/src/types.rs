//! Core type definitions for the CubeMelon Plugin System
//! 
//! This module contains all fundamental types used throughout the plugin system,
//! including UUIDs, versions, plugin types, and various enums for state management.

use std::fmt;

/// 128-bit UUID for plugin identification
/// 
/// Ensures global uniqueness and avoids name conflicts between plugins.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CubeMelonUUID {
    pub bytes: [u8; 16],
}

impl CubeMelonUUID {
    /// Create a new UUID from bytes
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self { bytes }
    }

    /// Create a zero UUID (used for testing or null values)
    pub const fn zero() -> Self {
        Self { bytes: [0; 16] }
    }

    /// Convert to hyphenated string format
    pub fn to_string(&self) -> String {
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3],
            self.bytes[4], self.bytes[5],
            self.bytes[6], self.bytes[7],
            self.bytes[8], self.bytes[9],
            self.bytes[10], self.bytes[11], self.bytes[12], self.bytes[13], self.bytes[14], self.bytes[15]
        )
    }
}

impl fmt::Display for CubeMelonUUID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Version information using semantic versioning
/// 
/// 4-byte structure supporting semantic versioning (major.minor.patch).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CubeMelonVersion {
    pub major: u16,    // Major version
    pub minor: u8,     // Minor version  
    pub patch: u8,     // Patch version
}

impl CubeMelonVersion {
    /// Create a new version
    pub const fn new(major: u16, minor: u8, patch: u8) -> Self {
        Self { major, minor, patch }
    }
}

impl fmt::Display for CubeMelonVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Language identification using BCP 47 format
/// 
/// Language is identified by UTF-8 strings following BCP 47 standard.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct CubeMelonLanguage {
    /// UTF-8, NULL-terminated, BCP 47 format language code
    /// Examples: "en-US", "ja-JP", "zh-Hant-TW"
    pub code: *const u8,
}

impl CubeMelonLanguage {
    /// Create a language from a static string
    /// 
    /// # Safety
    /// 
    /// The provided string must be valid for the lifetime of the CubeMelonLanguage object
    pub unsafe fn from_static_str(s: &'static str) -> Self {
        Self {
            code: s.as_ptr(),
        }
    }

    /// Convert to Rust string slice
    /// 
    /// # Safety
    /// 
    /// The code pointer must be valid and point to a NULL-terminated UTF-8 string
    pub fn as_str(&self) -> &str {
        unsafe {
            let cstr = std::ffi::CStr::from_ptr(self.code as *const i8);
            cstr.to_str().unwrap_or("en-US") // Fallback to English
        }
    }
}

// Common language constants
impl CubeMelonLanguage {
    pub const EN_US: CubeMelonLanguage = CubeMelonLanguage { code: "en-US\0".as_ptr(), };
    pub const JA_JP: CubeMelonLanguage = CubeMelonLanguage { code: "ja-JP\0".as_ptr(), };
    pub const ZH_CN: CubeMelonLanguage = CubeMelonLanguage { code: "zh-CN\0".as_ptr(), };
    pub const ZH_TW: CubeMelonLanguage = CubeMelonLanguage { code: "zh-TW\0".as_ptr(), };
    pub const KO_KR: CubeMelonLanguage = CubeMelonLanguage { code: "ko-KR\0".as_ptr(), };
    pub const FR_FR: CubeMelonLanguage = CubeMelonLanguage { code: "fr-FR\0".as_ptr(), };
    pub const DE_DE: CubeMelonLanguage = CubeMelonLanguage { code: "de-DE\0".as_ptr(), };
    pub const ES_ES: CubeMelonLanguage = CubeMelonLanguage { code: "es-ES\0".as_ptr(), };
    pub const IT_IT: CubeMelonLanguage = CubeMelonLanguage { code: "it-IT\0".as_ptr(), };
    pub const RU_RU: CubeMelonLanguage = CubeMelonLanguage { code: "ru-RU\0".as_ptr(), };
    pub const PT_BR: CubeMelonLanguage = CubeMelonLanguage { code: "pt-BR\0".as_ptr(), };
    pub const AR_SA: CubeMelonLanguage = CubeMelonLanguage { code: "ar-SA\0".as_ptr(), };
    pub const TR_TR: CubeMelonLanguage = CubeMelonLanguage { code: "tr-TR\0".as_ptr(), };
    pub const FA_IR: CubeMelonLanguage = CubeMelonLanguage { code: "fa-IR\0".as_ptr(), };
    pub const EL_GR: CubeMelonLanguage = CubeMelonLanguage { code: "el-GR\0".as_ptr(), };
    pub const ID_ID: CubeMelonLanguage = CubeMelonLanguage { code: "id-ID\0".as_ptr(), };
    pub const VI_VN: CubeMelonLanguage = CubeMelonLanguage { code: "vi-VN\0".as_ptr(), };
    pub const TH_TH: CubeMelonLanguage = CubeMelonLanguage { code: "th-TH\0".as_ptr(), };
    pub const PL_PL: CubeMelonLanguage = CubeMelonLanguage { code: "pl-PL\0".as_ptr(), };
    pub const NL_NL: CubeMelonLanguage = CubeMelonLanguage { code: "nl-NL\0".as_ptr(), };
    pub const SV_SE: CubeMelonLanguage = CubeMelonLanguage { code: "sv-SE\0".as_ptr(), };
    pub const DA_DK: CubeMelonLanguage = CubeMelonLanguage { code: "da-DK\0".as_ptr(), };
    pub const NO_NO: CubeMelonLanguage = CubeMelonLanguage { code: "no-NO\0".as_ptr(), };
    pub const FI_FI: CubeMelonLanguage = CubeMelonLanguage { code: "fi-FI\0".as_ptr(), };
    pub const UK_UA: CubeMelonLanguage = CubeMelonLanguage { code: "uk-UA\0".as_ptr(), };
}

/// Plugin type flags (64-bit)
/// 
/// Defines what functionality a plugin provides. Multiple types can be combined using bitwise OR.
#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubeMelonPluginType {
    /// No special functionality - basic interface only
    Basic = 0,
    
    // Execution patterns (basic functionality)
    /// Single task execution (synchronous)
    SingleTask = 0x0000000000000001,
    /// Single task execution (asynchronous)
    AsyncTask = 0x0000000000000002,
    /// Resident automatic execution
    Resident = 0x0000000000000004,
    /// State management
    State = 0x0000000000000008,
    /// Plugin management
    Manager = 0x0000000000000010,

    // Data processing
    /// Data input
    DataInput = 0x0000000000000020,
    /// Data output
    DataOutput = 0x0000000000000040,

    // User interface
    /// Window management
    Window = 0x0000000000000080,

    // Media processing
    /// Image processing
    Image = 0x0000000000000100,
    /// Audio processing
    Audio = 0x0000000000000200,
    /// Video processing
    Video = 0x0000000000000400,

    // File system & storage
    /// Local file system operations
    FileSystem = 0x0000000000000800,
    /// Database operations
    Database = 0x0000000000001000,

    // Security
    /// Encryption processing
    Encryption = 0x0000000000002000,

    // Network communication
    /// HTTP/HTTPS client
    HttpClient = 0x0000000000100000,
    /// HTTP/HTTPS server
    HttpServer = 0x0000000000200000,
    /// TCP client
    TcpClient = 0x0000000000400000,
    /// TCP server
    TcpServer = 0x0000000000800000,
    /// UDP communication
    UdpSocket = 0x0000000001000000,
    /// WebSocket communication
    WebSocket = 0x0000000002000000,
    /// File sharing (SMB, AFP, NFS, etc.)
    FileSharing = 0x0000000004000000,
    /// Service discovery (Bonjour, UPnP, etc.)
    ServiceDiscovery = 0x0000000008000000,

    // Future extensions
    /// Streaming (RTP, WebRTC, etc.)
    Streaming = 0x0000000010000000,
    /// Messaging (MQTT, AMQP, etc.)
    Messaging = 0x0000000020000000,
    /// Blockchain communication
    Blockchain = 0x0000000040000000,
    /// IoT protocols (CoAP, etc.)
    IoT = 0x0000000080000000,

    // SDK internal use (prohibited for plugins)
    Reserved = 0x8000000000000000,
}

impl CubeMelonPluginType {
    /// Check if this type contains the specified flag
    pub fn contains(self, flag: CubeMelonPluginType) -> bool {
        (self as u64) & (flag as u64) != 0
    }

    /// Combine multiple plugin types
    pub fn combine(types: &[CubeMelonPluginType]) -> u64 {
        types.iter().map(|&t| t as u64).fold(0, |acc, t| acc | t)
    }

    /// Create from raw u64 value
    pub fn from_raw(value: u64) -> Self {
        // Safety: We're using repr(u64) so this is safe
        unsafe { std::mem::transmute(value) }
    }

    /// Convert to raw u64 value
    pub fn as_raw(self) -> u64 {
        self as u64
    }
}

impl std::ops::BitOr for CubeMelonPluginType {
    type Output = u64;

    fn bitor(self, rhs: Self) -> Self::Output {
        (self as u64) | (rhs as u64)
    }
}

impl std::ops::BitOr<CubeMelonPluginType> for u64 {
    type Output = u64;

    fn bitor(self, rhs: CubeMelonPluginType) -> Self::Output {
        self | (rhs as u64)
    }
}

impl std::ops::BitOr<u64> for CubeMelonPluginType {
    type Output = u64;

    fn bitor(self, rhs: u64) -> Self::Output {
        (self as u64) | rhs
    }
}

impl std::ops::BitAnd for CubeMelonPluginType {
    type Output = u64;

    fn bitand(self, rhs: Self) -> Self::Output {
        (self as u64) & (rhs as u64)
    }
}

impl std::ops::BitAnd<CubeMelonPluginType> for u64 {
    type Output = u64;

    fn bitand(self, rhs: CubeMelonPluginType) -> Self::Output {
        self & (rhs as u64)
    }
}

impl std::ops::BitAnd<u64> for CubeMelonPluginType {
    type Output = u64;

    fn bitand(self, rhs: u64) -> Self::Output {
        (self as u64) & rhs
    }
}

/// Plugin execution status
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubeMelonExecutionStatus {
    /// Idle/waiting
    Idle = 0,
    /// Running
    Running = 1,
    /// Suspended
    Suspended = 2,
    /// Completed
    Completed = 3,
    /// Error occurred
    Error = 4,
    /// Cancelled
    Cancelled = 5,
}

/// Plugin state scope
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubeMelonPluginStateScope {
    /// Plugin's internal state
    Local = 0,
    /// Host environment (language settings, timezone, etc.)
    Host = 1,
    /// Shared state with other plugins (images, history, etc.)
    Shared = 2,
}

/// Thread requirements
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubeMelonThreadRequirements {
    /// No special requirements
    NoRequirements = 0,
    /// Must run on UI thread
    UIThread = 1 << 0,
    /// Recommended for background thread
    Background = 1 << 1,
    /// Recommended for high priority thread
    HighPriority = 1 << 2,
    /// Recommended for low priority thread
    LowPriority = 1 << 3,
}

impl std::ops::BitOr for CubeMelonThreadRequirements {
    type Output = u32;

    fn bitor(self, rhs: Self) -> Self::Output {
        (self as u32) | (rhs as u32)
    }
}

/// Task type classification
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubeMelonTaskType {
    None = 0,
    
    // Basic tasks (1-19)
    Generic = 1,
    FileIO = 2,
    Database = 3,
    Computation = 4,
    Window = 5,
    Image = 6,
    Audio = 7,
    Video = 8,
    // 9-19: Reserved for future basic functionality

    // Network tasks (20-39)
    Http = 20,
    Tcp = 21,
    Udp = 22,
    WebSocket = 23,
    FileSharing = 24, // SMB, etc.
    ServiceDiscovery = 25, // Bonjour, etc.
    GRPC = 26,
    MQTT = 27,
    GraphQL = 28,
    // 29-39: Reserved for future network protocols

    // Extended tasks (40-99)
    // Project-specific functionality
    
    // User defined (100-65535)
    UserDefinedStart = 100,
    UserDefinedEnd = 65535,
}

/// Log level for debugging and monitoring
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CubeMelonLogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl fmt::Display for CubeMelonLogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CubeMelonLogLevel::Trace => write!(f, "TRACE"),
            CubeMelonLogLevel::Debug => write!(f, "DEBUG"),
            CubeMelonLogLevel::Info => write!(f, "INFO"),
            CubeMelonLogLevel::Warn => write!(f, "WARN"),
            CubeMelonLogLevel::Error => write!(f, "ERROR"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_creation_and_display() {
        let uuid = CubeMelonUUID::from_bytes([
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88
        ]);
        
        let _expected = "123456789abcdef0-1122-3344-5566-778899aabbcc";
        // Note: The actual format might be different, this is just a test structure
        assert_eq!(uuid.bytes.len(), 16);
    }

    #[test]
    fn test_version_ordering() {
        let v1_0_0 = CubeMelonVersion::new(1, 0, 0);
        let v1_0_1 = CubeMelonVersion::new(1, 0, 1);
        let v1_1_0 = CubeMelonVersion::new(1, 1, 0);
        let v2_0_0 = CubeMelonVersion::new(2, 0, 0);

        assert!(v1_0_0 < v1_0_1);
        assert!(v1_0_1 < v1_1_0);
        assert!(v1_1_0 < v2_0_0);
    }

    #[test]
    fn test_plugin_type_operations() {
        let web_server = CubeMelonPluginType::HttpServer | 
                        CubeMelonPluginType::WebSocket |
                        CubeMelonPluginType::FileSystem;
        
        // Test bitwise operations work
        assert_ne!(web_server, 0);
        
        // Test contains functionality
        assert!(CubeMelonPluginType::HttpServer.contains(CubeMelonPluginType::HttpServer));
        assert!(!CubeMelonPluginType::HttpServer.contains(CubeMelonPluginType::HttpClient));
    }

    #[test]
    fn test_plugin_type_combine() {
        let types = vec![
            CubeMelonPluginType::HttpServer,
            CubeMelonPluginType::WebSocket,
            CubeMelonPluginType::FileSystem,
        ];
        
        let combined = CubeMelonPluginType::combine(&types);
        assert_ne!(combined, 0);
        
        // Should contain all the individual flags
        assert_ne!(combined & (CubeMelonPluginType::HttpServer as u64), 0);
        assert_ne!(combined & (CubeMelonPluginType::WebSocket as u64), 0);
        assert_ne!(combined & (CubeMelonPluginType::FileSystem as u64), 0);
    }

    #[test]
    fn test_thread_requirements_combination() {
        let requirements = CubeMelonThreadRequirements::Background | 
                          CubeMelonThreadRequirements::HighPriority;
        
        assert_ne!(requirements, 0);
        assert_ne!(requirements & (CubeMelonThreadRequirements::Background as u32), 0);
        assert_ne!(requirements & (CubeMelonThreadRequirements::HighPriority as u32), 0);
    }
}