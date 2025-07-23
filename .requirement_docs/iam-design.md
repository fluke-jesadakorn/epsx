# EPSX IAM Core Module Design

## 1. IAM System Overview

### Core Principles
- **Modular Design**: Standalone module for cross-project use
- **Plugin Architecture**: Easy integration with any project
- **Firebase Integration**: Built on Firebase Auth with extensibility
- **Role-Based Access Control**: Hierarchical permission system
- **Template-Driven**: Dynamic permission templates

### Design Goals
- Reusable across multiple projects
- Minimal integration effort
- Scalable permission management
- Audit trail and compliance
- Performance optimized

## 2. IAM Architecture

### Module Structure
```
@epsx/iam-core/
├── src/
│   ├── auth/              # Authentication layer
│   ├── rbac/              # Role-based access control
│   ├── templates/         # Permission templates
│   ├── audit/             # Audit logging
│   ├── policies/          # Policy engine
│   └── integrations/      # External integrations
├── types/                 # TypeScript definitions
├── hooks/                 # React hooks
├── components/            # UI components (optional)
└── docs/                  # Integration documentation
```

### Core Components
```typescript
// Core IAM interfaces
interface User {
  id: string;
  email: string;
  roles: Role[];
  permissions: Permission[];
  customClaims?: Record<string, any>;
}

interface Role {
  id: string;
  name: string;
  description: string;
  permissions: Permission[];
  hierarchy: number;
}

interface Permission {
  id: string;
  resource: string;
  action: string;
  conditions?: Condition[];
}
```

## 3. Authentication Layer

### Firebase Auth Integration
```typescript
class AuthService {
  // Core auth methods
  async login(email: string, password: string): Promise<User>;
  async logout(): Promise<void>;
  async refreshToken(): Promise<string>;
  async validateToken(token: string): Promise<User>;
  
  // Custom claims management
  async setCustomClaims(uid: string, claims: Claims): Promise<void>;
  async getCustomClaims(uid: string): Promise<Claims>;
}

// Session management
class SessionManager {
  async createSession(user: User): Promise<Session>;
  async validateSession(sessionId: string): Promise<Session>;
  async destroySession(sessionId: string): Promise<void>;
}
```

### Backend Auth Middleware (Rust)
```rust
// Auth middleware for Axum
pub async fn auth_middleware(
    State(auth_service): State<AuthService>,
    headers: HeaderMap,
    mut req: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let token = extract_bearer_token(&headers)?;
    let user = auth_service.verify_token(token).await?;
    
    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

// Permission checking
pub fn require_permission(
    resource: &str,
    action: &str,
) -> impl Filter<Extract = (), Error = Rejection> + Clone {
    // Implementation
}
```

## 4. Role-Based Access Control (RBAC)

### Role Hierarchy
```
Super Admin (100)
├── Organization Admin (80)
│   ├── Department Manager (60)
│   │   ├── Team Lead (40)
│   │   │   └── Member (20)
│   │   └── Specialist (30)
│   └── Analyst (50)
└── Guest (10)
```

### Permission Model
```typescript
interface ResourcePermission {
  resource: string;          // e.g., "stocks", "trades", "users"
  actions: string[];         // e.g., ["read", "write", "delete"]
  conditions?: Condition[];  // Optional conditions
}

interface Condition {
  field: string;    // e.g., "user_id", "department"
  operator: string; // e.g., "eq", "in", "contains"
  value: any;       // Condition value
}

// Example permission
const stockPermission: ResourcePermission = {
  resource: "stocks",
  actions: ["read", "analyze"],
  conditions: [
    {
      field: "subscription_level",
      operator: "gte",
      value: "premium"
    }
  ]
};
```

### Role Management
```typescript
class RoleManager {
  async createRole(role: CreateRoleRequest): Promise<Role>;
  async updateRole(id: string, updates: UpdateRoleRequest): Promise<Role>;
  async deleteRole(id: string): Promise<void>;
  async assignRole(userId: string, roleId: string): Promise<void>;
  async revokeRole(userId: string, roleId: string): Promise<void>;
  
  // Role inheritance
  async getEffectivePermissions(userId: string): Promise<Permission[]>;
  async checkPermission(userId: string, resource: string, action: string): Promise<boolean>;
}
```

## 5. Permission Templates

### Template System
```typescript
interface PermissionTemplate {
  id: string;
  name: string;
  description: string;
  category: string;
  permissions: TemplatePermission[];
  variables: TemplateVariable[];
}

interface TemplatePermission {
  resource: string;
  actions: string[];
  conditions: TemplateCondition[];
}

interface TemplateVariable {
  name: string;
  type: 'string' | 'number' | 'boolean' | 'array';
  required: boolean;
  defaultValue?: any;
}

// Example template
const tradingTemplate: PermissionTemplate = {
  id: "trading_basic",
  name: "Basic Trading Access",
  description: "Standard trading permissions for regular users",
  category: "trading",
  permissions: [
    {
      resource: "stocks",
      actions: ["read", "analyze"],
      conditions: []
    },
    {
      resource: "trades",
      actions: ["create", "read"],
      conditions: [
        {
          field: "user_id",
          operator: "eq",
          value: "{{current_user_id}}"
        }
      ]
    }
  ],
  variables: [
    {
      name: "max_trade_amount",
      type: "number",
      required: true,
      defaultValue: 10000
    }
  ]
};
```

### Template Engine
```typescript
class TemplateEngine {
  async createTemplate(template: PermissionTemplate): Promise<void>;
  async applyTemplate(userId: string, templateId: string, variables: Record<string, any>): Promise<void>;
  async validateTemplate(template: PermissionTemplate): Promise<ValidationResult>;
  async renderTemplate(templateId: string, variables: Record<string, any>): Promise<Permission[]>;
}
```

## 6. Policy Engine

### Dynamic Policy Evaluation
```typescript
interface Policy {
  id: string;
  name: string;
  description: string;
  rules: PolicyRule[];
  effect: 'allow' | 'deny';
}

interface PolicyRule {
  condition: string;  // e.g., "user.subscription === 'premium'"
  resources: string[];
  actions: string[];
}

class PolicyEngine {
  async evaluateAccess(
    user: User,
    resource: string,
    action: string,
    context?: Record<string, any>
  ): Promise<AccessDecision>;
  
  async compilePolicies(policies: Policy[]): Promise<CompiledPolicy>;
}
```

### Context-Aware Permissions
```typescript
// Example: Time-based permissions
const timeBasedPolicy: Policy = {
  id: "business_hours_trading",
  name: "Business Hours Trading Only",
  description: "Allow trading only during business hours",
  rules: [
    {
      condition: "now() >= 09:00 && now() <= 17:00 && isWeekday()",
      resources: ["trades"],
      actions: ["create", "update"]
    }
  ],
  effect: "allow"
};

// Example: Resource ownership
const ownershipPolicy: Policy = {
  id: "own_data_access",
  name: "Own Data Access",
  description: "Users can only access their own data",
  rules: [
    {
      condition: "resource.user_id === user.id",
      resources: ["trades", "portfolios", "settings"],
      actions: ["read", "update", "delete"]
    }
  ],
  effect: "allow"
};
```

## 7. Audit System

### Audit Logging
```typescript
interface AuditEvent {
  id: string;
  timestamp: Date;
  userId: string;
  action: string;
  resource: string;
  resourceId?: string;
  result: 'success' | 'failure';
  details: Record<string, any>;
  ip?: string;
  userAgent?: string;
}

class AuditLogger {
  async logEvent(event: Omit<AuditEvent, 'id' | 'timestamp'>): Promise<void>;
  async getAuditTrail(filters: AuditFilters): Promise<AuditEvent[]>;
  async generateReport(params: ReportParams): Promise<AuditReport>;
}
```

### Compliance Features
```typescript
// GDPR compliance
class ComplianceManager {
  async exportUserData(userId: string): Promise<UserDataExport>;
  async deleteUserData(userId: string): Promise<DeletionReport>;
  async anonymizeUserData(userId: string): Promise<AnonymizationReport>;
}
```

## 8. Integration Patterns

### Plugin Architecture
```typescript
// IAM Plugin interface
interface IAMPlugin {
  name: string;
  version: string;
  initialize(config: PluginConfig): Promise<void>;
  authenticate(credentials: any): Promise<User>;
  authorize(user: User, resource: string, action: string): Promise<boolean>;
}

// Firebase plugin
class FirebaseIAMPlugin implements IAMPlugin {
  name = 'firebase';
  version = '1.0.0';
  
  async initialize(config: FirebaseConfig): Promise<void> {
    // Initialize Firebase
  }
  
  async authenticate(token: string): Promise<User> {
    // Verify Firebase token
  }
  
  async authorize(user: User, resource: string, action: string): Promise<boolean> {
    // Check permissions
  }
}

// Usage in any project
const iam = new IAMCore();
await iam.use(new FirebaseIAMPlugin());
await iam.initialize(config);
```

### Frontend Integration
```typescript
// React hooks for easy integration
export function useAuth() {
  const context = useContext(IAMContext);
  return {
    user: context.user,
    login: context.login,
    logout: context.logout,
    isAuthenticated: !!context.user,
  };
}

export function usePermission(resource: string, action: string) {
  const { user } = useAuth();
  const [hasPermission, setHasPermission] = useState(false);
  
  useEffect(() => {
    if (user) {
      checkPermission(user, resource, action).then(setHasPermission);
    }
  }, [user, resource, action]);
  
  return hasPermission;
}

// Permission gate component
export function PermissionGate({ 
  resource, 
  action, 
  children, 
  fallback = null 
}: PermissionGateProps) {
  const hasPermission = usePermission(resource, action);
  return hasPermission ? children : fallback;
}
```

### Backend Integration (Rust)
```rust
// IAM service trait
#[async_trait]
pub trait IAMService: Send + Sync {
    async fn authenticate(&self, token: &str) -> Result<User>;
    async fn authorize(&self, user: &User, resource: &str, action: &str) -> Result<bool>;
    async fn get_permissions(&self, user_id: &str) -> Result<Vec<Permission>>;
}

// Firebase implementation
pub struct FirebaseIAMService {
    auth_client: FirebaseAuth,
    db_client: Firestore,
}

#[async_trait]
impl IAMService for FirebaseIAMService {
    async fn authenticate(&self, token: &str) -> Result<User> {
        // Implementation
    }
    
    async fn authorize(&self, user: &User, resource: &str, action: &str) -> Result<bool> {
        // Implementation
    }
}
```

## 9. Configuration & Setup

### Configuration Structure
```typescript
interface IAMConfig {
  auth: {
    provider: 'firebase' | 'custom';
    config: any;
  };
  rbac: {
    enableHierarchy: boolean;
    defaultRoles: string[];
  };
  audit: {
    enabled: boolean;
    storage: 'firestore' | 'postgres' | 'custom';
  };
  policies: {
    enableDynamic: boolean;
    cacheTimeout: number;
  };
}
```

### Quick Setup Guide
```typescript
// 1. Install the module
npm install @epsx/iam-core

// 2. Initialize in your app
import { IAMCore } from '@epsx/iam-core';

const iam = new IAMCore({
  auth: {
    provider: 'firebase',
    config: firebaseConfig
  },
  rbac: {
    enableHierarchy: true,
    defaultRoles: ['member']
  },
  audit: {
    enabled: true,
    storage: 'firestore'
  }
});

// 3. Wrap your app
function App() {
  return (
    <IAMProvider iam={iam}>
      <Router>
        <Routes>
          {/* Your routes */}
        </Routes>
      </Router>
    </IAMProvider>
  );
}

// 4. Use in components
function TradingPage() {
  return (
    <PermissionGate resource="stocks" action="read">
      <StockList />
    </PermissionGate>
  );
}
```

## 10. Migration & Deployment

### Module Separation Steps
1. **Extract Current IAM Code**: Move existing IAM logic to separate module
2. **Create Plugin Interface**: Define standardized plugin architecture
3. **Implement Firebase Plugin**: Current Firebase integration as plugin
4. **Update Integration Points**: Modify apps to use new IAM module
5. **Add Documentation**: Complete integration guide

### Testing Strategy
```typescript
// Unit tests for IAM core
describe('IAM Core', () => {
  test('should authenticate valid user', async () => {
    const user = await iam.authenticate(validToken);
    expect(user).toBeDefined();
  });
  
  test('should authorize user with correct permissions', async () => {
    const hasPermission = await iam.authorize(user, 'stocks', 'read');
    expect(hasPermission).toBe(true);
  });
});

// Integration tests
describe('IAM Integration', () => {
  test('should work with Firebase', async () => {
    // Test Firebase integration
  });
  
  test('should handle permission templates', async () => {
    // Test template application
  });
});
```

### Performance Considerations
- **Caching**: Permission results caching
- **Lazy Loading**: Load permissions on demand
- **Batching**: Batch permission checks
- **Indexing**: Optimize database queries

---

**Document Version**: 1.0  
**Last Updated**: 2025-01-23  
**Status**: Design Phase  
**Dependencies**: architecture-design.md  
**Implementation Priority**: High  
**Estimated Effort**: 5-7 development days