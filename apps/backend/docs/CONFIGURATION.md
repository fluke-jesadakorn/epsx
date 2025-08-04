# EPSX Backend Configuration Guide

This document describes how to configure the EPSX backend application through environment variables and configuration files.

## Configuration System Overview

The EPSX backend uses a comprehensive configuration system that eliminates hard-coded values throughout the codebase. All configuration is loaded from environment variables with sensible defaults.

### Configuration Structure

The configuration is organized into logical sections:

- **Core Application Settings**: Basic server configuration
- **Database Configuration**: PostgreSQL connection settings
- **Authentication & Security**: JWT, Firebase, and session settings
- **Email Configuration**: SendGrid integration settings
- **Payment Gateway Configuration**: Payment processing settings
- **Market Data Services**: External data provider settings
- **External Services Configuration**: Third-party API settings
- **Rate Limiting Configuration**: Request throttling settings
- **Business Configuration**: Business logic parameters
- **Branding & UI Configuration**: Customizable branding elements
- **Network Configuration**: Network-related settings
- **Cache & Redis Configuration**: Caching settings
- **Logging & Monitoring**: Observability settings

## Environment Variables

### Core Application Settings

```bash
PORT=8080                    # Server port (default: 8080)
HOST=0.0.0.0                # Host to bind to (default: 0.0.0.0)
BIND_ADDRESS=0.0.0.0        # Bind address (default: 0.0.0.0)
DEFAULT_HOST=127.0.0.1      # Default host for internal connections
FRONTEND_URL=http://localhost:3000  # Frontend application URL
ENVIRONMENT=development      # Environment: development, staging, production
```

### Database Configuration

```bash
DATABASE_URL=postgresql://postgres:password@localhost:5432/epsx_db
DATABASE_HOST=localhost
DATABASE_PORT=5432
DATABASE_NAME=epsx_db
DATABASE_USER=postgres
DATABASE_PASSWORD=password
DATABASE_MAX_CONNECTIONS=10
DATABASE_QUERY_TIMEOUT_SECONDS=30
```

### Authentication & Security

```bash
JWT_SECRET=change_this_in_production_to_a_secure_random_string
JWT_EXPIRY_HOURS=24
SESSION_TTL_SECONDS=3600

# Firebase Configuration
FIREBASE_PROJECT_ID=epsx-trading-platform
FIREBASE_SERVICE_KEY_PATH=/path/to/firebase-service-key.json
FIREBASE_AUTH_API_BASE_URL=https://identitytoolkit.googleapis.com/v1/accounts
```

### Email Configuration (SendGrid)

```bash
SENDGRID_API_KEY=your_sendgrid_api_key_here
FROM_EMAIL=noreply@epsx.com
FROM_NAME=EPSX Trading Platform
EMAIL_ENABLED=true
```

### Payment Gateway Configuration

```bash
COINPAYMENTS_API_KEY=your_coinpayments_api_key
COINPAYMENTS_API_SECRET=your_coinpayments_api_secret
COINPAYMENTS_API_URL=https://www.coinpayments.net/api.php
PAYMENT_WEBHOOK_URL=https://yourdomain.com/api/webhooks/payments
PAYMENT_CONFIRMATION_COUNT=3
CHECKOUT_URL_TEMPLATE=https://checkout.example.com/pay/{}
```

### Market Data Services

```bash
ALPHA_VANTAGE_API_KEY=your_alpha_vantage_api_key
ALPHA_VANTAGE_API_URL=https://www.alphavantage.co/query
TRADINGVIEW_ORIGIN_URL=https://www.tradingview.com
TRADINGVIEW_SCANNER_API_URL=https://scanner.tradingview.com/global/scan
TRADINGVIEW_HTTP_TIMEOUT=10
```

### External Services Configuration

```bash
# QR Code Service
QR_CODE_API_BASE_URL=https://api.qrserver.com/v1/create-qr-code/
QR_CODE_DEFAULT_SIZE=200x200
QR_CODE_MAX_SIZE=1000

# Firebase Auth API
FIREBASE_SIGNIN_ENDPOINT=signInWithPassword
FIREBASE_LOOKUP_ENDPOINT=lookup
FIREBASE_SIGNUP_ENDPOINT=signUp
```

### Rate Limiting Configuration

```bash
# Default Rate Limits
DEFAULT_RATE_LIMIT_PER_MINUTE=60
DEFAULT_RATE_LIMIT_PER_HOUR=1000
DEFAULT_RATE_LIMIT_PER_DAY=10000

# Endpoint-Specific Rate Limits
RATE_LIMIT_LOGIN_PER_MINUTE=5
RATE_LIMIT_PAYMENT_PER_MINUTE=10
```

### Business Configuration

```bash
SUPPORTED_CURRENCIES=USD,EUR,GBP,JPY,AUD,CAD,CHF,CNY,SEK,NZD
DEFAULT_CURRENCY=USD
FEATURE_EXPIRATION_WARNING_DAYS=30,7,3,1
```

### Branding & UI Configuration

```bash
COMPANY_NAME=EPSX
PLATFORM_NAME=EPSX Trading Platform
SUPPORT_EMAIL=support@epsx.com
DASHBOARD_URL=https://epsx.com/dashboard
WELCOME_MESSAGE_TEMPLATE=Welcome to {}, {}!
```

## Configuration Best Practices

### Security

1. **Never commit secrets to version control**
2. **Use strong, unique values for JWT_SECRET in production**
3. **Store API keys and passwords securely**
4. **Use environment-specific configurations**

### Environment Management

1. **Development**: Use `.env` file with development values
2. **Staging**: Use staging-specific environment variables
3. **Production**: Use secure secret management (AWS Secrets Manager, etc.)

### Validation

The configuration system includes validation for:
- Required fields (will cause startup failure if missing)
- Numeric values (ports, timeouts, etc.)
- URL format validation
- Currency code validation
- Rate limit ranges

## Configuration Loading

Configuration is loaded in the following order (later values override earlier ones):

1. Default values (hardcoded in configuration structs)
2. Environment variables
3. `.env` file (if present)

## Runtime Configuration Updates

Some configuration values can be updated at runtime through:

1. **Database-stored settings** (for business logic parameters)
2. **Admin API endpoints** (for rate limits and feature flags)
3. **Configuration reload endpoints** (for non-security-critical settings)

## Troubleshooting

### Common Issues

1. **Startup Failure**: Check required environment variables
2. **Database Connection**: Verify DATABASE_URL format and credentials
3. **External Service Errors**: Check API keys and endpoint URLs
4. **Rate Limiting Issues**: Verify rate limit configurations

### Debug Mode

Enable debug logging with:
```bash
RUST_LOG=debug
LOG_LEVEL=debug
DEBUG_MODE=true
```

### Configuration Validation

The application validates configuration on startup and will:
- **Fail fast** for critical missing configuration
- **Log warnings** for non-critical missing configuration
- **Use defaults** where appropriate

## Migration from Hard-coded Values

This configuration system replaces the following hard-coded values:

- External API URLs (TradingView, QR code service, etc.)
- Company branding and messaging
- Rate limiting thresholds
- Timeout values
- Currency lists
- Email templates and URLs
- Network configuration

## Examples

### Development Environment

```bash
# .env.development
PORT=8080
FRONTEND_URL=http://localhost:3000
DATABASE_URL=postgresql://postgres:password@localhost:5432/epsx_dev
EMAIL_ENABLED=false
USE_MOCK_EMAIL=true
DEBUG_MODE=true
```

### Production Environment

```bash
# Production environment variables
PORT=80
FRONTEND_URL=https://app.epsx.com
DATABASE_URL=postgresql://prod_user:secure_password@prod-db:5432/epsx_prod
EMAIL_ENABLED=true
SENDGRID_API_KEY=SG.production_key_here
JWT_SECRET=very_secure_production_secret_at_least_32_chars
DEBUG_MODE=false
```

## Support

For configuration questions or issues:
1. Check the `.env.example` file for reference
2. Review this documentation
3. Check application logs for configuration warnings
4. Contact the development team for production configuration assistance