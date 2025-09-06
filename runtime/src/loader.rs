// Plugin loader and execution related methods for RuntimeData
// Note: Keep comments in English per repository guidelines.

use anyhow::{anyhow, Context, Result};
use libloading::Library;
use std::path::{PathBuf};

use cubemelon_sdk::{
    CubeMelonInterface, CubeMelonPlugin, CubeMelonPluginErrorCode, CubeMelonLogLevel,
};

use crate::host_services::runtime_log;
use crate::{PluginInfo, RuntimeData};

impl RuntimeData {
    /// Scan plugins directory for valid plugins
    pub fn scan_plugins(&mut self) -> Result<()> {
        let plugins_dir = self.get_plugins_directory();
        runtime_log(CubeMelonLogLevel::Info, &format!("Scanning plugins directory: {:?}", plugins_dir));

        if !plugins_dir.exists() {
            runtime_log(
                CubeMelonLogLevel::Warn,
                &format!(
                    "Plugins directory does not exist, creating: {:?}",
                    plugins_dir
                ),
            );
            std::fs::create_dir_all(&plugins_dir)?;
            return Ok(());
        }

        let entries = std::fs::read_dir(&plugins_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            // Check file extension (platform-specific)
            #[cfg(windows)]
            let is_library = path.extension() == Some(std::ffi::OsStr::new("dll"));
            #[cfg(target_os = "macos")]
            let is_library = path.extension() == Some(std::ffi::OsStr::new("dylib"));
            #[cfg(all(unix, not(target_os = "macos")))]
            let is_library = path.extension() == Some(std::ffi::OsStr::new("so"));

            if !is_library {
                continue;
            }

            match self.validate_and_extract_info(&path) {
                Ok(plugin_info) => {
                    runtime_log(
                        CubeMelonLogLevel::Info,
                        &format!(
                            "Found valid plugin: {} ({})",
                            plugin_info.name, plugin_info.uuid
                        ),
                    );
                    self.discovered_plugins.push(plugin_info);
                }
                Err(e) => {
                    runtime_log(
                        CubeMelonLogLevel::Warn,
                        &format!("Invalid plugin at {:?}: {}", path, e),
                    );
                }
            }
        }

        runtime_log(
            CubeMelonLogLevel::Info,
            &format!("Found {} valid plugins", self.discovered_plugins.len()),
        );
        Ok(())
    }

    /// Validate plugin and extract basic information
    pub fn validate_and_extract_info(&self, plugin_path: &PathBuf) -> Result<PluginInfo> {
        // Load library temporarily
        let library = unsafe { Library::new(plugin_path).context("Failed to load plugin library")? };

        // Get required functions
        let get_plugin_interface: libloading::Symbol<
            unsafe extern "C" fn(u64, u32, *mut *const std::ffi::c_void) -> CubeMelonPluginErrorCode,
        > = unsafe {
            library
                .get(b"get_plugin_interface")
                .context("Plugin missing get_plugin_interface function")?
        };

        let create_plugin: libloading::Symbol<unsafe extern "C" fn() -> *mut CubeMelonPlugin> =
            unsafe {
                library
                    .get(b"create_plugin")
                    .context("Plugin missing create_plugin function")?
            };

        let destroy_plugin: libloading::Symbol<unsafe extern "C" fn(*mut CubeMelonPlugin)> =
            unsafe {
                library
                    .get(b"destroy_plugin")
                    .context("Plugin missing destroy_plugin function")?
            };

        // Get basic interface
        let mut interface_ptr: *const std::ffi::c_void = std::ptr::null();
        let result = unsafe { get_plugin_interface(0, 1, &mut interface_ptr as *mut *const std::ffi::c_void) };

        if result != CubeMelonPluginErrorCode::Success {
            return Err(anyhow!("Failed to get plugin interface: {:?}", result));
        }

        if interface_ptr.is_null() {
            return Err(anyhow!("Plugin returned null interface"));
        }

        let interface = unsafe { &*(interface_ptr as *const CubeMelonInterface) };

        // Create temporary plugin instance to get metadata
        let plugin = unsafe { create_plugin() };
        if plugin.is_null() {
            return Err(anyhow!("Failed to create plugin instance"));
        }

        // Extract metadata
        let uuid = (interface.get_uuid)();
        let version = (interface.get_version)();
        let supported_types = (interface.get_supported_types)();

        // Get name and description using system language
        let name_ptr = (interface.get_name)(plugin, self.system_language.clone());
        let desc_ptr = (interface.get_description)(plugin, self.system_language.clone());

        let name = if name_ptr.is_null() {
            "Unknown Plugin".to_string()
        } else {
            unsafe { std::ffi::CStr::from_ptr(name_ptr as *const i8).to_string_lossy().into_owned() }
        };

        let description = if desc_ptr.is_null() {
            "No description".to_string()
        } else {
            unsafe { std::ffi::CStr::from_ptr(desc_ptr as *const i8).to_string_lossy().into_owned() }
        };

        // Clean up
        unsafe { destroy_plugin(plugin) };

        Ok(PluginInfo {
            uuid,
            version,
            name,
            description,
            supported_types,
            path: plugin_path.clone(),
        })
    }

    /// Load a plugin by name, UUID, or number
    pub fn load_plugin(&mut self, plugin_id: &str) -> Result<&PluginInfo> {
        // Try to parse as number first
        let plugin_info = if let Ok(index) = plugin_id.parse::<usize>() {
            if index == 0 || index > self.discovered_plugins.len() {
                return Err(anyhow!(
                    "Invalid plugin number: {}. Valid range: 1-{}",
                    index,
                    self.discovered_plugins.len()
                ));
            }
            self.discovered_plugins[index - 1].clone()
        } else {
            // Find plugin by name or UUID
            self.discovered_plugins
                .iter()
                .find(|p| p.name == plugin_id || p.uuid.to_string() == plugin_id)
                .ok_or_else(|| anyhow!("Plugin not found: {}", plugin_id))?
                .clone()
        };

        if self.loaded_libraries.contains_key(&plugin_info.uuid) {
            runtime_log(CubeMelonLogLevel::Info, &format!("Plugin already loaded: {}", plugin_info.name));
            return Ok(self
                .discovered_plugins
                .iter()
                .find(|p| p.uuid == plugin_info.uuid)
                .unwrap());
        }

        runtime_log(CubeMelonLogLevel::Info, &format!("Loading plugin: {}", plugin_info.name));
        runtime_log(CubeMelonLogLevel::Info, &format!("Plugin path: {:?}", plugin_info.path));

        // Load library
        let library = unsafe { Library::new(&plugin_info.path).context("Failed to load plugin library")? };
        runtime_log(CubeMelonLogLevel::Info, "Plugin library loaded successfully");

        // Get interface
        let get_plugin_interface: libloading::Symbol<
            unsafe extern "C" fn(u64, u32, *mut *const std::ffi::c_void) -> CubeMelonPluginErrorCode,
        > = unsafe {
            library
                .get(b"get_plugin_interface")
                .context("Plugin missing get_plugin_interface function")?
        };

        let mut interface_ptr: *const std::ffi::c_void = std::ptr::null();
        let result = unsafe { get_plugin_interface(0, 1, &mut interface_ptr as *mut *const std::ffi::c_void) };

        if result != CubeMelonPluginErrorCode::Success {
            return Err(anyhow!("Failed to get plugin interface: {:?}", result));
        }

        // Store loaded library
        self.loaded_libraries.insert(plugin_info.uuid, library);

        runtime_log(CubeMelonLogLevel::Info, &format!("Plugin loaded successfully: {}", plugin_info.name));

        Ok(self
            .discovered_plugins
            .iter()
            .find(|p| p.uuid == plugin_info.uuid)
            .unwrap())
    }

    /// Execute a plugin
    pub fn execute_plugin(&self, plugin_info: &PluginInfo) -> Result<()> {
        let library = self
            .loaded_libraries
            .get(&plugin_info.uuid)
            .ok_or_else(|| anyhow!("Plugin not loaded: {}", plugin_info.name))?;

        runtime_log(CubeMelonLogLevel::Info, &format!("Executing plugin: {}", plugin_info.name));

        // Get interface from library
        let get_plugin_interface: libloading::Symbol<
            unsafe extern "C" fn(u64, u32, *mut *const std::ffi::c_void) -> CubeMelonPluginErrorCode,
        > = unsafe {
            library
                .get(b"get_plugin_interface")
                .context("Plugin missing get_plugin_interface function")?
        };

        let mut interface_ptr: *const std::ffi::c_void = std::ptr::null();
        let result = unsafe { get_plugin_interface(0, 1, &mut interface_ptr as *mut *const std::ffi::c_void) };

        if result != CubeMelonPluginErrorCode::Success {
            return Err(anyhow!("Failed to get plugin interface: {:?}", result));
        }

        let interface = unsafe { &*(interface_ptr as *const CubeMelonInterface) };
        let create_plugin: libloading::Symbol<unsafe extern "C" fn() -> *mut CubeMelonPlugin> = unsafe {
            library
                .get(b"create_plugin")
                .context("Plugin missing create_plugin function")?
        };

        let destroy_plugin: libloading::Symbol<unsafe extern "C" fn(*mut CubeMelonPlugin)> = unsafe {
            library
                .get(b"destroy_plugin")
                .context("Plugin missing destroy_plugin function")?
        };

        // Create instance
        let instance = unsafe { create_plugin() };
        if instance.is_null() {
            return Err(anyhow!("Failed to create plugin instance"));
        }

        // Initialize
        runtime_log(CubeMelonLogLevel::Info, "Initializing plugin instance...");
        let init_result = (interface.initialize)(instance, &self.host_services);
        if init_result != CubeMelonPluginErrorCode::Success {
            unsafe { destroy_plugin(instance) };
            return Err(anyhow!("Plugin initialization failed: {:?}", init_result));
        }
        runtime_log(CubeMelonLogLevel::Info, "Plugin initialization completed successfully");

        println!("Plugin '{}' executed successfully!", plugin_info.name);
        println!("Description: {}", plugin_info.description);

        // Uninitialize
        runtime_log(CubeMelonLogLevel::Info, "Uninitializing plugin instance...");
        let uninit_result = (interface.uninitialize)(instance);
        if uninit_result != CubeMelonPluginErrorCode::Success {
            runtime_log(
                CubeMelonLogLevel::Warn,
                &format!("Plugin uninitialization failed: {:?}", uninit_result),
            );
        } else {
            runtime_log(CubeMelonLogLevel::Info, "Plugin uninitialization completed successfully");
        }

        // Destroy instance
        unsafe { destroy_plugin(instance) };

        Ok(())
    }

    /// List all discovered plugins
    pub fn list_plugins(&self) {
        if self.discovered_plugins.is_empty() {
            println!("No plugins found.");
            return;
        }

        println!("Available plugins:");
        for (i, plugin) in self.discovered_plugins.iter().enumerate() {
            let status = if self.loaded_libraries.contains_key(&plugin.uuid) {
                "loaded"
            } else {
                "discovered"
            };

            println!("  {}. {} [{}]", i + 1, plugin.name, status);
            println!("     Description: {}", plugin.description);
            println!("     Version: {}", plugin.version);
            println!("     UUID: {}", plugin.uuid);
        }
    }
}
