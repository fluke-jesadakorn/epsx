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

### Professional API Structure with Access Control
```
/api/v1/
├── authentication/
│   ├── POST /login                  # Public
│   ├── POST /logout                 # Authenticated
│   ├── POST /register               # Public
│   └── GET  /me                     # Authenticated
├── market-data/
│   ├── GET    /market-data          # Bronze+ profiles
│   ├── GET    /market-data/basic    # Bronze+ profiles
│   ├── GET    /market-data/{symbol} # Silver+ profiles
│   └── POST   /analyze/{symbol}     # Profile-specific limits
├── analytics/
│   ├── GET    /analytics/eps        # Bronze+ profiles
│   ├── GET    /analytics/patterns   # Silver+ profiles
│   └── GET    /analytics/ai-insights # Gold+ profiles
├── alerts/
│   ├── GET    /alerts/basic         # Silver+ profiles
│   └── POST   /alerts/custom        # Gold+ profiles
├── payments/
│   ├── POST   /crypto-payments/initiate      # Authenticated
│   ├── GET    /crypto-payments/{id}/status   # Authenticated
│   └── POST   /crypto-payments/webhook       # System only
├── system/
│   ├── GET    /health               # Public
│   └── GET    /cache/status         # Admin only
└── webhooks/
    └── POST   /musepay              # System only

/api/admin/
├── analytics/
│   ├── GET    /analytics/overview   # Admin profile required
│   └── GET    /analytics/users      # Admin profile required
├── user-management/
│   ├── GET    /users                # Admin profile required
│   ├── POST   /users/{id}/assign-permission-profile # Admin profile
│   └── POST   /users/bulk-assign    # Admin profile required
├── permission-profiles/
│   ├── GET    /profiles             # Admin profile required
│   ├── POST   /profiles/{id}/update # Admin profile required
│   └── GET    /profiles/{id}/users  # Admin profile required
└── authentication/
    ├── POST   /admin/login          # Admin profile required
    └── GET    /admin/permissions    # Admin profile required
```

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
1. **Network Security**: HTTPS, CORS, Rate limiting ✅
2. **Authentication**: Firebase Auth tokens with PostgreSQL data ✅
3. **Authorization**: Enhanced permission profiles with: ✅
   - ✅ **API Endpoint Access Control**: Granular API endpoint permissions with wildcard support
   - ✅ **Frontend Route Guards**: Middleware-based route access control
   - ✅ **Rate Limiting**: Per-profile API rate limits (per minute/hour)
   - ✅ **Expiration Handling**: Automatic feature expiration with renewal notifications
4. **Data Security**: Encryption at rest and in transit ✅
5. **Session Security**: HTTP-only cookies with secure flags ✅

## Performance Architecture

### Scalability Design ✅ FULLY IMPLEMENTED
- **Stateless Services**: No server-side session storage ✅
- **SSR Optimization**: Server-side rendering with edge caching ✅
- **Database Scaling**: PostgreSQL with connection pooling and optimized indexes ✅
- **Caching Strategy**: Multi-layer caching: ✅
  - ✅ **Permission Profile Cache**: In-memory/Redis-based permission profile caching (switchable)
  - ✅ **Route Pattern Cache**: Pre-compiled route patterns for fast matching
  - ✅ **API Response Cache**: Market data and analytics results
  - ✅ **Rate Limit State**: In-memory/Redis rate limiting (production-ready)
- **Payment Processing**: Async webhook processing with retry logic ✅
- **Expiration Management**: Background job for feature expiration checks ✅

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