# Phase 1: Backend API Foundation

## Overview
Create comprehensive backend APIs to handle all auth/IAM/AC logic that's currently distributed across frontends.

## Backend Auth API Endpoints

### 1. Session Management APIs
**Location**: `apps/backend/src/web/auth/handlers.rs`

#### 1.1 Session Validation Endpoint
- [ ] `POST /api/v1/auth/validate-session`
  - Validates current user session
  - Returns user info, permissions, roles
  - Handles both regular and admin sessions
  - Includes rate limiting and security headers

```rust
// Handler function to add
pub async fn validate_session_handler(
    Extension(session_repo): Extension<Arc<dyn SessionRepository>>,
    headers: HeaderMap,
) -> Result<Json<SessionValidationResponse>, AuthError>
```

#### 1.2 Session Refresh Endpoint  
- [ ] `POST /api/v1/auth/refresh-session`
  - Extends session expiry
  - Updates last activity timestamp
  - Returns new session info

#### 1.3 Session Info Endpoint
- [ ] `GET /api/v1/auth/session-info`
  - Lightweight session status check
  - Returns minimal user data for UI
  - Optimized for frequent calls

### 2. Route Protection APIs
**Location**: `apps/backend/src/web/auth/handlers.rs`

#### 2.1 Route Access Validation
- [ ] `POST /api/v1/auth/validate-access`
  - Validates user access to specific routes
  - Handles permission profiles (Bronze/Silver/Gold)
  - Includes admin role hierarchy checking
  - Returns detailed permission info

```rust
#[derive(Deserialize)]
pub struct RouteAccessRequest {
    pub route: String,
    pub method: String,
    pub app_type: AppType, // Frontend or Admin
}

#[derive(Serialize)]
pub struct RouteAccessResponse {
    pub allowed: bool,
    pub permissions: Vec<String>,
    pub user_level: String,
    pub rate_limit_info: RateLimitInfo,
}
```

#### 2.2 Bulk Route Validation
- [ ] `POST /api/v1/auth/validate-routes`
  - Validates multiple routes at once
  - Optimized for middleware pre-loading
  - Returns permission map for caching

### 3. Permission Checking APIs
**Location**: `apps/backend/src/web/auth/handlers.rs`

#### 3.1 User Permissions Endpoint
- [ ] `GET /api/v1/auth/permissions`
  - Returns current user's permissions
  - Includes role hierarchy
  - Permission profile details

#### 3.2 Permission Validation
- [ ] `POST /api/v1/auth/check-permission`
  - Validates specific permission
  - Supports wildcard matching
  - Context-aware checking

```rust
#[derive(Deserialize)]
pub struct PermissionCheckRequest {
    pub permission: String,
    pub resource: Option<String>,
    pub context: Option<serde_json::Value>,
}
```

### 4. User State APIs
**Location**: `apps/backend/src/web/auth/handlers.rs`

#### 4.1 Current User Endpoint
- [ ] `GET /api/v1/auth/me`
  - Returns current user profile
  - Includes permissions and roles
  - Frontend-optimized response

#### 4.2 Auth Status Endpoint
- [ ] `GET /api/v1/auth/status`
  - Quick auth status check
  - Returns boolean + basic info
  - Minimal response for performance

## Backend Service Enhancements

### 1. Enhanced Permission Checker
**Location**: `apps/backend/src/dom/services/permission_checker.rs`

#### 1.1 Route-based Permission Validation
- [ ] Add route pattern matching
- [ ] Implement permission profile integration
- [ ] Add admin role hierarchy support

```rust
impl PermissionChecker {
    pub async fn validate_route_access(
        &self,
        user_id: &UserId,
        route: &str,
        method: &str,
        app_type: AppType,
    ) -> Result<RouteAccessResult, PermissionError> {
        // Implementation
    }
}
```

#### 1.2 Bulk Permission Checking
- [ ] Implement batch permission validation
- [ ] Add caching for frequent checks
- [ ] Optimize database queries

### 2. Session Service Enhancement
**Location**: `apps/backend/src/infra/db/postgres/session_repo.rs`

#### 2.1 Unified Session Management
- [ ] Remove admin/regular session separation
- [ ] Use single `sess_id` for both apps
- [ ] Enhanced session metadata tracking

```rust
impl SessionRepository for PostgresSessionRepo {
    async fn create_unified_session(
        &self,
        user_id: UserId,
        session_type: SessionType,
        metadata: SessionMetadata,
    ) -> Result<Session, RepositoryError> {
        // Implementation
    }
}
```

#### 2.2 Session Validation Service
- [ ] Add comprehensive session validation
- [ ] Implement session security checks
- [ ] Add device fingerprinting support

### 3. Auth Middleware Enhancement
**Location**: `apps/backend/src/web/middleware/auth_middleware.rs`

#### 3.1 Unified Cookie Handling
- [ ] Single cookie configuration
- [ ] HTTP-only session cookies
- [ ] Proper security attributes (Secure, SameSite)

```rust
pub struct UnifiedAuthMiddleware {
    session_repo: Arc<dyn SessionRepository>,
    permission_checker: Arc<PermissionChecker>,
}

impl UnifiedAuthMiddleware {
    pub async fn validate_request(
        &self,
        req: &Request,
    ) -> Result<AuthContext, AuthError> {
        // Implementation
    }
}
```

#### 3.2 CSRF Protection
- [ ] Implement CSRF token generation
- [ ] Add token validation middleware
- [ ] Integrate with session management

## Database Schema Updates
**Location**: `apps/backend/migrations/`

### 1.1 Session Table Enhancement
- [ ] Add session metadata columns
- [ ] Remove admin/regular separation
- [ ] Add security tracking fields

```sql
-- Migration: Unify session table
ALTER TABLE sessions ADD COLUMN session_type VARCHAR(20) DEFAULT 'regular';
ALTER TABLE sessions ADD COLUMN metadata JSONB DEFAULT '{}';
ALTER TABLE sessions ADD COLUMN security_info JSONB DEFAULT '{}';
```

### 1.2 Audit Log Enhancement
- [ ] Track auth API calls
- [ ] Log permission checks
- [ ] Session lifecycle events

## Configuration Updates

### 1.1 Cookie Configuration
**Location**: `apps/backend/src/config.rs`

- [ ] Unified cookie settings
- [ ] Security-first defaults
- [ ] Environment-based configuration

```rust
#[derive(Clone)]
pub struct CookieConfig {
    pub name: String,
    pub domain: Option<String>,
    pub path: String,
    pub max_age: i64,
    pub http_only: bool,
    pub secure: bool,
    pub same_site: SameSite,
}

impl Default for CookieConfig {
    fn default() -> Self {
        Self {
            name: "sess_id".to_string(),
            http_only: true,
            secure: true,
            same_site: SameSite::Lax,
            // ... other defaults
        }
    }
}
```

## Testing Requirements

### 1.1 Unit Tests
- [ ] Session validation logic
- [ ] Permission checking functions
- [ ] Route access validation
- [ ] Cookie handling

### 1.2 Integration Tests
- [ ] API endpoint functionality
- [ ] Database operations
- [ ] Middleware integration
- [ ] Security features

### 1.3 Security Tests
- [ ] CSRF protection
- [ ] Session hijacking prevention
- [ ] Permission bypass attempts
- [ ] Rate limiting validation

## Dependencies
- Phase 1 has no dependencies (can start immediately)
- Must be completed before Phase 2 frontend simplification
- All backend tests must pass before proceeding

## Completion Criteria
- [ ] All API endpoints implemented and tested
- [ ] Session management unified and secure
- [ ] Permission checking centralized
- [ ] Database migrations completed
- [ ] Security features active
- [ ] Comprehensive test coverage (>90%)
- [ ] Performance benchmarks met
- [ ] Documentation updated