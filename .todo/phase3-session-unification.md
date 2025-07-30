# Phase 3: Session Unification

## Overview
Completely centralize session management in the backend, using single `sess_id` for both apps, and remove all frontend session handling logic.

## Backend Session Management

### 1. Unified Session System
**Location**: `apps/backend/src/infra/db/postgres/session_repo.rs`

#### 1.1 Single Session Table Schema
- [ ] Remove admin/regular session separation
- [ ] Use single `sess_id` for both frontend and admin
- [ ] Add session type metadata for differentiation

```sql
-- Migration: Unify session management
ALTER TABLE sessions DROP COLUMN IF EXISTS admin_sess_id;
ALTER TABLE sessions ADD COLUMN session_type VARCHAR(20) DEFAULT 'regular';
ALTER TABLE sessions ADD COLUMN app_origin VARCHAR(20) DEFAULT 'frontend';
ALTER TABLE sessions ADD COLUMN capabilities JSONB DEFAULT '[]';
```

#### 1.2 Enhanced Session Repository
- [ ] Implement unified session creation
- [ ] Add session type tracking
- [ ] Enhanced session validation

```rust
impl PostgresSessionRepo {
    pub async fn create_unified_session(
        &self,
        user_id: UserId,
        session_type: SessionType,
        app_origin: AppOrigin,
        capabilities: Vec<String>,
    ) -> Result<UnifiedSession, RepositoryError> {
        let session_id = Uuid::new_v4().to_string();
        
        sqlx::query!(
            r#"
            INSERT INTO sessions (id, user_id, session_type, app_origin, capabilities, created_at, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            session_id,
            user_id.to_string(),
            session_type.to_string(),
            app_origin.to_string(),
            serde_json::to_value(capabilities)?,
            Utc::now(),
            Utc::now() + Duration::days(7)
        )
        .execute(&self.pool)
        .await?;
        
        Ok(UnifiedSession {
            id: session_id,
            user_id,
            session_type,
            app_origin,
            capabilities,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::days(7),
        })
    }
    
    pub async fn validate_unified_session(
        &self,
        session_id: &str,
        app_origin: AppOrigin,
    ) -> Result<Option<UnifiedSession>, RepositoryError> {
        let session = sqlx::query_as!(
            UnifiedSession,
            r#"
            SELECT id, user_id, session_type, app_origin, capabilities, created_at, expires_at
            FROM sessions 
            WHERE id = $1 AND expires_at > $2
            "#,
            session_id,
            Utc::now()
        )
        .fetch_optional(&self.pool)
        .await?;
        
        // Validate app origin compatibility
        if let Some(ref s) = session {
            if !self.is_session_valid_for_app(s, app_origin) {
                return Ok(None);
            }
        }
        
        Ok(session)
    }
    
    fn is_session_valid_for_app(&self, session: &UnifiedSession, app_origin: AppOrigin) -> bool {
        match (session.session_type, app_origin) {
            (SessionType::Admin, AppOrigin::Admin) => true,
            (SessionType::Regular, AppOrigin::Frontend) => true,
            (SessionType::Admin, AppOrigin::Frontend) => {
                // Admin sessions can access frontend
                session.capabilities.contains(&"frontend_access".to_string())
            },
            (SessionType::Regular, AppOrigin::Admin) => false, // Regular users can't access admin
        }
    }
}
```

### 2. Session Types and Capabilities
**Location**: `apps/backend/src/dom/entities/auth.rs`

#### 2.1 Define Session Types
- [ ] Create session type enums
- [ ] Define app origin types
- [ ] Implement capability system

```rust
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "session_type")]
pub enum SessionType {
    Regular,
    Admin,
    Service, // Future: for service-to-service auth
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "app_origin")]
pub enum AppOrigin {
    Frontend,
    Admin,
    Mobile, // Future: for mobile apps
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSession {
    pub id: String,
    pub user_id: UserId,
    pub session_type: SessionType,
    pub app_origin: AppOrigin,
    pub capabilities: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_activity: Option<DateTime<Utc>>,
    pub security_info: SecurityInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityInfo {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_fingerprint: Option<String>,
    pub login_method: LoginMethod,
}
```

### 3. Enhanced Auth Middleware
**Location**: `apps/backend/src/web/middleware/auth_middleware.rs`

#### 3.1 Unified Cookie Handling
- [ ] Single cookie configuration
- [ ] Secure cookie attributes
- [ ] App-aware session validation

```rust
pub struct UnifiedAuthMiddleware {
    session_repo: Arc<dyn SessionRepository>,
    cookie_config: CookieConfig,
}

impl UnifiedAuthMiddleware {
    pub async fn extract_and_validate_session(
        &self,
        headers: &HeaderMap,
        app_origin: AppOrigin,
    ) -> Result<Option<AuthContext>, AuthError> {
        // Extract session cookie
        let session_id = self.extract_session_cookie(headers)?;
        
        if let Some(session_id) = session_id {
            // Validate session with app origin check
            if let Some(session) = self.session_repo
                .validate_unified_session(&session_id, app_origin)
                .await? 
            {
                // Update last activity
                self.session_repo
                    .update_last_activity(&session_id)
                    .await?;
                
                return Ok(Some(AuthContext {
                    user_id: session.user_id,
                    session_id: session.id,
                    session_type: session.session_type,
                    capabilities: session.capabilities,
                    permissions: self.load_user_permissions(session.user_id).await?,
                }));
            }
        }
        
        Ok(None)
    }
    
    fn extract_session_cookie(&self, headers: &HeaderMap) -> Result<Option<String>, AuthError> {
        if let Some(cookie_header) = headers.get(COOKIE) {
            let cookie_str = cookie_header.to_str().map_err(|_| AuthError::InvalidCookie)?;
            
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if let Some(value) = cookie.strip_prefix("sess_id=") {
                    return Ok(Some(value.to_string()));
                }
            }
        }
        Ok(None)
    }
    
    pub fn create_session_cookie(&self, session_id: &str) -> String {
        format!(
            "sess_id={}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age={}",
            session_id,
            self.cookie_config.max_age
        )
    }
}
```

## Backend API Updates

### 1. Session Management Endpoints
**Location**: `apps/backend/src/web/auth/handlers.rs`

#### 1.1 Unified Login Endpoint
- [ ] Create sessions with app origin tracking
- [ ] Set unified session cookies
- [ ] Handle admin vs regular user distinction

```rust
pub async fn unified_login_handler(
    Extension(auth_service): Extension<Arc<AuthService>>,
    Extension(session_repo): Extension<Arc<dyn SessionRepository>>,
    headers: HeaderMap,
    Json(login_request): Json<LoginRequest>,
) -> Result<(StatusCode, HeaderMap, Json<LoginResponse>), AuthError> {
    // Authenticate user
    let user = auth_service.authenticate(
        &login_request.email,
        &login_request.password
    ).await?;
    
    // Determine session type based on user roles
    let session_type = if user.has_admin_role() {
        SessionType::Admin
    } else {
        SessionType::Regular
    };
    
    // Create unified session
    let session = session_repo.create_unified_session(
        user.id,
        session_type,
        login_request.app_origin,
        user.get_capabilities(),
    ).await?;
    
    // Set session cookie
    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        SET_COOKIE,
        format!(
            "sess_id={}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=604800",
            session.id
        ).parse().unwrap()
    );
    
    Ok((
        StatusCode::OK,
        response_headers,
        Json(LoginResponse {
            user: user.into(),
            session_type: session.session_type,
            capabilities: session.capabilities,
        })
    ))
}
```

#### 1.2 Session Context Endpoint
- [ ] Return session info with app context
- [ ] Include capabilities and permissions
- [ ] Optimize for frequent calls

```rust
pub async fn session_context_handler(
    Extension(auth_context): Extension<AuthContext>,
    Extension(permission_checker): Extension<Arc<PermissionChecker>>,
) -> Result<Json<SessionContextResponse>, AuthError> {
    let permissions = permission_checker
        .get_user_permissions(auth_context.user_id)
        .await?;
    
    Ok(Json(SessionContextResponse {
        authenticated: true,
        user_id: auth_context.user_id,
        session_type: auth_context.session_type,
        capabilities: auth_context.capabilities,
        permissions,
        expires_at: auth_context.expires_at,
    }))
}
```

### 2. Logout Enhancement
**Location**: `apps/backend/src/web/auth/handlers.rs`

#### 2.1 Unified Logout
- [ ] Invalidate session in database
- [ ] Clear session cookie
- [ ] Handle both frontend and admin logout

```rust
pub async fn unified_logout_handler(
    Extension(auth_context): Extension<AuthContext>,
    Extension(session_repo): Extension<Arc<dyn SessionRepository>>,
) -> Result<(StatusCode, HeaderMap), AuthError> {
    // Invalidate session
    session_repo.invalidate_session(&auth_context.session_id).await?;
    
    // Clear session cookie
    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        "sess_id=; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=0".parse().unwrap()
    );
    
    Ok((StatusCode::OK, headers))
}
```

## Frontend Cookie Manager Removal

### 1. Remove Admin Cookie Manager
**Location**: `packages/api-client/src/cookie-manager.ts`

#### 1.1 Remove Cookie Conversion Logic
- [ ] Delete `admin_sess_id` conversion
- [ ] Remove cookie setting utilities  
- [ ] Keep only cookie reading functionality

```typescript
// REMOVE: Complex cookie manager
export class CookieManager {
  // Delete entire class implementation
}

// REPLACE WITH: Simple cookie reader
export class CookieReader {
  static get(name: string): string | undefined {
    if (typeof document !== 'undefined') {
      const cookies = document.cookie.split(';');
      for (const cookie of cookies) {
        const [key, value] = cookie.trim().split('=');
        if (key === name) {
          return decodeURIComponent(value);
        }
      }
    }
    return undefined;
  }
  
  static has(name: string): boolean {
    return this.get(name) !== undefined;
  }
}
```

### 2. Update Frontend Cookie Usage
**Location**: `apps/frontend/lib/cookies.ts`

#### 2.1 Simplify Cookie Utilities
- [ ] Remove cookie setting functions
- [ ] Keep only reading functionality
- [ ] Remove server-side cookie manipulation

```typescript
// REMOVE: Complex cookie utilities
export const ServerCookies = {
  set: (name: string, value: string, options?: CookieOptions) => {
    // Remove implementation
  },
  // ... other methods to remove
};

// REPLACE WITH: Read-only utilities
export const SessionCookies = {
  getSessionId(): string | undefined {
    return cookies().get('sess_id')?.value;
  },
  
  hasSession(): boolean {
    return cookies().has('sess_id');
  }
};
```

### 3. Update Admin Frontend Cookie Usage
**Location**: `apps/admin-frontend/` (various files)

#### 3.1 Remove Admin-specific Cookie Logic
- [ ] Replace `admin_sess_id` with `sess_id`
- [ ] Remove cookie setting from frontend
- [ ] Use unified cookie reading

```typescript
// BEFORE: Admin-specific cookie handling (REMOVE)
const adminSessionId = getCookie('admin_sess_id');

// AFTER: Unified cookie handling (IMPLEMENT)
const sessionId = getCookie('sess_id');
```

## Session Validation Updates

### 1. Frontend Session Validation
**Location**: `apps/frontend/middleware.ts`

#### 1.1 Backend-only Session Validation
- [ ] Remove client-side session parsing
- [ ] Use backend API for validation
- [ ] Handle app origin in requests

```typescript
async function validateSession(request: NextRequest): Promise<SessionValidationResult> {
  const sessionId = request.cookies.get('sess_id')?.value;
  
  if (!sessionId) {
    return { authenticated: false };
  }
  
  try {
    const response = await fetch(`${process.env.BACKEND_URL}/api/v1/auth/validate-session`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Cookie': `sess_id=${sessionId}`
      },
      body: JSON.stringify({ app_origin: 'Frontend' })
    });
    
    if (response.ok) {
      const sessionInfo = await response.json();
      return {
        authenticated: true,
        user: sessionInfo.user,
        permissions: sessionInfo.permissions,
        capabilities: sessionInfo.capabilities
      };
    }
  } catch (error) {
    console.error('Session validation failed:', error);
  }
  
  return { authenticated: false };
}
```

### 2. Admin Session Validation
**Location**: `apps/admin-frontend/middleware.ts`

#### 2.1 Use Same Session Validation
- [ ] Replace admin-specific logic with unified approach
- [ ] Specify admin app origin
- [ ] Use same session cookie

```typescript
async function validateAdminSession(request: NextRequest): Promise<SessionValidationResult> {
  const sessionId = request.cookies.get('sess_id')?.value;
  
  if (!sessionId) {
    return { authenticated: false };
  }
  
  const response = await fetch(`${process.env.BACKEND_URL}/api/v1/auth/validate-session`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Cookie': `sess_id=${sessionId}`
    },
    body: JSON.stringify({ app_origin: 'Admin' })
  });
  
  if (response.ok) {
    const sessionInfo = await response.json();
    
    // Ensure user has admin capabilities
    if (sessionInfo.session_type === 'Admin' || 
        sessionInfo.capabilities.includes('frontend_access')) {
      return {
        authenticated: true,
        user: sessionInfo.user,
        permissions: sessionInfo.permissions,
        capabilities: sessionInfo.capabilities
      };
    }
  }
  
  return { authenticated: false };
}
```

## Security Enhancements

### 1. Enhanced Security Headers
**Location**: `apps/backend/src/web/middleware/auth_middleware.rs`

#### 1.1 Comprehensive Security Headers
- [ ] Add security headers to all auth responses
- [ ] Implement CSRF protection
- [ ] Add session security monitoring

```rust
impl UnifiedAuthMiddleware {
    fn add_security_headers(&self, headers: &mut HeaderMap) {
        headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
        headers.insert("X-Frame-Options", "DENY".parse().unwrap());
        headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
        headers.insert("Referrer-Policy", "strict-origin-when-cross-origin".parse().unwrap());
        headers.insert(
            "Content-Security-Policy", 
            "default-src 'self'; script-src 'self' 'unsafe-inline'".parse().unwrap()
        );
    }
}
```

### 2. Session Security Monitoring
**Location**: `apps/backend/src/dom/services/session_security.rs`

#### 2.1 Session Anomaly Detection
- [ ] Track session usage patterns
- [ ] Detect suspicious activity
- [ ] Implement session invalidation triggers

```rust
pub struct SessionSecurityService {
    session_repo: Arc<dyn SessionRepository>,
    audit_service: Arc<AuditService>,
}

impl SessionSecurityService {
    pub async fn validate_session_security(
        &self,
        session_id: &str,
        request_info: RequestInfo,
    ) -> Result<SecurityValidationResult, SecurityError> {
        let session = self.session_repo.get_session(session_id).await?;
        
        // Check for IP address changes
        if let Some(ref stored_ip) = session.security_info.ip_address {
            if stored_ip != &request_info.ip_address {
                self.audit_service.log_security_event(
                    SecurityEvent::IpAddressChange {
                        session_id: session_id.to_string(),
                        old_ip: stored_ip.clone(),
                        new_ip: request_info.ip_address.clone(),
                    }
                ).await?;
                
                // Could trigger additional verification or session invalidation
            }
        }
        
        // Check for user agent changes
        if let Some(ref stored_ua) = session.security_info.user_agent {
            if stored_ua != &request_info.user_agent {
                self.audit_service.log_security_event(
                    SecurityEvent::UserAgentChange {
                        session_id: session_id.to_string(),
                        old_ua: stored_ua.clone(),
                        new_ua: request_info.user_agent.clone(),
                    }
                ).await?;
            }
        }
        
        Ok(SecurityValidationResult::Valid)
    }
}
```

## Migration Strategy

### 1.1 Database Migration
- [ ] Create migration scripts for session table changes
- [ ] Migrate existing sessions to new format
- [ ] Ensure zero-downtime migration

```sql
-- Migration script: 003_unify_sessions.sql
BEGIN;

-- Add new columns
ALTER TABLE sessions ADD COLUMN IF NOT EXISTS session_type VARCHAR(20) DEFAULT 'regular';
ALTER TABLE sessions ADD COLUMN IF NOT EXISTS app_origin VARCHAR(20) DEFAULT 'frontend';
ALTER TABLE sessions ADD COLUMN IF NOT EXISTS capabilities JSONB DEFAULT '[]';
ALTER TABLE sessions ADD COLUMN IF NOT EXISTS security_info JSONB DEFAULT '{}';

-- Migrate existing admin sessions
UPDATE sessions 
SET session_type = 'admin', app_origin = 'admin'
WHERE user_id IN (
    SELECT user_id FROM user_roles 
    WHERE role_name IN ('admin', 'system_administrator')
);

-- Remove old admin session columns if they exist
-- ALTER TABLE sessions DROP COLUMN IF EXISTS admin_sess_id;

COMMIT;
```

### 1.2 Gradual Rollout
- [ ] Feature flag for unified session system
- [ ] A/B test with subset of users
- [ ] Monitor for issues and rollback capability

## Testing Requirements

### 1.1 Session Management Tests
- [ ] Test unified session creation
- [ ] Verify app origin validation
- [ ] Test session security features
- [ ] Capability-based access control

### 1.2 Cross-app Session Tests
- [ ] Admin users accessing frontend
- [ ] Session sharing between apps
- [ ] Security boundary enforcement
- [ ] Cookie handling across domains

### 1.3 Security Tests
- [ ] Session hijacking prevention
- [ ] CSRF protection validation
- [ ] Security header verification
- [ ] Anomaly detection testing

## Dependencies
- **Requires**: Phase 1 (Backend APIs) and Phase 2 (Frontend Simplification) completed
- **Blocks**: Phase 4 (Permission Consolidation)

## Completion Criteria
- [ ] Single `sess_id` cookie for both apps
- [ ] All session logic centralized in backend
- [ ] No frontend cookie setting or session manipulation
- [ ] Enhanced security monitoring and headers
- [ ] Database migration completed successfully
- [ ] All existing functionality preserved
- [ ] Comprehensive test coverage
- [ ] Performance benchmarks met
- [ ] Security audit passed