//! CubeMelon Plugin Runtime - Simple Version
//! 
//! A simple host application for loading and executing CubeMelon plugins.

use anyhow::{Context, Result};
use libloading::Library;
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::fs;
use serde::{Serialize, Deserialize};

use cubemelon_sdk::{
    CubeMelonUUID, CubeMelonVersion, CubeMelonLanguage, CubeMelonHostServices, CubeMelonLogLevel,
    CubeMelonTaskRequest, CubeMelonTaskResult, CubeMelonTaskType, CubeMelonString, CubeMelonPluginErrorCode,
};
use cubemelon_sdk::CubeMelonPluginManagerInterface; // bring trait methods (execute_task, etc.) into scope

mod host_services;
use host_services::{
    runtime_log, language_to_string, get_system_language_callback, plugin_log_callback,
};

mod manager;
mod state;
mod loader;

/// Top-level runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Settings section containing host-level options
    pub settings: Settings,
}

/// [settings] section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Plugin installation directory (relative to executable)
    pub plugins_directory: String,

    /// System language setting
    pub language: String,

    /// Additional settings (flattened)
    #[serde(flatten)]
    pub extras: HashMap<String, toml::Value>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            plugins_directory: "plugins".to_string(),
            language: "auto".to_string(), // "auto" means detect from system
            extras: HashMap::new(),
        }
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self { settings: Settings::default() }
    }
}

/// Runtime Data Manager
/// This struct implements both plugin manager and state interfaces and manages the runtime
pub struct RuntimeData {
    /// Discovered plugins from the runtime
    pub discovered_plugins: Vec<PluginInfo>,
    
    /// Loaded plugin libraries
    pub loaded_libraries: HashMap<CubeMelonUUID, Library>,
    
    /// Current system language (resolved from config)
    pub system_language: CubeMelonLanguage,
    
    /// Path to the configuration file (empty if unavailable)
    pub config_path: PathBuf,
    
    /// Current runtime configuration
    pub config: RuntimeConfig,
    
    /// Host services for plugins
    pub host_services: CubeMelonHostServices,
}

/// Basic plugin information
#[derive(Debug, Clone)]
pub struct PluginInfo {
    uuid: CubeMelonUUID,
    version: CubeMelonVersion,
    supported_types: u64,
    name: String,
    description: String,
    path: PathBuf,
}

impl RuntimeData {
    /// Create a new RuntimeData
    pub fn new() -> Self {
        let config_path_opt = Self::get_config_file_path();
        let (config_path, config) = if let Some(path) = config_path_opt {
            runtime_log(CubeMelonLogLevel::Info, &format!("Config file path: {:?}", path));
            if path.exists() {
                runtime_log(CubeMelonLogLevel::Info, "Loading existing configuration file");
                match Self::load_config(&path) {
                    Ok(cfg) => (path, cfg),
                    Err(e) => {
                        runtime_log(
                            CubeMelonLogLevel::Warn,
                            &format!(
                                "Failed to load configuration, using defaults in-memory. error={}",
                                e
                            ),
                        );
                        // Do not rewrite file; keep in-memory defaults
                        (path, RuntimeConfig::default())
                    }
                }
            } else {
                runtime_log(CubeMelonLogLevel::Info, "Creating default configuration");
                let default_config = RuntimeConfig::default();
                if let Err(e) = Self::save_config(&path, &default_config) {
                    runtime_log(
                        CubeMelonLogLevel::Warn,
                        &format!(
                            "Failed to save default configuration (will continue without file): {}",
                            e
                        ),
                    );
                }
                (path, default_config)
            }
        } else {
            runtime_log(
                CubeMelonLogLevel::Warn,
                "Could not determine config path; running with in-memory defaults",
            );
            (PathBuf::new(), RuntimeConfig::default())
        };

        // Determine effective language: config > system (strict BCP 47, case-sensitive)
        let system_language = if config.settings.language != "auto" {
            let lang = crate::host_services::parse_language(&config.settings.language);
            runtime_log(
                CubeMelonLogLevel::Info,
                &format!("Using language from config: {}", config.settings.language),
            );
            lang
        } else {
            let sys = unsafe { get_system_language_callback() };
            runtime_log(
                CubeMelonLogLevel::Info,
                &format!("Using system language: {}", language_to_string(&sys)),
            );
            sys
        };

        // Create host services with plugin log callback and system language detection
        let host_services = CubeMelonHostServices::new(
            Some(plugin_log_callback),           // Enable plugin logging
            Some(get_system_language_callback),  // Enable system language detection
            Some(host_services::get_host_interface_callback), // Host interface provider
        );

        Self {
            discovered_plugins: Vec::new(),
            loaded_libraries: HashMap::new(),
            system_language,
            config_path,
            config,
            host_services,
        }
    }
    
    /// Get the path to the configuration file
    fn get_config_file_path() -> Option<PathBuf> {
        // Derive from current executable; if unavailable, return None
        let exe_path = std::env::current_exe().ok()?;
        let exe_dir = exe_path.parent()?;
        let exe_name = exe_path.file_stem()?.to_string_lossy();
        let config_file = format!("{}.toml", exe_name);
        Some(exe_dir.join(config_file))
    }
    
    /// Load configuration from TOML file
    fn load_config(config_path: &Path) -> Result<RuntimeConfig> {
        let content = fs::read_to_string(config_path)?;
        let config: RuntimeConfig = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// Save configuration to TOML file
    fn save_config(config_path: &Path, config: &RuntimeConfig) -> Result<()> {
        let content = toml::to_string_pretty(config)?;
        
        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(config_path, content)?;
        Ok(())
    }
    
    /// Get the current plugins directory path (absolute)
    pub fn get_plugins_directory(&self) -> PathBuf {
        if Path::new(&self.config.settings.plugins_directory).is_absolute() {
            PathBuf::from(&self.config.settings.plugins_directory)
        } else {
            // Relative to executable directory
            let exe_path = std::env::current_exe().unwrap_or_default();
            let exe_dir = exe_path.parent().unwrap_or(Path::new("."));
            exe_dir.join(&self.config.settings.plugins_directory)
        }
    }
    
    /// Get the current language setting
    pub fn get_language(&self) -> &str {
        &self.config.settings.language
    }
    
    /// Set plugins directory
    pub fn set_plugins_directory(&mut self, directory: String) -> Result<()> {
        self.config.settings.plugins_directory = directory;
        if !self.config_path.as_os_str().is_empty() {
            if let Err(e) = Self::save_config(&self.config_path, &self.config) {
                runtime_log(
                    CubeMelonLogLevel::Warn,
                    &format!("Failed to persist config (plugins_directory): {}", e),
                );
            }
        } else {
            runtime_log(
                CubeMelonLogLevel::Info,
                "Config path unavailable; new setting kept in-memory only",
            );
        }
        runtime_log(CubeMelonLogLevel::Info, "Plugins directory updated in configuration");
        Ok(())
    }
    
    /// Set language setting
    pub fn set_language(&mut self, language: String) -> Result<()> {
        self.config.settings.language = language;
        if !self.config_path.as_os_str().is_empty() {
            if let Err(e) = Self::save_config(&self.config_path, &self.config) {
                runtime_log(
                    CubeMelonLogLevel::Warn,
                    &format!("Failed to persist config (language): {}", e),
                );
            }
        } else {
            runtime_log(
                CubeMelonLogLevel::Info,
                "Config path unavailable; new setting kept in-memory only",
            );
        }
        runtime_log(CubeMelonLogLevel::Info, "Language setting updated in configuration");
        Ok(())
    }
    
    // Loader-related methods are implemented in `loader.rs`.
    
    /// Interactive prompt loop
    pub fn run_interactive(&mut self) -> Result<()> {
        runtime_log(CubeMelonLogLevel::Info, "Starting interactive mode");
        println!("CubeMelon Plugin Runtime v{}", env!("CARGO_PKG_VERSION"));
        println!("Type 'help' for commands, 'quit' to exit");
        println!();
        
        loop {
            print!("cubemelon> ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(0) => {
                    // EOF reached, exit gracefully
                    runtime_log(CubeMelonLogLevel::Info, "EOF reached, exiting interactive mode");
                    println!("Goodbye!");
                    break;
                }
                Ok(_) => {
                    // Successfully read input
                }
                Err(e) => {
                    runtime_log(CubeMelonLogLevel::Warn, &format!("Error reading input: {}", e));
                    println!("Error reading input: {}", e);
                    continue;
                }
            }
            
            let input = input.trim();
            if input.is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = input.split_whitespace().collect();
            let command = parts[0];
            
            match command {
                "help" | "h" => {
                    println!("Available commands:");
                    println!("  help, h              - Show this help");
                    println!("  list, ls             - List all plugins");
                    println!("  scan                 - Rescan plugins directory");
                    println!("  load <name|number>   - Load a plugin by name or number");
                    println!("  run <name|number>    - Run a plugin by name or number");
                    println!("  host-exec <id>       - Execute via host manager (index|uuid|name)");
                    println!("  quit, exit, q        - Exit the runtime");
                    println!();
                }
                "list" | "ls" => {
                    self.list_plugins();
                    println!();
                }
                "scan" => {
                    runtime_log(CubeMelonLogLevel::Info, "User requested plugin rescan");
                    println!("Rescanning plugins directory...");
                    self.discovered_plugins.clear();
                    match self.scan_plugins() {
                        Ok(()) => {
                            runtime_log(CubeMelonLogLevel::Info, "Plugin rescan completed successfully");
                            println!("Scan completed successfully.");
                        },
                        Err(e) => {
                            runtime_log(CubeMelonLogLevel::Warn, &format!("Plugin rescan failed: {}", e));
                            println!("Scan failed: {}", e);
                        },
                    }
                    println!();
                }
                "load" => {
                    if parts.len() < 2 {
                        println!("Usage: load <plugin_name|number>");
                        continue;
                    }
                    
                    match self.load_plugin(parts[1]) {
                        Ok(plugin_info) => {
                            runtime_log(CubeMelonLogLevel::Info, &format!("Successfully loaded plugin via user request: {}", plugin_info.name));
                            println!("Plugin '{}' loaded successfully!", plugin_info.name);
                        }
                        Err(e) => {
                            runtime_log(CubeMelonLogLevel::Warn, &format!("Failed to load plugin '{}': {}", parts[1], e));
                            println!("Failed to load plugin: {}", e);
                        }
                    }
                    println!();
                }
                "run" => {
                    if parts.len() < 2 {
                        println!("Usage: run <plugin_name|number>");
                        continue;
                    }
                    
                    // Load plugin if not already loaded
                    let plugin_info = match self.load_plugin(parts[1]) {
                        Ok(info) => info.clone(),
                        Err(e) => {
                            runtime_log(CubeMelonLogLevel::Warn, &format!("Failed to load plugin '{}' for execution: {}", parts[1], e));
                            println!("Failed to load plugin: {}", e);
                            continue;
                        }
                    };
                    
                    // Execute plugin
                    match self.execute_plugin(&plugin_info) {
                        Ok(()) => {
                            runtime_log(CubeMelonLogLevel::Info, &format!("Plugin execution completed successfully: {}", plugin_info.name));
                        }
                        Err(e) => {
                            runtime_log(CubeMelonLogLevel::Warn, &format!("Failed to execute plugin '{}': {}", plugin_info.name, e));
                            println!("Failed to execute plugin: {}", e);
                        }
                    }
                    println!();
                }
                "host-exec" => {
                    if parts.len() < 2 {
                        println!("Usage: host-exec <plugin_id>");
                        println!("  plugin_id: index|uuid|name");
                        continue;
                    }

                    // Load plugin if needed and get its info
                    let plugin_info = match self.load_plugin(parts[1]) {
                        Ok(info) => info.clone(),
                        Err(e) => {
                            runtime_log(CubeMelonLogLevel::Warn, &format!("Failed to load plugin '{}': {}", parts[1], e));
                            println!("Failed to load plugin: {}", e);
                            continue;
                        }
                    };

                    // Build minimal request/result
                    let mut result = CubeMelonTaskResult::empty();
                    let request = CubeMelonTaskRequest::new(
                        std::ptr::null(),
                        std::ptr::null_mut(),
                        CubeMelonString::empty(),
                        CubeMelonTaskType::Generic,
                        self.system_language.clone(),
                        0,
                        5_000_000,
                    );

                    // Execute through host manager path
                    let rc = self.execute_task(plugin_info.uuid, &request, &mut result);
                    match rc {
                        CubeMelonPluginErrorCode::Success => {
                            println!("host-exec: Success. status={:?}, code={:?}", result.status, result.error_code);

                            // Show additional result details if available
                            if result.progress_ratio >= 0.0 {
                                println!("  progress: {:.0}%", result.progress_ratio * 100.0);
                            }
                            if !result.progress_stage.str.is_null() {
                                if let Ok(s) = result.progress_stage.as_str() {
                                    if !s.is_empty() { println!("  stage: {}", s); }
                                }
                            }
                            if !result.progress_message.str.is_null() {
                                if let Ok(s) = result.progress_message.as_str() {
                                    if !s.is_empty() { println!("  message: {}", s); }
                                }
                            }
                            if !result.output_json.str.is_null() {
                                if let Ok(s) = result.output_json.as_str() {
                                    if !s.is_empty() { println!("  output_json: {}", s); }
                                }
                                if let Some(free_fn) = result.output_json.free_string {
                                    unsafe { free_fn(result.output_json.str); }
                                }
                            }

                            runtime_log(CubeMelonLogLevel::Info, &format!(
                                "host-exec completed: {} (status={:?}, code={:?})",
                                plugin_info.name, result.status, result.error_code
                            ));
                        }
                        other => {
                            println!("host-exec: Failed: {:?}", other);
                            runtime_log(CubeMelonLogLevel::Warn, &format!(
                                "host-exec failed for {}: {:?}", plugin_info.name, other
                            ));
                        }
                    }
                    println!();
                }
                "quit" | "exit" | "q" => {
                    runtime_log(CubeMelonLogLevel::Info, "User requested exit");
                    println!("Goodbye!");
                    break;
                }
                _ => {
                    println!("Unknown command: '{}'. Type 'help' for available commands.", command);
                    println!();
                }
            }
        }
        
        Ok(())
    }
}




fn main() -> Result<()> {
    runtime_log(CubeMelonLogLevel::Info, &format!("Starting CubeMelon Plugin Runtime v{}", env!("CARGO_PKG_VERSION")));
    
    // Create runtime data (always succeeds, falls back to defaults)
    let mut runtime = RuntimeData::new();
    // Register runtime singleton for host interface delegation
    host_services::set_runtime_singleton(&mut runtime as *mut RuntimeData);
    runtime_log(
        CubeMelonLogLevel::Info,
        &format!(
            "Effective language: {}",
            language_to_string(&runtime.system_language)
        ),
    );
    
    // Scan for plugins
    runtime.scan_plugins()
        .context("Failed to scan plugins")?;
    
    // Run interactive mode
    runtime.run_interactive()
        .context("Interactive mode failed")?;
    
    runtime_log(CubeMelonLogLevel::Info, "CubeMelon Plugin Runtime shutting down");
    Ok(())
}
