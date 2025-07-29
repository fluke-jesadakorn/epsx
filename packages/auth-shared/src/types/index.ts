// Export consolidated auth types that eliminate duplication with @epsx/types
export type {
  // Main auth types from @epsx/types
  UserProfile,
  LoginRequest,
  RegisterRequest,
  EnhancedRegisterRequest,
  PasswordChangeRequest,
  PasswordResetRequest,
  ProfileUpdateRequest,
  RegistrationResponse,
  AuthCookies,
  UserPreferences,
  NotificationPreferences,
  
  // Auth-shared specific types
  AuthContextState,
  AuthService,
  AuthResult,
  AuthError,
  
  // Legacy types (deprecated)
  BaseUser,
  AuthenticatedUser,
  FrontendUser,
  AdminUser,
  BackendUser,
  SignInCredentials,
  SignUpData,
  UserCredentials
} from './consolidated-auth';

export type {
  // Main permission types from @epsx/types
  Permission,
  Role,
  PermissionCheckRequest,
  PermissionCheckResponse,
  UserPermissionStatus,
  DynamicPermissionProfile,
  PermissionProfilePermission,
  PermissionCondition,
  PermissionCategory,
  PermissionScope,
  PermissionProfileScope,
  PermissionProfileStatus,
  PackageTier,
  
  // Auth-shared specific permission types
  PermissionCheckResult,
  RoutePermissionConfig,
  PermissionCacheEntry,
  PermissionProfile,
  AuthMiddlewareConfig,
  AuthGuardProps,
  PermissionChecker,
  RoleChecker
} from './consolidated-permissions';

export {
  UserRole,
  ROLE_HIERARCHY
} from './consolidated-permissions';