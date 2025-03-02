/// Shared library for the Rust SAM application
///
/// This library contains common code shared between the API handler and event processor
/// Lambda functions. It includes data models, repository access, error handling, and configuration.
///
/// # Modules
///
/// * `models` - Data models for items and events
/// * `repository` - DynamoDB repository for data access
/// * `error` - Error handling
/// * `config` - Configuration management

pub mod models;
pub mod repository;
pub mod error;
pub mod config;

// Re-export common types
pub use error::AppError;

#[cfg(test)]
mod tests; 