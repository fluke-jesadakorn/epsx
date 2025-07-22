// Main package exports
export * from './types';
export * from './session';
export * from './service';
export * from './context';
export * from './hooks';

// Permission service with template support
export * from './permission-service';

// Actions with specific naming to avoid conflicts
export {
  handleSignIn,
  handleSignOut,
  getCurrentUser,
  getAuthStatus,
  requireAuth,
  requireGuest,
  getSessionInfoAction,
  refreshSession as refreshSessionAction
} from './actions';

// Default session configuration
export { DEFAULT_SESSION_CONFIG } from './session';
