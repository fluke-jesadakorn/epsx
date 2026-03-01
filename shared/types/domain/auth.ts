/**
 * CANONICAL AUTHENTICATION DOMAIN TYPES
 * Single source of truth for all authentication-related interfaces across EPSX
 * Consolidates OIDC, JWT, Web3, and Firebase authentication types
 */

import type { JWTPayload } from 'jose'
import type { EPSXPermission } from './permission'
import type { SessionData } from './session'
import type { AdminUserProfile, PackageTier, UserProfile, UserRole } from './user'

// Re-export core auth types from shared system
export type {
  AuthResponse, User
} from '../../types/auth'

// ============================================================================
// JWT TOKEN TYPES
// ============================================================================

/**
 * Base JWT payload for all EPSX tokens
 */
export interface BaseJWTPayload extends JWTPayload {
  sub: string
  email: string
  name: string
  iat: number
  exp: number
  iss: string
  aud: string
}

/**
 * User JWT payload for frontend application
 */
export interface UserJWTPayload extends BaseJWTPayload {
  token_type: 'user_access'
  role: UserRole
  permissions: {
    permissions: EPSXPermission[]
    package_tier: PackageTier
    expires_at?: number
  }
  user_context: {
    package_tier: PackageTier
    wallet_address?: string
    platform_preferences: string[]
  }
  platforms: string[]
  primary_platform: string
}

/**
 * Admin JWT payload for admin application
 */
export interface AdminJWTPayload extends BaseJWTPayload {
  token_type: 'admin_access'
  role: 'admin' | 'super_admin'
  permissions: {
    system_access: {
      capabilities: EPSXPermission[]
      level: 'admin' | 'super_admin'
    }
  }
  admin_context: {
    managed_platforms: string[]
    access_level: string
    granted_by?: string
    granted_at: number
  }
}

/**
 * Refresh token payload
 */
export interface RefreshTokenPayload extends BaseJWTPayload {
  token_type: 'refresh'
  sid: string
  device_id?: string
}

/**
 * API key token payload
 */
export interface APIKeyPayload extends BaseJWTPayload {
  token_type: 'api_key'
  key_id: string
  permissions: EPSXPermission[]
  rate_limits: {
    requests_per_minute: number
    requests_per_hour: number
    requests_per_day: number
  }
}

/**
 * Union type for all JWT payloads
 */
export type EPSXJWTPayload = UserJWTPayload | AdminJWTPayload | RefreshTokenPayload | APIKeyPayload

// ============================================================================
// AUTHENTICATION REQUEST/RESPONSE TYPES
// ============================================================================

/**
 * Login request for email/password authentication
 */
export interface LoginRequest {
  email: string
  password: string
  remember_me?: boolean
  device_info?: {
    user_agent: string
    ip_address: string
    device_name?: string
  }
}

/**
 * Login response with tokens and user data
 */
export interface LoginResponse {
  success: boolean
  user?: UserProfile | AdminUserProfile
  tokens?: {
    access_token: string
    refresh_token: string
    id_token?: string
    expires_at: number
  }
  session?: SessionData
  requires_2fa?: boolean
  error?: string
}

/**
 * OAuth/OIDC authorization request
 */
export interface OAuthAuthorizationRequest {
  client_id: string
  redirect_uri: string
  scope: string
  state: string
  code_challenge?: string
  code_challenge_method?: 'S256'
  response_type: 'code'
  app_type: 'user' | 'admin'
}

/**
 * OAuth/OIDC token exchange request
 */
export interface OAuthTokenRequest {
  grant_type: 'authorization_code' | 'refresh_token'
  client_id: string
  client_secret?: string
  code?: string
  refresh_token?: string
  redirect_uri?: string
  code_verifier?: string
}

/**
 * OAuth/OIDC token response
 */
export interface OAuthTokenResponse {
  access_token: string
  token_type: 'Bearer'
  expires_in: number
  refresh_token?: string
  id_token?: string
  scope?: string
}

/**
 * OIDC UserInfo response
 */
export interface OIDCUserInfo {
  sub: string
  email: string
  email_verified: boolean
  name: string
  given_name?: string
  family_name?: string
  picture?: string
  locale?: string
  updated_at?: number
}

// ============================================================================
// WEB3 AUTHENTICATION TYPES
// ============================================================================

/**
 * Web3 wallet connection request
 */
export interface Web3ConnectRequest {
  wallet_type: 'metamask' | 'walletconnect' | 'coinbase' | 'other'
  chain_id?: number
  account?: string
}

/**
 * Web3 authentication challenge
 */
export interface Web3AuthChallenge {
  nonce: string
  message: string
  expires_at: number
  challenge_id: string
}

/**
 * SIWE (Sign-In with Ethereum) message format
 */
export interface SIWEMessage {
  domain: string
  address: string
  statement: string
  uri: string
  version: string
  chainId: number
  nonce: string
  issuedAt: string
  expirationTime?: string
  notBefore?: string
  requestId?: string
  resources?: string[]
}

/**
 * Web3 authentication verification
 */
export interface Web3AuthVerification {
  wallet_address: string
  signature: string
  message: string
  nonce: string
  chain_id?: number
}

/**
 * Web3 authentication response
 */
export interface Web3AuthResponse {
  success: boolean
  user?: UserProfile
  tokens?: {
    access_token: string
    refresh_token: string
    expires_at: number
  }
  wallet_data?: {
    address: string
    chain_id: number
    permissions: EPSXPermission[]
  }
  error?: string
}

// ============================================================================
// MULTI-FACTOR AUTHENTICATION TYPES
// ============================================================================

/**
 * 2FA method types
 */
export type TwoFactorMethod = 'totp' | 'sms' | 'email' | 'backup_codes'

/**
 * 2FA setup request
 */
export interface TwoFactorSetupRequest {
  method: TwoFactorMethod
  phone_number?: string
}

/**
 * 2FA setup response
 */
export interface TwoFactorSetupResponse {
  success: boolean
  method: TwoFactorMethod
  secret?: string // For TOTP
  qr_code?: string // For TOTP
  backup_codes?: string[]
  error?: string
}

/**
 * 2FA verification request
 */
export interface TwoFactorVerificationRequest {
  method: TwoFactorMethod
  code: string
}

/**
 * 2FA verification response
 */
export interface TwoFactorVerificationResponse {
  success: boolean
  tokens?: {
    access_token: string
    refresh_token: string
    expires_at: number
  }
  error?: string
}

// ============================================================================
// PASSWORD & ACCOUNT MANAGEMENT TYPES
// ============================================================================

/**
 * Password reset request
 */
export interface PasswordResetRequest {
  email: string
}

/**
 * Password reset confirmation
 */
export interface PasswordResetConfirmation {
  token: string
  new_password: string
}

/**
 * Password change request
 */
export interface PasswordChangeRequest {
  current_password: string
  new_password: string
}

/**
 * Email verification request
 */
export interface EmailVerificationRequest {
  email: string
}

/**
 * Email verification confirmation
 */
export interface EmailVerificationConfirmation {
  token: string
}

/**
 * Account registration request
 */
export interface RegistrationRequest {
  email: string
  password: string
  name: string
  terms_accepted: boolean
  marketing_consent?: boolean
  referral_code?: string
}

/**
 * Account registration response
 */
export interface RegistrationResponse {
  success: boolean
  user?: UserProfile
  tokens?: {
    access_token: string
    refresh_token: string
    expires_at: number
  }
  requires_email_verification?: boolean
  error?: string
}

// ============================================================================
// AUTHENTICATION STATE TYPES
// ============================================================================

/**
 * Authentication state for client applications
 */
export interface AuthenticationState {
  isAuthenticated: boolean
  isLoading: boolean
  user: UserProfile | AdminUserProfile | null
  session: SessionData | null
  tokens: {
    access_token: string | null
    refresh_token: string | null
    expires_at: number | null
  }
  permissions: EPSXPermission[]
  error: string | null
  lastRefresh: number | null
}

/**
 * Authentication context for React providers
 */
export interface AuthenticationContext extends AuthenticationState {
  login: (credentials: LoginRequest) => Promise<LoginResponse>
  logout: () => Promise<void>
  refreshSession: () => Promise<boolean>
  connectWallet: (request: Web3ConnectRequest) => Promise<Web3AuthResponse>
  disconnectWallet: () => Promise<void>
  updatePermissions: () => Promise<void>
}

// ============================================================================
// AUTHENTICATION CONFIGURATION TYPES
// ============================================================================

/**
 * OIDC client configuration
 */
export interface OIDCClientConfig {
  client_id: string
  client_secret?: string
  redirect_uri: string
  scope: string
  response_type: 'code'
  code_challenge_method: 'S256'
  issuer: string
  authorization_endpoint: string
  token_endpoint: string
  userinfo_endpoint: string
  jwks_uri: string
}

/**
 * Authentication provider configuration
 */
export interface AuthProviderConfig {
  oidc: OIDCClientConfig
  web3: {
    supported_chains: number[]
    wallet_connect_project_id: string
    default_chain_id: number
  }
  session: {
    duration: number
    refresh_threshold: number
    max_concurrent_sessions: number
  }
}

// ============================================================================
// TYPE GUARDS & HELPERS
// ============================================================================

/**
 * Check if JWT payload is for user
 */
export function isUserJWT(payload: EPSXJWTPayload): payload is UserJWTPayload {
  return payload.token_type === 'user_access'
}

/**
 * Check if JWT payload is for admin
 */
export function isAdminJWT(payload: EPSXJWTPayload): payload is AdminJWTPayload {
  return payload.token_type === 'admin_access'
}

/**
 * Check if JWT payload is refresh token
 */
export function isRefreshToken(payload: EPSXJWTPayload): payload is RefreshTokenPayload {
  return payload.token_type === 'refresh'
}

/**
 * Check if JWT payload is API key
 */
export function isAPIKeyToken(payload: EPSXJWTPayload): payload is APIKeyPayload {
  return payload.token_type === 'api_key'
}

/**
 * Extract user ID from any JWT payload
 */
export function extractUserId(payload: EPSXJWTPayload): string {
  return payload.sub
}

/**
 * Check if token is expired
 */
export function isTokenExpired(payload: EPSXJWTPayload): boolean {
  return Date.now() / 1000 > payload.exp
}

/**
 * Check if token expires soon (within 5 minutes)
 */
export function isTokenExpiringSoon(payload: EPSXJWTPayload, thresholdSeconds = 300): boolean {
  return Date.now() / 1000 > (payload.exp - thresholdSeconds)
}

/**
 * Get token time remaining in seconds
 */
export function getTokenTimeRemaining(payload: EPSXJWTPayload): number {
  return Math.max(0, payload.exp - Math.floor(Date.now() / 1000))
}

// ============================================================================
// LEGACY COMPATIBILITY ALIASES
// ============================================================================

/** @deprecated Use UserJWTPayload instead */
export type UserClaims = UserJWTPayload

/** @deprecated Use AdminJWTPayload instead */
export type AdminClaims = AdminJWTPayload

/** @deprecated Use AuthenticationState instead */
export type AuthState = AuthenticationState

/** @deprecated Use LoginResponse instead */
export type AuthResult = LoginResponse