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
  permission_group: 'Basic Access Group' | 'Standard Access Group' | 'Premium Access Group' | 'Professional Access Group' | 'Enterprise Access Group';
  enterprise_tier?: 'Starter' | 'Business' | 'Enterprise' | 'Whale'; // @deprecated Use permission_group instead
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

// Backward compatibility alias
export type EnterpriseAdminUser = AdminWallet;

export interface Web3AdminAuthState {
  wallet: AdminWallet | null;
  user: EnterpriseAdminUser | null; // @deprecated Use wallet instead
  isLoading: boolean;
  isAuthenticated: boolean;
  error: string | null;
  expiresAt: number | null;
  walletAddress?: string;
  isConnecting: boolean;
  isAuthenticating: boolean;
}

// Transform Web3 user to admin wallet format
// TODO: Backend should send permission_group directly, not derive on client
function transformWeb3UserToAdminWallet(web3User: UserInfoResponse): AdminWallet {
  const permissions = web3User.permissions || [];
  const isAdmin = permissions.some(p => p.startsWith('admin:'));

  return {
    wallet_address: web3User.wallet_address,
    permission_group: 'Basic Access Group', // TODO: Get from backend
    enterprise_tier: 'Starter', // @deprecated
    permissions,
    has_api_access: true, // Admin users have API access
    verified_tokens_usd: 0, // Could be derived from permissions or separate API
    nft_collections: [],
    dao_memberships: [],
    is_admin: isAdmin,
    admin_permissions: permissions.filter(p => p.startsWith('admin:')),
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
  getAdminUser: () => Promise<EnterpriseAdminUser | null>; // @deprecated Use getAdminWallet instead
  refreshSession: () => Promise<void>;
  clearError: () => void;
  // Permission helpers
  can: (permission: string) => boolean;
  hasAnyPermission: (permissions: string[]) => boolean;
  hasAllPermissions: (permissions: string[]) => boolean;
  hasMinimumPermissionGroup: (requiredGroup: string) => boolean;
  hasEnterpriseTier: (tier: 'Starter' | 'Business' | 'Enterprise' | 'Whale') => boolean;
  isAdmin: () => boolean;
  canManageUsers: () => boolean;
  canManageSystem: () => boolean;
  canViewAnalytics: () => boolean;
}>((set, get) => ({
  wallet: null,
  user: null, // @deprecated Use wallet instead
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
            user: adminWallet, // For backward compatibility
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
      // eslint-disable-next-line no-console
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
          user: adminWallet, // For backward compatibility
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
      // eslint-disable-next-line no-console
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
      // eslint-disable-next-line no-console
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
        user: null,
        isAuthenticated: false,
        expiresAt: null,
        walletAddress: undefined,
        isLoading: false,
        error: null,
      });

      window.location.href = '/auth';
    } catch (_error) {
      // eslint-disable-next-line no-console
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
            user: null,
            isAuthenticated: false,
            error: 'Insufficient admin permissions'
          });
          return null;
        }

        set({
          wallet: adminWallet,
          user: adminWallet,
          isAuthenticated: true,
          expiresAt: Date.now() + (24 * 60 * 60 * 1000)
        });

        return adminWallet;
      }
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('❌ Failed to load admin wallet:', _error);
    }

    // Clear session if unable to load
    set({
      wallet: null,
      user: null,
      isAuthenticated: false,
      expiresAt: null
    });

    return null;
  },

  // Get current admin user (@deprecated - use getAdminWallet)
  getAdminUser: async () => {
    // Delegate to getAdminWallet for Web3-first approach
    return await get().getAdminWallet(); // AdminWallet and EnterpriseAdminUser are type-compatible
  },

  // Refresh admin session
  refreshSession: async () => {
    try {
      await get().getAdminWallet();
    } catch (_error) {
      // eslint-disable-next-line no-console
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
  // These no longer do any real validation - backend is the single source of truth
  // Kept for backward compatibility with UI components that conditionally show elements
  // All actual permission checks happen in backend via JWT middleware
  // ============================================================================

  /** @deprecated Backend handles permission enforcement. This always returns true. */
  can: (_permission: string) => {
    console.warn('[DEPRECATED] can() - Permission enforcement moved to backend. This always returns true.');
    return true;
  },

  /** @deprecated Backend handles permission enforcement. This always returns true. */
  hasAnyPermission: (_permissions: string[]) => {
    console.warn('[DEPRECATED] hasAnyPermission() - Permission enforcement moved to backend. This always returns true.');
    return true;
  },

  /** @deprecated Backend handles permission enforcement. This always returns true. */
  hasAllPermissions: (_permissions: string[]) => {
    console.warn('[DEPRECATED] hasAllPermissions() - Permission enforcement moved to backend. This always returns true.');
    return true;
  },

  /** @deprecated Backend handles permission enforcement. This always returns true. */
  hasMinimumPermissionGroup: (_requiredGroup: string) => {
    console.warn('[DEPRECATED] hasMinimumPermissionGroup() - Permission enforcement moved to backend. This always returns true.');
    return true;
  },

  /** @deprecated Backend handles permission enforcement. This always returns true. */
  hasEnterpriseTier: (_tier: 'Starter' | 'Business' | 'Enterprise' | 'Whale') => {
    console.warn('[DEPRECATED] hasEnterpriseTier() - Permission enforcement moved to backend. This always returns true.');
    return true;
  },

  /** @deprecated Backend handles permission enforcement. This always returns true. */
  isAdmin: () => {
    console.warn('[DEPRECATED] isAdmin() - Permission enforcement moved to backend. This always returns true.');
    return true;
  },

  /** @deprecated Backend handles permission enforcement. This always returns true. */
  canManageUsers: () => {
    console.warn('[DEPRECATED] canManageUsers() - Permission enforcement moved to backend. This always returns true.');
    return true;
  },

  /** @deprecated Backend handles permission enforcement. This always returns true. */
  canManageSystem: () => {
    console.warn('[DEPRECATED] canManageSystem() - Permission enforcement moved to backend. This always returns true.');
    return true;
  },

  /** @deprecated Backend handles permission enforcement. This always returns true. */
  canViewAnalytics: () => {
    console.warn('[DEPRECATED] canViewAnalytics() - Permission enforcement moved to backend. This always returns true.');
    return true;
  },
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
          user: adminWallet,
          walletAddress: adminWallet.wallet_address,
          isAuthenticated: true,
          expiresAt: Date.now() + (24 * 60 * 60 * 1000),
          error: null
        });
      } else {
        // Clear session if no admin permissions
        useAuth.setState({
          wallet: null,
          user: null,
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
        user: null,
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
 */
export function getAdminDisplayName(user: EnterpriseAdminUser | null): string {
  if (!user) { return 'Unknown Admin'; }
  return user.wallet_address ?
    `${user.wallet_address.slice(0, 6)}...${user.wallet_address.slice(-4)}` :
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

// Legacy compatibility exports (deprecated)
export interface AuthState {
  user: any | null;
  isLoading: boolean;
  isAuthenticated: boolean;
  error: string | null;
}