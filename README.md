# Deep Risk Model Lambda Function

This project contains a Lambda function implementation of the Deep Risk Model, deployed using AWS SAM.

## Prerequisites

- AWS CLI configured with appropriate credentials
- AWS SAM CLI installed
- Rust toolchain installed
- Docker (for local testing)

## Project Structure

```
.
├── src/
│   ├── lambda.rs      # Lambda handler implementation
│   └── ...           # Other source files
├── template.yaml     # SAM template
├── samconfig.toml    # SAM configuration
├── deploy.sh         # Deployment script
└── Cargo.toml        # Rust dependencies
```

## Local Development

1. Build the project:
```bash
cargo build
```

2. Run tests:
```bash
cargo test
```

3. Test locally using SAM:
```bash
sam local start-api
```

## Deployment

1. Update the `samconfig.toml` with your AWS account details:
   - Replace `XXXXXXXXXXXX` in the S3 bucket name
   - Replace `XXXXXXXXXXXX` in the ECR repository URL
   - Update the region if needed

2. Run the deployment script:
```bash
./deploy.sh
```

3. Follow the SAM CLI prompts to complete the deployment.

## API Endpoints

The Lambda function exposes the following endpoints:

- `POST /factors`: Generate risk factors
- `POST /covariance`: Estimate covariance matrix
- `GET /health`: Health check endpoint

## Environment Variables

The following environment variables are required:

- `MODEL_BUCKET`: S3 bucket name for model data
- `AWS_REGION`: AWS region (default: us-east-1)

## Monitoring and Logging

- Logs are available in CloudWatch Logs
- Metrics are available in CloudWatch Metrics
- X-Ray tracing is enabled for request tracking

## Security

- IAM roles and policies are managed through the SAM template
- API Gateway endpoints are secured with IAM authentication
- S3 bucket access is restricted to the Lambda function

## Troubleshooting

1. Check CloudWatch Logs for detailed error messages
2. Verify IAM permissions in the Lambda execution role
3. Ensure all environment variables are set correctly
4. Check API Gateway logs for request/response issues

## Contributing

Please read CONTRIBUTING.md for details on our code of conduct and the process for submitting pull requests.

## License

This project is licensed under the MIT License - see the LICENSE file for details. 