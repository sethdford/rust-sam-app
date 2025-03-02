use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("DynamoDB error: {0}")]
    DynamoDb(#[from] aws_sdk_dynamodb::Error),
    
    #[error("SQS error: {0}")]
    Sqs(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<AppError> for lambda_runtime::Error {
    fn from(error: AppError) -> Self {
        lambda_runtime::Error::from(error.to_string())
    }
} 