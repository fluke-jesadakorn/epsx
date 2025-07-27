# EPSX Technical Architecture

## System Overview

### High-Level Architecture ✅ IMPLEMENTED
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Frontend      │    │  Admin Frontend │    │    Backend      │
│   (Next.js SSR)│◄───┤   (Next.js SSR) │◄───┤ (Rust Clean)    │
│                 │    │                 │    │                 │
│  - SSR Auth ✅  │    │  - IAM Mgmt ✅  │    │  - Domain ✅    │
│  - Theme Sys ✅ │    │  - Permissions ✅│    │  - Use Cases ✅ │
│  - Mobile PWA ✅│    │  - Analytics ✅ │    │  - Infra ✅     │
│  - Access Ctrl✅│    │  - Rule Blder✅ │    │  - Jobs ✅      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
        ┌─────────────────────────┼─────────────────────────┐
        │              Data & Services Layer ✅             │
        │  ┌─────────────┐  ┌──────────────┐  ┌─────────┐  │
        │  │  Firebase   │  │  PostgreSQL  │  │ Real-   │  │
        │  │  Auth Only  │  │  Primary DB  │  │ time    │  │
        │  │     ✅      │  │      ✅      │  │   ✅    │  │
        │  └─────────────┘  └──────────────┘  └─────────┘  │
        └─────────────────────────────────────────────────┘
```

### Monorepo Structure ✅ IMPLEMENTED
```
epsx/
├── apps/
│   ├── frontend/              # User-facing analytics platform (Next.js 15 SSR) ✅
│   │   ├── app/              # App Router with Server Components ✅
│   │   ├── components/       # Server/Client/Shared component architecture ✅
│   │   ├── lib/             # Server-side utilities and auth ✅
│   │   └── hooks/           # Client-side React hooks ✅
│   ├── admin-frontend/        # Administrative interface (Next.js SSR) ✅
│   │   ├── components/       # Admin-specific UI components ✅
│   │   ├── hooks/           # Admin analytics and IAM hooks ✅
│   │   ├── components/     # Analytics: permission_analytics.tsx, assignment_analytics.tsx ✅
│   │   └── components/     # Rule Builders: visual_rule_builder.tsx, condition_builder.tsx ✅
│   │   └── services/        # Admin service integrations ✅
│   └── backend/              # Rust API server with Clean Architecture ✅
│       ├── src/dom/         # Domain entities, values, services ✅
│       ├── src/app/         # Use cases, DTOs, ports ✅
│       ├── src/infra/       # Repositories, services, auth ✅
│       ├── src/web/         # HTTP handlers, routes, middleware ✅
│       ├── src/core/        # Shared core utilities ✅
│       ├── migrations/      # Database migration scripts ✅
│       └── tests/           # Comprehensive test suites ✅
├── packages/
│   ├── ui/                   # Shared UI components & design system ✅
│   │   ├── src/components/  # Reusable UI components ✅
│   │   ├── src/providers/   # Theme and context providers ✅
│   │   └── src/tokens/      # Design tokens and theming ✅
│   ├── types/               # Shared TypeScript type definitions ✅
│   ├── config/              # Shared configuration utilities ✅
│   └── utils/               # Cross-platform utility functions ✅
├── .requirement_docs/        # Business & technical documentation ✅
├── .github/workflows/        # CI/CD pipeline configuration ✅
└── docs/                    # Additional project documentation ✅
```

## Backend Architecture (Rust)

### Clean Architecture + Hexagonal Pattern
```rust
src/
├── main.rs                 # Application entry point
├── lib.rs                  # Library root
├── web/                    # Web layer (adapters)
│   ├── auth/              # Auth & IAM endpoints
│   ├── analytics/         # Market data analytics
│   ├── permission_profiles/ # Dynamic permission profile management
│   ├── realtime/          # Real-time notifications
│   ├── user/              # User management
│   └── middleware/        # Auth, CORS, rate limiting
├── app/                    # Application layer
│   ├── use_cases/         # Business use cases
│   ├── dtos/              # Data transfer objects
│   └── ports/             # Repository & service interfaces
├── dom/                    # Domain layer (core business logic)
│   ├── entities/          # Business entities
│   ├── values/            # Value objects
│   ├── services/          # Domain services
│   └── events/            # Domain events
├── core/                   # Core utilities & shared logic
└── infra/                  # Infrastructure layer
    ├── db/               # Database implementations
    ├── repos/            # Repository implementations
    ├── services/         # External service integrations
    └── auth/             # Authentication implementations
```

## Frontend Architecture (Next.js SSR)

### SSR-First App Structure
```
apps/frontend/
├── app/
│   ├── layout.tsx              # Root layout with SSR theme system
│   ├── page.tsx               # SSR home page with analytics
│   ├── loading.tsx            # Loading UI for SSR transitions
│   ├── error.tsx              # Error boundaries for SSR
│   ├── auth/                  # SSR authentication pages
│   │   ├── login/page.tsx     # SSR login page
│   │   ├── register/page.tsx  # SSR registration page
│   │   └── forgot-password/   # SSR password recovery
│   ├── analytics/             # SSR analytics interface
│   │   ├── market-data/       # SSR market data view
│   │   ├── eps/              # SSR EPS analysis
│   │   └── patterns/         # SSR pattern recognition
│   ├── api/                  # API routes & server actions
│   │   ├── auth/             # Authentication endpoints
│   │   └── analytics/        # Analytics data endpoints
│   └── actions/              # Server Actions for mutations
└── components/
    ├── server/               # Server-only components
    ├── client/               # Client-only components ('use client')
    └── shared/               # Isomorphic components
```

### SSR State Management
```typescript
// Server-side state with client hydration
interface ServerState {
  initialAuth: AuthState;        // SSR auth from cookies/headers
  initialTheme: ThemeState;      // SSR theme from preferences
  initialAnalytics: AnalyticsState; // Pre-fetched analytics data
  initialUser: UserState;        // Server-rendered user data
}

interface ClientState extends ServerState {
  notifications: NotificationState; // Client-only real-time alerts
  ui: UIState;                   // Client-only UI states
  realtime: RealtimeState;       // WebSocket connections
}

// Server Actions for data mutations
async function loginAction(formData: FormData) {
  'use server'
  // Server-side authentication with cookie setting
}
```

## Database Architecture

### PostgreSQL Schema (Primary Storage) ✅ FULLY IMPLEMENTED
```sql
-- Core user management (linked to Firebase UID) ✅
-- UPDATED: Registration now stores Firebase UID, not user email
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    firebase_uid VARCHAR(128) UNIQUE NOT NULL,
    email VARCHAR(255) NOT NULL, -- Placeholder email (format: firebase_uid@firebase.user)
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);


-- Package-based IAM system (no roles, permission profiles only) ✅

-- Dynamic permission profile system ✅ ENHANCED
CREATE TABLE permission_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(50) NOT NULL, -- 'analytics', 'premium', 'admin'
    version VARCHAR(20) NOT NULL DEFAULT '1.0',
    status VARCHAR(50) DEFAULT 'active',
    profile_data JSONB NOT NULL, -- Features, modules, limits
    pricing_tier JSONB,
    auto_assignment_rules JSONB,
    api_endpoints JSONB DEFAULT '{}', -- NEW: API access control
    frontend_routes JSONB DEFAULT '{}', -- NEW: Frontend route access
    compliance_level VARCHAR(50) DEFAULT 'educational',
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE user_features (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) NOT NULL,
    profile_id UUID REFERENCES permission_profiles(id) NOT NULL,
    feature_id VARCHAR(255) NOT NULL,
    status VARCHAR(50) DEFAULT 'active',
    configuration JSONB,
    activated_at TIMESTAMP DEFAULT NOW(),
    expires_at TIMESTAMP,
    activated_via VARCHAR(50) DEFAULT 'auto' -- 'auto', 'admin', 'payment'
);

-- Admin assignment system ✅ IMPLEMENTED
CREATE TABLE admin_permission_profile_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) NOT NULL,
    profile_id UUID REFERENCES permission_profiles(id) NOT NULL,
    assigned_by UUID REFERENCES users(id) NOT NULL,
    assignment_type VARCHAR(50) NOT NULL, -- 'promotional', 'trial', 'permanent'
    assignment_reason TEXT NOT NULL,
    expires_at TIMESTAMP,
    variables JSONB,
    override_pricing BOOLEAN DEFAULT FALSE,
    notification_settings JSONB,
    status VARCHAR(50) DEFAULT 'active',
    created_at TIMESTAMP DEFAULT NOW()
);

-- Analytics and market data ✅ SCHEMA IMPLEMENTED
CREATE TABLE market_data (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    symbol VARCHAR(20) NOT NULL,
    data_type VARCHAR(50) NOT NULL,
    data_value JSONB NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    source VARCHAR(100),
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE analytics_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    analysis_type VARCHAR(100) NOT NULL,
    result JSONB NOT NULL,
    confidence_score DECIMAL(3,2),
    created_at TIMESTAMP DEFAULT NOW()
);

-- Comprehensive audit system ✅
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL,
    resource VARCHAR(100) NOT NULL,
    resource_id UUID,
    details JSONB,
    ip_address INET,
    user_agent TEXT,
    timestamp TIMESTAMP DEFAULT NOW()
);

-- Feature payment integration (multiple providers) ✅
CREATE TABLE feature_payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) NOT NULL,
    profile_id UUID REFERENCES permission_profiles(id) NOT NULL,
    external_payment_id VARCHAR(255) NOT NULL, -- External payment system reference
    payment_provider VARCHAR(50) NOT NULL, -- 'stripe', 'crypto', 'manual'
    amount DECIMAL(10,2),
    currency VARCHAR(10) DEFAULT 'USD',
    features_unlocked JSONB NOT NULL,
    activation_status VARCHAR(50) DEFAULT 'pending',
    activated_at TIMESTAMP,
    expires_at TIMESTAMP
);

-- Indexes for performance ✅ ENHANCED
CREATE INDEX idx_users_firebase_uid ON users(firebase_uid);
CREATE INDEX idx_user_features_user_id ON user_features(user_id);
CREATE INDEX idx_user_features_profile_id ON user_features(profile_id);
CREATE INDEX idx_permission_profiles_category ON permission_profiles(category);
CREATE INDEX idx_permission_profiles_status ON permission_profiles(status);
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_timestamp ON audit_logs(timestamp);
CREATE INDEX idx_market_data_symbol_timestamp ON market_data(symbol, timestamp);
-- NEW: Composite index for payment-based auto-assignment
CREATE INDEX idx_feature_payments_status_expires 
ON feature_payments(activation_status, expires_at) 
WHERE activation_status = 'active';
```

### Firebase Authentication (Auth Only)
```
Firebase Authentication:
- User accounts (email/password only)
- JWT token generation and validation
- User management (create, delete, reset password)
- Custom claims (minimal - just user_id reference to PostgreSQL)

No Firestore collections - all data stored in PostgreSQL
```

## API Design

### Professional API Structure with Access Control (V1 Only)
```
# Public Routes (No Authentication Required)
GET  /health                         # System health check
GET  /auth/me-public                 # Public auth status check

# V1 API Routes (/api/v1/)
/api/v1/
├── authentication/
│   ├── POST /auth/login             # Public - User/Admin login
│   ├── POST /auth/register          # Public - User registration
│   ├── POST /auth/register-auto     # Public - Auto registration
│   ├── POST /auth/password-reset    # Public - Password reset
│   ├── POST /auth/logout            # Authenticated - Logout
│   ├── POST /auth/refresh           # Authenticated - Token refresh
│   ├── GET  /auth/profile           # Authenticated - Get user profile
│   └── POST /auth/session/clear     # Authenticated - Clear session
├── users/
│   ├── GET  /users/profile          # Authenticated - Get current user profile
│   ├── PUT  /users/profile          # Authenticated - Update user profile
│   ├── GET  /users                  # Authenticated - List users (admin)
│   ├── GET  /users/:id              # Authenticated - Get user by ID (admin)
│   └── DELETE /users/:id            # Authenticated - Delete user (admin)
├── market-data/
│   └── GET  /market-data/symbols    # Authenticated - Get available symbols
├── payments/
│   ├── GET  /payments/crypto/deposit-address  # Authenticated - Get crypto deposit
│   ├── POST /payments/musepay/create          # Authenticated - Create payment
│   └── POST /webhooks/payments/musepay        # System - Payment webhook
├── premium/
│   └── GET  /premium/rankings       # Permission required - Premium rankings
├── system/
│   └── POST /system/cache           # Authenticated - Clear cache
├── audit/
│   ├── POST /audit/logs             # Public - Create audit entry (frontend logging)
│   ├── GET  /audit/logs             # Admin - Search audit logs
│   ├── GET  /audit/logs/:log_id     # Admin - Get specific audit log
│   ├── GET  /audit/statistics       # Admin - Get audit statistics
│   └── GET  /audit/export           # Admin - Export audit logs
├── iam/
│   ├── POST /iam/roles              # Admin - Create role
│   ├── GET  /iam/roles              # Admin - List roles
│   ├── GET  /iam/roles/:role_id     # Admin - Get role
│   ├── PUT  /iam/roles/:role_id     # Admin - Update role
│   ├── DELETE /iam/roles/:role_id   # Admin - Delete role
│   ├── POST /iam/policies           # Admin - Create policy
│   ├── GET  /iam/policies           # Admin - List policies
│   ├── GET  /iam/policies/:policy_id # Admin - Get policy
│   ├── DELETE /iam/policies/:policy_id # Admin - Delete policy
│   ├── POST /iam/evaluate           # Admin - Evaluate permission
│   ├── POST /iam/users/:user_id/overrides # Admin - Set user overrides
│   ├── GET  /iam/users/:user_id/overrides # Admin - Get user overrides
│   ├── POST /iam/users/:user_id/roles/:role_id # Admin - Assign role
│   ├── DELETE /iam/users/:user_id/roles/:role_id # Admin - Remove role
│   └── GET  /iam/users/:user_id/roles # Admin - Get user roles
├── permission-profiles/
│   ├── POST /permission-profiles/permission-profiles # Admin - Create profile
│   ├── GET  /permission-profiles/permission-profiles # Admin - Search profiles
│   ├── GET  /permission-profiles/permission-profiles/:profile_id # Admin - Get profile
│   ├── PUT  /permission-profiles/permission-profiles/:profile_id # Admin - Update profile
│   ├── DELETE /permission-profiles/permission-profiles/:profile_id # Admin - Delete profile
│   ├── POST /permission-profiles/permission-profiles/:profile_id/apply # Admin - Apply profile
│   ├── GET  /permission-profiles/permission-profiles/:profile_id/history # Admin - Get history
│   └── POST /permission-profiles/initialize-defaults # Super Admin - Initialize defaults
├── realtime/
│   ├── GET  /realtime/ws            # Authenticated - WebSocket connection
│   ├── GET  /realtime/events        # Authenticated - Server-sent events
│   ├── GET  /realtime/events/health # Authenticated - SSE health check
│   ├── POST /realtime/admin/broadcast # Admin - Broadcast notification
│   ├── POST /realtime/admin/simulate/payment # Admin - Simulate payment
│   ├── POST /realtime/admin/simulate/stock # Admin - Simulate stock update
│   ├── GET  /realtime/admin/stats   # Admin - Connection statistics
│   └── POST /realtime/admin/notify/:user_id # Admin - Send user notification
└── admin/
    ├── POST /admin/auth/logout      # Admin - Logout
    ├── GET  /admin/auth/profile     # Admin - Get admin profile
    ├── GET  /admin/analytics/user-statistics # Admin - User statistics
    ├── GET  /admin/users            # Admin - List users
    ├── POST /admin/users            # Admin - Create user
    ├── GET  /admin/users/:user_id   # Admin - Get user
    ├── PUT  /admin/users/:user_id   # Admin - Update user role
    ├── DELETE /admin/users/:user_id # Admin - Soft delete user
    ├── POST /admin/users/batch-update-roles # Admin - Bulk update roles
    ├── GET  /admin/users/:user_id/role-history # Admin - Get role history
    ├── GET  /admin/permission-profiles # Admin - List permission profiles
    ├── GET  /admin/permission-profiles/:profile_id # Admin - Get profile details
    └── POST /admin/permission-profiles/assign # Admin - Assign profile directly

# Admin API Routes (/api/admin/)
/api/admin/
├── auth/
│   └── POST /auth/login             # Public - Admin login
└── [Protected admin routes mounted under /api/admin/ with auth middleware]
```

## Advanced System Features

### Enterprise Rate Limiting System ✅ FULLY IMPLEMENTED

The system includes a sophisticated rate limiting infrastructure that provides granular control over API access:

#### Features:
- **Multi-Window Rate Limiting**: Per-minute, per-hour, and per-day limits
- **Per-User & Per-Endpoint**: Granular tracking for each user-endpoint combination
- **Redis-Compatible Architecture**: Designed for distributed scaling in production
- **Automatic Cleanup**: Old entries are automatically purged to prevent memory leaks
- **Admin Controls**: Ability to reset user limits and monitor usage

#### Implementation:
```rust
// Rate limiting configuration per permission profile
pub struct RateLimitConfig {
    pub requests_per_minute: Option<u32>,
    pub requests_per_hour: Option<u32>, 
    pub requests_per_day: Option<u32>,
}

// Example usage in middleware
let result = rate_limiter.check_rate_limit(
    &user_id,
    "/api/v1/analytics/eps",
    "GET",
    &config
).await?;
```

#### Rate Limits by Permission Profile:
- **Bronze**: 10/minute, 100/hour, 1000/day
- **Silver**: 50/minute, 500/hour, 5000/day  
- **Gold**: 200/minute, 2000/hour, 20000/day
- **Admin**: Unlimited

### Comprehensive Audit System ✅ FULLY IMPLEMENTED

Enterprise-grade audit logging with complete traceability and compliance features:

#### Features:
- **25+ Audit Action Types**: Covering all system operations (login, user management, permission changes, etc.)
- **Export Capabilities**: JSON, CSV, XML formats for compliance reporting
- **Statistical Analysis**: Top actions, top actors, failure rates over time periods
- **Rich Metadata**: IP tracking, session correlation, error details, duration tracking
- **Search & Filter**: Advanced querying with time ranges, actors, resources

#### Audit Actions Tracked:
```rust
pub enum AuditAction {
    Login, LoginFailed, Logout, PasswordReset,
    UserCreated, UserUpdated, UserDeleted, UserRoleChanged,
    RoleCreated, RoleUpdated, RoleDeleted, RoleAssigned,
    PolicyCreated, PolicyUpdated, PolicyDeleted,
    PermissionGranted, PermissionDenied, PermissionEvaluated,
    ConfigurationChanged, SecurityPolicyUpdated,
    AuditLogAccessed, DataExported, BackupCreated,
    // ... and more
}
```

#### API Endpoints:
- `POST /api/v1/audit/logs` - Create audit entries (public for frontend logging)
- `GET /api/v1/audit/search` - Search with filters (admin only)
- `GET /api/v1/audit/stats` - Statistical analysis (admin only)
- `GET /api/v1/audit/export` - Export data for compliance (admin only)

### Real-time Infrastructure ✅ FULLY IMPLEMENTED

WebSocket-based real-time communication system for live updates:

#### Features:
- **WebSocket Connection Management**: Automatic reconnection, heartbeat monitoring
- **Event-driven Notifications**: Real-time alerts, system status updates
- **Server-sent Events (SSE)**: Alternative for environments that don't support WebSockets
- **Connection Status Monitoring**: Health checks and diagnostics

#### Supported Event Types:
- User permission changes
- System maintenance notifications
- Real-time analytics updates
- Payment status changes
- Security alerts

#### API Endpoints:
- `GET /api/v1/realtime/connect` - WebSocket upgrade
- `POST /api/v1/realtime/events` - Server-sent events fallback
- `GET /api/v1/realtime/status` - Connection health check

### Soft Delete User Management ✅ IMPLEMENTED

Advanced user lifecycle management with restoration capabilities:

#### Features:
- **Soft Delete**: Users are marked as deleted but data is preserved
- **Restoration**: Ability to restore deleted users with full data integrity
- **Audit Trail**: All user lifecycle events are tracked
- **Data Retention**: Configurable retention periods for deleted user data

## Security Architecture

### Authentication Flow
```typescript
// Frontend: SSR auth with HTTP-only cookies
const authMiddleware = async (request: NextRequest) => {
  const token = request.cookies.get('auth-token')?.value;
  const user = await verifyFirebaseToken(token);
  // Load complete user profile from PostgreSQL
  const userProfile = await loadUserFromPostgres(user.uid);
  return userProfile;
};

// Backend: Token verification with PostgreSQL data
async fn auth_middleware(req: Request) -> Result<User> {
    let token = extract_token(&req)?;
    let firebase_user = verify_firebase_token(token).await?;
    // Load complete user data from PostgreSQL
    let user = load_user_from_postgres(&firebase_user.uid).await?;
    Ok(user)
}
```

### Security Layers ✅ FULLY IMPLEMENTED
1. **Network Security**: HTTPS, CORS, Advanced rate limiting ✅
2. **Authentication**: Firebase Auth tokens with PostgreSQL data ✅
3. **Authorization**: Enhanced permission profiles with: ✅
   - ✅ **API Endpoint Access Control**: Granular API endpoint permissions with wildcard support
   - ✅ **Frontend Route Guards**: Middleware-based route access control
   - ✅ **Enterprise Rate Limiting**: Multi-window rate limiting (per minute/hour/day) with Redis compatibility
   - ✅ **Expiration Handling**: Automatic feature expiration with renewal notifications
4. **Data Security**: Encryption at rest and in transit ✅
5. **Session Security**: HTTP-only cookies with secure flags ✅
6. **Comprehensive Audit System**: Full audit trail with export capabilities ✅

## Performance Architecture

### Scalability Design ✅ FULLY IMPLEMENTED
- **Stateless Services**: No server-side session storage ✅
- **SSR Optimization**: Server-side rendering with edge caching ✅
- **Database Scaling**: PostgreSQL with connection pooling and optimized indexes ✅
- **Real-time Infrastructure**: WebSocket connection management with event-driven notifications ✅
- **Caching Strategy**: Multi-layer caching: ✅
  - ✅ **Permission Profile Cache**: In-memory/Redis-based permission profile caching (switchable)
  - ✅ **Route Pattern Cache**: Pre-compiled route patterns for fast matching
  - ✅ **API Response Cache**: Market data and analytics results
  - ✅ **Enterprise Rate Limit State**: Multi-window rate limiting with automatic cleanup
- **Payment Processing**: Async webhook processing with retry logic ✅
- **Expiration Management**: Background job for feature expiration checks ✅
- **Audit & Compliance**: Comprehensive audit logging with statistical analysis and export ✅

### Performance Targets
- **API Response**: <2 seconds for all endpoints
- **SSR Rendering**: <1 second initial page load
- **Database Queries**: <500ms for complex analytics queries
- **User Capacity**: Support 20,000+ concurrent users
- **Permission Checks**: <50ms for API endpoint validation
- **Rate Limiting**: <10ms overhead per request

## Deployment Architecture

### Vercel Deployment
```
Production Domains:
- epsx.com              # User-facing analytics platform (SSR)
- admin.epsx.com        # Administrative interface
- api.epsx.com          # Backend API endpoints

Environment Management:
- Development:  dev.epsx.com (Firebase + Local PostgreSQL)
- Staging:      stage.epsx.com (Firebase + Cloud PostgreSQL)
- Production:   epsx.com (Firebase + Production PostgreSQL + Redis)
```

### Infrastructure Components
- **Frontend**: Vercel Edge Functions with SSR
- **Backend**: Vercel Serverless Functions (Rust)
- **Database**: Managed PostgreSQL with automated backups and optimized indexes
- **Authentication**: Firebase Auth (email/password only)
- **Cache**: Vercel KV (Redis) for:
  - Permission profile caching
  - Rate limiting state management
  - API response caching
  - Route pattern compilation
- **Background Jobs**: Scheduled functions for:
  - Feature expiration checks
  - Payment reconciliation
  - Analytics aggregation
- **Monitoring**: Comprehensive logging and metrics collection

---

**Document Version**: 5.0  
**Last Updated**: 2025-07-25  
**Status**: ✅ **COMPLETE ENTERPRISE TECHNICAL ARCHITECTURE** - Enhanced IAM + Advanced Analytics Fully Operational  
**Implementation Status**: ✅ **100% Complete Enterprise Solution** (All core systems, enhanced IAM, and advanced analytics operational)  
**Major Updates**:
- ✅ **COMPLETE**: Enhanced database schema with API endpoint and route access control
- ✅ **COMPLETE**: Granular permission system with rate limiting fully operational
- ✅ **COMPLETE**: Payment-based auto-assignment infrastructure implemented
- ✅ **COMPLETE**: Feature expiration and renewal system operational
- ✅ **COMPLETE**: Performance optimization with multi-layer caching implemented
- ✅ **COMPLETE**: Background job system with comprehensive monitoring
- ✅ **NEW**: Advanced analytics components (PermissionAnalytics, AssignmentAnalytics) with real-time dashboards
- ✅ **NEW**: Visual rule building tools (VisualRuleBuilder, ConditionBuilder) with drag-and-drop interfaces
- ✅ **NEW**: Complete enterprise-ready admin UI with comprehensive management capabilities
**System Status**: ✅ **Enterprise-Ready Production System** with complete feature set