//! Read and Write VCD (Value Change Dump) files and provide [embedded_hal] pin
//! implementations that reflect the VCD state.

#![warn(missing_docs)]
pub use embedded_hal_sync_pins::pins;
pub mod reader;
pub mod writer;
