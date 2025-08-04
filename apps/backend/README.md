# EPSX Backend - Clean Architecture Implementation

A high-performance Rust backend implementing Clean Architecture principles for the EPSX trading platform, featuring comprehensive authentication, permission management, real-time data processing, and financial analytics.

[![Rust](https://img.shields.io/badge/Rust-2021-000000?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/Axum-0.7-orange?style=flat-square)](https://docs.rs/axum/)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-16-blue?style=flat-square&logo=postgresql&logoColor=white)](https://www.postgresql.org/)
[![WebSocket](https://img.shields.io/badge/WebSocket-Support-green?style=flat-square)](https://tools.ietf.org/html/rfc6455)

---

## 🏗️ Clean Architecture Overview

The EPSX backend implements a well-structured Clean Architecture pattern with five distinct layers, ensuring maintainability, testability, and scalability:

```
┌─────────────────────────────────────────────────────────────┐
│                        Web Layer                            │
│  (Handlers, Middleware, Routes, API Endpoints)             │
├─────────────────────────────────────────────────────────────┤
│                    Infrastructure Layer                     │
│  (Repositories, External Services, Database, Cache)        │
├─────────────────────────────────────────────────────────────┤
│                     Application Layer                       │
│         (Use Cases, DTOs, Ports, Services)                 │
├─────────────────────────────────────────────────────────────┤
│                      Domain Layer                           │
│    (Entities, Value Objects, Domain Services, Events)      │
├─────────────────────────────────────────────────────────────┤
│                       Core Layer                            │
│      (Errors, Events, Telemetry, Plugins, Config)         │
└─────────────────────────────────────────────────────────────┘
```

---

## 🧱 Architecture Layers

### 1. Core Layer (`src/core/`)
**Shared kernel providing cross-cutting concerns:**

- **Error Handling** (`errors.rs`): Enhanced error system with correlation tracking and circuit breakers
- **Error Recovery** (`error_recovery.rs`): Retry strategies, circuit breakers, and fallback mechanisms
- **Events** (`events.rs`): Domain event system with event sourcing capabilities
- **Telemetry** (`telemetry.rs`): Performance monitoring, alerting, and logging infrastructure
- **Database** (`db.rs`): Generic database abstractions and connection management
- **Plugins** (`plugins.rs`): Extensible plugin system for trading algorithms and data providers

### 2. Domain Layer (`src/dom/`)
**Business logic and core domain models:**

#### Entities (`entities/`)
- **User** (`user.rs`): Core user aggregate with role-based permissions
- **IAM** (`iam.rs`): Identity and Access Management entities (roles, policies, groups)
- **Permission Profiles** (`permission_profile.rs`): Reusable permission templates
- **Audit** (`audit.rs`): Audit logging entities for compliance
- **Payment** (`payment.rs`): Payment processing entities
- **Stock** (`stock.rs`): Financial data entities
- **Module** (`module.rs`): Modular system entities

#### Value Objects (`values/`)
- **Identifiers** (`identifiers.rs`): Strongly-typed IDs (UserId, SessionId, etc.)
- **Auth** (`auth.rs`): Authentication-related value objects
- **Permissions** (`permissions.rs`): Permission sets and access controls
- **Payments** (`payments.rs`): Payment-related value objects

#### Domain Services (`services/`)
- **Role Hierarchy** (`role_hierarchy.rs`): Role inheritance and hierarchy management
- **Permission Resolver** (`permission_resolver.rs`): Complex permission resolution with caching
- **Permission Cache Service** (`permission_cache_service.rs`): High-performance permission caching
- **Policy Engine** (`policy_engine.rs`): Business rule evaluation engine
- **Auto Assignment** (`auto_assignment.rs`): Automated feature assignment
- **Feature Expiration** (`feature_expiration.rs`): Time-based feature expiration management

### 3. Application Layer (`src/app/`)
**Use cases and application services:**

#### Use Cases (`use_cases/`)
- **Auth** (`auth.rs`): Authentication workflows (login, registration, session management)
- **User** (`user.rs`): User management operations
- **IAM** (`iam.rs`): Identity and access management workflows
- **Module Management** (`module_management.rs`): Module lifecycle management
- **Payment** (`payment.rs`): Payment processing workflows
- **Stock** (`stock.rs`): Financial data processing workflows

#### Ports (`ports/`)
- **Repository Interfaces** (`repositories.rs`): Data persistence abstractions
- **Service Interfaces** (`services.rs`): External service abstractions
- **Event Interfaces** (`events.rs`): Event handling abstractions

#### DTOs (`dtos/`)
- Data transfer objects for API communication with validation

### 4. Infrastructure Layer (`src/infra/`)
**External adapters and technical implementations:**

#### Database (`db/postgres/`)
- **PostgreSQL Repositories**: Concrete implementations for all entities
- **Migrations**: Database schema versioning
- **Connection Management**: Database connection pooling

#### Services (`services/`)
- **Email** (`email.rs`): SendGrid integration with fallback support
- **Encryption** (`encryption.rs`): Data encryption and security services
- **Market Data** (`market_data.rs`): External market data integration
- **Notification** (`notification.rs`): Multi-channel notification system
- **Payment** (`payment.rs`): Payment gateway integrations

#### Caching (`cache/`)
- **Redis Cache** (`redis_cache.rs`): Distributed caching implementation
- **Memory Cache** (`memory_cache.rs`): In-memory caching for development
- **Cache Abstraction**: Pluggable caching strategies

#### Background Jobs (`jobs/`)
- **Job Scheduler** (`job_scheduler.rs`): Background job processing
- **Expiration Checker** (`expiration_checker.rs`): Automated feature expiration
- **Notification Service** (`notification_service.rs`): Batch notification processing

### 5. Web Layer (`src/web/`)
**HTTP API and presentation layer:**

#### API Modules
- **Auth** (`auth/`): Authentication and session management endpoints
- **IAM** (`iam/`): Identity and access management API
- **User** (`user/`): User profile management
- **Admin** (`admin/`): Administrative endpoints
- **Modules** (`modules/`): Feature module management
- **Real-time** (`realtime/`): WebSocket and SSE endpoints

#### Middleware (`middleware/`)
- **Auth Middleware** (`auth_middleware.rs`): JWT and session-based authentication
- **Permission Middleware** (`permission_middleware.rs`): Role-based access control
- **Module Auth** (`module_auth_middleware.rs`): Module-specific authentication
- **Rate Limiting** (`rate_limit.rs`): Request throttling and abuse prevention
- **Error Handling** (`error_handling.rs`): Comprehensive error response formatting
- **Validation** (`validation/`): Request validation with custom validators

#### Validation (`validation/`)
- **Request DTOs** (`request_dtos.rs`): API request validation
- **Validators** (`validators.rs`): Custom validation rules
- **Middleware** (`middleware.rs`): Validation middleware integration

---

## 🚀 Key Features

### Authentication & Authorization
- **Multi-tier Role System**: Basic, Premium, Moderator, Admin roles
- **IAM Profiles**: Reusable permission templates
- **Firebase Integration**: Secure token verification
- **Session Management**: HTTP-only cookie handling
- **JWT Authentication**: Secure token-based authentication
- **Permission Matrices**: Feature-based access control

### Financial Data Processing
- **Real-time Stock Data**: WebSocket streaming
- **Market Screener**: Advanced filtering capabilities
- **Portfolio Analysis**: Investment performance metrics
- **Trading Signals**: Algorithm-generated trading recommendations
- **Historical Data**: Comprehensive price and volume data

### Payment System
- **Crypto Payments**: Multi-currency support (BTC, ETH, USDT, BNB)
- **Payment Webhooks**: Real-time payment status updates
- **Feature Unlocking**: Automatic permission assignment
- **Payment Tracking**: Comprehensive transaction monitoring

### Real-time Features
- **WebSocket Support**: Live trading data and notifications
- **Server-Sent Events**: Real-time updates for admin dashboard
- **Live Data Streaming**: Market data and user activity streams
- **Background Jobs**: Automated feature expiration and notifications

---

## 🛠️ Setup Instructions

### Prerequisites

- **Rust** (latest stable version)
- **PostgreSQL** 16+
- **Redis** (for caching)
- **Node.js** 18+ (for running the monorepo)

### Environment Configuration

1. **Copy environment template:**
```bash
cp .env.example .env
```

2. **Configure Firebase Admin:**
```env
FIREBASE_PROJECT_ID=your_project_id
FIREBASE_CLIENT_EMAIL=your_service_account_email
FIREBASE_PRIVATE_KEY="-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----\n"
```

3. **Configure Database:**
```env
DATABASE_URL=postgresql://username:password@localhost:5432/epsx_dev
REDIS_URL=redis://localhost:6379
```

4. **Configure Server:**
```env
PORT=8080
HOST=0.0.0.0
FRONTEND_URL=http://localhost:3000
JWT_SECRET=your_jwt_secret_here
```

### Build and Run

```bash
# Development
cargo run

# Production build
cargo build --release

# Run with specific environment
RUST_ENV=production cargo run --release

# Run tests
cargo test

# Run integration tests
cargo test --test integration_tests

# Run E2E tests
./tests/e2e/test_rate_limit.sh
```

---

## 🔌 API Endpoints

### Authentication (`/api/v1/auth`)
```
POST   /login              # User authentication
POST   /logout             # Session termination
GET    /session/validate   # Session validation
POST   /refresh            # Token refresh
```

### User Management (`/api/v1/users`)
```
GET    /profile            # User profile
PUT    /profile            # Update profile
GET    /permissions        # User permissions
POST   /level/assign       # Assign user level
```

### IAM (`/api/v1/iam`)
```
GET    /roles              # List roles
POST   /roles              # Create role
GET    /profiles           # List permission profiles
POST   /profiles           # Create permission profile
POST   /assign             # Assign permissions
```

### Stock Data (`/api/v1/stocks`)
```
GET    /prices             # Real-time prices
GET    /screener           # Stock screening
GET    /rankings           # Stock rankings
GET    /financial-data     # Company financials
```

### Payments (`/api/v1/payments`)
```
POST   /create             # Create payment
GET    /status/:id         # Payment status
POST   /webhook            # Payment webhook
GET    /history            # Payment history
```

### Real-time (`/api/v1/realtime`)
```
GET    /ws                 # WebSocket connection
GET    /sse                # Server-Sent Events
```

---

## 🏛️ Database Schema

### Core Tables
- **users**: User accounts and profiles
- **user_levels**: User permission levels and features
- **roles**: Role definitions and hierarchies
- **permission_profiles**: Reusable permission templates
- **sessions**: User session management
- **audit_logs**: Comprehensive audit trail

### Financial Tables
- **stocks**: Stock metadata and information
- **stock_prices**: Real-time and historical prices
- **user_watchlists**: User stock tracking
- **rankings**: Stock ranking data

### Payment Tables
- **payments**: Payment transactions
- **payment_confirmations**: Blockchain confirmations
- **user_features**: Feature assignments and expiration

---

## 🧪 Testing Strategy

### Unit Tests
```bash
# Run all unit tests
cargo test

# Run specific module tests
cargo test dom::services::permission_resolver
cargo test infra::repositories
```

### Integration Tests
```bash
# Database integration tests
cargo test --test integration_tests

# API endpoint tests
cargo test --test api_tests
```

### End-to-End Tests
```bash
# Rate limiting tests
./tests/e2e/test_rate_limit.sh

# Premium feature tests
./tests/e2e/test_premium_rate_limit.sh

# Permission system tests
cargo test --test permission_e2e_tests
```

### Performance Tests
```bash
# Load testing with wrk
wrk -t12 -c400 -d30s http://localhost:8080/api/v1/health

# Memory profiling
cargo run --release --features profiling
```

---

## 🔧 Development Guidelines

### Code Organization
- Follow Clean Architecture layer boundaries
- Use dependency injection through the AppContainer
- Implement comprehensive error handling with context
- Write tests for all business logic
- Use async/await for all I/O operations

### Error Handling
```rust
// Use rich error types with context
use crate::core::errors::{AppError, AppResult};

async fn example_function() -> AppResult<User> {
    let user = repository.find_user(id)
        .await
        .map_err(|e| AppError::Database {
            message: "Failed to find user".to_string(),
            source: Some(Box::new(e)),
            correlation_id: Some(generate_correlation_id()),
        })?;
    
    Ok(user)
}
```

### Domain Events
```rust
// Publish domain events for loose coupling
use crate::core::events::{DomainEvent, EventPublisher};

async fn update_user_level(user_id: UserId, level: UserLevel) -> AppResult<()> {
    // Update user level
    repository.update_user_level(user_id, level).await?;
    
    // Publish domain event
    let event = UserLevelChanged { user_id, old_level, new_level: level };
    event_publisher.publish(DomainEvent::UserLevelChanged(event)).await?;
    
    Ok(())
}
```

### Plugin Development
```rust
// Implement custom plugins
use crate::core::plugins::{Plugin, PluginContext};

pub struct CustomTradingPlugin;

impl Plugin for CustomTradingPlugin {
    fn name(&self) -> &str { "custom_trading" }
    
    async fn initialize(&self, context: &PluginContext) -> AppResult<()> {
        // Plugin initialization logic
        Ok(())
    }
    
    async fn execute(&self, input: &PluginInput) -> AppResult<PluginOutput> {
        // Plugin execution logic
        Ok(PluginOutput::default())
    }
}
```

---

## 📊 Performance & Monitoring

### Metrics Collection
- Request latency and throughput
- Database query performance
- Cache hit rates
- WebSocket connection counts
- Background job processing times

### Health Checks
```bash
# Health endpoint
curl http://localhost:8080/health

# Detailed system status
curl http://localhost:8080/health/detailed
```

### Logging
- Structured logging with correlation IDs
- Configurable log levels per module
- Error tracking with context
- Performance metrics collection

---

## 🔒 Security Features

### Authentication Security
- Firebase Admin SDK token verification
- Secure session management with HTTP-only cookies
- JWT signature validation
- Rate limiting on authentication endpoints

### Authorization Security
- Role-based access control (RBAC)
- Feature-based permissions
- Permission inheritance and hierarchies
- Audit logging for all access attempts

### Data Security
- Encryption service for sensitive data
- Secure database connections
- Input validation and sanitization
- SQL injection prevention with SQLx

---

## 🚀 Deployment

### Production Configuration
```env
RUST_ENV=production
PORT=8080
HOST=0.0.0.0
DATABASE_URL=postgresql://prod_user:password@db:5432/epsx_prod
REDIS_URL=redis://redis:6379
```

### Docker Support
```bash
# Build Docker image
docker build -t epsx-backend .

# Run with Docker Compose
docker-compose up -d

# Health check
curl http://localhost:8080/health
```

### Performance Tuning
- Database connection pooling
- Redis caching strategies
- Background job processing
- WebSocket connection management

---

## 📚 Additional Resources

- [Axum Documentation](https://docs.rs/axum/)
- [SQLx Documentation](https://docs.rs/sqlx/)
- [PostgreSQL Performance](https://www.postgresql.org/docs/current/performance-tips.html)
- [Redis Best Practices](https://redis.io/docs/management/optimization/)
- [Clean Architecture in Rust](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)

---

## 🤝 Contributing

1. Follow Clean Architecture principles
2. Write comprehensive tests
3. Document all public APIs
4. Use meaningful commit messages
5. Ensure all tests pass before submitting PRs

---

*Built with ❤️ using modern Rust practices and Clean Architecture principles*