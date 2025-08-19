#!/bin/bash

echo "🔍 Debugging PancakeSwap Template Serving"
echo "=========================================="

echo "1. Testing OIDC endpoint directly..."
curl -v "http://localhost:8080/oauth/authorize?response_type=code&client_id=epsx-frontend&redirect_uri=https%3A%2F%2Fepsx.io%2Fapi%2Fauth%2Fcallback%2Fepsx-backend&scope=openid+profile+email&state=test&code_challenge=test&code_challenge_method=S256" > /tmp/pancake_debug.html

echo ""
echo "2. Checking if PancakeSwap theme elements exist in response..."
if grep -q "Pancake Stack Login" /tmp/pancake_debug.html; then
    echo "✅ Found 'Pancake Stack Login' - PancakeSwap theme is working!"
else
    echo "❌ Missing 'Pancake Stack Login' - Default template being served"
fi

if grep -q "🥞" /tmp/pancake_debug.html; then
    echo "✅ Found pancake emoji - Theme elements present"
else
    echo "❌ Missing pancake emoji - Theme elements missing"
fi

if grep -q "FFB347\|pancake-primary" /tmp/pancake_debug.html; then
    echo "✅ Found PancakeSwap colors - CSS styling present"
else
    echo "❌ Missing PancakeSwap colors - CSS styling missing"
fi

echo ""
echo "3. Template content preview:"
echo "----------------------------"
head -30 /tmp/pancake_debug.html

echo ""
echo "4. Checking for error indicators..."
if grep -q "error\|Error\|ERROR" /tmp/pancake_debug.html; then
    echo "⚠️ Found error indicators in response"
    grep -i "error" /tmp/pancake_debug.html
else
    echo "✅ No obvious error indicators found"
fi

echo ""
echo "Debug complete. Check /tmp/pancake_debug.html for full response."