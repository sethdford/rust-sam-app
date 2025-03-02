#!/bin/bash

# GitHub username
USERNAME="sethdford"
# Repository name
REPO_NAME="rust-sam-app"
# Repository description
DESCRIPTION="A serverless application built with AWS SAM and Rust, following AWS Well-Architected best practices"

# Prompt for GitHub personal access token
echo "Please enter your GitHub personal access token:"
read -s TOKEN
echo

# Create the repository using GitHub API
echo "Creating repository $REPO_NAME..."
curl -i -H "Authorization: token $TOKEN" \
     -d "{\"name\":\"$REPO_NAME\",\"description\":\"$DESCRIPTION\"}" \
     https://api.github.com/user/repos

echo
echo "If the repository was created successfully, run:"
echo "git push -u origin master" 