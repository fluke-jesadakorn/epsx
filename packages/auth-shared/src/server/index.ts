// Server-side auth utilities
export {
  getServerAuth,
  requireAuth,
  hasServerPermission,
  requirePermission,
  checkRoleHierarchy,
  requireRole
} from './auth';

export type {
  ServerAuthResult,
  AuthServerConfig
} from './auth';

// Server-side guards
export {
  SSRAuthGuard,
  SSRRoleContent,
  SSRUserInfo,
  SSRAdminGuard
} from './guards';