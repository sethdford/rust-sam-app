# Testing Guide for Rust SAM Application

This document provides guidance on how to test the Rust SAM application, including unit tests, integration tests, and local testing with SAM CLI.

## Test Structure

The application includes test modules for each component:

- **API Handler Tests**: `api-handler/src/tests.rs`
- **Event Processor Tests**: `event-processor/src/tests.rs`
- **Shared Library Tests**: `shared/src/tests.rs`

## Running Unit Tests

You can run all tests using Cargo:

```bash
# Run all tests
cargo test --all

# Run tests for a specific component
cargo test -p api-handler
cargo test -p event-processor
cargo test -p shared
```

## Test Events

The `events/test/` directory contains sample events for testing the Lambda functions:

- `get-items.json` - API Gateway event for listing all items
- `get-item.json` - API Gateway event for getting a specific item
- `create-item.json` - API Gateway event for creating a new item
- `delete-item.json` - API Gateway event for deleting an item
- `sqs-event.json` - SQS event for the event processor

## Local Testing with SAM CLI

You can use the SAM CLI to test the Lambda functions locally:

### Testing the API Handler

```bash
# Start the local API Gateway
sam local start-api

# Test with curl
curl http://localhost:3000/items
curl -X POST http://localhost:3000/items -H "Content-Type: application/json" -d '{"name":"Test Item","description":"This is a test item"}'
curl http://localhost:3000/items/{id}
curl -X DELETE http://localhost:3000/items/{id}

# Invoke with a specific event
sam local invoke ApiHandler -e events/test/get-items.json
sam local invoke ApiHandler -e events/test/get-item.json
sam local invoke ApiHandler -e events/test/create-item.json
sam local invoke ApiHandler -e events/test/delete-item.json
```

### Testing the Event Processor

```bash
# Invoke with a specific event
sam local invoke EventProcessor -e events/test/sqs-event.json
```

## Mock Implementations

The test modules include mock implementations for external dependencies:

### API Handler Mocks

- `MockDynamoDbRepository`: Mocks the DynamoDB repository for testing CRUD operations
- `MockSqsClient`: Mocks the SQS client for testing event publishing

### Event Processor Mocks

- `MockDynamoDbRepository`: Mocks the DynamoDB repository for testing event processing

## Writing New Tests

When writing new tests, follow these guidelines:

1. **Use the existing mock implementations** to avoid making actual AWS calls
2. **Create test helper functions** for common setup tasks
3. **Test both success and error cases** for comprehensive coverage
4. **Use descriptive test names** that explain what is being tested

Example test structure:

```rust
#[tokio::test]
async fn test_get_item_success() {
    // Setup
    let mock_repo = MockDynamoDbRepository::new();
    let test_item = create_test_item();
    mock_repo.create_item(&test_item).await.unwrap();
    
    // Execute
    let result = get_item(&mock_repo, &test_item.id).await;
    
    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(200, response.status_code);
    // Additional assertions...
}
```

## Integration Testing

For integration testing with actual AWS services, you can deploy a test stack:

```bash
# Deploy a test stack
sam deploy --stack-name rust-sam-test --parameter-overrides Environment=test

# Run integration tests against the test stack
# (You would need to implement these tests separately)

# Clean up the test stack
aws cloudformation delete-stack --stack-name rust-sam-test
```

## Continuous Integration

The project is set up for continuous integration testing. When you push changes to the repository, the CI pipeline will:

1. Build the application
2. Run unit tests
3. Deploy to a test environment
4. Run integration tests
5. Clean up the test environment

## Troubleshooting

If you encounter issues with tests:

1. **Check the test logs** for detailed error messages
2. **Verify that mock implementations** correctly simulate the behavior of real services
3. **Ensure that test events** contain valid data
4. **Check for environment variables** that might be required by the tests 