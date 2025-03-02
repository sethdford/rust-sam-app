use std::env;
use tracing::info;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub environment: String,
    pub log_level: String,
    pub table_name: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "dev".to_string());
        let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "INFO".to_string());
        let table_name = env::var("TABLE_NAME").unwrap_or_else(|_| "Items".to_string());
        
        let config = Self {
            environment,
            log_level,
            table_name,
        };
        
        info!("Loaded configuration: {:?}", config);
        
        config
    }
    
    pub fn is_production(&self) -> bool {
        self.environment == "prod"
    }
} 