use lambda_http::{run, service_fn, Body, Error, Request, Response};
use tracing::{info, error};
use shared::{
    models::{Item, ApiResponse, ErrorResponse},
    repository::DynamoDbRepository,
    config::AppConfig,
    AppError,
};
use aws_sdk_sqs::Client as SqsClient;
use serde_json::json;
use std::env;
use uuid;
use md5;
use chrono::{Utc};

/// Test module for unit testing the API handler
#[cfg(test)]
mod tests;

/// Main entry point for the API handler Lambda function
///
/// This function initializes the AWS SDK, sets up logging, and starts the Lambda runtime.
/// It handles HTTP requests from API Gateway and routes them to the appropriate handler functions.
#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    info!("API Handler Lambda starting up");
    
    // Load configuration from environment variables
    let config = AppConfig::from_env();
    
    // Initialize AWS SDK clients
    let aws_config = aws_config::load_from_env().await;
    let repo = DynamoDbRepository::new(&aws_config);
    let sqs_client = SqsClient::new(&aws_config);
    
    // Get SQS queue URL from environment or construct a default one
    let queue_url = env::var("EVENT_QUEUE_URL").unwrap_or_else(|_| {
        let stack_name = env::var("AWS_LAMBDA_FUNCTION_NAME")
            .unwrap_or_else(|_| "rust-sam-app".to_string());
        format!("https://sqs.{}.amazonaws.com/123456789012/{}-Events-{}",
            env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            stack_name,
            config.environment
        )
    });
    
    info!("Using SQS queue URL: {}", queue_url);

    // Run the Lambda service with our request handler
    run(service_fn(|event: Request| {
        handle_request(event, &repo, &sqs_client, &queue_url)
    })).await?;

    Ok(())
}

/// Main request handler for the API Lambda
///
/// This function routes incoming HTTP requests to the appropriate handler function
/// based on the HTTP method and path.
///
/// # Arguments
///
/// * `event` - The HTTP request from API Gateway
/// * `repo` - The DynamoDB repository for data access
/// * `sqs_client` - The SQS client for sending events
/// * `queue_url` - The URL of the SQS queue for events
///
/// # Returns
///
/// * `Result<Response<Body>, Error>` - The HTTP response or an error
async fn handle_request(
    event: Request,
    repo: &DynamoDbRepository,
    sqs_client: &SqsClient,
    queue_url: &str,
) -> Result<Response<Body>, Error> {
    let path = event.uri().path().to_string();
    let method = event.method().as_str().to_string();
    
    info!("Handling request: {} {}", method, path);

    let result = match (method.as_str(), path.as_str()) {
        // Route GET /items to get_items handler
        ("GET", "/items") => get_items(repo).await,
        
        // Route GET /items/{id} to get_item handler
        ("GET", p) if p.starts_with("/items/") => {
            let id = p.trim_start_matches("/items/");
            get_item(repo, id).await
        },
        
        // Route POST /items to create_item handler
        ("POST", "/items") => {
            let body = event.body();
            let item: Item = match body {
                Body::Text(text) => serde_json::from_str(text)?,
                Body::Binary(bytes) => serde_json::from_slice(bytes)?,
                _ => return Ok(error_response("Invalid request body".to_string(), 400)),
            };
            create_item(repo, sqs_client, queue_url, item).await
        },
        
        // Route DELETE /items/{id} to delete_item handler
        ("DELETE", p) if p.starts_with("/items/") => {
            let id = p.trim_start_matches("/items/");
            delete_item(repo, sqs_client, queue_url, id).await
        },
        
        // Return 404 for any other routes
        _ => Ok(error_response("Not found".to_string(), 404)),
    };

    // Handle errors and convert to appropriate HTTP responses
    match result {
        Ok(response) => Ok(response),
        Err(err) => {
            error!("Error processing request: {:?}", err);
            let status_code = match err {
                AppError::NotFound(_) => 404,
                AppError::Validation(_) => 400,
                _ => 500,
            };
            Ok(error_response(err.to_string(), status_code))
        }
    }
}

/// Handler for GET /items endpoint
///
/// Retrieves all items from the database and returns them as a JSON array.
///
/// # Arguments
///
/// * `repo` - The DynamoDB repository for data access
///
/// # Returns
///
/// * `Result<Response<Body>, AppError>` - A JSON response with all items or an error
async fn get_items(repo: &DynamoDbRepository) -> Result<Response<Body>, AppError> {
    // Retrieve all items from the database
    let items = repo.list_items().await?;
    
    // Create a successful response
    let response = ApiResponse {
        status_code: 200,
        body: items,
    };
    
    // Serialize the response body to JSON
    let body = serde_json::to_string(&response.body)?;
    
    // Build and return the HTTP response
    Ok(Response::builder()
        .status(response.status_code)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .map_err(|e| AppError::Internal(e.to_string()))?)
}

/// Handler for GET /items/{id} endpoint
///
/// Retrieves a specific item by ID from the database.
///
/// # Arguments
///
/// * `repo` - The DynamoDB repository for data access
/// * `id` - The ID of the item to retrieve
///
/// # Returns
///
/// * `Result<Response<Body>, AppError>` - A JSON response with the item or an error
async fn get_item(repo: &DynamoDbRepository, id: &str) -> Result<Response<Body>, AppError> {
    // Retrieve the item from the database
    let item = repo.get_item(id).await?;
    
    match item {
        Some(item) => {
            // Create a successful response
            let response = ApiResponse {
                status_code: 200,
                body: item,
            };
            
            // Serialize the response body to JSON
            let body = serde_json::to_string(&response.body)?;
            
            // Build and return the HTTP response
            Ok(Response::builder()
                .status(response.status_code)
                .header("Content-Type", "application/json")
                .body(Body::from(body))
                .map_err(|e| AppError::Internal(e.to_string()))?)
        },
        None => Err(AppError::NotFound(format!("Item with ID {} not found", id))),
    }
}

/// Validates an item before processing
///
/// This function performs comprehensive validation on an item to ensure it meets
/// security and business requirements. It checks for empty fields, malicious content,
/// and other validation rules.
///
/// # Arguments
///
/// * `item` - The item to validate
///
/// # Returns
///
/// * `Result<(), AppError>` - Ok if valid, Err with validation error otherwise
fn validate_item(item: &Item) -> Result<(), AppError> {
    // Check for empty or invalid fields
    if item.name.is_empty() {
        return Err(AppError::Validation("Item name cannot be empty".to_string()));
    }
    
    // Check name length
    if item.name.len() > 100 {
        return Err(AppError::Validation("Item name too long (max 100 characters)".to_string()));
    }
    
    // Check for malicious content in name
    if item.name.contains('<') || item.name.contains('>') || item.name.contains('&') {
        return Err(AppError::Validation("Item name contains invalid characters".to_string()));
    }
    
    // Check description if present
    if let Some(desc) = &item.description {
        if desc.len() > 1000 {
            return Err(AppError::Validation("Description too long (max 1000 characters)".to_string()));
        }
        
        // Check for malicious content in description
        if desc.contains('<') || desc.contains('>') || desc.contains('&') {
            return Err(AppError::Validation("Description contains invalid characters".to_string()));
        }
    }
    
    // Validate classification
    match item.classification.as_str() {
        "PUBLIC" | "INTERNAL" | "CONFIDENTIAL" | "RESTRICTED" => (),
        _ => return Err(AppError::Validation("Invalid classification level".to_string())),
    }
    
    Ok(())
}

/// Masks sensitive data for logging
///
/// This function masks sensitive data to prevent it from appearing in logs.
///
/// # Arguments
///
/// * `data` - The data to mask
///
/// # Returns
///
/// * `String` - The masked data
fn mask_sensitive_data(data: &str) -> String {
    if data.len() <= 4 {
        return "****".to_string();
    }
    let visible = &data[0..4];
    let masked = "*".repeat(data.len() - 4);
    format!("{}{}", visible, masked)
}

/// Creates an audit record for an action
///
/// This function creates an audit record for an action performed on an item.
///
/// # Arguments
///
/// * `action` - The action performed (create, update, delete)
/// * `item` - The item affected
/// * `previous_state` - The previous state of the item (for updates and deletes)
/// * `request_id` - The ID of the request that triggered the action
///
/// # Returns
///
/// * `AuditRecord` - The audit record
fn create_audit_record(
    action: &str,
    item: &Item,
    previous_state: Option<String>,
    request_id: &str,
) -> shared::models::AuditRecord {
    let new_state = if action != "delete" {
        Some(serde_json::to_string(item).unwrap_or_default())
    } else {
        None
    };
    
    // In a real application, you would get the user ID from authentication
    let user_id = "system".to_string();
    
    // Create a hash of the item for non-repudiation
    let item_json = serde_json::to_string(item).unwrap_or_default();
    let item_hash = format!("{:x}", md5::compute(item_json.as_bytes()));
    
    shared::models::AuditRecord {
        event_id: uuid::Uuid::new_v4().to_string(),
        user_id,
        action: action.to_string(),
        resource_id: item.id.clone(),
        resource_type: "item".to_string(),
        timestamp: Utc::now(),
        previous_state,
        new_state,
        request_id: request_id.to_string(),
        hash: Some(item_hash),
    }
}

/// Handler for POST /items endpoint
///
/// Creates a new item in the database and sends a creation event to SQS.
///
/// # Arguments
///
/// * `repo` - The DynamoDB repository for data access
/// * `sqs_client` - The SQS client for sending events
/// * `queue_url` - The URL of the SQS queue for events
/// * `item` - The item to create
///
/// # Returns
///
/// * `Result<Response<Body>, AppError>` - A JSON response with the created item or an error
async fn create_item(
    repo: &DynamoDbRepository,
    sqs_client: &SqsClient,
    queue_url: &str,
    item: Item,
) -> Result<Response<Body>, AppError> {
    // Validate item
    validate_item(&item)?;
    
    // Save item to DynamoDB
    repo.create_item(&item).await?;
    
    // Create an audit record
    let audit = create_audit_record("create", &item, None, "request-id");
    
    // In a real application, you would store the audit record
    // For now, we'll just log it
    info!(
        action = %audit.action,
        resource_id = %audit.resource_id,
        resource_type = %audit.resource_type,
        user_id = %audit.user_id,
        "Item created"
    );
    
    // Create an event for the item creation
    let event = shared::models::ItemEvent {
        event_type: shared::models::ItemEventType::Created,
        item: item.clone(),
        timestamp: chrono::Utc::now(),
    };
    
    // Serialize the event to JSON
    let event_json = serde_json::to_string(&event)?;
    
    // Send the event to SQS
    sqs_client.send_message()
        .queue_url(queue_url)
        .message_body(event_json)
        .send()
        .await
        .map_err(|e| AppError::Sqs(e.to_string()))?;
    
    // Create a successful response
    let response = ApiResponse {
        status_code: 201,
        body: item,
    };
    
    // Serialize the response body to JSON
    let body = serde_json::to_string(&response.body)?;
    
    // Build and return the HTTP response
    Ok(Response::builder()
        .status(response.status_code)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .map_err(|e| AppError::Internal(e.to_string()))?)
}

/// Handler for DELETE /items/{id} endpoint
///
/// Deletes an item from the database and sends a deletion event to SQS.
///
/// # Arguments
///
/// * `repo` - The DynamoDB repository for data access
/// * `sqs_client` - The SQS client for sending events
/// * `queue_url` - The URL of the SQS queue for events
/// * `id` - The ID of the item to delete
///
/// # Returns
///
/// * `Result<Response<Body>, AppError>` - A success response or an error
async fn delete_item(
    repo: &DynamoDbRepository,
    sqs_client: &SqsClient,
    queue_url: &str,
    id: &str,
) -> Result<Response<Body>, AppError> {
    // Check if item exists
    let item = repo.get_item(id).await?;
    
    match item {
        Some(item) => {
            // Create an audit record with the previous state
            let previous_state = serde_json::to_string(&item).ok();
            let audit = create_audit_record("delete", &item, previous_state, "request-id");
            
            // Delete item from DynamoDB
            repo.delete_item(id).await?;
            
            // In a real application, you would store the audit record
            // For now, we'll just log it
            info!(
                action = %audit.action,
                resource_id = %audit.resource_id,
                resource_type = %audit.resource_type,
                user_id = %audit.user_id,
                "Item deleted"
            );
            
            // Create an event for the item deletion
            let event = shared::models::ItemEvent {
                event_type: shared::models::ItemEventType::Deleted,
                item,
                timestamp: chrono::Utc::now(),
            };
            
            // Serialize the event to JSON
            let event_json = serde_json::to_string(&event)?;
            
            // Send the event to SQS
            sqs_client.send_message()
                .queue_url(queue_url)
                .message_body(event_json)
                .send()
                .await
                .map_err(|e| AppError::Sqs(e.to_string()))?;
            
            // Build and return a 204 No Content response
            Ok(Response::builder()
                .status(204)
                .body(Body::Empty)
                .map_err(|e| AppError::Internal(e.to_string()))?)
        },
        None => Err(AppError::NotFound(format!("Item with ID {} not found", id))),
    }
}

/// Helper function to create an error response
///
/// # Arguments
///
/// * `message` - The error message
/// * `status_code` - The HTTP status code
///
/// # Returns
///
/// * `Response<Body>` - An HTTP response with the error message
fn error_response(message: String, status_code: u16) -> Response<Body> {
    let error = ErrorResponse { message };
    let body = serde_json::to_string(&error).unwrap_or_else(|_| {
        json!({ "message": "Error serializing error response" }).to_string()
    });
    
    Response::builder()
        .status(status_code)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .unwrap_or_else(|_| {
            let fallback_body = json!({ "message": "Internal server error" }).to_string();
            Response::builder()
                .status(500)
                .header("Content-Type", "application/json")
                .body(Body::from(fallback_body))
                .unwrap()
        })
} 