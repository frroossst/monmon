#![feature(thread_id_value)]

// Core modules
pub mod condition_variables;
pub mod message;
pub mod semaphore;
pub mod tests;

// Monitor-related modules
pub mod futex_monitor;
pub mod ipc_monitor;
pub mod monitor_guard;
pub mod monitor_trait;
pub mod semaphore_monitor;

// Main monitors module that re-exports everything
pub mod monitors;

// Utility module
pub mod critical_section;
