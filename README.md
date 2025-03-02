# Rust SAM Application

A serverless application built with AWS SAM and Rust, following AWS Well-Architected best practices.

## Architecture

This project implements a serverless API for managing items with the following components:

- **API Handler Lambda**: Processes HTTP requests and interacts with DynamoDB
- **Event Processor Lambda**: Processes events from SQS for asynchronous workflows
- **DynamoDB Table**: Stores item data
- **SQS Queue**: Handles asynchronous event processing
- **Dead Letter Queue**: Captures failed event processing for retry/analysis
- **CloudWatch Alarms**: Monitors for errors and DLQ messages

## Well-Architected Best Practices

This project follows AWS Well-Architected Framework best practices:

### Operational Excellence
- Structured logging with tracing
- CloudWatch alarms for monitoring
- CI/CD pipeline support
- Environment-based configuration

### Security
- Least privilege IAM permissions
- Input validation
- Error handling that doesn't leak implementation details

### Reliability
- Dead Letter Queue for failed messages
- Retry mechanisms
- Proper error handling

### Performance Efficiency
- ARM64 architecture for better price/performance
- Asynchronous processing with SQS
- DynamoDB on-demand capacity

### Cost Optimization
- Serverless architecture
- Pay-per-request DynamoDB
- Right-sized Lambda functions

### Sustainability
- ARM64 architecture (lower energy consumption)
- Efficient code with Rust
- Serverless architecture (shared resources)

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [AWS SAM CLI](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-sam-cli-install.html)
- [AWS CLI](https://aws.amazon.com/cli/)
- [Docker](https://www.docker.com/products/docker-desktop) (for local testing)
- [cargo-lambda](https://www.cargo-lambda.info/guide/installation.html)

## Project Structure

```
rust-sam-app/
├── api-handler/            # API Lambda function
│   ├── src/
│   └── Cargo.toml
├── event-processor/        # Event processor Lambda function
│   ├── src/
│   └── Cargo.toml
├── shared/                 # Shared code library
│   ├── src/
│   └── Cargo.toml
├── Cargo.toml              # Workspace configuration
├── template.yml           # SAM template
├── samconfig.toml          # SAM CLI configuration
└── README.md
```

## Building and Deploying

### Local Development

1. Build the project:
   ```
   sam build
   ```

2. Run locally:
   ```
   sam local start-api
   ```

3. Invoke a function locally:
   ```
   sam local invoke ApiFunction --event events/api-event.json
   ```

### Deployment

1. Deploy to AWS:
   ```
   sam deploy --guided
   ```

2. For subsequent deployments:
   ```
   sam deploy
   ```

## API Endpoints

- `GET /items` - List all items
- `GET /items/{id}` - Get a specific item
- `POST /items` - Create a new item
- `DELETE /items/{id}` - Delete an item

## Example Item JSON

```json
{
  "name": "Example Item",
  "description": "This is an example item"
}
```

## Testing

Run unit tests:
```
cargo test
```

## License

This project is licensed under the MIT License - see the LICENSE file for details. 