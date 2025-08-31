//! Interface implementation modules
//! 
//! This module contains procedural macro implementations for various plugin interfaces.
//! Each interface has its own submodule that implements the corresponding `*_plugin_impl` macro.

// === Standard Interfaces ===

pub mod single_task;       // #[single_task_plugin_impl]
pub mod manager;           // #[manager_plugin_impl]

// TODO: Implement these interface macros
// pub mod async_task;        // #[async_task_plugin_impl]
// pub mod resident;          // #[resident_plugin_impl]
// pub mod state;             // #[state_plugin_impl]
// pub mod data_input;        // #[data_input_plugin_impl]
// pub mod data_output;       // #[data_output_plugin_impl]
// pub mod window;            // #[window_plugin_impl]

// === Extended Interfaces ===

// TODO: Implement these extended interface macros
// pub mod image;             // #[image_plugin_impl]
// pub mod audio;             // #[audio_plugin_impl]
// pub mod video;             // #[video_plugin_impl]
// pub mod file_system;       // #[file_system_plugin_impl]
// pub mod database;          // #[database_plugin_impl]
// pub mod encryption;        // #[encryption_plugin_impl]
// pub mod compression;       // #[compression_plugin_impl]

// === Network Interfaces ===

// TODO: Implement these network interface macros
// pub mod http_client;       // #[http_client_plugin_impl]
// pub mod tcp_client;        // #[tcp_client_plugin_impl]
// pub mod tcp_server;        // #[tcp_server_plugin_impl]
// pub mod udp_socket;        // #[udp_socket_plugin_impl]
// pub mod websocket;         // #[websocket_plugin_impl]
// pub mod file_sharing;      // #[file_sharing_plugin_impl]
// pub mod service_discovery; // #[service_discovery_plugin_impl]

/// Re-export the single_task implementation function for use in lib.rs
pub use single_task::process_single_task_impl_attribute;
pub use manager::process_manager_impl_attribute;

// TODO: Re-export other interface implementation functions as they are implemented
// pub use async_task::process_async_task_impl_attribute;
// pub use resident::process_resident_impl_attribute;
// pub use state::process_state_impl_attribute;
// pub use data_input::process_data_input_impl_attribute;
// pub use data_output::process_data_output_impl_attribute;
// pub use window::process_window_impl_attribute;
// pub use http_client::process_http_client_impl_attribute;
// pub use image::process_image_impl_attribute;
// pub use audio::process_audio_impl_attribute;
// pub use video::process_video_impl_attribute;
// pub use file_system::process_file_system_impl_attribute;
// pub use database::process_database_impl_attribute;
// pub use encryption::process_encryption_impl_attribute;
// pub use compression::process_compression_impl_attribute;
// pub use tcp_client::process_tcp_client_impl_attribute;
// pub use tcp_server::process_tcp_server_impl_attribute;
// pub use udp_socket::process_udp_socket_impl_attribute;
// pub use websocket::process_websocket_impl_attribute;
// pub use file_sharing::process_file_sharing_impl_attribute;
// pub use service_discovery::process_service_discovery_impl_attribute;