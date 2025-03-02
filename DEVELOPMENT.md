# Development Guide for Rust SAM Application

This guide provides information for developers working on the Rust SAM application.

## Development Environment Setup

### Prerequisites

1. **Install Rust**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Install AWS SAM CLI**
   ```bash
   # macOS
   brew tap aws/tap
   brew install aws-sam-cli

   # Linux/Windows
   # See https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-sam-cli-install.html
   ```

3. **Install Docker**
   Docker is required for local testing with SAM CLI.
   ```bash
   # macOS
   brew install --cask docker

   # Linux/Windows
   # See https://docs.docker.com/get-docker/
   ```

4. **Configure AWS CLI**
   ```bash
   aws configure
   ```

### Clone the Repository

```bash
git clone https://github.com/yourusername/rust-sam.git
cd rust-sam
```

## Project Structure

The application is organized as a workspace with three main components:

- **api-handler**: Lambda function for handling API Gateway requests
- **event-processor**: Lambda function for processing SQS events
- **shared**: Common code shared between the Lambda functions

Each component has its own `Cargo.toml` file and source code.

## Building the Application

```bash
# Build with SAM CLI
sam build

# Build with Cargo
cargo build --release
```

## Local Development

### Running the API Locally

```bash
sam local start-api
```

This will start a local API Gateway that you can use to test the API handler.

### Invoking Lambda Functions Locally

```bash
# Invoke the API handler with a test event
sam local invoke ApiHandler -e events/test/get-items.json

# Invoke the event processor with a test event
sam local invoke EventProcessor -e events/test/sqs-event.json
```

### Local DynamoDB

For local development, you can use DynamoDB Local:

```bash
# Start DynamoDB Local
docker run -p 8000:8000 amazon/dynamodb-local

# Create a table
aws dynamodb create-table \
    --table-name Items \
    --attribute-definitions AttributeName=id,AttributeType=S \
    --key-schema AttributeName=id,KeyType=HASH \
    --billing-mode PAY_PER_REQUEST \
    --endpoint-url http://localhost:8000
```

Set the `DYNAMODB_TABLE` and `AWS_ENDPOINT_URL` environment variables to use DynamoDB Local:

```bash
export DYNAMODB_TABLE=Items
export AWS_ENDPOINT_URL=http://localhost:8000
```

### Local SQS

For local development, you can use LocalStack for SQS:

```bash
# Start LocalStack
docker run -p 4566:4566 localstack/localstack

# Create an SQS queue
aws sqs create-queue \
    --queue-name ItemEvents \
    --endpoint-url http://localhost:4566
```

Set the `EVENT_QUEUE_URL` environment variable to use the local SQS queue:

```bash
export EVENT_QUEUE_URL=http://localhost:4566/000000000000/ItemEvents
```

## Code Organization

### API Handler

The API handler (`api-handler/src/main.rs`) is responsible for:

1. Processing HTTP requests from API Gateway
2. Performing CRUD operations on items in DynamoDB
3. Publishing events to SQS when items are created or deleted

Key functions:
- `handle_request`: Routes requests to the appropriate handler
- `get_items`: Lists all items
- `get_item`: Gets a specific item
- `create_item`: Creates a new item
- `delete_item`: Deletes an item

### Event Processor

The event processor (`event-processor/src/main.rs`) is responsible for:

1. Processing events from SQS
2. Performing additional processing based on the event type

Key functions:
- `handle_event`: Processes SQS events
- `process_sqs_message`: Processes a single SQS message

### Shared Library

The shared library (`shared/src/lib.rs`) contains common code used by both Lambda functions:

- `models.rs`: Data models for items and events
- `repository.rs`: DynamoDB repository for data access
- `error.rs`: Error handling
- `config.rs`: Configuration management

## Adding a New Feature

To add a new feature to the application:

1. **Update the shared library** if needed (e.g., add a new model or repository method)
2. **Update the API handler** if needed (e.g., add a new endpoint)
3. **Update the event processor** if needed (e.g., add handling for a new event type)
4. **Add tests** for the new feature
5. **Update the SAM template** if needed (e.g., add a new resource or permission)

Example: Adding a new endpoint to update an item:

1. Add an update method to the repository:
   ```rust
   // shared/src/repository.rs
   impl DynamoDbRepository {
       pub async fn update_item(&self, item: &Item) -> Result<(), aws_sdk_dynamodb::Error> {
           // Implementation
       }
   }
   ```

2. Add a handler function to the API handler:
   ```rust
   // api-handler/src/main.rs
   async fn update_item(
       repo: &DynamoDbRepository,
       sqs_client: &SqsClient,
       queue_url: &str,
       id: &str,
       item: Item,
   ) -> Result<Response<Body>, AppError> {
       // Implementation
   }
   ```

3. Update the request handler to route PUT requests:
   ```rust
   // api-handler/src/main.rs
   let result = match (method.as_str(), path.as_str()) {
       // Existing routes...
       ("PUT", p) if p.starts_with("/items/") => {
           let id = p.trim_start_matches("/items/");
           let body = event.body();
           let item: Item = match body {
               Body::Text(text) => serde_json::from_str(text)?,
               Body::Binary(bytes) => serde_json::from_slice(bytes)?,
               _ => return Ok(error_response("Invalid request body".to_string(), 400)),
           };
           update_item(repo, sqs_client, queue_url, id, item).await
       },
       // Other routes...
   };
   ```

4. Add a test for the new endpoint:
   ```rust
   // api-handler/src/tests.rs
   #[tokio::test]
   async fn test_update_item() {
       // Implementation
   }
   ```

## Deployment

### Deploying to AWS

```bash
# First-time deployment
sam deploy --guided

# Subsequent deployments
sam deploy
```

### Deployment Parameters

The SAM template supports the following parameters:

- `Environment`: The deployment environment (e.g., dev, test, prod)
- `DynamoDBTableName`: The name of the DynamoDB table
- `SQSQueueName`: The name of the SQS queue

Example:

```bash
sam deploy --parameter-overrides Environment=dev DynamoDBTableName=Items-dev SQSQueueName=ItemEvents-dev
```

## Monitoring and Debugging

### CloudWatch Logs

You can view the Lambda function logs in CloudWatch:

```bash
# View API handler logs
aws logs filter-log-events --log-group-name /aws/lambda/rust-sam-ApiHandler

# View event processor logs
aws logs filter-log-events --log-group-name /aws/lambda/rust-sam-EventProcessor
```

### X-Ray Tracing

The application is configured for X-Ray tracing. You can view traces in the AWS X-Ray console.

## Best Practices

### Error Handling

- Use the `AppError` enum for application-specific errors
- Log errors with appropriate context
- Return user-friendly error messages in API responses

### Logging

- Use structured logging with the `tracing` crate
- Include relevant context in log messages
- Use appropriate log levels (info, warn, error)

### Security

- Follow the principle of least privilege for IAM roles
- Validate input data
- Use environment variables for configuration
- Don't log sensitive information

### Performance

- Keep Lambda functions small and focused
- Use async/await for I/O-bound operations
- Minimize cold start times by keeping dependencies minimal

## Troubleshooting

### Common Issues

1. **Lambda function times out**
   - Check the Lambda function timeout in the SAM template
   - Optimize the function code to complete within the timeout

2. **Permission denied errors**
   - Check the IAM role permissions in the SAM template
   - Ensure the Lambda function has the necessary permissions

3. **DynamoDB errors**
   - Check the DynamoDB table name in the environment variables
   - Ensure the table exists and has the correct schema

4. **SQS errors**
   - Check the SQS queue URL in the environment variables
   - Ensure the queue exists and the Lambda function has permission to access it 