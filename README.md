# Rust SAM Application

A serverless application built with Rust and AWS SAM (Serverless Application Model).

## Architecture Overview

This application implements a serverless microservices architecture with the following components:

1. **API Handler** - A Lambda function that processes API Gateway requests for CRUD operations on items
2. **Event Processor** - A Lambda function that processes events from SQS for asynchronous workflows
3. **Shared Library** - Common code shared between the Lambda functions

### Data Flow

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  API Gateway │────▶│  API Handler │────▶│  DynamoDB   │
└─────────────┘     └──────┬──────┘     └─────────────┘
                           │
                           ▼
                    ┌─────────────┐     ┌─────────────┐
                    │  SQS Queue  │────▶│Event Processor│
                    └─────────────┘     └─────────────┘
```

## Project Structure

```
rust-sam/
├── api-handler/           # API Gateway Lambda handler
│   ├── src/
│   │   ├── main.rs        # Main entry point for API handler
│   │   └── tests.rs       # Tests for API handler
│   └── Cargo.toml         # API handler dependencies
├── event-processor/       # SQS event processor Lambda
│   ├── src/
│   │   ├── main.rs        # Main entry point for event processor
│   │   └── tests.rs       # Tests for event processor
│   └── Cargo.toml         # Event processor dependencies
├── shared/                # Shared code library
│   ├── src/
│   │   ├── lib.rs         # Library entry point
│   │   ├── models.rs      # Data models
│   │   ├── repository.rs  # DynamoDB repository
│   │   ├── error.rs       # Error handling
│   │   ├── config.rs      # Configuration
│   │   └── tests.rs       # Tests for shared library
│   └── Cargo.toml         # Shared library dependencies
├── events/                # Test event payloads
│   └── test/              # Test events for local testing
├── template.yaml         # SAM template
└── README.md             # This file
```

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70.0 or later)
- [AWS SAM CLI](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-sam-cli-install.html)
- [Docker](https://docs.docker.com/get-docker/) (for local testing)
- [AWS CLI](https://aws.amazon.com/cli/) (configured with appropriate credentials)

### Local Development

1. **Clone the repository**

```bash
git clone https://github.com/yourusername/rust-sam.git
cd rust-sam
```

2. **Build the application**

```bash
sam build
```

3. **Run locally**

```bash
sam local start-api
```

4. **Test the API**

```bash
# List all items
curl http://localhost:3000/items

# Create a new item
curl -X POST http://localhost:3000/items \
  -H "Content-Type: application/json" \
  -d '{"name":"Test Item","description":"This is a test item"}'

# Get a specific item
curl http://localhost:3000/items/{id}

# Delete an item
curl -X DELETE http://localhost:3000/items/{id}
```

### Running Tests

```bash
# Run all tests
cargo test --all

# Run tests for a specific component
cargo test -p api-handler
cargo test -p event-processor
cargo test -p shared
```

## Deployment

1. **Build the application**

```bash
sam build
```

2. **Deploy to AWS**

```bash
sam deploy --guided
```

Follow the prompts to configure your deployment.

## Testing with Sample Events

You can use the provided sample events to test your Lambda functions:

```bash
# Test the API handler
sam local invoke ApiHandler -e events/test/get-items.json
sam local invoke ApiHandler -e events/test/get-item.json
sam local invoke ApiHandler -e events/test/create-item.json
sam local invoke ApiHandler -e events/test/delete-item.json

# Test the event processor
sam local invoke EventProcessor -e events/test/sqs-event.json
```

## Code Structure and Design

### API Handler

The API handler processes HTTP requests from API Gateway and performs CRUD operations on items:

- `GET /items` - List all items
- `GET /items/{id}` - Get a specific item
- `POST /items` - Create a new item
- `DELETE /items/{id}` - Delete an item

When items are created or deleted, events are published to an SQS queue for asynchronous processing.

### Event Processor

The event processor consumes events from the SQS queue and performs additional processing based on the event type:

- `Created` - Process item creation events
- `Updated` - Process item update events
- `Deleted` - Process item deletion events

### Shared Library

The shared library contains common code used by both Lambda functions:

- `models.rs` - Data models for items and events
- `repository.rs` - DynamoDB repository for data access
- `error.rs` - Error handling
- `config.rs` - Configuration management

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Commit your changes (`git commit -am 'Add my feature'`)
4. Push to the branch (`git push origin feature/my-feature`)
5. Create a new Pull Request 