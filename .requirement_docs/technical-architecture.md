# EPSX Technical Architecture

## System Overview

### High-Level Architecture
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Frontend      │    │  Admin Frontend │    │    Backend      │
│   (Next.js SSR)│    │   (Next.js)     │    │    (Rust)       │
│                 │    │                 │    │                 │
│  - Analytics UI │    │  - User Mgmt    │    │  - API Layer    │
│  - Auth & IAM   │    │  - IAM Core     │    │  - Analytics    │
│  - Theme Switch │    │  - Data Mgmt    │    │  - Domain       │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
        ┌─────────────────────────┼─────────────────────────┐
        │              Data & Services Layer               │
        │  ┌─────────────┐  ┌──────────────┐  ┌─────────┐  │
        │  │  Firebase   │  │  PostgreSQL  │  │ IAM     │  │
        │  │  - Auth     │  │  - All Data  │  │ Core    │  │
        │  │  (Email/Pwd)│  │  - Analytics │  │ Module  │  │
        │  └─────────────┘  └──────────────┘  └─────────┘  │
        └─────────────────────────────────────────────────┘
```

### Monorepo Structure
```
epsx/
├── apps/
│   ├── frontend/           # User-facing analytics platform (SSR-first)
│   ├── admin-frontend/     # Administrative interface
│   └── backend/           # Rust API server with Clean Architecture
├── packages/
│   ├── ui/               # Shared UI components & theme system
│   └── utils/            # Shared utilities
└── .requirement_docs/     # Business & technical documentation
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
│   ├── templates/         # Dynamic template management
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

### PostgreSQL Schema (Primary Storage)
```sql
-- Users and authentication (linked to Firebase UID)
users (id, firebase_uid, email, created_at, updated_at)
user_profiles (user_id, name, preferences, settings)
roles (id, name, description, hierarchy)
user_roles (user_id, role_id, granted_at)
permissions (id, resource, action, conditions)
role_permissions (role_id, permission_id)

-- Analytics and market data
market_data (id, symbol, data_type, value, timestamp)
analytics_results (id, user_id, symbol, analysis_type, result, created_at)
subscriptions (id, user_id, plan, status, created_at, expires_at)
notifications (id, user_id, type, content, sent_at, read_at)

-- Dynamic template system
feature_templates (id, name, description, category, version, status, template_data JSONB)
template_variables (id, template_id, name, type, required, default_value)
template_conditions (id, template_id, field, operator, value, logic_operator)
user_features (id, user_id, template_id, feature_id, status, configuration JSONB)
feature_usage (id, user_id, feature_id, usage_data JSONB, timestamp)

-- Admin template assignments
admin_template_assignments (id, user_id, template_id, assigned_by, assignment_type, expires_at)
assignment_audit_log (id, assignment_id, action, performed_by, details JSONB, timestamp)

-- Crypto payments integration
feature_payments (id, user_id, template_id, payment_id, features_unlocked JSONB)
crypto_price_quotes (id, template_id, quote_id, usd_price, crypto_amount, currency, network)

-- Audit and compliance
audit_logs (id, user_id, action, resource, details, timestamp)
sessions (id, user_id, token_hash, created_at, expires_at)
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

### Professional API Structure
```
/api/v1/
├── authentication/
│   ├── POST /login
│   ├── POST /logout
│   ├── POST /register
│   └── GET  /me
├── market-data/
│   ├── GET    /market-data
│   ├── GET    /market-data/{symbol}
│   └── POST   /analyze/{symbol}
├── payments/
│   ├── POST   /crypto-payments/initiate
│   ├── GET    /crypto-payments/{id}/status
│   └── POST   /crypto-payments/webhook
├── system/
│   ├── GET    /health
│   └── GET    /cache/status
└── webhooks/
    └── POST   /musepay

/api/admin/
├── analytics/
│   ├── GET    /analytics/overview
│   └── GET    /analytics/users
├── user-management/
│   ├── GET    /users
│   ├── POST   /users/{id}/assign-template
│   └── POST   /users/bulk-assign
└── authentication/
    ├── POST   /admin/login
    └── GET    /admin/permissions
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

### Security Layers
1. **Network Security**: HTTPS, CORS, Rate limiting
2. **Authentication**: Firebase Auth tokens with PostgreSQL data
3. **Authorization**: IAM system with RBAC and dynamic templates
4. **Data Security**: Encryption at rest and in transit
5. **Session Security**: HTTP-only cookies with secure flags

## Performance Architecture

### Scalability Design
- **Stateless Services**: No server-side session storage
- **SSR Optimization**: Server-side rendering with edge caching
- **Database Scaling**: PostgreSQL with connection pooling
- **Caching Strategy**: Strategic data caching at multiple layers

### Performance Targets
- **API Response**: <2 seconds for all endpoints
- **SSR Rendering**: <1 second initial page load
- **Database Queries**: <500ms for complex analytics queries
- **User Capacity**: Support 20,000+ concurrent users

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
- **Database**: Managed PostgreSQL with automated backups
- **Authentication**: Firebase Auth with custom claims
- **Monitoring**: Comprehensive logging and metrics collection

---

**Document Version**: 2.1  
**Last Updated**: 2025-01-24  
**Status**: Consolidated Technical Architecture  
**Next Review**: Implementation Phase