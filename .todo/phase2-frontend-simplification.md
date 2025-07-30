# Phase 2: Frontend Simplification

## Overview
Replace complex frontend auth logic with thin API clients that call the centralized backend services.

## Frontend App Changes (`apps/frontend/`)

### 1. Middleware Simplification
**Location**: `apps/frontend/middleware.ts`

#### 1.1 Replace Complex Logic with API Calls
- [ ] Remove route-to-permission mapping
- [ ] Remove permission profile checking logic
- [ ] Remove permission validation cache
- [ ] Replace with simple backend API calls

```typescript
// Before: Complex middleware logic (REMOVE)
const hasAccess = await checkPermissionAccess(pathname, userLevel, permissions);

// After: Simple API call (IMPLEMENT)
const { allowed } = await fetch('/api/v1/auth/validate-access', {
  method: 'POST',
  body: JSON.stringify({ route: pathname, method: 'GET', app_type: 'Frontend' })
}).then(r => r.json());
```

#### 1.2 Simplified Middleware Structure
- [ ] Keep only basic session validation
- [ ] Add API call for route protection
- [ ] Remove complex permission logic
- [ ] Add minimal caching (30 seconds max)

```typescript
export async function middleware(request: NextRequest) {
  // Simple session check
  const sessionValid = await validateSession(request);
  if (!sessionValid.authenticated) {
    return redirectToLogin();
  }

  // Route access validation via API
  const routeAccess = await validateRouteAccess(request.nextUrl.pathname);
  if (!routeAccess.allowed) {
    return redirectToUnauthorized();
  }

  return NextResponse.next();
}
```

### 2. Auth Context Simplification
**Location**: `apps/frontend/context/auth-context.tsx`

#### 2.1 Remove Complex State Management
- [ ] Remove permission validation logic
- [ ] Remove optimistic updates for auth
- [ ] Remove client-side session parsing
- [ ] Simplify to data container only

```typescript
// Before: Complex context (REMOVE)
interface AuthContextType {
  // ... complex state management
  validatePermission: (permission: string) => boolean;
  refreshSession: () => Promise<void>;
  // ... other complex methods
}

// After: Simple data container (IMPLEMENT)
interface AuthContextType {
  user: User | null;
  loading: boolean;
  authenticated: boolean;
  permissions: string[];
  refreshUser: () => Promise<void>;
}
```

#### 2.2 API-based State Updates
- [ ] Replace local state management with API calls
- [ ] Add simple data fetching hooks
- [ ] Remove auth decision logic

### 3. Session Management Simplification
**Location**: `apps/frontend/lib/session.ts`

#### 3.1 Remove Client-side Session Logic
- [ ] Remove session parsing functions
- [ ] Remove fallback session handling
- [ ] Remove client-side validation
- [ ] Keep only API communication

```typescript
// Before: Complex session handling (REMOVE)
export function parseSession(sessionData: string): Session | null {
  // Complex parsing logic
}

// After: Simple API wrapper (IMPLEMENT)
export async function getSessionInfo(): Promise<SessionInfo> {
  return fetch('/api/v1/auth/session-info').then(r => r.json());
}

export async function validateCurrentSession(): Promise<boolean> {
  const response = await fetch('/api/v1/auth/validate-session', { method: 'POST' });
  return response.ok;
}
```

### 4. Auth Server Utilities Simplification
**Location**: `apps/frontend/lib/auth-server.ts`

#### 4.1 Replace with API Client
- [ ] Remove duplicate auth logic
- [ ] Replace with backend API calls
- [ ] Simplify to API wrapper functions

```typescript
// Before: Complex server auth (REMOVE)
export async function getServerAuth(): Promise<AuthResult> {
  // Complex validation logic with fallbacks
}

// After: Simple API client (IMPLEMENT)
export async function getServerAuth(): Promise<AuthResult> {
  return fetch('/api/v1/auth/me').then(r => r.json());
}
```

### 5. Cookie Management Simplification
**Location**: `apps/frontend/lib/cookies.ts`

#### 5.1 Remove Client-side Cookie Manipulation
- [ ] Remove complex cookie utilities
- [ ] Keep only read-only cookie access
- [ ] Backend handles all cookie setting

```typescript
// Before: Complex cookie management (REMOVE)
export const ServerCookies = {
  set: (name: string, value: string, options?: CookieOptions) => {
    // Complex setting logic
  },
  // ... other complex methods
};

// After: Read-only access (IMPLEMENT)
export const CookieReader = {
  get: (name: string): string | undefined => {
    return cookies().get(name)?.value;
  },
  has: (name: string): boolean => {
    return cookies().has(name);
  }
};
```

### 6. Permission Hook Simplification
**Location**: `apps/frontend/hooks/usePermissions.ts`

#### 6.1 API-based Permission Checking
- [ ] Remove client-side permission logic
- [ ] Replace with API calls
- [ ] Add simple caching

```typescript
// Before: Complex permission logic (REMOVE)
export function usePermissions() {
  // Complex client-side checking
}

// After: API-based checking (IMPLEMENT)
export function usePermissions() {
  const { data: permissions } = useSWR('/api/v1/auth/permissions');
  
  const hasPermission = useCallback(async (permission: string) => {
    const response = await fetch('/api/v1/auth/check-permission', {
      method: 'POST',
      body: JSON.stringify({ permission })
    });
    return response.json();
  }, []);
  
  return { permissions, hasPermission };
}
```

## Admin Frontend Changes (`apps/admin-frontend/`)

### 1. Admin Middleware Simplification
**Location**: `apps/admin-frontend/middleware.ts`

#### 1.1 Use Shared Auth Logic
- [ ] Remove duplicate permission mapping
- [ ] Use same API endpoints as regular frontend
- [ ] Specify admin app type in requests

```typescript
export async function middleware(request: NextRequest) {
  // Use same validation as frontend, but specify admin app
  const routeAccess = await fetch('/api/v1/auth/validate-access', {
    method: 'POST',
    body: JSON.stringify({ 
      route: request.nextUrl.pathname, 
      method: 'GET', 
      app_type: 'Admin' 
    })
  }).then(r => r.json());
  
  if (!routeAccess.allowed) {
    return redirectToUnauthorized();
  }
  
  return NextResponse.next();
}
```

### 2. Admin Auth Context Simplification
**Location**: `apps/admin-frontend/auth/ctx.tsx`

#### 2.1 Align with Frontend Context
- [ ] Use same structure as frontend context
- [ ] Remove admin-specific auth logic
- [ ] Use unified API endpoints

```typescript
// Unified admin context (same structure as frontend)
interface AdminAuthContextType {
  user: AdminUser | null;
  loading: boolean;
  authenticated: boolean;
  permissions: string[];
  roles: string[];
  refreshUser: () => Promise<void>;
}
```

### 3. Admin Service Simplification
**Location**: `apps/admin-frontend/services/adminService.ts`

#### 3.1 Remove Duplicate Logic
- [ ] Remove auth-related service methods
- [ ] Use shared auth API endpoints
- [ ] Keep only admin-specific business logic

## Shared Package Updates

### 1. Auth Shared Package Simplification
**Location**: `packages/auth-shared/src/server/auth.ts`

#### 1.1 Create Unified API Client
- [ ] Remove complex middleware utilities
- [ ] Create simple API client for both apps
- [ ] Add shared types and interfaces

```typescript
export class UnifiedAuthClient {
  constructor(private baseUrl: string) {}
  
  async validateSession(): Promise<SessionValidationResponse> {
    return this.post('/api/v1/auth/validate-session');
  }
  
  async validateRouteAccess(route: string, appType: AppType): Promise<RouteAccessResponse> {
    return this.post('/api/v1/auth/validate-access', { route, method: 'GET', app_type: appType });
  }
  
  async getCurrentUser(): Promise<User> {
    return this.get('/api/v1/auth/me');
  }
  
  private async post(endpoint: string, data?: any) {
    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: data ? JSON.stringify(data) : undefined,
      credentials: 'include'
    });
    return response.json();
  }
  
  private async get(endpoint: string) {
    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      credentials: 'include'
    });
    return response.json();
  }
}
```

### 2. Server Actions Simplification
**Location**: `packages/server-actions/src/actions/enhanced-auth.ts`

#### 2.1 Replace with API Calls
- [ ] Remove complex server action logic
- [ ] Replace with backend API calls
- [ ] Keep validation and error handling

## Firebase Analytics Preservation

### 1.1 Keep Analytics in Frontends
- [ ] **PRESERVE**: `packages/firebase-analytics/` - Keep unchanged
- [ ] **PRESERVE**: `apps/frontend/lib/firebase-analytics.ts` - Keep unchanged
- [ ] **PRESERVE**: `apps/admin-frontend/lib/firebase-analytics.ts` - Keep unchanged
- [ ] **PRESERVE**: Analytics hooks and components in both frontends

### 1.2 Ensure Analytics Still Works
- [ ] Test analytics after auth changes
- [ ] Verify user tracking still functions
- [ ] Confirm event logging continues

## Cookie Management Updates

### 1. Remove Frontend Cookie Setting
**Location**: Multiple files

#### 1.1 Frontend Cookie Changes
- [ ] Remove all cookie setting from frontend
- [ ] Keep only cookie reading for session info
- [ ] Backend handles all auth cookies

#### 1.2 Unified Session Cookie
- [ ] Use single `sess_id` cookie for both apps
- [ ] Remove `admin_sess_id` completely
- [ ] Backend sets all cookie attributes

## Performance Considerations

### 1.1 API Call Optimization
- [ ] Add minimal caching for frequent checks (30 seconds)
- [ ] Batch route validation where possible
- [ ] Use lightweight endpoints for status checks

### 1.2 Loading States
- [ ] Add proper loading states for auth checks
- [ ] Implement graceful fallbacks
- [ ] Optimize initial page load

## Testing Requirements

### 1.1 Frontend Tests
- [ ] Test simplified middleware
- [ ] Verify auth context functionality
- [ ] Test API client integration
- [ ] Ensure Firebase Analytics still works

### 1.2 Integration Tests
- [ ] Test frontend-backend auth flow
- [ ] Verify session management
- [ ] Test permission checking
- [ ] Admin app functionality

### 1.3 Regression Tests
- [ ] Ensure no loss of functionality
- [ ] Verify all auth flows work
- [ ] Test error handling
- [ ] Performance benchmarks

## Migration Strategy

### 1.1 Gradual Migration
- [ ] Migrate one component at a time
- [ ] Keep old logic as fallback initially
- [ ] Test thoroughly before removing old code
- [ ] Monitor for issues

### 1.2 Feature Flags
- [ ] Use feature flags to toggle new auth logic
- [ ] Gradual rollout to users
- [ ] Quick rollback if needed

## Dependencies
- **Requires**: Phase 1 backend APIs completed and tested
- **Blocks**: Phase 3 session unification

## Completion Criteria
- [ ] Frontend middleware simplified to API calls only
- [ ] Auth contexts are thin data containers
- [ ] All auth logic removed from frontend
- [ ] Firebase Analytics preserved and functional
- [ ] Cookie management centralized in backend
- [ ] All tests passing
- [ ] Performance benchmarks met
- [ ] No regression in functionality
- [ ] Admin frontend aligned with regular frontend