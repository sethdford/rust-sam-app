#[cfg(test)]
mod tests {
    use aws_lambda_events::sqs::{SqsEvent, SqsMessage};
    use lambda_runtime::LambdaEvent;
    use shared::{
        models::{Item, ItemEvent, ItemEventType},
        repository::DynamoDbRepository,
    };
    use chrono::{DateTime, Utc};
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;

    // Mock DynamoDB repository
    struct MockDynamoDbRepository {
        items: Arc<Mutex<HashMap<String, Item>>>,
        processed_events: Arc<Mutex<Vec<ItemEvent>>>,
    }

    impl MockDynamoDbRepository {
        fn new() -> Self {
            Self {
                items: Arc::new(Mutex::new(HashMap::new())),
                processed_events: Arc::new(Mutex::new(Vec::new())),
            }
        }

        async fn create_item(&self, item: &Item) -> Result<(), aws_sdk_dynamodb::Error> {
            let mut items = self.items.lock().unwrap();
            items.insert(item.id.clone(), item.clone());
            Ok(())
        }

        async fn get_item(&self, id: &str) -> Result<Option<Item>, aws_sdk_dynamodb::Error> {
            let items = self.items.lock().unwrap();
            Ok(items.get(id).cloned())
        }

        async fn list_items(&self) -> Result<Vec<Item>, aws_sdk_dynamodb::Error> {
            let items = self.items.lock().unwrap();
            Ok(items.values().cloned().collect())
        }

        async fn delete_item(&self, id: &str) -> Result<(), aws_sdk_dynamodb::Error> {
            let mut items = self.items.lock().unwrap();
            items.remove(id);
            Ok(())
        }

        fn record_processed_event(&self, event: ItemEvent) {
            let mut events = self.processed_events.lock().unwrap();
            events.push(event);
        }

        fn get_processed_events(&self) -> Vec<ItemEvent> {
            let events = self.processed_events.lock().unwrap();
            events.clone()
        }
    }

    // Helper function to create a test item
    fn create_test_item() -> Item {
        Item {
            id: "test-id".to_string(),
            name: "Test Item".to_string(),
            description: Some("Test Description".to_string()),
            created_at: DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        }
    }

    // Helper function to create a test SQS event
    fn create_test_sqs_event(event_type: ItemEventType, item: Item) -> SqsEvent {
        let item_event = ItemEvent {
            event_type,
            item,
            timestamp: Utc::now(),
        };

        let event_json = serde_json::to_string(&item_event).unwrap();

        SqsEvent {
            records: vec![SqsMessage {
                message_id: Some("test-message-id".to_string()),
                receipt_handle: Some("test-receipt-handle".to_string()),
                body: Some(event_json),
                md5_of_body: Some("test-md5".to_string()),
                md5_of_message_attributes: None,
                attributes: Default::default(),
                message_attributes: Default::default(),
                event_source_arn: Some("arn:aws:sqs:us-east-1:123456789012:test-queue".to_string()),
                event_source: Some("aws:sqs".to_string()),
                aws_region: Some("us-east-1".to_string()),
            }],
        }
    }

    // Tests would go here, but they require access to the handler functions
    // which are not easily testable without refactoring the main.rs file
    // to separate the handler logic from the AWS Lambda runtime setup.
    //
    // In a real-world scenario, we would:
    // 1. Refactor the handler functions to be more testable
    // 2. Create unit tests for each event type
    // 3. Use the mock repository for testing
} 