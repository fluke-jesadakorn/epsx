// ============================================================================
// SHARED AUTHENTICATION - CONSOLIDATED EXPORTS
// Single source of truth for all auth-related imports
// ============================================================================

// Core client
export { SharedWeb3AuthClient, createFrontendClient, createAdminClient } from './client';
export type {
  Web3TokenResponse,
  Web3AuthRequest,
  Web3ChallengeResponse,
  UserInfoResponse,
  UnifiedApiResponse,
} from './client';

// Provider
export { SharedOpenIDWeb3Provider, useSharedAuth, useAuth } from '../components/auth/Provider';

// Server actions
export { loginAction, logoutAction } from './actions';

// Cookies
export {
  COOKIES,
  COOKIE_OPTIONS,
  setClientCookie,
  getClientCookie,
  removeClientCookie,
  setClientCookieJSON,
  getClientCookieJSON,
  clearClientSideCookies,
  isHttpOnlyCookie,
  isClientSideCookie,
  getServerAuthToken,
} from './cookies';

// Middleware
export { createAuthMiddleware, AuthMiddlewareConfig } from './middleware';

// Direct API client (for wallet authentication without Bearer tokens)
export { directWeb3Api, requestWalletChallenge, verifyWalletSignature, authenticateWallet } from './api';
export type {
  ChallengeRequest,
  ChallengeResponse,
  SignatureVerificationRequest,
  SignatureVerificationResponse,
} from './api';
