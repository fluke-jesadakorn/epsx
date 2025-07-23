# EPSX Architecture Design

## 1. System Overview

### High-Level Architecture
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Frontend      │    │  Admin Frontend │    │    Backend      │
│   (Next.js)     │    │   (Next.js)     │    │    (Rust)       │
│                 │    │                 │    │                 │
│  - Trading UI   │    │  - User Mgmt    │    │  - API Layer    │
│  - Auth         │    │  - IAM          │    │  - Business     │
│  - Theme        │    │  - Analytics    │    │  - Data Layer   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │    Firebase     │
                    │                 │
                    │  - Auth         │
                    │  - Firestore    │
                    │  - Functions    │
                    └─────────────────┘
```

### Monorepo Structure
```
epsx/
├── apps/
│   ├── frontend/           # User-facing trading platform
│   ├── admin-frontend/     # Admin dashboard
│   └── backend/           # Rust API server
├── packages/              # Shared libraries (future)
├── .requirement_docs/     # Documentation
└── configs/              # Shared configurations
```

## 2. Backend Architecture (Rust)

### Hexagonal + Clean Code Architecture
```
┌─────────────────────────────────────────────────────────────┐
│                        Web Layer                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │   Routes    │  │  Handlers   │  │ Middleware  │       │
│  │             │  │             │  │             │       │
│  │ - Auth      │  │ - Request   │  │ - CORS      │       │
│  │ - Stock     │  │ - Response  │  │ - Auth      │       │
│  │ - Payment   │  │ - Error     │  │ - Rate Lmt  │       │
│  └─────────────┘  └─────────────┘  └─────────────┘       │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │ Use Cases   │  │    DTOs     │  │   Ports     │       │
│  │             │  │             │  │             │       │
│  │ - AuthUC    │  │ - AuthDTO   │  │ - Repos     │       │
│  │ - StockUC   │  │ - StockDTO  │  │ - Services  │       │
│  │ - PaymentUC │  │ - PaymentDTO│  │ - Events    │       │
│  └─────────────┘  └─────────────┘  └─────────────┘       │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                     Domain Layer                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │  Entities   │  │   Values    │  │  Services   │       │
│  │             │  │             │  │             │       │
│  │ - User      │  │ - UserId    │  │ - AuthSvc   │       │
│  │ - Stock     │  │ - StockId   │  │ - PermSvc   │       │
│  │ - Payment   │  │ - Amount    │  │ - PolicySvc │       │
│  └─────────────┘  └─────────────┘  └─────────────┘       │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                  Infrastructure Layer                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │ Repositories│  │  Services   │  │   Events    │       │
│  │             │  │             │  │             │       │
│  │ - UserRepo  │  │ - EmailSvc  │  │ - EventBus  │       │
│  │ - StockRepo │  │ - PaySvc    │  │ - Dispatcher│       │
│  │ - AuditRepo │  │ - NotifSvc  │  │ - Handlers  │       │
│  └─────────────┘  └─────────────┘  └─────────────┘       │
└─────────────────────────────────────────────────────────────┘
```

### Backend Module Organization
```rust
src/
├── main.rs                 # Application entry point
├── lib.rs                  # Library root
├── web/                    # Web layer (adapters)
│   ├── auth/
│   ├── stock/
│   ├── payment/
│   └── middleware/
├── app/                    # Application layer
│   ├── use_cases/
│   ├── dtos/
│   └── ports/
├── dom/                    # Domain layer
│   ├── entities/
│   ├── values/
│   ├── services/
│   └── events/
└── infra/                  # Infrastructure layer
    ├── repos/
    ├── services/
    └── events/
```

## 3. Frontend Architecture

### Next.js App Structure
```
app/
├── layout.tsx              # Root layout
├── page.tsx               # Home page
├── auth/                  # Authentication pages
├── trading/               # Trading interface
├── dashboard/             # User dashboard
├── api/                   # API routes
└── components/            # Shared components
```

### Component Architecture
```
components/
├── ui/                    # Base UI components
├── auth/                  # Authentication components
├── trading/               # Trading-specific components
├── shared/                # Cross-app shared components
└── layout/                # Layout components
```

### State Management
```typescript
// Context-based state management
interface AppState {
  auth: AuthState;
  theme: ThemeState;
  trading: TradingState;
}

// Hooks for state access
const useAuth = () => useContext(AuthContext);
const useTheme = () => useContext(ThemeContext);
const useTrade = () => useContext(TradingContext);
```

## 4. Database Architecture

### Firebase Firestore Structure
```
/users/{userId}
  - profile: UserProfile
  - permissions: UserPermissions
  - settings: UserSettings

/stocks/{stockId}  
  - data: StockData
  - analysis: EPSAnalysis
  - recommendations: Recommendations

/trades/{tradeId}
  - userId: string
  - stockId: string
  - action: 'buy' | 'sell'
  - timestamp: Timestamp
  - strategy: string

/subscriptions/{subId}
  - userId: string
  - plan: string
  - status: 'active' | 'inactive'
  - permissions: string[]
```

### Database Abstraction Layer
```rust
// Repository trait for database operations
#[async_trait]
pub trait UserRepository {
    async fn find_by_id(&self, id: &UserId) -> Result<User>;
    async fn save(&self, user: &User) -> Result<()>;
    async fn delete(&self, id: &UserId) -> Result<()>;
}

// Firebase implementation
pub struct FirebaseUserRepo {
    client: FirestoreClient,
}

// Future database implementations
pub struct PostgresUserRepo {
    pool: PgPool,
}
```

## 5. Authentication & Authorization

### Firebase Auth Integration
```typescript
// Frontend auth service
interface AuthService {
  login: (email: string, password: string) => Promise<User>;
  logout: () => Promise<void>;
  getCurrentUser: () => Promise<User | null>;
  refreshToken: () => Promise<string>;
}

// Backend auth middleware
async fn auth_middleware(req: Request) -> Result<User> {
    let token = extract_token(&req)?;
    let user = verify_firebase_token(token).await?;
    Ok(user)
}
```

### IAM System (See iam-design.md)
- Role-based access control
- Permission templates
- Dynamic policy evaluation

## 6. API Design

### RESTful API Structure
```
/api/v1/
├── auth/
│   ├── POST /login
│   ├── POST /logout
│   └── GET  /me
├── stocks/
│   ├── GET    /stocks
│   ├── GET    /stocks/{id}
│   └── POST   /stocks/{id}/analyze
├── trades/
│   ├── POST   /trades
│   ├── GET    /trades
│   └── GET    /trades/{id}
└── users/
    ├── GET    /users/profile
    ├── PUT    /users/profile
    └── GET    /users/permissions
```

### Error Handling
```rust
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Authentication failed")]
    Unauthorized,
    #[error("Resource not found")]
    NotFound,
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error("Internal server error")]
    Internal,
}
```

## 7. Security Architecture

### Security Layers
1. **Network Security**: HTTPS, CORS, Rate limiting
2. **Authentication**: Firebase Auth tokens
3. **Authorization**: IAM system with RBAC
4. **Data Security**: Encryption at rest and in transit
5. **Session Security**: Secure cookie management

### Security Patterns
```rust
// Secure cookie configuration
let cookie = Cookie::build("session", token)
    .secure(true)
    .http_only(true)
    .same_site(SameSite::Strict)
    .max_age(Duration::hours(24))
    .finish();
```

## 8. Scalability Design

### Horizontal Scaling
- **Stateless Services**: No server-side session storage
- **Load Balancing**: Vercel edge functions
- **Database Scaling**: Firebase auto-scaling
- **Caching**: Strategic data caching

### Microservices Preparation
```
Current: Monolithic backend
Future: Service separation
- Auth Service
- Trading Service  
- Analytics Service
- Notification Service
```

### Performance Optimizations
- **Database**: Indexed queries, connection pooling
- **API**: Request/response compression
- **Frontend**: Code splitting, lazy loading
- **Caching**: Redis for session data (future)

## 9. Deployment Architecture

### Vercel Deployment
```
epsx/
├── vercel.json            # Deployment configuration
├── apps/frontend/         # Auto-deployed to epsx.com
├── apps/admin-frontend/   # Auto-deployed to admin.epsx.com
└── apps/backend/         # Serverless functions
```

### Environment Management
```
Development:  dev.epsx.com
Staging:      stage.epsx.com  
Production:   epsx.com
```

## 10. Monitoring & Observability

### Logging Strategy
```rust
use tracing::{info, warn, error};

// Structured logging
info!(
    user_id = %user.id,
    action = "trade_executed",
    stock_id = %stock.id,
    "User executed trade"
);
```

### Metrics Collection
- **Performance**: Response times, throughput
- **Business**: User engagement, trade volume
- **System**: Error rates, resource usage
- **Security**: Auth failures, suspicious activity

---

**Document Version**: 1.0  
**Last Updated**: 2025-01-23  
**Status**: Active Development  
**Dependencies**: development-requirements.md, iam-design.md  
**Next Review**: Implementation Phase