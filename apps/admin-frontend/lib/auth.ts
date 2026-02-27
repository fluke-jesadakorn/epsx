'use client'

/**
 * Web3 Admin Wallet Authentication
 * Pure Web3 wallet-based authentication for admin dashboard
 */

import { create } from 'zustand';

import { loginAction, logoutAction } from '@/shared/auth/actions';
import { createAdminClient, type UserInfoResponse } from '@/shared/auth/client';

// Web3 Admin Wallet interface (migrated from EnterpriseAdminUser)
export interface AdminWallet {
  wallet_address: string;
  /** user's role from backend: 'user', 'admin', or 'super_admin' */
  role: 'user' | 'admin' | 'super_admin';
  permissions: string[];
  has_api_access: boolean;
  verified_tokens_usd: number;
  nft_collections: string[];
  dao_memberships: string[];
  is_admin: boolean;
  admin_permissions: string[];
}

/**
 * Backend Web3 User Response
 */
interface ExtendedUserInfoResponse extends UserInfoResponse {
  is_admin?: boolean;
  admin_permissions?: string[];
  role?: string;
}

interface AdminAuthParams {
  walletAddress?: string;
  signature?: string;
  message?: string;
  nonce?: string;
}

// Web3 admin client instance
const adminWeb3Client = createAdminClient();

export interface Web3AdminAuthState {
  wallet: AdminWallet | null;
  isLoading: boolean;
  isAuthenticated: boolean;
  error: string | null;
  expiresAt: number | null;
  walletAddress?: string;
  isConnecting: boolean;
  isAuthenticating: boolean;
}

// Transform Web3 user to admin wallet format
// Uses backend-provided role directly - no client-side derivation
function transformWeb3UserToAdminWallet(web3User: ExtendedUserInfoResponse): AdminWallet {
  const permissions = web3User.permissions;
  // Use backend-provided is_admin flag
  const isAdmin = web3User.is_admin ?? permissions.some(p => p.startsWith('admin:'));
  // Use backend-provided admin_permissions
  const adminPermissions: string[] = web3User.admin_permissions ?? permissions.filter(p => p.startsWith('admin:'));
  // Use backend-provided role
  const role = (web3User.role ?? (isAdmin ? 'admin' : 'user')) as 'user' | 'admin' | 'super_admin';

  return {
    wallet_address: web3User.wallet_address,
    role,
    permissions,
    has_api_access: true,
    verified_tokens_usd: 0,
    nft_collections: [],
    dao_memberships: [],
    is_admin: isAdmin,
    admin_permissions: adminPermissions,
  };
}

type SetFn = (partial: Partial<Web3AdminAuthState>) => void;
type GetFn = () => Web3AdminAuthState & { getAdminWallet: () => Promise<AdminWallet | null>; disconnectWallet: () => Promise<void> };

async function connectWalletImpl(set: SetFn): Promise<void> {
  set({ isConnecting: true, error: null });
  try {
    if (adminWeb3Client.isAuthenticated()) {
      const web3User = await adminWeb3Client.loadCurrentUser() as ExtendedUserInfoResponse | null;
      if (web3User !== null) {
        const adminWallet = transformWeb3UserToAdminWallet(web3User);
        set({ wallet: adminWallet, walletAddress: adminWallet.wallet_address, isAuthenticated: true, isConnecting: false, expiresAt: Date.now() + 86400000 });
        return;
      }
    }
    window.dispatchEvent(new CustomEvent('epsx:admin-connect-wallet'));
    set({ isConnecting: false });
  } catch (_error) {
    set({ error: _error instanceof Error ? _error.message : 'Wallet connection failed', isConnecting: false });
  }
}

async function disconnectWalletImpl(set: SetFn): Promise<void> {
  set({ isLoading: true });
  try {
    try { await logoutAction(); } catch (_e) { /* silently fail */ }
    adminWeb3Client.logout();
    set({ wallet: null, isAuthenticated: false, expiresAt: null, walletAddress: undefined, isLoading: false });
  } catch (_error) {
    set({ error: 'Disconnect failed. Please try again.', isLoading: false });
  }
}

async function getAdminWalletImpl(set: SetFn, get: GetFn): Promise<AdminWallet | null> {
  const { wallet, expiresAt } = get();
  if (wallet !== null && expiresAt !== null && Date.now() < expiresAt) { return wallet; }
  try {
    const web3User = await adminWeb3Client.loadCurrentUser();
    if (web3User !== null) {
      const adminWallet = transformWeb3UserToAdminWallet(web3User as ExtendedUserInfoResponse);
      set({ wallet: adminWallet, isAuthenticated: true, expiresAt: Date.now() + 86400000 });
      return adminWallet;
    }
  } catch (_error) { /* silently fail */ }
  set({ wallet: null, isAuthenticated: false, expiresAt: null });
  return null;
}

async function authenticateAdminImpl(set: SetFn, get: GetFn, params?: AdminAuthParams): Promise<void> {
  set({ isAuthenticating: true, error: null });
  try {
    if (hasSignatureParams(params)) {
      const { adminWallet } = await doSignatureAuth(params);
      set({ wallet: adminWallet, walletAddress: adminWallet.wallet_address, isAuthenticated: true, isAuthenticating: false, expiresAt: Date.now() + 86400000 });
      return;
    }
    const { walletAddress: currentWalletAddress } = get();
    if (currentWalletAddress === undefined) {
      set({ error: 'Please connect wallet first', isAuthenticating: false });
      return;
    }
    window.dispatchEvent(new CustomEvent('epsx:admin-authenticate', { detail: { walletAddress: currentWalletAddress } }));
    set({ isAuthenticating: false });
  } catch (_error) {
    set({ error: _error instanceof Error ? _error.message : 'Authentication failed', isAuthenticating: false, isAuthenticated: false });
  }
}

function hasSignatureParams(params: AdminAuthParams | undefined): params is Required<AdminAuthParams> {
  return (
    params?.walletAddress !== undefined && params.walletAddress !== '' &&
    params.signature !== undefined && params.signature !== '' &&
    params.message !== undefined && params.message !== '' &&
    params.nonce !== undefined && params.nonce !== ''
  );
}

async function doSignatureAuth(params: Required<AdminAuthParams>): Promise<{ adminWallet: AdminWallet; access?: string }> {
  const result = await adminWeb3Client.authenticateWithSignature({
    wallet_address: params.walletAddress,
    signature: params.signature,
    message: params.message,
    nonce: params.nonce,
  });

  if (result.success !== true || result.user === undefined) {
    throw new Error(result.error ?? 'Authentication failed');
  }

  if (result.user.access !== undefined) {
    await loginAction(result.user.access, result.user);
  }

  return {
    adminWallet: transformWeb3UserToAdminWallet(result.user as ExtendedUserInfoResponse),
    access: result.user.access,
  };
}

// Create Web3 admin wallet auth store
export const useAuth = create<Web3AdminAuthState & {
  connectWallet: () => Promise<void>;
  authenticateAdmin: (params?: AdminAuthParams) => Promise<void>;
  requestAdminChallenge: (walletAddress: string) => Promise<{ nonce: string; message: string; wallet_address: string }>;
  disconnectWallet: () => Promise<void>;
  getAdminWallet: () => Promise<AdminWallet | null>;
  refreshSession: () => Promise<void>;
  clearError: () => void;
}>((set, get) => ({
  wallet: null,
  isLoading: false,
  isAuthenticated: false,
  error: null,
  expiresAt: null,
  walletAddress: undefined,
  isConnecting: false,
  isAuthenticating: false,
  connectWallet: () => connectWalletImpl(set),
  authenticateAdmin: (params) => authenticateAdminImpl(set, get as GetFn, params),
  requestAdminChallenge: (walletAddress) => adminWeb3Client.requestChallenge(walletAddress),
  disconnectWallet: () => disconnectWalletImpl(set),
  getAdminWallet: () => getAdminWalletImpl(set, get as GetFn),
  refreshSession: async () => {
    try {
      await getAdminWalletImpl(set, get as GetFn);
    } catch (_error) {
      void disconnectWalletImpl(set);
    }
  },
  clearError: () => { set({ error: null }); },
}));

// Subscribe to Web3 client changes
if (typeof window !== 'undefined') {
  // Subscribe to Web3 client user changes
  adminWeb3Client.subscribe((web3User) => {
    const _state = useAuth.getState();

    if (web3User !== null) {
      const adminWallet = transformWeb3UserToAdminWallet(web3User as ExtendedUserInfoResponse);

      // Server handling permission enforcement - we just display state
      useAuth.setState({
        wallet: adminWallet,
        walletAddress: adminWallet.wallet_address,
        isAuthenticated: true,
        expiresAt: Date.now() + (24 * 60 * 60 * 1000),
        error: null
      });
    } else {
      // Clear session if Web3 user is null
      useAuth.setState({
        wallet: null,
        walletAddress: undefined,
        isAuthenticated: false,
        expiresAt: null,
        error: null
      });
    }
  });

  // Auto-fetch admin user on app start
  void useAuth.getState().getAdminWallet();
}

// Utility Functions for Web3 Enterprise Admin UI
/**
 *
 * @param wallet
 */
export function getAdminDisplayName(wallet: AdminWallet | null): string {
  if (wallet === null) { return 'Unknown Admin'; }
  return wallet.wallet_address !== '' ?
    `${wallet.wallet_address.slice(0, 6)}...${wallet.wallet_address.slice(-4)}` :
    'Enterprise Admin';
}

/**
 *
 * @param permissions
 */
export function getEnterprisePermissionLabels(permissions: string[]): string[] {
  const permissionLabels: Record<string, string> = {
    // Enterprise Admin permissions
    // 'admin:*:*': 'Global Enterprise Administrator', // REMOVED: Strict separation
    'admin:enterprise:*:*': 'Enterprise Management',
    'admin:enterprise:users:manage': 'Enterprise User Management',
    'admin:enterprise:system:manage': 'Enterprise System Management',
    'admin:enterprise:analytics:view': 'Enterprise Analytics',
    'admin:enterprise:tiers:manage': 'Enterprise Tier Management',

    // DAO Management
    'admin:dao:*:*': 'Full DAO Management',
    'admin:dao:manage': 'DAO Administration',
    'admin:governance:manage': 'Governance Management',

    // Compliance Management
    'admin:compliance:*:*': 'Full Compliance Management',
    'admin:compliance:manage': 'Compliance Administration',
    'admin:kyc:manage': 'KYC Management',

    // Marketplace Management
    'admin:marketplace:*:*': 'Full Marketplace Management',
    'admin:marketplace:manage': 'Marketplace Administration',
    'admin:products:manage': 'Product Management',
  };

  return permissions.map(permission => {
    return permissionLabels[permission] ?? permission;
  });
}

// Enterprise tier utility functions
/**
 *
 * @param tier
 */
export function getEnterpriseTierDisplayName(tier: string): string {
  const tierNames: Record<string, string> = {
    'Starter': 'Starter ($1K+ tokens)',
    'Business': 'Business ($10K+ tokens)',
    'Enterprise': 'Enterprise ($100K+ tokens)',
    'Whale': 'Whale ($1M+ tokens)'
  };

  return tierNames[tier] ?? tier;
}

/**
 *
 * @param tier
 */
export function getEnterpriseTierIcon(tier: string): string {
  const tierIcons: Record<string, string> = {
    'Starter': '🚀',
    'Business': '💼',
    'Enterprise': '🏢',
    'Whale': '🐋'
  };

  return tierIcons[tier] ?? '⭐';
}

// Web3 wallet connection helper for admin
/**
 *
 */
export async function connectAdminWallet() {
  const { connectWallet } = useAuth.getState();
  await connectWallet();
}

// Web3 admin authentication helper
/**
 *
 */
export async function authenticateAdminWallet() {
  const { authenticateAdmin } = useAuth.getState();
  await authenticateAdmin();
}
