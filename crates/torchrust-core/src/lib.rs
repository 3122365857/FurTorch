//! TorchRust Core - Game log parsing, price calculation, and drop processing logic.
//!
//! This crate contains the core business logic ported from the original Python implementation.
//! It is designed to be used by the Node.js API server via FFI or as a library.

pub mod config;
pub mod log_parser;
pub mod price;
pub mod drop;
pub mod process;

pub use config::Config;
pub use log_parser::{convert_from_log_structure, LogParseError};
pub use price::PriceInfo;
pub use drop::DropProcessor;
pub use process::{process_log_chunk, ProcessInput, ProcessOutput, DropState, PriceTableEntry};
