use cubemelon_sdk::prelude::*;

#[plugin]
pub struct Plugin {
    initialized: bool,
    host_services: Option<CubeMelonHostServices>,
}

#[plugin_impl]
impl Plugin {
    fn log_message(&self, level: CubeMelonLogLevel, message: &str) {
        if let Some(ref services) = self.host_services {
            services.log_message(level, "SingleTaskPlugin", message);
        }
    }

    pub fn new() -> Self {
        Self {
            initialized: false,
            host_services: None,
        }
    }

    pub fn get_uuid() -> CubeMelonUUID {
        uuid!("6ccc639d-b240-44ec-9c83-a006a66a590b")
    }

    pub fn get_version() -> CubeMelonVersion {
        version!(1, 0, 0)
    }

    pub fn get_supported_types() -> u64 {
        CubeMelonPluginType::SingleTask as u64
    }

    pub fn get_name(&self, language: CubeMelonLanguage) -> *const u8 {
        multilang_map!(language, "Single Task Plugin", {
            "ja-JP" => "単発実行プラグイン",
        })
    }

    pub fn get_description(&self, language: CubeMelonLanguage) -> *const u8 {
        multilang_map!(language, "Plugin for single task execution test", {
            "ja-JP" => "単発実行プラグインのテストです",
        })
    }

    pub fn initialize(
        &mut self,
        host_services: Option<&CubeMelonHostServices>,
    ) -> Result<(), CubeMelonPluginErrorCode> {
        if self.initialized {
            return Err(CubeMelonPluginErrorCode::AlreadyInitialized);
        }

        if let Some(services) = host_services {
            self.host_services = Some(*services);
        }

        self.initialized = true;
        self.log_message(CubeMelonLogLevel::Info, "Plugin initialized.");

        // Try acquiring host Manager interface and call a simple method
        if let Some(services) = &self.host_services {
            unsafe {
                if let Some(get_iface) = services.get_host_interface {
                    let mut host_plugin: *const CubeMelonPlugin = std::ptr::null();
                    let mut iface_ptr: *const std::ffi::c_void = std::ptr::null();
                    let ec = get_iface(
                        CubeMelonPluginType::Manager,
                        1,
                        &mut host_plugin as *mut *const CubeMelonPlugin,
                        &mut iface_ptr as *mut *const std::ffi::c_void,
                    );
                    if ec == CubeMelonPluginErrorCode::Success && !host_plugin.is_null() && !iface_ptr.is_null() {
                        let vtbl = &*(iface_ptr as *const cubemelon_sdk::interfaces::CubeMelonPluginManagerInterfaceImpl);
                        let mut infos = CubeMelonPluginBasicInfoArray::empty();
                        let lang = services.get_system_language();
                        let ec2 = (vtbl.get_all_plugins_basic_info)(host_plugin, lang, &mut infos as *mut _);
                        if ec2 == CubeMelonPluginErrorCode::Success {
                            let count = infos.count;
                            self.log_message(CubeMelonLogLevel::Info, &format!("Host Manager.get_all_plugins_basic_info: {} items", count));
                            if let Some(free_fn) = infos.free_info_array {
                                free_fn(infos.infos, infos.count);
                            }
                        } else {
                            self.log_message(CubeMelonLogLevel::Warn, &format!("Manager.get_all_plugins_basic_info failed: {:?}", ec2));
                        }
                    } else {
                        self.log_message(CubeMelonLogLevel::Warn, &format!("get_host_interface(Manager) failed: {:?}", ec));
                    }
                } else {
                    self.log_message(CubeMelonLogLevel::Warn, "Host does not provide get_host_interface");
                }
            }
        }

        // Try acquiring host State interface and call a simple method
        if let Some(services) = &self.host_services {
            unsafe {
                if let Some(get_iface) = services.get_host_interface {
                    let mut host_plugin: *const CubeMelonPlugin = std::ptr::null();
                    let mut iface_ptr: *const std::ffi::c_void = std::ptr::null();
                    let ec = get_iface(
                        CubeMelonPluginType::State,
                        1,
                        &mut host_plugin as *mut *const CubeMelonPlugin,
                        &mut iface_ptr as *mut *const std::ffi::c_void,
                    );
                    if ec == CubeMelonPluginErrorCode::Success && !host_plugin.is_null() && !iface_ptr.is_null() {
                        let vtbl = &*(iface_ptr as *const cubemelon_sdk::interfaces::CubeMelonPluginStateInterfaceImpl);
                        let fmt_ptr = (vtbl.get_format_name)(host_plugin, CubeMelonPluginStateScope::Host);
                        if !fmt_ptr.is_null() {
                            let fmt = std::ffi::CStr::from_ptr(fmt_ptr as *const i8).to_string_lossy().to_string();
                            self.log_message(CubeMelonLogLevel::Info, &format!("Host State.get_format_name: {}", fmt));
                        } else {
                            self.log_message(CubeMelonLogLevel::Warn, "Host State.get_format_name returned null");
                        }
                    } else {
                        self.log_message(CubeMelonLogLevel::Warn, &format!("get_host_interface(State) failed: {:?}", ec));
                    }
                }
            }
        }

        // NOTE: Do not call Manager.execute_task() from initialize.
        // It creates a new plugin instance and re-enters initialize(), causing recursion.

        Ok(())
    }

    pub fn uninitialize(&mut self) -> Result<(), CubeMelonPluginErrorCode> {
        if !self.initialized {
            return Err(CubeMelonPluginErrorCode::NotInitialized);
        }

        self.log_message(CubeMelonLogLevel::Info, "Plugin uninitialized.");
        self.host_services = None;
        self.initialized = false;

        Ok(())
    }
}

#[single_task_plugin_impl]
impl Plugin {
    pub fn execute(
        &mut self,
        request: &CubeMelonTaskRequest,
        result: &mut CubeMelonTaskResult,
    ) -> CubeMelonPluginErrorCode {
        if !self.initialized {
            return CubeMelonPluginErrorCode::NotInitialized;
        }

        let language = request.language.clone();
        self.log_message(CubeMelonLogLevel::Info, language.as_str());

        let ptr = multilang_map!(language, "Executing task...", {
            "ja-JP" => "タスクを実行しています...",
        });
        match c_str_to_str(ptr) {
            Ok(message) => {
                self.log_message(CubeMelonLogLevel::Info, message);
            },
            Err(_) => self.log_message(CubeMelonLogLevel::Error, "Invalid UTF-8 string"),
        };

        if request.input_json.str != std::ptr::null() {
            if let Some(json_str) = request.input_json.as_str().ok() {
                self.log_message(CubeMelonLogLevel::Info, &format!("Input JSON: {}", json_str));
            }
        }

        result.callee = self as *const _ as *mut _;
        result.output_data = std::ptr::null_mut();
        result.output_json = CubeMelonString::empty();
        result.status = CubeMelonExecutionStatus::Completed;
        result.error_code = CubeMelonPluginErrorCode::Success;
        result.completion_time_us = 0;
        result.progress_ratio = 1.0;

        let ptr = multilang_map!(language, "Task completed successfully.", {
            "ja-JP" => "タスクが正常に完了しました。",
        });
        result.progress_message = match c_str_to_str(ptr) {
            Ok(message) => CubeMelonString::from_static_str(message),
            Err(_) => CubeMelonString::from_static_str("Invalid UTF-8 string"),
        };

        let ptr = multilang_map!(language, "Done", {
            "ja-JP" => "完了",
        });
        result.progress_stage = match c_str_to_str(ptr) {
            Ok(message) => CubeMelonString::from_static_str(message),
            Err(_) => CubeMelonString::from_static_str("Invalid UTF-8 string"),
        };
        result.estimated_remaining_us = 0;

        let ptr = multilang_map!(language, "Completed task.", {
            "ja-JP" => "タスクが完了しました。",
        });
        match c_str_to_str(ptr) {
            Ok(message) => self.log_message(CubeMelonLogLevel::Info, message),
            Err(_) => self.log_message(CubeMelonLogLevel::Error, "Invalid UTF-8 string"),
        };

        CubeMelonPluginErrorCode::Success
    }
}

#[plugin_interface(basic, single_task)]
impl Plugin {}
