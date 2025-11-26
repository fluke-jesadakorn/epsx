// Web3 Authentication Types
// Mirrors backend UnifiedWeb3AuthService and UnifiedWeb3PermissionService types

// Core Web3 Authentication Types

export interface Web3Challenge {
  wallet_address: string;
  nonce: string;
  message: string;
  expires_at: string;
  created_at: string;
}

export interface Web3VerificationRequest {
  wallet_address: string;
  message: string;
  signature: string;
  nonce: string;
}

export interface Web3AuthResult {
  wallet_address: string;
  permissions: string[];
  access_token: string;
  is_new_user: boolean;
}

// Permission System Types

export interface PermissionInfo {
  permission: string;
  permission_type: string;
  is_active: boolean;
  expires_at?: string;
  granted_at: string;
  last_verified_at?: string;
  verification_data?: Record<string, any>;
}

export interface PermissionStats {
  total_permissions: number;
  permanent_permissions: number;
  temporary_permissions: number;
  expired_permissions: number;
}

export interface GroupMembership {
  id: string;
  group_id: string;
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

// Web3 Permission Types

export enum Web3PermissionType {
  Manual = 'Manual',
  NFT = 'NFT',
  Token = 'Token',
  DAO = 'DAO',
}

export interface Web3Permission {
  permission: string;
  permission_type: Web3PermissionType;
  is_active: boolean;
  expires_at?: string;
  granted_at: string;
  verification_data?: Record<string, any>;
}

// Blockchain Configuration Types

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

// Permission Delegation Types

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

export interface EIP712DelegationMessage {
  delegator: string;
  delegate: string;
  permission: string;
  expires_at: number;
  nonce: string;
  network: string;
}

// Error Types

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

// Auth State Types

export interface Web3AuthState {
  isConnected: boolean;
  isAuthenticated: boolean;
  isLoading: boolean;
  wallet_address?: string;
  permissions: string[];
  group_memberships: GroupMembership[];
  permission_stats?: PermissionStats;
  error?: Web3AuthError;
}

// API Response Types

export interface Web3ChallengeResponse {
  success: boolean;
  data?: Web3Challenge;
  error?: Web3AuthError;
}

export interface Web3VerifyResponse {
  success: boolean;
  data?: Web3AuthResult;
  error?: Web3AuthError;
}

export interface Web3PermissionsResponse {
  success: boolean;
  data?: PermissionInfo[];
  error?: Web3AuthError;
}

export interface Web3GroupMembershipsResponse {
  success: boolean;
  data?: GroupMembership[];
  error?: Web3AuthError;
}

// Permission Verification Types

export interface PermissionVerificationResult {
  wallet_address: string;
  permission: string;
  is_granted: boolean;
  verification_type: string;
  verification_data: Record<string, any>;
  cached_until?: string;
}

export interface BatchPermissionResult {
  [permission: string]: boolean;
}

export interface BatchPermissionResponse {
  success: boolean;
  data?: BatchPermissionResult;
  error?: Web3AuthError;
}

// Network Configuration

export interface NetworkConfig {
  ethereum_rpc_url: string;
  polygon_rpc_url: string;
  arbitrum_rpc_url: string;
  optimism_rpc_url: string;
  base_rpc_url: string;
  bsc_rpc_url: string;
}

// Conditional Permission Logic

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

// Tier/Plan Integration

export interface TierInfo {
  current_tier: string;
  available_tiers: string[];
  tier_permissions: Record<string, string[]>;
  upgrade_urls: Record<string, string>;
}

export interface PlanInfo {
  current_plan?: string;
  available_plans: string[];
  plan_permissions: Record<string, string[]>;
  plan_pricing: Record<string, number>;
  subscription_status?: string;
  next_billing_date?: string;
}

// Wallet Connection Types

export interface WalletConnectionState {
  isConnecting: boolean;
  isReconnecting: boolean;
  connector?: any; // WAGMI connector type
  chain?: any; // WAGMI chain type
  error?: Error;
}

// Combined Auth Context Type

export interface Web3AuthContextType {
  // Authentication State
  authState: Web3AuthState;
  walletState: WalletConnectionState;

  // Authentication Actions
  connectWallet: () => Promise<void>;
  disconnectWallet: () => Promise<void>;
  authenticate: () => Promise<Web3AuthResult>;
  logout: () => Promise<void>;

  // Permission Actions
  checkPermission: (permission: string) => Promise<boolean>;
  checkPermissions: (permissions: string[]) => Promise<BatchPermissionResult>;
  refreshPermissions: () => Promise<void>;

  // Data Fetching
  getGroupMemberships: () => Promise<GroupMembership[]>;
  getPermissionStats: () => Promise<PermissionStats>;

  // Utility
  hasPermission: (permission: string) => boolean;
  isAdmin: () => boolean;
  getCurrentTier: () => string;
}
