#[cfg(test)]
mod tests {
    use aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
    use http::Method;
    use lambda_runtime::LambdaEvent;
    use shared::{
        models::{Item, ItemEvent, ItemEventType},
        repository::DynamoDbRepository,
    };
    use aws_sdk_sqs::Client as SqsClient;
    use chrono::{DateTime, Utc};
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;
    use serde_json::json;

    // Mock SQS client
    struct MockSqsClient {
        sent_messages: Arc<Mutex<Vec<String>>>,
    }

    impl MockSqsClient {
        fn new() -> Self {
            Self {
                sent_messages: Arc::new(Mutex::new(Vec::new())),
            }
        }

        async fn send_message(&self, message: &str) -> Result<(), aws_sdk_sqs::Error> {
            let mut messages = self.sent_messages.lock().unwrap();
            messages.push(message.to_string());
            Ok(())
        }

        fn get_sent_messages(&self) -> Vec<String> {
            let messages = self.sent_messages.lock().unwrap();
            messages.clone()
        }
    }

    // Mock DynamoDB repository
    struct MockDynamoDbRepository {
        items: Arc<Mutex<HashMap<String, Item>>>,
    }

    impl MockDynamoDbRepository {
        fn new() -> Self {
            Self {
                items: Arc::new(Mutex::new(HashMap::new())),
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

    // Helper function to create a test API Gateway request
    fn create_api_request(method: Method, path: &str, body: Option<String>) -> ApiGatewayProxyRequest {
        ApiGatewayProxyRequest {
            resource: Some(path.to_string()),
            path: Some(path.to_string()),
            http_method: method.to_string(),
            headers: Default::default(),
            multi_value_headers: Default::default(),
            query_string_parameters: Default::default(),
            multi_value_query_string_parameters: Default::default(),
            path_parameters: Default::default(),
            stage_variables: Default::default(),
            body,
            is_base64_encoded: Some(false),
            request_context: Default::default(),
        }
    }

    // Tests would go here, but they require access to the handler functions
    // which are not easily testable without refactoring the main.rs file
    // to separate the handler logic from the AWS Lambda runtime setup.
    //
    // In a real-world scenario, we would:
    // 1. Refactor the handler functions to be more testable
    // 2. Create unit tests for each API endpoint
    // 3. Use the mock repository and SQS client for testing
    
    // Example test structure (not functional without refactoring):
    /*
    #[tokio::test]
    async fn test_get_items() {
        // Setup
        let mock_repo = MockDynamoDbRepository::new();
        let mock_sqs = MockSqsClient::new();
        
        // Add test items to the mock repository
        let test_item = create_test_item();
        mock_repo.create_item(&test_item).await.unwrap();
        
        // Create a request
        let request = create_api_request(Method::GET, "/items", None);
        let lambda_event = LambdaEvent::new(request, Default::default());
        
        // Call the handler (assuming it was refactored to be testable)
        let response = handle_request(lambda_event, &mock_repo, &mock_sqs).await.unwrap();
        
        // Assertions
        assert_eq!(200, response.status_code.unwrap());
        
        let body = response.body.unwrap();
        let items: Vec<Item> = serde_json::from_str(&body).unwrap();
        
        assert_eq!(1, items.len());
        assert_eq!(test_item.id, items[0].id);
    }
    */
} 