#!/bin/bash

# Test script for user creation API
# This script tests the updated user creation endpoint

echo "Testing user creation with display_name and password fields..."

# Test data
TEST_DATA='{
  "email": "test@example.com",
  "role": "user-basic-001", 
  "display_name": "Test User",
  "password": "temporary123"
}'

echo "Test payload:"
echo "$TEST_DATA" | jq

echo "Note: This requires a running backend server and valid JWT token"
echo "To test with curl:"
echo "curl -X POST http://localhost:8080/api/v1/admin/users \\"
echo "  -H \"Content-Type: application/json\" \\"
echo "  -H \"Authorization: Bearer YOUR_JWT_TOKEN\" \\"
echo "  -d '$TEST_DATA'"