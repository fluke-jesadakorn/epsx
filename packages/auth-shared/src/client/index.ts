'use client';

// ============================================================================
// CLIENT ENTRY POINT - React components and hooks only
// ============================================================================

// Client-side guards
export {
  ClientAuthGuard,
  ClientRoleContent,
  AuthLoadingSpinner
} from './guards';

// Client-side providers & hooks
export {
  UnifiedAuthProvider,
  useUnifiedAuth,
  useAuth,
  usePermissions
} from '../providers';

// All client-side hooks
export * from '../hooks';

// Client-safe guards (with React dependencies)
export {
  AuthGuard,
  PermissionGuard,
  RoleGuard,
  AdminGuard
} from '../guards';