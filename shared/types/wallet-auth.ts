/**
 * Comprehensive Wallet-Based Authentication Types
 * Matches the backend UnifiedWeb3AuthService and UnifiedWeb3PermissionService
 */

// ============================================================================
// CORE WALLET AUTH TYPES (Backend Compatible)
// ============================================================================

/**
 * Wallet-based user (replaces email-based User)
 */
export interface WalletUser {
  wallet_address: string;
  user_id?: string; // Optional, often same as wallet_address
  permissions: string[]; // Structured permissions: "platform:resource:action"
  tier?: string; // user's access tier (basic, premium, etc.)
  expires_at?: number; // Session expiration timestamp
}

/**
 * Enhanced permission information with metadata and expiry
 */
export interface PermissionInfo {
  permission: string;
  permission_type: string;
  is_active: boolean;
  expires_at?: string;
  granted_at: string;
  last_verified_at?: string;
  verification_data?: Record<string, unknown>;
}

/**
 * Permission statistics for wallet
 */
export interface PermissionStats {
  total_permissions: number;
  permanent_permissions: number;
  temporary_permissions: number;
  expired_permissions: number;
}

/**
 * Group membership information
 */
export interface GroupMembership {
  id: string;
  plan_id: string;
  group_name: string;
  group_type: string;
  assigned_at: string;
  expires_at?: string;
  is_active: boolean;
  assignment_source: string;
  assignment_reason?: string;
  assigned_by?: string;
  payment_reference?: string;
  subscription_id?: string;
  auto_renew: boolean;
  next_billing_date?: string;
}

/**
 * Permission types
 */
export enum PermissionType {
  Manual = 'Manual',
  NFT = 'NFT',
  Token = 'Token',
  DAO = 'DAO',
}

/**
 * @deprecated Use PermissionType instead
 */
export enum Web3PermissionType {
  Manual = 'Manual',
  NFT = 'NFT',
  Token = 'Token',
  DAO = 'DAO',
}

/**
 * Permission with verification data
 */
export interface Permission {
  permission: string;
  permission_type: PermissionType;
  is_active: boolean;
  expires_at?: string;
  granted_at: string;
  verification_data?: Record<string, unknown>;
}

/**
 * @deprecated Use Permission instead
 */
export interface Web3Permission {
  permission: string;
  permission_type: Web3PermissionType;
  is_active: boolean;
  expires_at?: string;
  granted_at: string;
  verification_data?: Record<string, unknown>;
}

/**
 * Enhanced wallet authentication state with comprehensive features
 */
export interface WalletAuthState {
  walletAddress: string | null;
  isConnected: boolean;
  isAuthenticated: boolean;
  isLoading: boolean;
  user: WalletUser | null;
  permissions: string[];
  userTier: string | null;
  error: string | null;
  expiresAt: number | null;

  // Enhanced state
  permissionInfo: PermissionInfo[];
  groupMemberships: GroupMembership[];
  permissionStats?: PermissionStats;
  web3Permissions: Web3Permission[]; // @deprecated Use permissions instead
  detailedPermissions: Permission[];

  // Actions
  connect: () => Promise<void>;
  disconnect: () => Promise<void>;
  authenticate: (
    signature: string,
    message: string,
    nonce: string
  ) => Promise<void>;
  logout: () => Promise<void>;

  // Permission checks (simplified)
  can: (permission: string) => boolean;
  hasPermission: (permission: string) => boolean;

  // Enhanced permission actions
  refreshPermissions: () => Promise<void>;
  getPermissionStats: () => Promise<PermissionStats>;
  getGroupMemberships: () => Promise<GroupMembership[]>;
  checkPermissions: (permissions: string[]) => Promise<Record<string, boolean>>;
}

/**
 * Admin wallet authentication state
 */
export interface AdminWalletAuthState extends WalletAuthState {
  isAdmin: () => boolean;
  hasAdminPermission: (permission: string) => boolean;
}

// ============================================================================
// WEB3 AUTH REQUEST/RESPONSE TYPES (Backend Compatible)
// ============================================================================

/**
 * Web3 challenge request
 */
export interface WalletChallengeRequest {
  wallet_address: string;
}

/**
 * Web3 challenge response (from backend)
 */
export interface WalletChallengeResponse {
  nonce: string;
  message: string;
}

/**
 * Web3 signature verification request
 */
export interface WalletVerificationRequest {
  wallet_address: string;
  signature: string;
  message: string;
  nonce: string;
}

/**
 * Web3 authentication response (from backend) - Enhanced
 */
export interface WalletAuthResponse {
  success: boolean;
  wallet_address?: string;
  user_id?: string;
  permissions?: string[];
  tier?: string;
  expires_at?: number;
  access_token?: string;
  refresh_token?: string;
  is_new_user?: boolean;
  error?: string;
}

/**
 * Wallet session data (from backend /api/auth/web3/session)
 */
export interface WalletSessionData {
  wallet_address: string;
  user_id?: string;
  permissions: string[];
  tier?: string;
  expires_at?: number;
  access_token?: string;
  refresh_token?: string;
}

/**
 * Wallet permissions response (from backend /api/auth/web3/permissions)
 */
export interface WalletPermissionsResponse {
  permissions: WalletPermissionEntry[];
  total_count?: number;
  limit?: number;
  offset?: number;
}

/**
 * Individual wallet permission entry
 */
export interface WalletPermissionEntry {
  id?: string;
  wallet_address: string;
  permission: string;
  source?: string;
  expires_at?: number;
  granted_at?: number;
  granted_by?: string;
  is_active?: boolean;
}

// ============================================================================
// BLOCKCHAIN CONFIGURATION TYPES (Backend Compatible)
// ============================================================================

/**
 * NFT-based permission configuration
 */
export interface NFTConfig {
  contract_address: string;
  network: string;
  permission: string;
  collection_name?: string;
  require_specific_token: boolean;
  specific_token_ids: string[];
  minimum_tokens: number;
  check_ownership_live: boolean;
}

/**
 * Token-based permission configuration
 */
export interface TokenConfig {
  contract_address: string;
  network: string;
  permission: string;
  token_name?: string;
  token_symbol?: string;
  minimum_balance: string;
  token_decimals: number;
  check_balance_live: boolean;
}

/**
 * DAO proposal for governance-based permissions
 */
export interface DAOProposal {
  dao_contract_address: string;
  network: string;
  proposal_id: string;
  title: string;
  description?: string;
  target_wallet_address: string;
  permission: string;
  proposal_status: string;
  voting_end?: string;
}

/**
 * Network configuration for blockchain interactions
 */
export interface NetworkConfig {
  ethereum_rpc_url: string;
  polygon_rpc_url: string;
  arbitrum_rpc_url: string;
  optimism_rpc_url: string;
  base_rpc_url: string;
  bsc_rpc_url: string;
}

/**
 * Permission delegation between wallets
 */
export interface PermissionDelegation {
  delegator: string;
  delegate: string;
  permission: string;
  signature: string;
  expires_at: string;
  delegation_depth: number;
  network: string;
  nonce: string;
}

/**
 * EIP-712 message structure for delegation
 */
export interface EIP712DelegationMessage {
  delegator: string;
  delegate: string;
  permission: string;
  expires_at: number;
  nonce: string;
  network: string;
}

// ============================================================================
// ENHANCED ERROR HANDLING TYPES (Backend Compatible)
// ============================================================================

/**
 * Comprehensive Web3 authentication error
 */
export interface Web3AuthError {
  error_type: string;
  message: string;
  user_message: string;
  details: {
    permission?: string;
    required_group?: string;
    current_group?: string;
    wallet_address?: string;
  };
  suggested_actions: string[];
  upgrade_info?: {
    current_group: string;
    required_group: string;
    upgrade_url?: string;
    benefits: string[];
  };
}

/**
 * Permission verification result
 */
export interface PermissionVerificationResult {
  wallet_address: string;
  permission: string;
  is_granted: boolean;
  verification_type: string;
  verification_data: Record<string, unknown>;
  cached_until?: string;
}

/**
 * Batch permission checking result
 */
export interface BatchPermissionResult {
  [permission: string]: boolean;
}

// ============================================================================
// CONDITIONAL PERMISSION LOGIC (Backend Compatible)
// ============================================================================

/**
 * Conditional permission logic for complex permission rules
 */
export type PermissionCondition =
  | { type: 'And'; conditions: PermissionCondition[] }
  | { type: 'Or'; conditions: PermissionCondition[] }
  | { type: 'Not'; condition: PermissionCondition }
  | {
    type: 'TokenBalance';
    contract: string;
    network: string;
    min_balance: string;
  }
  | {
    type: 'NFTOwnership';
    contract: string;
    network: string;
    token_id?: string;
  }
  | { type: 'DelegatedBy'; delegator: string }
  | { type: 'TimeWindow'; start: string; end: string };

// ============================================================================
// SIMPLIFIED AUTH UTILITIES
// ============================================================================

/**
 * Check if user has admin permissions
 */
export function hasAdminPermissions(permissions: string[]): boolean {
  return permissions.some(
    p =>
      p === 'admin:*:*' ||
      p.startsWith('admin:') ||
      p === 'epsx:admin:*' ||
      p === 'epsx:*:*'
  );
}

/**
 * Check if user has specific permission
 */
export function hasPermission(
  userPermissions: string[],
  requiredPermission: string
): boolean {
  return userPermissions.includes(requiredPermission);
}

/**
 * Extract platform from permission string

 * e.g., "admin:users:manage" -> "admin"
 */
export function getPermissionPlatform(permission: string): string {
  return permission.split(':')[0] || '';
}

/**
 * Extract resource from permission string
 * e.g., "admin:users:manage" -> "users"
 */
export function getPermissionResource(permission: string): string {
  return permission.split(':')[1] || '';
}

/**
 * Extract action from permission string
 * e.g., "admin:users:manage" -> "manage"
 */
export function getPermissionAction(permission: string): string {
  return permission.split(':')[2] || '';
}

/**
 * Check if session is expired
 */
export function isSessionExpired(expiresAt: number | null): boolean {
  if (expiresAt === null) { return true; }
  return Date.now() >= expiresAt;
}

/**
 * Check if session expires soon (within 5 minutes)
 */
export function isSessionExpiringSoon(
  expiresAt: number | null,
  thresholdMs = 300000
): boolean {
  if (expiresAt === null) { return true; }
  return Date.now() >= expiresAt - thresholdMs;
}

// ============================================================================
// FRONTEND SESSION TYPES (API Compatible)
// ============================================================================

/**
 * Frontend session API response format
 */
export interface FrontendSessionResponse {
  isAuthenticated: boolean;
  user?: {
    wallet_address: string;
    user_id?: string;
    permissions: string[];
    tier?: string;
    has_access?: boolean;
  };
  expiresAt?: number;
  error?: string;
}

/**
 * Admin session API response format
 */
export interface AdminSessionResponse {
  isAuthenticated: boolean;
  user?: {
    wallet_address: string;
    user_id?: string;
    permissions: string[];
    tier?: string;
    admin_level?: string;
    has_access?: boolean;
  };
  expiresAt?: number;
  error?: string;
}
