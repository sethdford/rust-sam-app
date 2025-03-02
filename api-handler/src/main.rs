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

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    info!("API Handler Lambda starting up");
    
    // Load configuration
    let config = AppConfig::from_env();
    
    // Initialize AWS SDK
    let aws_config = aws_config::load_from_env().await;
    let repo = DynamoDbRepository::new(&aws_config);
    let sqs_client = SqsClient::new(&aws_config);
    
    // Get SQS queue URL from environment
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

    // Run the Lambda service
    run(service_fn(|event: Request| {
        handle_request(event, &repo, &sqs_client, &queue_url)
    })).await?;

    Ok(())
}

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
        ("GET", "/items") => get_items(repo).await,
        ("GET", p) if p.starts_with("/items/") => {
            let id = p.trim_start_matches("/items/");
            get_item(repo, id).await
        },
        ("POST", "/items") => {
            let body = event.body();
            let item: Item = match body {
                Body::Text(text) => serde_json::from_str(text)?,
                Body::Binary(bytes) => serde_json::from_slice(bytes)?,
                _ => return Ok(error_response("Invalid request body".to_string(), 400)),
            };
            create_item(repo, sqs_client, queue_url, item).await
        },
        ("DELETE", p) if p.starts_with("/items/") => {
            let id = p.trim_start_matches("/items/");
            delete_item(repo, sqs_client, queue_url, id).await
        },
        _ => Ok(error_response("Not found".to_string(), 404)),
    };

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

async fn get_items(repo: &DynamoDbRepository) -> Result<Response<Body>, AppError> {
    let items = repo.list_items().await?;
    
    let response = ApiResponse {
        status_code: 200,
        body: items,
    };
    
    let body = serde_json::to_string(&response.body)?;
    
    Ok(Response::builder()
        .status(response.status_code)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .map_err(|e| AppError::Internal(e.to_string()))?)
}

async fn get_item(repo: &DynamoDbRepository, id: &str) -> Result<Response<Body>, AppError> {
    let item = repo.get_item(id).await?;
    
    match item {
        Some(item) => {
            let response = ApiResponse {
                status_code: 200,
                body: item,
            };
            
            let body = serde_json::to_string(&response.body)?;
            
            Ok(Response::builder()
                .status(response.status_code)
                .header("Content-Type", "application/json")
                .body(Body::from(body))
                .map_err(|e| AppError::Internal(e.to_string()))?)
        },
        None => Err(AppError::NotFound(format!("Item with ID {} not found", id))),
    }
}

async fn create_item(
    repo: &DynamoDbRepository,
    sqs_client: &SqsClient,
    queue_url: &str,
    item: Item,
) -> Result<Response<Body>, AppError> {
    // Validate item
    if item.name.is_empty() {
        return Err(AppError::Validation("Item name cannot be empty".to_string()));
    }
    
    // Save item to DynamoDB
    repo.create_item(&item).await?;
    
    // Send event to SQS
    let event = shared::models::ItemEvent {
        event_type: shared::models::ItemEventType::Created,
        item: item.clone(),
        timestamp: chrono::Utc::now(),
    };
    
    let event_json = serde_json::to_string(&event)?;
    
    sqs_client.send_message()
        .queue_url(queue_url)
        .message_body(event_json)
        .send()
        .await
        .map_err(|e| AppError::Sqs(e.to_string()))?;
    
    // Return response
    let response = ApiResponse {
        status_code: 201,
        body: item,
    };
    
    let body = serde_json::to_string(&response.body)?;
    
    Ok(Response::builder()
        .status(response.status_code)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .map_err(|e| AppError::Internal(e.to_string()))?)
}

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
            // Delete item from DynamoDB
            repo.delete_item(id).await?;
            
            // Send event to SQS
            let event = shared::models::ItemEvent {
                event_type: shared::models::ItemEventType::Deleted,
                item,
                timestamp: chrono::Utc::now(),
            };
            
            let event_json = serde_json::to_string(&event)?;
            
            sqs_client.send_message()
                .queue_url(queue_url)
                .message_body(event_json)
                .send()
                .await
                .map_err(|e| AppError::Sqs(e.to_string()))?;
            
            // Return response
            Ok(Response::builder()
                .status(204)
                .body(Body::Empty)
                .map_err(|e| AppError::Internal(e.to_string()))?)
        },
        None => Err(AppError::NotFound(format!("Item with ID {} not found", id))),
    }
}

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