//! The Tauri command boundary. Each submodule holds the commands for one feature area;
//! commands stay thin and delegate to the application and repository layers.
//!
//! `tauri::generate_handler!` resolves a command through the module path it is written with,
//! so `lib.rs` refers to the submodules directly rather than through re-exports.

pub mod detection;
pub mod file_monitoring;
pub mod security_events;
pub mod settings;
pub mod system;
