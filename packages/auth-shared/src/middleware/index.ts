// Middleware auth utilities
export {
  getSessionToken,
  checkPermissionAccess,
  checkRoleHierarchy,
  clearPermissionCache,
  addSecurityHeaders,
  addAdminSecurityHeaders
} from './auth';

export type {
  MiddlewareAuthConfig
} from './auth';

// Unified middleware
export {
  createUnifiedMiddleware,
  createFrontendMiddleware,
  createAdminMiddleware,
  type UnifiedMiddlewareConfig,
} from './unified-middleware';