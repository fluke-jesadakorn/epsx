#!/bin/bash

echo "Testing rate limiting on health endpoint..."

# Test basic functionality first
echo "Testing basic health endpoint:"
curl -s http://127.0.0.1:8080/health | jq .

echo -e "\nTesting rate limiting by making multiple rapid requests:"

# Make 10 rapid requests to see if rate limiting kicks in
for i in {1..10}; do
    response=$(curl -s -w "%{http_code}" http://127.0.0.1:8080/health)
    status_code="${response: -3}"
    echo "Request $i: Status Code $status_code"
    
    if [ "$status_code" == "429" ]; then
        echo "Rate limiting detected!"
        break
    fi
    
    # Small delay to avoid overwhelming
    sleep 0.1
done

echo "Rate limiting test completed."