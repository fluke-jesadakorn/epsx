'use client'

/**
 * Web3 Admin Wallet Authentication
 * Pure Web3 wallet-based authentication for admin dashboard
 */

import { create } from 'zustand';

import { createAdminClient, UserInfoResponse } from '@/shared/auth/client';

// Web3 Admin Wallet interface (migrated from EnterpriseAdminUser)
export interface AdminWallet {
  wallet_address: string;
  /** User's role from backend: 'user', 'admin', or 'super_admin' */
  role: 'user' | 'admin' | 'super_admin';
  permissions: string[];
  has_api_access: boolean;
  verified_tokens_usd: number;
  nft_collections: string[];
  dao_memberships: string[];
  is_admin: boolean;
  admin_permissions: string[];
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
function transformWeb3UserToAdminWallet(web3User: UserInfoResponse): AdminWallet {
  const permissions = web3User.permissions || [];
  // Use backend-provided is_admin flag
  const isAdmin = (web3User as any).is_admin ?? permissions.some(p => p.startsWith('admin:'));
  // Use backend-provided admin_permissions
  const adminPermissions: string[] = (web3User as any).admin_permissions || permissions.filter(p => p.startsWith('admin:'));
  // Use backend-provided role
  const role = ((web3User as any).role || (isAdmin ? 'admin' : 'user')) as 'user' | 'admin' | 'super_admin';

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

// Create Web3 admin wallet auth store
export const useAuth = create<Web3AdminAuthState & {
  // Web3 Admin Actions
  connectWallet: () => Promise<void>;
  authenticateAdmin: (walletAddress?: string, signature?: string, message?: string, nonce?: string) => Promise<void>;
  requestAdminChallenge: (walletAddress: string) => Promise<{ nonce: string; message: string; wallet_address: string }>;
  disconnectWallet: () => Promise<void>;
  getAdminWallet: () => Promise<AdminWallet | null>;
  refreshSession: () => Promise<void>;
  clearError: () => void;
  // Permission enforcement handled by backend - no local methods needed
}>((set, get) => ({
  wallet: null,
  isLoading: false,
  isAuthenticated: false,
  error: null,
  expiresAt: null,
  walletAddress: undefined,
  isConnecting: false,
  isAuthenticating: false,

  // Connect wallet for admin authentication
  connectWallet: async () => {
    set({ isConnecting: true, error: null });

    try {

      // Check if already authenticated
      if (adminWeb3Client.isAuthenticated()) {
        const web3User = await adminWeb3Client.loadCurrentUser();
        if (web3User) {
          const adminWallet = transformWeb3UserToAdminWallet(web3User);

          // Check admin permissions
          if (!adminWallet.is_admin) {
            throw new Error('Insufficient admin permissions');
          }

          set({
            wallet: adminWallet,
            walletAddress: adminWallet.wallet_address,
            isAuthenticated: true,
            isConnecting: false,
            expiresAt: Date.now() + (24 * 60 * 60 * 1000) // 24 hours
          });

          return;
        }
      }

      // Trigger wallet connection event for Web3 components
      const walletConnectEvent = new CustomEvent('epsx:admin-connect-wallet');
      window.dispatchEvent(walletConnectEvent);

      set({ isConnecting: false });

    } catch (_error) {

      console.error('❌ Admin: Wallet connection failed:', _error);
      set({
        error: _error instanceof Error ? _error.message : 'Wallet connection failed',
        isConnecting: false
      });
    }
  },

  // Authenticate admin with Web3 signature (called by Web3 components)
  authenticateAdmin: async (walletAddress?: string, signature?: string, message?: string, nonce?: string) => {
    set({ isAuthenticating: true, error: null });

    try {

      // If signature provided, use it directly
      if (walletAddress && signature && message && nonce) {
        const result = await adminWeb3Client.authenticateWithSignature({
          wallet_address: walletAddress,
          signature,
          message,
          nonce
        });

        if (!result.success || !result.user) {
          throw new Error(result.error || 'Authentication failed');
        }

        const adminWallet = transformWeb3UserToAdminWallet(result.user);

        // Check admin permissions
        if (!adminWallet.is_admin) {
          throw new Error('Insufficient admin permissions for admin dashboard');
        }

        set({
          wallet: adminWallet,
          walletAddress: adminWallet.wallet_address,
          isAuthenticated: true,
          isAuthenticating: false,
          expiresAt: Date.now() + (24 * 60 * 60 * 1000), // 24 hours
        });

        return;
      }

      // Otherwise, trigger auth flow
      const { walletAddress: currentWalletAddress } = get();
      if (!currentWalletAddress) {
        set({ error: 'Please connect wallet first', isAuthenticating: false });
        return;
      }

      // Trigger authentication event for Web3 components to handle
      const authEvent = new CustomEvent('epsx:admin-authenticate', {
        detail: { walletAddress: currentWalletAddress }
      });
      window.dispatchEvent(authEvent);

      set({ isAuthenticating: false });

    } catch (_error) {

      console.error('❌ Admin: Authentication failed:', _error);
      set({
        error: _error instanceof Error ? _error.message : 'Authentication failed',
        isAuthenticating: false,
        isAuthenticated: false,
      });
    }
  },

  // Request Web3 challenge for admin authentication
  requestAdminChallenge: async (walletAddress: string) => {
    try {

      return await adminWeb3Client.requestChallenge(walletAddress);

    } catch (_error) {

      console.error('❌ Admin: Challenge request failed:', _error);
      throw _error;
    }
  },

  // Disconnect wallet and clear session
  disconnectWallet: async () => {
    set({ isLoading: true });

    try {
      // Use Web3 client logout
      await adminWeb3Client.logout();

      set({
        wallet: null,
        isAuthenticated: false,
        expiresAt: null,
        walletAddress: undefined,
        isLoading: false,
      });

    } catch (_error) {

      console.error('❌ Admin: Disconnect failed:', _error);
      set({
        error: 'Disconnect failed. Please try again.',
        isLoading: false
      });
    }
  },

  // Get current admin wallet (NEW)
  getAdminWallet: async () => {
    const { wallet, expiresAt } = get();
    if (wallet && expiresAt && Date.now() < expiresAt) {
      return wallet;
    }

    // Try to load from Web3 client
    try {
      const web3User = await adminWeb3Client.loadCurrentUser();
      if (web3User) {
        const adminWallet = transformWeb3UserToAdminWallet(web3User);

        // Verify admin permissions
        if (!adminWallet.is_admin) {
          set({
            wallet: null,
            isAuthenticated: false,
            error: 'Insufficient admin permissions'
          });
          return null;
        }

        set({
          wallet: adminWallet,
          isAuthenticated: true,
          expiresAt: Date.now() + (24 * 60 * 60 * 1000)
        });

        return adminWallet;
      }
    } catch (_error) {

      console.error('❌ Failed to load admin wallet:', _error);
    }

    // Clear session if unable to load
    set({
      wallet: null,
      isAuthenticated: false,
      expiresAt: null
    });

    return null;
  },

  // Refresh admin session
  refreshSession: async () => {
    try {
      await get().getAdminWallet();
    } catch (_error) {

      console.error('❌ Admin session refresh failed:', _error);
      get().disconnectWallet();
    }
  },

  // Clear error state
  clearError: () => {
    set({ error: null });
  },

  // ============================================================================
  // DEPRECATED: Permission helpers
  // ============================================================================
  // PERMISSION ENFORCEMENT - HANDLED BY BACKEND
  // All permission checks are done server-side via JWT middleware
  // Frontend only displays role/permissions but does not enforce them
  // ============================================================================
}));

// Subscribe to Web3 client changes
if (typeof window !== 'undefined') {
  // Subscribe to Web3 client user changes
  adminWeb3Client.subscribe((web3User) => {
    const state = useAuth.getState();

    if (web3User) {
      const adminWallet = transformWeb3UserToAdminWallet(web3User);

      // Only update if user has admin permissions
      if (adminWallet.is_admin) {
        useAuth.setState({
          wallet: adminWallet,
          walletAddress: adminWallet.wallet_address,
          isAuthenticated: true,
          expiresAt: Date.now() + (24 * 60 * 60 * 1000),
          error: null
        });
      } else {
        // Clear session if no admin permissions
        useAuth.setState({
          wallet: null,
          walletAddress: undefined,
          isAuthenticated: false,
          expiresAt: null,
          error: 'Insufficient admin permissions'
        });
      }
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
  useAuth.getState().getAdminWallet();
}

// Utility Functions for Web3 Enterprise Admin UI
/**
 *
 * @param user
 * @param wallet
 */
export function getAdminDisplayName(wallet: AdminWallet | null): string {
  if (!wallet) { return 'Unknown Admin'; }
  return wallet.wallet_address ?
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
    'admin:*:*': 'Global Enterprise Administrator',
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
    return permissionLabels[permission] || permission;
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

  return tierNames[tier] || tier;
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

  return tierIcons[tier] || '⭐';
}

// Web3 wallet connection helper for admin
/**
 *
 */
export async function connectAdminWallet() {
  const { connectWallet } = useAuth.getState();
  return await connectWallet();
}

// Web3 admin authentication helper
/**
 *
 */
export async function authenticateAdminWallet() {
  const { authenticateAdmin } = useAuth.getState();
  return await authenticateAdmin();
}
