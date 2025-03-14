#!/bin/bash

# Exit on error
set -e

# Build the Lambda function
echo "Building Lambda function..."
cargo build --release --target x86_64-unknown-linux-gnu

# Create deployment package
echo "Creating deployment package..."
cp target/x86_64-unknown-linux-gnu/release/bootstrap bootstrap
zip function.zip bootstrap
rm bootstrap

# Deploy using SAM
echo "Deploying with SAM..."
sam deploy --guided

echo "Deployment complete!" 