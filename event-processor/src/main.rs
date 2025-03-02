use lambda_runtime::{service_fn, Error, LambdaEvent};
use aws_lambda_events::sqs::{SqsEvent, SqsMessage};
use tracing::{info, error};
use shared::{
    models::{ItemEvent, ItemEventType},
    repository::DynamoDbRepository,
    config::AppConfig,
    AppError,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    info!("Event Processor Lambda starting up");
    
    // Load configuration
    let config = AppConfig::from_env();
    
    // Initialize AWS SDK
    let aws_config = aws_config::load_from_env().await;
    let repo = DynamoDbRepository::new(&aws_config);

    // Run the Lambda service
    lambda_runtime::run(service_fn(|event: LambdaEvent<SqsEvent>| {
        handle_event(event, &repo)
    })).await?;

    Ok(())
}

async fn handle_event(
    event: LambdaEvent<SqsEvent>,
    repo: &DynamoDbRepository,
) -> Result<(), Error> {
    let (event, _context) = event.into_parts();
    
    info!("Processing {} SQS messages", event.records.len());
    
    for record in event.records {
        process_sqs_message(record, repo).await?;
    }
    
    Ok(())
}

async fn process_sqs_message(
    message: SqsMessage,
    repo: &DynamoDbRepository,
) -> Result<(), Error> {
    let body = message.body.as_deref().ok_or_else(|| {
        error!("SQS message has no body");
        AppError::Internal("SQS message has no body".to_string())
    })?;
    
    info!("Processing SQS message: {}", message.message_id.as_deref().unwrap_or("unknown"));
    
    // Parse the event
    let item_event: ItemEvent = serde_json::from_str(body)?;
    
    // Process based on event type
    match item_event.event_type {
        ItemEventType::Created => {
            info!("Item created event for item ID: {}", item_event.item.id);
            // In a real application, you might want to do additional processing here
            // For example, send a notification, update analytics, etc.
            tokio::time::sleep(Duration::from_millis(100)).await; // Simulate processing
        },
        ItemEventType::Updated => {
            info!("Item updated event for item ID: {}", item_event.item.id);
            // Process item update
            tokio::time::sleep(Duration::from_millis(100)).await; // Simulate processing
        },
        ItemEventType::Deleted => {
            info!("Item deleted event for item ID: {}", item_event.item.id);
            // Process item deletion
            // For example, clean up related resources
            tokio::time::sleep(Duration::from_millis(100)).await; // Simulate processing
        },
    }
    
    info!("Successfully processed event for item ID: {}", item_event.item.id);
    
    Ok(())
} 