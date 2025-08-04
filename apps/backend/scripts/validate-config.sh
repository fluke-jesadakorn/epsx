#!/bin/bash

# EPSX Backend Configuration Validation Script
# This script helps validate and check configuration settings

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration file path
ENV_FILE=".env"
ENV_EXAMPLE=".env.example"

echo "🔧 EPSX Backend Configuration Validator"
echo "========================================"

# Check if .env file exists
if [ ! -f "$ENV_FILE" ]; then
    echo -e "${YELLOW}Warning: .env file not found${NC}"
    if [ -f "$ENV_EXAMPLE" ]; then
        echo "📋 Found .env.example file. Copy it to .env and configure:"
        echo "cp $ENV_EXAMPLE $ENV_FILE"
    fi
    echo ""
fi

# Function to check if environment variable is set
check_env_var() {
    local var_name=$1
    local required=$2
    local description=$3
    
    if [ -z "${!var_name}" ]; then
        if [ "$required" = "true" ]; then
            echo -e "${RED}❌ Missing required: $var_name${NC} - $description"
            return 1
        else
            echo -e "${YELLOW}⚠️  Optional missing: $var_name${NC} - $description"
            return 0
        fi
    else
        echo -e "${GREEN}✅ Found: $var_name${NC}"
        return 0
    fi
}

# Load .env file if it exists
if [ -f "$ENV_FILE" ]; then
    echo "📁 Loading configuration from $ENV_FILE"
    set -a
    source "$ENV_FILE"
    set +a
    echo ""
fi

# Validation counters
ERRORS=0
WARNINGS=0

echo "🔍 Validating Core Application Settings"
echo "----------------------------------------"
check_env_var "PORT" false "Server port (default: 8080)" || ((ERRORS++))
check_env_var "FRONTEND_URL" true "Frontend application URL" || ((ERRORS++))
check_env_var "ENVIRONMENT" false "Environment: development, staging, production" || ((WARNINGS++))
echo ""

echo "🗄️  Validating Database Configuration" 
echo "--------------------------------------"
check_env_var "DATABASE_URL" true "PostgreSQL connection string" || ((ERRORS++))
if [ -n "$DATABASE_URL" ]; then
    # Validate DATABASE_URL format
    if [[ "$DATABASE_URL" =~ ^postgresql://.*@.*/.* ]]; then
        echo -e "${GREEN}✅ DATABASE_URL format appears valid${NC}"
    else
        echo -e "${RED}❌ DATABASE_URL format appears invalid${NC}"
        ((ERRORS++))
    fi
fi
echo ""

echo "🔐 Validating Authentication & Security"
echo "---------------------------------------"
check_env_var "JWT_SECRET" true "JWT secret key" || ((ERRORS++))
if [ -n "$JWT_SECRET" ] && [ ${#JWT_SECRET} -lt 32 ]; then
    echo -e "${YELLOW}⚠️  JWT_SECRET should be at least 32 characters${NC}"
    ((WARNINGS++))
fi
check_env_var "FIREBASE_PROJECT_ID" true "Firebase project ID" || ((ERRORS++))
echo ""

echo "📧 Validating Email Configuration"
echo "----------------------------------"
EMAIL_ENABLED=${EMAIL_ENABLED:-true}
if [ "$EMAIL_ENABLED" = "true" ]; then
    check_env_var "SENDGRID_API_KEY" true "SendGrid API key" || ((ERRORS++))
    check_env_var "FROM_EMAIL" false "From email address" || ((WARNINGS++))
else
    echo -e "${YELLOW}📧 Email disabled (EMAIL_ENABLED=false)${NC}"
fi
echo ""

echo "💳 Validating Payment Configuration"
echo "-----------------------------------"
check_env_var "PAYMENT_WEBHOOK_URL" false "Payment webhook URL" || ((WARNINGS++))
check_env_var "COINPAYMENTS_API_KEY" false "CoinPayments API key" || ((WARNINGS++))
echo ""

echo "📊 Validating Market Data Services"
echo "----------------------------------"
check_env_var "TRADINGVIEW_ORIGIN_URL" false "TradingView origin URL" || ((WARNINGS++))
check_env_var "ALPHA_VANTAGE_API_KEY" false "Alpha Vantage API key" || ((WARNINGS++))
echo ""

echo "⚡ Validating Rate Limiting"
echo "---------------------------"
check_env_var "DEFAULT_RATE_LIMIT_PER_MINUTE" false "Default rate limit per minute" || ((WARNINGS++))
if [ -n "$DEFAULT_RATE_LIMIT_PER_MINUTE" ]; then
    if ! [[ "$DEFAULT_RATE_LIMIT_PER_MINUTE" =~ ^[0-9]+$ ]]; then
        echo -e "${RED}❌ DEFAULT_RATE_LIMIT_PER_MINUTE must be a number${NC}"
        ((ERRORS++))
    fi
fi
echo ""

echo "🏢 Validating Branding Configuration"
echo "------------------------------------"
check_env_var "COMPANY_NAME" false "Company name" || ((WARNINGS++))
check_env_var "PLATFORM_NAME" false "Platform name" || ((WARNINGS++))
check_env_var "DASHBOARD_URL" false "Dashboard URL" || ((WARNINGS++))
echo ""

# Validate supported currencies
if [ -n "$SUPPORTED_CURRENCIES" ]; then
    echo "💱 Validating Currency Configuration"
    echo "------------------------------------"
    IFS=',' read -ra CURRENCIES <<< "$SUPPORTED_CURRENCIES"
    VALID_CURRENCIES=("USD" "EUR" "GBP" "JPY" "AUD" "CAD" "CHF" "CNY" "SEK" "NZD")
    
    for currency in "${CURRENCIES[@]}"; do
        currency=$(echo "$currency" | tr -d ' ')
        if [[ " ${VALID_CURRENCIES[@]} " =~ " ${currency} " ]]; then
            echo -e "${GREEN}✅ Valid currency: $currency${NC}"
        else
            echo -e "${YELLOW}⚠️  Unknown currency: $currency${NC}"
            ((WARNINGS++))
        fi
    done
    echo ""
fi

# Network connectivity tests (optional)
echo "🌐 Testing Network Connectivity (optional)"
echo "------------------------------------------"
if [ -n "$TRADINGVIEW_SCANNER_API_URL" ]; then
    if curl -s --connect-timeout 5 "$TRADINGVIEW_SCANNER_API_URL" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ TradingView API reachable${NC}"
    else
        echo -e "${YELLOW}⚠️  TradingView API not reachable (might be normal)${NC}"
    fi
fi

if [ -n "$QR_CODE_API_BASE_URL" ]; then
    if curl -s --connect-timeout 5 "$QR_CODE_API_BASE_URL" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ QR Code API reachable${NC}"
    else
        echo -e "${YELLOW}⚠️  QR Code API not reachable (might be normal)${NC}"
    fi
fi
echo ""

# Summary
echo "📋 Validation Summary"
echo "===================="
if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
    echo -e "${GREEN}🎉 All configuration checks passed!${NC}"
    exit 0
elif [ $ERRORS -eq 0 ]; then
    echo -e "${YELLOW}⚠️  Configuration valid with $WARNINGS warnings${NC}"
    echo "The application should start successfully, but some features may not work optimally."
    exit 0
else
    echo -e "${RED}❌ Configuration validation failed with $ERRORS errors and $WARNINGS warnings${NC}"
    echo ""
    echo "📝 Next steps:"
    echo "1. Fix the required configuration errors above"
    echo "2. Copy .env.example to .env and configure missing values"
    echo "3. Run this validator again"
    echo "4. Review the configuration documentation in docs/CONFIGURATION.md"
    exit 1
fi