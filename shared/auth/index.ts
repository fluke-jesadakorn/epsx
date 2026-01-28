// ============================================================================
// SHARED AUTHENTICATION - CONSOLIDATED EXPORTS
// Single source of truth for all auth-related imports
// ============================================================================

// Core client
export { SharedWeb3AuthClient, createAdminClient, createFrontendClient } from './client';
export type {
  UnifiedApiResponse, UserInfoResponse, Web3AuthRequest,
  Web3ChallengeResponse, Web3TokenResponse
} from './client';

// Provider
export { SharedOpenIDWeb3Provider, useAuth, useSharedAuth } from '../components/auth/Provider';

// Server actions
export { loginAction, logoutAction } from './actions';

// Cookies
export {
  COOKIES,
  COOKIE_OPTIONS, clearClientSideCookies, getClientCookie, getClientCookieJSON, getServerAuthToken, isClientSideCookie, isHttpOnlyCookie, removeClientCookie, setClientCookie, setClientCookieJSON
} from './cookies';

// Middleware
export { createAuthMiddleware } from './middleware';
export type { AuthMiddlewareConfig } from './middleware';

// Direct API client (for wallet authentication without Bearer tokens)
export { authenticateWallet, directWeb3Api, requestWalletChallenge, verifyWalletSignature } from './api';
export type {
  ChallengeRequest,
  ChallengeResponse,
  SignatureVerificationRequest,
  SignatureVerificationResponse
} from './api';

