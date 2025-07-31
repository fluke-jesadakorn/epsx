# Import Optimization Guide

## Overview
This guide provides the optimal import patterns to minimize bundle size, avoid circular dependencies, and reduce build times across the EPSX monorepo.

## Package Structure & Optimal Imports

### 1. Types Package (`@epsx/types`)
**Single entry point with clear import guidance**

```typescript
// ✅ OPTIMAL - Import specific types
import type { User, AdminUser, PaymentTier } from '@epsx/types';

// ✅ GOOD - Multiple related types
import type { ApiResponse, RequestConfig, PaginatedResponse } from '@epsx/types';

// ❌ AVOID - Full package import
import * as Types from '@epsx/types'; // Pulls entire types package
```

### 2. UI Package (`@epsx/ui`)
**Consolidated UI components, hooks, validation, and theme**

```typescript
// ✅ OPTIMAL - Import specific components
import { Button, Card, Input } from '@epsx/ui';

// ✅ OPTIMAL - Theme and validation
import { ThemeProvider, useTheme, ValidationPresets } from '@epsx/ui';

// ✅ OPTIMAL - Utilities
import { utilsCn } from '@epsx/ui';

// ❌ AVOID - Full package import
import * from '@epsx/ui'; // Pulls all components, validation, theme
```

### 3. Auth Shared Package (`@epsx/auth-shared`)
**Unified authentication with clear client/server boundaries**

```typescript
// ✅ OPTIMAL - Types only (no bundle impact)
import type { AuthConfig, AuthState } from '@epsx/auth-shared';

// ✅ OPTIMAL - Specific functionality
import { createFrontendMiddleware } from '@epsx/auth-shared';
import { UnifiedAuthProvider, useAuth } from '@epsx/auth-shared';
import { AuthGuard } from '@epsx/auth-shared';

// ❌ AVOID - Full package import
import * from '@epsx/auth-shared'; // Server/client conflicts
```

### 4. API Client Package (`@epsx/api-client`)
**Consolidated API operations with optimized type re-exports**

```typescript
// ✅ OPTIMAL - Standard usage (most common)
import { apiClient } from '@epsx/api-client';

// ✅ OPTIMAL - Specific functionality
import { AuthClient, isApiError } from '@epsx/api-client';

// ✅ OPTIMAL - Types only
import type { ApiResponse, RequestConfig } from '@epsx/api-client';

// ❌ AVOID - Full import with types
import * from '@epsx/api-client'; // Pulls all @epsx/types via re-export
```

### 5. Server Actions Package (`@epsx/server-actions`)
**Streamlined server operations with specific imports**

```typescript
// ✅ OPTIMAL - Auth operations
import { login, logout, getCurrentUser } from '@epsx/server-actions';

// ✅ OPTIMAL - Specific domain operations
import { createPayment, getPaymentStatus } from '@epsx/server-actions';
import { checkPermission, getUserPermissions } from '@epsx/server-actions';
import { getAdminUsers, getUserStats } from '@epsx/server-actions';

// ❌ AVOID - Full package import
import * from '@epsx/server-actions'; // Massive bundle impact
```

### 6. Shared Core Package (`@epsx/shared-core`)
**Core utilities with minimal footprint**

```typescript
// ✅ OPTIMAL - Error handling and logging
import { ErrorHandler, logger } from '@epsx/shared-core';

// ✅ OPTIMAL - Environment utilities
import { Environment, getApiBaseUrl } from '@epsx/shared-core';

// ✅ OPTIMAL - Types only
import type { Result, ValidationResult } from '@epsx/shared-core';

// ❌ AVOID - Full package import
import * from '@epsx/shared-core';
```

## Dependency Hierarchy (No Circular Dependencies)

```
Level 1 (Base): @epsx/types, @epsx/shared-core
Level 2 (Core): @epsx/ui, @epsx/api-client
Level 3 (Auth): @epsx/auth-shared
Level 4 (Actions): @epsx/server-actions
Level 5 (Apps): frontend, admin-frontend
```

**Rules:**
- Lower levels cannot import from higher levels
- Same level packages should avoid cross-imports
- Use types-only imports when possible: `import type { ... }`

## Common Anti-Patterns to Avoid

### 1. Circular Dependencies
```typescript
// ❌ BAD - Creates circular dependency
// @epsx/api-client importing from @epsx/server-actions
import { getCurrentUser } from '@epsx/server-actions';

// ✅ GOOD - Use shared types instead
import type { User } from '@epsx/types';
```

### 2. Full Package Imports
```typescript
// ❌ BAD - Pulls entire package
import * as UI from '@epsx/ui';
import * as Types from '@epsx/types';

// ✅ GOOD - Import only what you need
import { Button, Card } from '@epsx/ui';
import type { User, Payment } from '@epsx/types';
```

### 3. Deep Path Imports
```typescript
// ❌ BAD - Bypasses barrel exports
import { Button } from '@epsx/ui/src/components/button';

// ✅ GOOD - Use barrel exports
import { Button } from '@epsx/ui';
```

### 4. Mixed Type and Value Imports
```typescript
// ❌ BAD - Mixes types and values
import { User, apiClient } from '@epsx/api-client';

// ✅ GOOD - Separate type imports
import type { User } from '@epsx/api-client';
import { apiClient } from '@epsx/api-client';
```

## Bundle Size Impact

| Import Pattern | Bundle Impact | Build Time |
|---|---|---|
| `import { Button } from '@epsx/ui'` | Low | Fast |
| `import type { User } from '@epsx/types'` | None | Instant |
| `import * from '@epsx/ui'` | High | Slow |
| `import * from '@epsx/server-actions'` | Very High | Very Slow |

## Migration Strategy

### Phase 1: Fix Circular Dependencies
1. Identify circular imports using build errors
2. Move shared types to `@epsx/types`
3. Use type-only imports: `import type { ... }`

### Phase 2: Optimize Import Patterns
1. Replace `import *` with specific imports
2. Use barrel exports from package root
3. Separate type and value imports

### Phase 3: Test and Validate
1. Run build to verify no circular dependencies
2. Check bundle sizes in apps
3. Verify tree-shaking is working

## Tools for Validation

```bash
# Check for circular dependencies
npx madge --circular --extensions ts,tsx src/

# Analyze bundle size
npx webpack-bundle-analyzer dist/

# TypeScript dependency analysis
npx tsc --showConfig --listFiles
```

## Summary

- **Use specific imports** instead of `import *`
- **Separate type imports** with `import type`
- **Follow dependency hierarchy** to avoid circular dependencies
- **Use barrel exports** from package roots
- **Minimize cross-package dependencies** within same level