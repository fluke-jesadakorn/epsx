// Export all auth types
export type {
  BaseUser,
  AuthenticatedUser,
  FrontendUser,
  AdminUser,
  BackendUser
} from './user';

export type {
  SignInCredentials,
  SignUpData,
  UserCredentials,
  AuthContextState,
  AuthService,
  AuthResult,
  AuthError
} from './auth';

export {
  UserRole,
  ROLE_HIERARCHY
} from './permissions';

export type {
  PermissionCheckResult,
  RoutePermissionConfig,
  PermissionCacheEntry,
  PermissionProfile
} from './permissions';