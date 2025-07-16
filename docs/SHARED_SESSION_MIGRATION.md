# Shared Session Management Migration Guide

This guide helps you migrate from separate session management in each app to a unified session system using the new `@epsx/auth` package and Turborepo features.

## Overview

The shared auth package (`@epsx/auth`) provides:
- **Unified session management** across all apps
- **Common authentication types** and interfaces
- **Shared server actions** for session handling
- **Reusable auth hooks** and context providers
- **Consistent error handling** and state management

## Package Structure

```
packages/auth/
├── src/
│   ├── types.ts          # Common auth types and interfaces
│   ├── session.ts        # Server-side session management
│   ├── actions.ts        # Server actions for auth operations
│   ├── context.ts        # React context providers (base logic)
│   ├── hooks.ts          # Custom auth hooks
│   ├── service.ts        # Auth service interface
│   └── index.ts          # Main exports
├── package.json
├── tsconfig.json
└── tsup.config.ts
```

## Migration Steps

### 1. Update Dependencies

Both frontend and admin-frontend apps now include the shared auth package:

```json
// apps/frontend/package.json & apps/admin-frontend/package.json
{
  "dependencies": {
    "@epsx/auth": "workspace:*",
    // ... other dependencies
  }
}
```

### 2. Replace Session Management

#### Before (Old Approach)
Each app had its own session files:
- `apps/frontend/lib/session.ts`
- `apps/frontend/lib/session-improved.ts`
- `apps/frontend/app/actions/auth.ts`
- `apps/admin-frontend/` (similar files)

#### After (New Approach)
Use the shared session management:

```typescript
// Import shared session functions
import { 
  createSession, 
  verifySession, 
  destroySession,
  handleSignIn,
  handleSignOut,
  getCurrentUser 
} from '@epsx/auth';
```

### 3. Update Auth Context Providers

#### Frontend App
Replace your existing auth context with the shared implementation:

```typescript
// apps/frontend/context/shared-auth-provider.tsx
import { AppAuthProvider, useAuth } from './shared-auth-provider';

// In your layout or main app component:
function App({ children }: { children: React.ReactNode }) {
  return (
    <AppAuthProvider>
      {children}
    </AppAuthProvider>
  );
}
```

#### Admin Frontend App
```typescript
// apps/admin-frontend/context/shared-admin-auth-provider.tsx
import { AppAdminAuthProvider, useAdminAuth } from './shared-admin-auth-provider';

// In your admin layout:
function AdminApp({ children }: { children: React.ReactNode }) {
  return (
    <AppAdminAuthProvider>
      {children}
    </AppAdminAuthProvider>
  );
}
```

### 4. Update Components

#### Before (Old Hooks)
```typescript
// Old way - different hooks in each app
import { useAuth } from '@/context/auth-context';
import { useAdminAuth } from '@/hooks/useAdminAuth';
```

#### After (Shared Hooks)
```typescript
// New way - consistent hooks across apps
import { useAuth } from '@/context/shared-auth-provider';
import { useAdminAuth } from '@/context/shared-admin-auth-provider';

// Or use advanced hooks from the shared package
import { 
  useAuthWithActions, 
  useAuthRequirements, 
  useSessionUtils 
} from '@epsx/auth';
```

### 5. Update Server Actions

#### Before (App-specific Actions)
```typescript
// Old: apps/frontend/app/actions/auth.ts
export async function handleSignIn(idToken: string) {
  // App-specific implementation
}
```

#### After (Shared Actions)
```typescript
// New: Import from shared package
import { handleSignIn, handleSignOut, getCurrentUser } from '@epsx/auth/actions';

// Use directly in your components or server functions
export async function signInUser(idToken: string) {
  return await handleSignIn(idToken);
}
```

### 6. Configuration

The shared package uses a default session configuration that can be customized:

```typescript
import { createSession, type SessionConfig } from '@epsx/auth';

// Custom session configuration
const customConfig: SessionConfig = {
  sessionKey: '__session',
  maxAge: 60 * 60 * 24 * 7, // 7 days
  refreshThreshold: 60 * 60 * 2, // 2 hours
  secure: true,
  sameSite: 'strict'
};

// Use with session functions
await createSession(token, customConfig);
```

## Benefits

### 1. Code Reuse
- No more duplicated session logic between apps
- Single source of truth for authentication

### 2. Consistency
- Same session behavior across frontend and admin apps
- Unified error handling and state management

### 3. Maintainability
- Changes to auth logic only need to be made in one place
- Easier testing and debugging

### 4. Type Safety
- Shared types ensure consistency
- Better TypeScript support across apps

### 5. Turborepo Integration
- Proper dependency management
- Efficient builds with caching
- Better development workflow

## Turborepo Features Used

### 1. Workspace Dependencies
```json
// Root package.json scripts
{
  "build:auth": "turbo run build --filter=@epsx/auth",
  "type-check:auth": "turbo run type-check --filter=@epsx/auth"
}
```

### 2. Dependency Graph
```json
// turbo.json
{
  "tasks": {
    "build": {
      "dependsOn": ["^build"],
      "outputs": ["dist/**", ".next/**"]
    }
  }
}
```

### 3. Caching and Parallelization
- Auth package builds first due to dependencies
- Frontend and admin apps build in parallel after auth package
- Efficient caching of build artifacts

## Example Usage

### Basic Authentication
```typescript
'use client';

import { useAuth } from '@/context/shared-auth-provider';

export function LoginForm() {
  const { signInWithEmailAndPassword, loading, error } = useAuth();
  
  const handleSubmit = async (email: string, password: string) => {
    try {
      await signInWithEmailAndPassword({ email, password });
      // Session automatically created by shared session management
    } catch (error) {
      console.error('Login failed:', error);
    }
  };

  return (
    <form onSubmit={(e) => {
      e.preventDefault();
      const formData = new FormData(e.currentTarget);
      handleSubmit(
        formData.get('email') as string,
        formData.get('password') as string
      );
    }}>
      {/* Form fields */}
    </form>
  );
}
```

### Server-side Session Verification
```typescript
// app/dashboard/page.tsx
import { getCurrentUser, requireAuth } from '@epsx/auth/actions';

export default async function DashboardPage() {
  // Require authentication, redirect to login if not authenticated
  const user = await requireAuth('/login');
  
  return (
    <div>
      <h1>Welcome, {user.email}!</h1>
      {/* Dashboard content */}
    </div>
  );
}
```

### Advanced Auth Hooks
```typescript
'use client';

import { useAuthWithActions, useAuthRequirements } from '@epsx/auth';

export function ProtectedComponent() {
  const { 
    isAuthenticated, 
    isEmailVerified, 
    user,
    changePassword 
  } = useAuthWithActions();
  
  const { requireEmailVerified } = useAuthRequirements();
  
  const handlePasswordChange = async () => {
    try {
      requireEmailVerified(); // Throws if email not verified
      await changePassword('oldPassword', 'newPassword');
    } catch (error) {
      console.error('Password change failed:', error);
    }
  };

  if (!isAuthenticated) {
    return <div>Please log in</div>;
  }

  return (
    <div>
      <p>Email: {user?.email}</p>
      <p>Verified: {isEmailVerified ? 'Yes' : 'No'}</p>
      <button onClick={handlePasswordChange}>
        Change Password
      </button>
    </div>
  );
}
```

## Next Steps

1. **Remove old session files** after confirming the migration works
2. **Update tests** to use the shared auth package
3. **Consider extending** the shared package with additional features as needed
4. **Monitor** the session behavior across both apps to ensure consistency

## Common Issues and Solutions

### Issue: Import Errors
**Problem**: Cannot find module '@epsx/auth'
**Solution**: Run `pnpm install` and `pnpm build:auth` to build the shared package

### Issue: Type Conflicts
**Problem**: Conflicting types between old and new auth systems
**Solution**: Remove old type definitions and use types from `@epsx/auth`

### Issue: Session Not Persisting
**Problem**: Sessions don't work across apps
**Solution**: Ensure both apps use the same session key and domain configuration

### Issue: Build Failures
**Problem**: Apps fail to build due to auth package dependencies
**Solution**: Check turbo.json configuration and ensure proper dependency order

This migration provides a solid foundation for shared session management across your Turborepo monorepo while maintaining the flexibility to customize behavior per app when needed.
