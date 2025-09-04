//! Memory management for safe C ABI interoperability
//! 
//! This module provides safe memory management structures and functions for
//! interfacing between Rust and C ABI. It ensures proper allocation and deallocation
//! while maintaining memory safety.

use std::ffi::{CStr, CString};
use std::ptr;
use crate::types::CubeMelonUUID;
use crate::structs::CubeMelonPluginBasicInfo;

/// Safe string structure for C ABI
/// 
/// The string is allocated by the plugin and must be freed using the provided free function.
#[repr(C)]
#[derive(Debug)]
pub struct CubeMelonString {
    /// Pointer to UTF-8 string data (NULL-terminated)
    pub str: *const u8,
    /// Function to free the string memory
    pub free_string: Option<unsafe extern "C" fn(*const u8)>,
}

impl Clone for CubeMelonString {
    fn clone(&self) -> Self {
        if self.str.is_null() {
            return CubeMelonString::empty();
        }

        // If it's a static string (no free function), we can safely share the pointer
        if self.free_string.is_none() {
            return Self {
                str: self.str,
                free_string: None,
            };
        }

        // For allocated strings, we need to create a new allocation
        match self.as_str() {
            Ok(s) => CubeMelonString::from_string(s.to_string()),
            Err(_) => CubeMelonString::empty(),
        }
    }
}

impl CubeMelonString {
    /// Create a new CubeMelonString from a Rust string
    /// 
    /// The string is allocated on the heap and must be freed using free_string()
    pub fn from_string(s: String) -> Self {
        let c_string = CString::new(s).unwrap_or_else(|_| CString::new("").unwrap());
        let ptr = c_string.into_raw() as *const u8;
        
        Self {
            str: ptr,
            free_string: Some(free_string_impl),
        }
    }

    /// Create a new CubeMelonString from a static string
    /// 
    /// No allocation is performed, so no free function is needed
    pub fn from_static_str(s: &'static str) -> Self {
        Self {
            str: s.as_ptr(),
            free_string: None, // Static strings don't need to be freed
        }
    }

    /// Create an empty CubeMelonString
    pub fn empty() -> Self {
        Self {
            str: ptr::null(),
            free_string: None,
        }
    }

    /// Convert to Rust string slice
    /// 
    /// # Safety
    /// 
    /// The str pointer must be valid and point to a NULL-terminated UTF-8 string
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        if self.str.is_null() {
            return Ok("");
        }
        unsafe {
            let cstr = CStr::from_ptr(self.str as *const i8);
            cstr.to_str()
        }
    }

    /// Check if the string is empty or null
    pub fn is_empty(&self) -> bool {
        self.str.is_null() || unsafe { *self.str == 0 }
    }
}

unsafe impl Send for CubeMelonString {}
unsafe impl Sync for CubeMelonString {}

/// UUID array structure for C ABI
#[repr(C)]
#[derive(Debug)]
pub struct CubeMelonUUIDArray {
    /// Array of UUIDs
    pub uuids: *mut CubeMelonUUID,
    /// Number of UUIDs in the array
    pub count: usize,
    /// Function to free the UUID array
    pub free_uuid_array: Option<unsafe extern "C" fn(*mut CubeMelonUUID, usize)>,
}

impl CubeMelonUUIDArray {
    /// Create a new CubeMelonUUIDArray from a Vec of UUIDs
    pub fn from_vec(mut uuids: Vec<CubeMelonUUID>) -> Self {
        uuids.shrink_to_fit();
        let ptr = uuids.as_mut_ptr();
        let count = uuids.len();
        std::mem::forget(uuids);
        
        Self {
            uuids: ptr,
            count,
            free_uuid_array: Some(free_uuid_array_impl),
        }
    }

    /// Create an empty CubeMelonUUIDArray
    pub fn empty() -> Self {
        Self {
            uuids: ptr::null_mut(),
            count: 0,
            free_uuid_array: None,
        }
    }

    /// Convert to Rust slice
    /// 
    /// # Safety
    /// 
    /// The uuids pointer must be valid for the specified count
    pub unsafe fn as_slice(&self) -> &[CubeMelonUUID] {
        if self.uuids.is_null() || self.count == 0 {
            &[]
        } else {
            std::slice::from_raw_parts(self.uuids, self.count)
        }
    }
}

unsafe impl Send for CubeMelonUUIDArray {}
unsafe impl Sync for CubeMelonUUIDArray {}

/// Plugin basic info array structure for C ABI
#[repr(C)]
#[derive(Debug)]
pub struct CubeMelonPluginBasicInfoArray {
    /// Array of plugin basic info
    pub infos: *mut CubeMelonPluginBasicInfo,
    /// Number of info structures in the array
    pub count: usize,
    /// Function to free the info array
    pub free_info_array: Option<unsafe extern "C" fn(*mut CubeMelonPluginBasicInfo, usize)>,
}

impl CubeMelonPluginBasicInfoArray {
    /// Create a new CubeMelonPluginBasicInfoArray from a Vec of info structures
    pub fn from_vec(mut infos: Vec<CubeMelonPluginBasicInfo>) -> Self {
        infos.shrink_to_fit();
        let ptr = infos.as_mut_ptr();
        let count = infos.len();
        std::mem::forget(infos);
        
        Self {
            infos: ptr,
            count,
            free_info_array: Some(free_info_array_impl),
        }
    }

    /// Create an empty CubeMelonPluginBasicInfoArray
    pub fn empty() -> Self {
        Self {
            infos: ptr::null_mut(),
            count: 0,
            free_info_array: None,
        }
    }

    /// Convert to Rust slice
    /// 
    /// # Safety
    /// 
    /// The infos pointer must be valid for the specified count
    pub unsafe fn as_slice(&self) -> &[CubeMelonPluginBasicInfo] {
        if self.infos.is_null() || self.count == 0 {
            &[]
        } else {
            std::slice::from_raw_parts(self.infos, self.count)
        }
    }
}

unsafe impl Send for CubeMelonPluginBasicInfoArray {}
unsafe impl Sync for CubeMelonPluginBasicInfoArray {}

/// Value tag enumeration for CubeMelonValue
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubeMelonValueTag {
    Null = 0,
    Bool = 1,
    Int = 2,
    UInt = 3,
    Float = 4,
    Pointer = 5,
    String = 6,
    Buffer = 7,
    Array = 8,
    Custom = u32::MAX,
}

/// Generic hierarchical data structure for C ABI
/// 
/// This structure can hold various types of data and provides memory management
/// through the free_value function pointer.
#[repr(C)]
#[derive(Debug)]
pub struct CubeMelonValue {
    /// Type tag indicating which union member is active
    pub tag: CubeMelonValueTag,
    /// Reserved field for future use (padding)
    pub reserved: u32,
    /// Union containing the actual data
    pub data: CubeMelonValueData,
    /// Function to free the value contents (not the container itself)
    pub free_value: Option<unsafe extern "C" fn(*mut CubeMelonValue)>,
}

/// Union data for CubeMelonValue
#[repr(C)]
pub union CubeMelonValueData {
    /// Pointer data
    pub pointer: *mut std::ffi::c_void,
    /// Numeric data
    pub number: CubeMelonValueNumber,
    /// String data
    pub string: CubeMelonValueString,
    /// Buffer data
    pub buffer: CubeMelonValueBuffer,
    /// Array data
    pub array: CubeMelonValueArray,
}

/// Numeric union for CubeMelonValue
#[repr(C)]
#[derive(Clone, Copy)]
pub union CubeMelonValueNumber {
    /// Boolean value
    pub b: bool,
    /// Signed integer value
    pub i: isize,
    /// Unsigned integer value
    pub u: usize,
    /// Float value
    pub f: f64,
}

/// String structure for CubeMelonValue
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CubeMelonValueString {
    /// Pointer to UTF-8 string data (NULL-terminated)
    pub str: *const u8,
}

/// Buffer structure for CubeMelonValue
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CubeMelonValueBuffer {
    /// Number of bytes in the buffer
    pub count: usize,
    /// Pointer to buffer data
    pub data: *const std::ffi::c_void,
}

/// Array structure for CubeMelonValue
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CubeMelonValueArray {
    /// Number of items in the array
    pub count: usize,
    /// Pointer to array of CubeMelonValue items (not pointers to CubeMelonValue)
    pub items: *const CubeMelonValue,
}

impl CubeMelonValue {
    /// Create a null value
    pub fn null() -> Self {
        Self {
            tag: CubeMelonValueTag::Null,
            reserved: 0,
            data: CubeMelonValueData { pointer: std::ptr::null_mut() },
            free_value: None,
        }
    }

    /// Create a boolean value
    pub fn bool(value: bool) -> Self {
        Self {
            tag: CubeMelonValueTag::Bool,
            reserved: 0,
            data: CubeMelonValueData {
                number: CubeMelonValueNumber { b: value },
            },
            free_value: None,
        }
    }

    /// Create a signed integer value
    pub fn int(value: isize) -> Self {
        Self {
            tag: CubeMelonValueTag::Int,
            reserved: 0,
            data: CubeMelonValueData {
                number: CubeMelonValueNumber { i: value },
            },
            free_value: None,
        }
    }

    /// Create an unsigned integer value
    pub fn uint(value: usize) -> Self {
        Self {
            tag: CubeMelonValueTag::UInt,
            reserved: 0,
            data: CubeMelonValueData {
                number: CubeMelonValueNumber { u: value },
            },
            free_value: None,
        }
    }

    /// Create a float value
    pub fn float(value: f64) -> Self {
        Self {
            tag: CubeMelonValueTag::Float,
            reserved: 0,
            data: CubeMelonValueData {
                number: CubeMelonValueNumber { f: value },
            },
            free_value: None,
        }
    }

    /// Create a pointer value
    pub fn pointer(ptr: *mut std::ffi::c_void) -> Self {
        Self {
            tag: CubeMelonValueTag::Pointer,
            reserved: 0,
            data: CubeMelonValueData { pointer: ptr },
            free_value: None,
        }
    }

    /// Create a string value from a Rust string
    /// 
    /// The string is moved (not copied) and converted to C string format.
    /// Must be freed using free_value()
    pub fn string(s: String) -> Self {
        let c_string = CString::new(s).unwrap_or_else(|_| CString::new("").unwrap());
        let ptr = c_string.into_raw() as *const u8;
        
        Self {
            tag: CubeMelonValueTag::String,
            reserved: 0,
            data: CubeMelonValueData {
                string: CubeMelonValueString { str: ptr },
            },
            free_value: Some(free_value_impl),
        }
    }

    /// Create a string value from a string slice (creates a copy)
    /// 
    /// The string slice is copied and must be freed using free_value()
    pub fn string_from_str(s: &str) -> Self {
        Self::string(s.to_string())
    }

    /// Create a string value from a static string
    /// 
    /// No allocation is performed, so no free function is needed
    pub fn static_string(s: &'static str) -> Self {
        Self {
            tag: CubeMelonValueTag::String,
            reserved: 0,
            data: CubeMelonValueData {
                string: CubeMelonValueString { str: s.as_ptr() },
            },
            free_value: None, // Static strings don't need to be freed
        }
    }

    /// Create a buffer value from a Vec<u8>
    /// 
    /// The vector data is transferred to the heap and must be freed using free_value()
    pub fn buffer(mut data: Vec<u8>) -> Self {
        data.shrink_to_fit();
        let ptr = data.as_ptr() as *const std::ffi::c_void;
        let count = data.len();
        std::mem::forget(data); // Prevent automatic deallocation
        
        Self {
            tag: CubeMelonValueTag::Buffer,
            reserved: 0,
            data: CubeMelonValueData {
                buffer: CubeMelonValueBuffer { count, data: ptr },
            },
            free_value: Some(free_value_impl),
        }
    }

    /// Create an array value from a Vec of CubeMelonValue
    /// 
    /// The array is allocated on the heap and must be freed using free_value()
    pub fn array(mut values: Vec<CubeMelonValue>) -> Self {
        values.shrink_to_fit();
        let ptr = values.as_ptr();
        let count = values.len();
        std::mem::forget(values); // Prevent automatic deallocation
        
        Self {
            tag: CubeMelonValueTag::Array,
            reserved: 0,
            data: CubeMelonValueData {
                array: CubeMelonValueArray { count, items: ptr },
            },
            free_value: Some(free_value_impl),
        }
    }

    /// Get boolean value
    /// 
    /// # Safety
    /// 
    /// The value must be of type Bool
    pub unsafe fn as_bool(&self) -> bool {
        debug_assert_eq!(self.tag, CubeMelonValueTag::Bool);
        self.data.number.b
    }

    /// Get signed integer value
    /// 
    /// # Safety
    /// 
    /// The value must be of type Int
    pub unsafe fn as_int(&self) -> isize {
        debug_assert_eq!(self.tag, CubeMelonValueTag::Int);
        self.data.number.i
    }

    /// Get unsigned integer value
    /// 
    /// # Safety
    /// 
    /// The value must be of type UInt
    pub unsafe fn as_uint(&self) -> usize {
        debug_assert_eq!(self.tag, CubeMelonValueTag::UInt);
        self.data.number.u
    }

    /// Get float value
    /// 
    /// # Safety
    /// 
    /// The value must be of type Float
    pub unsafe fn as_float(&self) -> f64 {
        debug_assert_eq!(self.tag, CubeMelonValueTag::Float);
        self.data.number.f
    }

    /// Get pointer value
    /// 
    /// # Safety
    /// 
    /// The value must be of type Pointer
    pub unsafe fn as_pointer(&self) -> *mut std::ffi::c_void {
        debug_assert_eq!(self.tag, CubeMelonValueTag::Pointer);
        self.data.pointer
    }

    /// Get string value as Rust str
    /// 
    /// # Safety
    /// 
    /// The value must be of type String and contain valid UTF-8
    pub unsafe fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        debug_assert_eq!(self.tag, CubeMelonValueTag::String);
        if self.data.string.str.is_null() {
            return Ok("");
        }
        let cstr = CStr::from_ptr(self.data.string.str as *const i8);
        cstr.to_str()
    }

    /// Get buffer value as slice
    /// 
    /// # Safety
    /// 
    /// The value must be of type Buffer
    pub unsafe fn as_buffer(&self) -> &[u8] {
        debug_assert_eq!(self.tag, CubeMelonValueTag::Buffer);
        if self.data.buffer.data.is_null() || self.data.buffer.count == 0 {
            &[]
        } else {
            std::slice::from_raw_parts(
                self.data.buffer.data as *const u8,
                self.data.buffer.count,
            )
        }
    }

    /// Get array value as slice of CubeMelonValue
    /// 
    /// # Safety
    /// 
    /// The value must be of type Array
    pub unsafe fn as_array(&self) -> &[CubeMelonValue] {
        debug_assert_eq!(self.tag, CubeMelonValueTag::Array);
        if self.data.array.items.is_null() || self.data.array.count == 0 {
            &[]
        } else {
            std::slice::from_raw_parts(self.data.array.items, self.data.array.count)
        }
    }

    /// Check if the value needs to be freed
    pub fn needs_free(&self) -> bool {
        self.free_value.is_some()
    }

    /// Check if the value is null
    pub fn is_null(&self) -> bool {
        self.tag == CubeMelonValueTag::Null
    }
}

unsafe impl Send for CubeMelonValue {}
unsafe impl Sync for CubeMelonValue {}

// Debug implementation for union data
impl std::fmt::Debug for CubeMelonValueData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CubeMelonValueData")
            .field("data", &"<union>")
            .finish()
    }
}

/// Free function implementation for CubeMelonValue
unsafe extern "C" fn free_value_impl(value_ptr: *mut CubeMelonValue) {
    if value_ptr.is_null() {
        return;
    }

    let value = &mut *value_ptr;
    
    match value.tag {
        CubeMelonValueTag::String => {
            if !value.data.string.str.is_null() {
                let _ = CString::from_raw(value.data.string.str as *mut i8);
                // CString's Drop implementation will free the memory
            }
        }
        CubeMelonValueTag::Buffer => {
            if !value.data.buffer.data.is_null() && value.data.buffer.count > 0 {
                let _ = Vec::from_raw_parts(
                    value.data.buffer.data as *mut u8,
                    value.data.buffer.count,
                    value.data.buffer.count,
                );
                // Vec's Drop implementation will free the memory
            }
        }
        CubeMelonValueTag::Array => {
            if !value.data.array.items.is_null() && value.data.array.count > 0 {
                // Get the array as a Vec and let it handle the cleanup
                let items_vec = Vec::from_raw_parts(
                    value.data.array.items as *mut CubeMelonValue,
                    value.data.array.count,
                    value.data.array.count,
                );
                
                // Free each item in the array that has a free function
                for mut item in items_vec {
                    if let Some(free_fn) = item.free_value {
                        free_fn(&mut item as *mut CubeMelonValue);
                    }
                }
                // Vec's Drop implementation will free the array memory
            }
        }
        _ => {
            // Other types don't need special cleanup
        }
    }
}
// Implementation of free functions

/// Free function for CubeMelonString
unsafe extern "C" fn free_string_impl(str_ptr: *const u8) {
    if !str_ptr.is_null() {
        let _ = CString::from_raw(str_ptr as *mut i8);
        // CString's Drop implementation will free the memory
    }
}

/// Free function for CubeMelonUUIDArray
unsafe extern "C" fn free_uuid_array_impl(uuids_ptr: *mut CubeMelonUUID, count: usize) {
    if !uuids_ptr.is_null() && count > 0 {
        let _ = Vec::from_raw_parts(uuids_ptr, count, count);
        // Vec's Drop implementation will free the memory
    }
}

/// Free function for CubeMelonPluginBasicInfoArray
unsafe extern "C" fn free_info_array_impl(infos_ptr: *mut CubeMelonPluginBasicInfo, count: usize) {
    if !infos_ptr.is_null() && count > 0 {
        let infos = Vec::from_raw_parts(infos_ptr, count, count);
        // Each info structure will be freed when dropped
        for info in infos {
            // Free the name and description strings if they have free functions
            if let Some(free_fn) = info.name.free_string {
                if !info.name.str.is_null() {
                    free_fn(info.name.str);
                }
            }
            if let Some(free_fn) = info.description.free_string {
                if !info.description.str.is_null() {
                    free_fn(info.description.str);
                }
            }
        }
    }
}

/// Initialize the memory management system
/// 
/// This function is called during SDK initialization
pub fn initialize_memory_system() {
    // Currently no global initialization is needed
    // This function exists for future extensibility
}

/// Cleanup the memory management system
/// 
/// This function is called during SDK cleanup
pub fn cleanup_memory_system() {
    // Currently no global cleanup is needed
    // This function exists for future extensibility
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cubemelon_string_from_string() {
        let rust_string = "Hello, World!".to_string();
        let cube_string = CubeMelonString::from_string(rust_string);
        
        assert!(!cube_string.str.is_null());
        assert!(cube_string.free_string.is_some());
        
        let converted = cube_string.as_str().unwrap();
        assert_eq!(converted, "Hello, World!");
        
        // Cleanup
        if let Some(free_fn) = cube_string.free_string {
            unsafe { free_fn(cube_string.str); }
        }
    }

    #[test]
    fn test_cubemelon_string_from_static() {
        let cube_string = CubeMelonString::from_static_str("Static string");
        
        assert!(!cube_string.str.is_null());
        
        let converted = cube_string.as_str().unwrap();
        assert_eq!(converted, "Static string");
        
        // Static strings don't need freeing
        assert!(cube_string.free_string.is_none());
    }
}