# Clean Implementation Plan: AWS IAM-Inspired Permission System

## 🎯 Overview
Since you're starting fresh without Firestore, here's a clean implementation plan that doesn't rely on legacy migration code.

## 📋 Clean Architecture

### **1. Core Permission Service** ✅ Ready
```
/packages/auth/src/permission-service/
├── types.ts              # Core interfaces
├── PermissionService.ts  # Permission evaluation engine
├── PolicyTemplates.ts    # Policy templates
├── hooks/
│   └── usePermissions.ts # React hooks
├── components/
│   └── PermissionGates.tsx # React components
├── database/
│   ├── schema.sql        # Fresh database schema
│   └── seed.sql          # Initial data (no migration)
└── index.ts              # Clean exports
```

### **2. Fresh Database Schema** 
```sql
-- No migration needed - fresh start
-- Core tables:
- users (id, email, created_at, updated_at)
- policies (id, name, document, created_at)
- roles (id, name, description, created_at)
- groups (id, name, description, created_at)
- user_groups (user_id, group_id, granted_at)
- user_roles (user_id, role_id, granted_at)
- role_policies (role_id, policy_id, attached_at)
- group_policies (group_id, policy_id, attached_at)
- permission_cache (user_id, resource, action, allowed, expires_at)
- audit_log (id, user_id, action, resource, result, timestamp)
```

### **3. React Hooks - Clean Implementation**
```typescript
// No legacy dependencies
export function useStockAnalyticsPermissions() {
  const { user } = useAuth(); // Your auth system
  const { hasPermission } = usePermission();
  
  return {
    canRead: hasPermission('stock:rankings', 'read'),
    canAnalyze: hasPermission('stock:analytics', 'analyze'),
    canExport: hasPermission('stock:data', 'export'),
    canScreen: hasPermission('stock:screener', 'screen'),
    maxRankings: calculateMaxRankings(user),
    userTier: user?.tier || 'BRONZE'
  };
}
```

### **4. Clean Component Implementation**
```tsx
// No legacy hooks or migration logic
export default function StockRankingTable() {
  const { canRead, canAnalyze, canExport, maxRankings, userTier } = useStockAnalyticsPermissions();
  
  return (
    <PermissionGate resource="stock:rankings" action="read">
      <div>
        {/* Clean implementation */}
        <RankingDisplay maxItems={maxRankings} />
        
        {canAnalyze && <AnalysisTools />}
        {canExport && <ExportButton />}
        
        <TierIndicator tier={userTier} />
      </div>
    </PermissionGate>
  );
}
```

## 🧹 Cleanup Tasks

### **1. Remove Legacy Files**
```bash
# Remove migration-related files
rm -rf /apps/frontend/hooks/usePermissionAwareAccess.ts
rm -rf /apps/frontend/hooks/useRankingAccess.ts
rm -rf /apps/frontend/middleware/rankingAccess.ts
rm -rf /apps/frontend/middleware/userAccess.ts
rm -rf /MIGRATION_GUIDE.md

# Remove Firestore dependencies
rm -rf /apps/frontend/services/firestore/
rm -rf /apps/frontend/lib/firebase*.ts
```

### **2. Update Package Dependencies**
```json
{
  "dependencies": {
    // Remove Firebase
    // "firebase": "^x.x.x",
    // "firebase-admin": "^x.x.x",
    
    // Add new dependencies
    "pg": "^8.11.3",
    "@types/pg": "^8.10.9",
    "redis": "^4.6.10",
    "jsonwebtoken": "^9.0.2"
  }
}
```

### **3. Clean Environment Variables**
```env
# Remove Firestore config
# FIREBASE_PROJECT_ID=
# FIREBASE_CLIENT_EMAIL=
# FIREBASE_PRIVATE_KEY=

# Add new config
DATABASE_URL=postgresql://user:password@localhost:5432/epsx
REDIS_URL=redis://localhost:6379
JWT_SECRET=your-jwt-secret
PERMISSION_CACHE_TTL=300
```

## 🗄️ Database Setup

### **1. Fresh Database Schema**
```sql
-- Create fresh database
CREATE DATABASE epsx_permissions;

-- Run schema creation
\i /packages/auth/src/permission-service/database/schema.sql

-- Seed initial data
\i /packages/auth/src/permission-service/database/seed.sql
```

### **2. Initial Data Seeding**
```sql
-- Create default policies
INSERT INTO policies (name, document) VALUES
('BronzePolicy', '{
  "Version": "2024-01-01",
  "Statement": [{
    "Effect": "Allow",
    "Action": ["stock:rankings:read"],
    "Resource": "*",
    "Condition": {
      "NumericLessThan": {
        "stock:rankings:count": 10
      }
    }
  }]
}'),
('SilverPolicy', '{
  "Version": "2024-01-01",
  "Statement": [{
    "Effect": "Allow",
    "Action": ["stock:rankings:read", "stock:analytics:analyze"],
    "Resource": "*",
    "Condition": {
      "NumericLessThan": {
        "stock:rankings:count": 25
      }
    }
  }]
}');

-- Create default roles
INSERT INTO roles (name, description) VALUES
('Bronze', 'Basic tier with limited access'),
('Silver', 'Premium tier with analytics'),
('Gold', 'Advanced tier with export capabilities'),
('Platinum', 'Full access tier');

-- Attach policies to roles
INSERT INTO role_policies (role_id, policy_id) VALUES
(1, 1), -- Bronze -> BronzePolicy
(2, 2); -- Silver -> SilverPolicy
```

## 🔧 Implementation Steps

### **Step 1: Database Setup**
```bash
# Create fresh database
createdb epsx_permissions

# Run schema
psql epsx_permissions < /packages/auth/src/permission-service/database/schema.sql

# Seed data
psql epsx_permissions < /packages/auth/src/permission-service/database/seed.sql
```

### **Step 2: Update tsconfig.json**
```json
{
  "compilerOptions": {
    "paths": {
      "@epsx/auth/*": ["./packages/auth/src/*"],
      "@epsx/auth/permission-service": ["./packages/auth/src/permission-service"]
    }
  }
}
```

### **Step 3: Create Clean Auth Context**
```typescript
// /apps/frontend/context/auth-context.tsx
export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  
  // Your auth logic here (JWT, OAuth, etc.)
  // No Firebase dependencies
  
  return (
    <AuthContext.Provider value={{ user, setUser }}>
      {children}
    </AuthContext.Provider>
  );
}
```

### **Step 4: Update Components**
```typescript
// Clean implementation - no legacy code
export default function StockRankingTable() {
  const { canRead, canAnalyze, maxRankings } = useStockAnalyticsPermissions();
  
  return (
    <PermissionGate resource="stock:rankings" action="read">
      <div>Rankings: {maxRankings}</div>
      {canAnalyze && <AnalysisButton />}
    </PermissionGate>
  );
}
```

## 🎯 Benefits of Clean Implementation

### **1. No Legacy Baggage**
- ✅ No migration complexity
- ✅ No backward compatibility issues
- ✅ Clean, modern codebase

### **2. Simpler Architecture**
- ✅ Direct permission checks
- ✅ No bridge components
- ✅ Clear separation of concerns

### **3. Better Performance**
- ✅ No legacy code overhead
- ✅ Optimized database queries
- ✅ Efficient caching

### **4. Easier Maintenance**
- ✅ Single source of truth
- ✅ Clear code patterns
- ✅ Better testing coverage

## 🚀 Next Steps

1. **Remove Legacy Files**: Clean up old Firestore and migration code
2. **Setup Fresh Database**: Create new PostgreSQL database with clean schema
3. **Update Dependencies**: Remove Firebase, add new dependencies
4. **Update Components**: Use clean permission-based components
5. **Test Implementation**: Verify all functionality works

This gives you a clean, modern permission system without any legacy dependencies! 🎉
