#!/bin/bash

echo "Testing rate limiting on premium endpoint with permission middleware..."

# Test the premium endpoint that has permission middleware
echo "Testing premium endpoint with rapid requests:"

# Make 20 rapid requests to see if rate limiting kicks in
for i in {1..20}; do
    response=$(curl -s -w "%{http_code}" http://127.0.0.1:8080/api/v1/premium/rankings)
    status_code="${response: -3}"
    echo "Request $i: Status Code $status_code"
    
    if [ "$status_code" == "429" ]; then
        echo "Rate limiting detected!"
        break
    fi
    
    # Very small delay
    sleep 0.05
done

echo "Rate limiting test on premium endpoint completed."