#!/bin/bash

# Test script for bearer token authentication flow
BACKEND_URL="http://localhost:8080"

echo "🔐 Testing Bearer Token Authentication Flow"
echo "========================================="

# Test 1: Health check (public endpoint)
echo ""
echo "1. Testing health check (public endpoint)..."
HEALTH_RESPONSE=$(curl -s -w "%{http_code}" -o /dev/null "${BACKEND_URL}/health")
if [ "$HEALTH_RESPONSE" = "200" ]; then
    echo "✅ Health check passed"
else
    echo "❌ Health check failed: HTTP $HEALTH_RESPONSE"
    exit 1
fi

# Test 2: Try accessing protected endpoint without token
echo ""
echo "2. Testing protected endpoint without token..."
PROTECTED_RESPONSE=$(curl -s -w "%{http_code}" -o /dev/null "${BACKEND_URL}/api/v1/auth/me")
if [ "$PROTECTED_RESPONSE" = "401" ]; then
    echo "✅ Protected endpoint correctly returns 401 without token"
else
    echo "❌ Protected endpoint should return 401 without token, got: HTTP $PROTECTED_RESPONSE"
fi

# Test 3: Login and get bearer token
echo ""
echo "3. Testing login to get bearer token..."
LOGIN_RESPONSE=$(curl -s -X POST "${BACKEND_URL}/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "type": "credentials",
    "email": "test@example.com",
    "password": "password123"
  }')

# Check if login response contains access_token
if echo "$LOGIN_RESPONSE" | grep -q "access_token"; then
    BEARER_TOKEN=$(echo "$LOGIN_RESPONSE" | grep -o '"access_token":"[^"]*"' | cut -d'"' -f4)
    echo "✅ Login successful, got bearer token: ${BEARER_TOKEN:0:10}..."
    
    # Test 4: Use bearer token to access protected endpoint
    echo ""
    echo "4. Testing protected endpoint with bearer token..."
    PROTECTED_WITH_TOKEN_RESPONSE=$(curl -s -w "%{http_code}" -o /dev/null \
      "${BACKEND_URL}/api/v1/auth/me" \
      -H "Authorization: Bearer $BEARER_TOKEN")
    
    if [ "$PROTECTED_WITH_TOKEN_RESPONSE" = "200" ]; then
        echo "✅ Protected endpoint accessible with bearer token"
        
        # Test 5: Get user profile with bearer token
        echo ""
        echo "5. Testing user profile retrieval..."
        PROFILE_RESPONSE=$(curl -s "${BACKEND_URL}/api/v1/auth/me" \
          -H "Authorization: Bearer $BEARER_TOKEN")
        
        if echo "$PROFILE_RESPONSE" | grep -q "user_id"; then
            echo "✅ User profile retrieved successfully"
            echo "Profile: $(echo "$PROFILE_RESPONSE" | jq -r '.email // "No email found"') ($(echo "$PROFILE_RESPONSE" | jq -r '.role // "No role found"'))"
        else
            echo "❌ Failed to retrieve user profile"
            echo "Response: $PROFILE_RESPONSE"
        fi
        
        # Test 6: Test session validation
        echo ""
        echo "6. Testing session validation..."
        VALIDATION_RESPONSE=$(curl -s -X POST "${BACKEND_URL}/api/v1/auth/validate-session" \
          -H "Authorization: Bearer $BEARER_TOKEN" \
          -H "Content-Type: application/json" \
          -d '{"app_type": "frontend"}')
        
        if echo "$VALIDATION_RESPONSE" | grep -q "authenticated"; then
            echo "✅ Session validation successful"
        else
            echo "❌ Session validation failed"
            echo "Response: $VALIDATION_RESPONSE"
        fi
        
        # Test 7: Test logout
        echo ""
        echo "7. Testing logout..."
        LOGOUT_RESPONSE=$(curl -s -w "%{http_code}" -o /dev/null \
          -X POST "${BACKEND_URL}/api/v1/auth/logout" \
          -H "Authorization: Bearer $BEARER_TOKEN")
        
        if [ "$LOGOUT_RESPONSE" = "200" ]; then
            echo "✅ Logout successful"
            
            # Test 8: Verify token is invalid after logout
            echo ""
            echo "8. Testing token validity after logout..."
            POST_LOGOUT_RESPONSE=$(curl -s -w "%{http_code}" -o /dev/null \
              "${BACKEND_URL}/api/v1/auth/me" \
              -H "Authorization: Bearer $BEARER_TOKEN")
            
            if [ "$POST_LOGOUT_RESPONSE" = "401" ]; then
                echo "✅ Token correctly invalidated after logout"
            else
                echo "⚠️  Token still valid after logout (HTTP $POST_LOGOUT_RESPONSE) - this might be expected if sessions persist"
            fi
        else
            echo "❌ Logout failed: HTTP $LOGOUT_RESPONSE"
        fi
    else
        echo "❌ Protected endpoint not accessible with bearer token: HTTP $PROTECTED_WITH_TOKEN_RESPONSE"
    fi
else
    echo "❌ Login failed or no access_token in response"
    echo "Response: $LOGIN_RESPONSE"
fi

echo ""
echo "🎯 Bearer Token Authentication Flow Test Complete"
echo "================================================"