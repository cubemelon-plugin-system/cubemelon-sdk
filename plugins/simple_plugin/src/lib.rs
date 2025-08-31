use cubemelon_sdk::prelude::*;

#[plugin]
pub struct SimplePlugin {
}

#[plugin_impl]
impl SimplePlugin {
    pub fn new() -> Self { Self {} }
    pub fn get_uuid() -> CubeMelonUUID { uuid!("ac02f3d9-0354-4012-91d3-d8f5bddd5b23") }
    pub fn get_version() -> CubeMelonVersion { version!(1, 0, 0) }
    pub fn get_supported_types() -> u64 { CubeMelonPluginType::Basic as u64 }
    // Other methods are optional
}