/**
 * Authentication System Exports
 * Central export point for all authentication functionality
 */

// Client-side hooks and provider
export { 
  useAuth, 
  useAuthStatus, 
  useAdminAuth, 
  useSignIn, 
  useSignOut 
} from './hooks';

export { 
  AuthProvider,
  type User,
  type Session,
  type AuthContextType 
} from './provider';

// Server-side session management (only export types for client components)
export type { SessionData } from './session';

// Note: OAuth client functions should be imported directly from './client' in server components