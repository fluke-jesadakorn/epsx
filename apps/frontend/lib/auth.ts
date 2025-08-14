/**
 * Frontend Custom Authentication System
 * Replacement for Auth.js with custom OAuth 2.0 implementation
 */

export { OIDC_CONFIG, OAUTH_ENDPOINTS } from './auth/client';
export { SessionData, getSession, saveSession, clearSession } from './auth/session';
export { 
  AuthProvider, 
  useAuth, 
  useSignIn, 
  useSignOut, 
  useUser, 
  useSession,
  usePermissions,
  usePackageTier
} from './auth/hooks';

// Legacy compatibility exports for existing components
export const signIn = () => {
  window.location.href = '/api/auth/signin';
};

export const signOut = async () => {
  await fetch('/api/auth/signout', { method: 'POST' });
  window.location.href = '/login';
};

// Server-side auth function for middleware compatibility
export const auth = async () => {
  try {
    const session = await getSession();
    return session.isLoggedIn ? { user: session.user } : null;
  } catch (error) {
    console.error('Auth function error:', error);
    return null;
  }
};