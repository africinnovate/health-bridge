// src/lib.rs
pub mod error;
pub mod models;
pub mod schema;
pub mod utils;

// Optionally re-export commonly used types
pub use error::AppError;