'use client'

/**
 * Web3 Admin Wallet Authentication
 * Pure Web3 wallet-based authentication for admin dashboard
 */

import { create } from 'zustand'
import { config } from '@/config/env'
import { derivePermissionGroupFromPermissions, getPermissionGroupLevel } from '../../../../shared/permissions/utils/platform'
import { createAdminClient, SharedWeb3AuthClient, UserInfoResponse } from '../../../../shared/auth/openid-web3-client'

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

// Helper function to check structured permissions with wildcard support
function checkPermissionAccess(userPermissions: string[], requiredPermission: string): boolean {
  const required = parsePermission(requiredPermission);
  if (!required) return false;
  
  for (const permStr of userPermissions) {
    const userPerm = parsePermission(permStr);
    if (!userPerm) continue;
    
    if (userPerm.platform === required.platform && 
        userPerm.resource === required.resource && 
        userPerm.action === required.action) {
      return true;
    }
    
    if (userPerm.platform === required.platform) {
      if (userPerm.resource === '*' && userPerm.action === '*') {
        return true;
      }
      
      if (userPerm.resource === required.resource && userPerm.action === '*') {
        return true;
      }
    }
    
    if (userPerm.platform === 'admin' && userPerm.resource === '*' && userPerm.action === '*') {
      return true;
    }
  }
  
  return false;
}

function parsePermission(permissionString: string): { platform: string; resource: string; action: string } | null {
  const parts = permissionString.split(':');
  if (parts.length !== 3) return null;
  
  return {
    platform: parts[0],
    resource: parts[1],
    action: parts[2]
  };
}

// Transform Web3 user to admin wallet format
function transformWeb3UserToAdminWallet(web3User: UserInfoResponse): AdminWallet {
  const permissions = web3User.permissions || [];
  const isAdmin = permissions.some(p => p.startsWith('admin:'));
  
  return {
    wallet_address: web3User.wallet_address,
    permission_group: derivePermissionGroupFromPermissions(permissions) as any,
    enterprise_tier: 'Starter', // Default tier, could be derived from permissions
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
      console.log('🔄 Admin: Connecting Web3 wallet...');
      
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
          
          console.log('✅ Admin: Already authenticated with wallet:', adminWallet.wallet_address.slice(0, 8) + '...');
          return;
        }
      }
      
      // Trigger wallet connection event for Web3 components
      const walletConnectEvent = new CustomEvent('epsx:admin-connect-wallet');
      window.dispatchEvent(walletConnectEvent);
      
      set({ isConnecting: false });
      
    } catch (error) {
      console.error('❌ Admin: Wallet connection failed:', error);
      set({ 
        error: error instanceof Error ? error.message : 'Wallet connection failed',
        isConnecting: false 
      });
    }
  },

  // Authenticate admin with Web3 signature (called by Web3 components)
  authenticateAdmin: async (walletAddress?: string, signature?: string, message?: string, nonce?: string) => {
    set({ isAuthenticating: true, error: null });

    try {
      console.log('🔄 Admin: Starting Web3 enterprise authentication...');
      
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
        
        console.log('✅ Admin: Web3 enterprise authentication successful');
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
      
    } catch (error) {
      console.error('❌ Admin: Authentication failed:', error);
      set({
        error: error instanceof Error ? error.message : 'Authentication failed',
        isAuthenticating: false,
        isAuthenticated: false,
      });
    }
  },
  
  // Request Web3 challenge for admin authentication
  requestAdminChallenge: async (walletAddress: string) => {
    try {
      console.log('🔄 Admin: Requesting Web3 challenge...', { walletAddress: walletAddress.slice(0, 8) + '...' });
      
      const challenge = await adminWeb3Client.requestChallenge(walletAddress);
      
      console.log('✅ Admin: Challenge received for admin authentication');
      return challenge;
      
    } catch (error) {
      console.error('❌ Admin: Challenge request failed:', error);
      throw error;
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

      console.log('✅ Admin: Wallet disconnected and session cleared');
      window.location.href = '/auth';
    } catch (error) {
      console.error('❌ Admin: Disconnect failed:', error);
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
    } catch (error) {
      console.error('❌ Failed to load admin wallet:', error);
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
    const wallet = await get().getAdminWallet();
    return wallet; // AdminWallet and EnterpriseAdminUser are type-compatible
  },

  // Refresh admin session
  refreshSession: async () => {
    try {
      await get().getAdminWallet();
    } catch (error) {
      console.error('❌ Admin session refresh failed:', error);
      get().disconnectWallet();
    }
  },

  // Clear error state
  clearError: () => {
    set({ error: null });
  },

  // Permission checking methods for Web3 enterprise admin
  can: (permission: string) => {
    const { user } = get();
    if (!user || !user.is_admin) return false;
    
    // Check both regular permissions and admin-specific permissions
    const allPermissions = [...user.permissions, ...user.admin_permissions];
    return checkPermissionAccess(allPermissions, permission);
  },

  hasAnyPermission: (permissions: string[]) => {
    const { user } = get();
    if (!user || !user.is_admin) return false;
    
    return permissions.some(permission => get().can(permission));
  },

  hasAllPermissions: (permissions: string[]) => {
    const { user } = get();
    if (!user || !user.is_admin) return false;
    
    return permissions.every(permission => get().can(permission));
  },

  // Permission group checking (NEW)
  hasMinimumPermissionGroup: (requiredGroup: string) => {
    const { wallet, user } = get();
    const currentWallet = wallet || user; // Support both new and legacy
    if (!currentWallet) return false;
    
    const currentGroup = currentWallet.permission_group || 
                        derivePermissionGroupFromPermissions(currentWallet.permissions || []);
    
    const currentLevel = getPermissionGroupLevel(currentGroup);
    const requiredLevel = getPermissionGroupLevel(requiredGroup);
    
    return currentLevel >= requiredLevel;
  },

  // @deprecated Use hasMinimumPermissionGroup instead
  hasEnterpriseTier: (tier: 'Starter' | 'Business' | 'Enterprise' | 'Whale') => {
    const { user } = get();
    if (!user) return false;
    
    // Map legacy tiers to permission groups for backward compatibility
    const tierToGroupMap = {
      'Starter': 'Basic Access Group',
      'Business': 'Standard Access Group', 
      'Enterprise': 'Professional Access Group',
      'Whale': 'Enterprise Access Group'
    };
    
    const requiredGroup = tierToGroupMap[tier] || 'Basic Access Group';
    return get().hasMinimumPermissionGroup(requiredGroup);
  },

  // Admin-specific permission checks for Web3 enterprise
  isAdmin: () => {
    const { user } = get();
    return user?.is_admin === true && get().hasAnyPermission([
      'admin:*:*',
      'admin:users:*',
      'admin:system:*'
    ]);
  },

  canManageUsers: () => {
    return get().hasAnyPermission([
      'admin:*:*',
      'admin:users:*',
      'admin:users:manage',
      'admin:enterprise:users:manage'
    ]);
  },

  canManageSystem: () => {
    return get().hasAnyPermission([
      'admin:*:*',
      'admin:system:*',
      'admin:system:manage',
      'admin:enterprise:system:manage'
    ]);
  },

  canViewAnalytics: () => {
    return get().hasAnyPermission([
      'admin:*:*',
      'admin:analytics:*',
      'admin:analytics:view',
      'admin:enterprise:analytics:view'
    ]);
  },

  canManageEnterprise: () => {
    return get().hasAnyPermission([
      'admin:*:*',
      'admin:enterprise:*:*',
      'admin:enterprise:manage',
      'admin:enterprise:tiers:manage'
    ]);
  },

  canManageDAO: () => {
    return get().hasAnyPermission([
      'admin:*:*',
      'admin:dao:*:*',
      'admin:dao:manage',
      'admin:governance:manage'
    ]);
  },

  canManageCompliance: () => {
    return get().hasAnyPermission([
      'admin:*:*',
      'admin:compliance:*:*',
      'admin:compliance:manage',
      'admin:kyc:manage'
    ]);
  },

  canManageMarketplace: () => {
    return get().hasAnyPermission([
      'admin:*:*',
      'admin:marketplace:*:*',
      'admin:marketplace:manage',
      'admin:products:manage'
    ]);
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

// Helper functions for Web3 enterprise admin components
export function checkAdminPermission(permission: string): boolean {
  return useAuth.getState().can(permission);
}

export function checkAnyAdminPermission(permissions: string[]): boolean {
  return useAuth.getState().hasAnyPermission(permissions);
}

export function checkEnterpriseTier(tier: 'Starter' | 'Business' | 'Enterprise' | 'Whale'): boolean {
  return useAuth.getState().hasEnterpriseTier(tier);
}

// Web3 Enterprise Admin Permissions Hook
export function useEnterpriseAdminPermissions() {
  const { 
    isAdmin,
    canManageUsers,
    canManageSystem,
    canViewAnalytics,
    canManageEnterprise,
    canManageDAO,
    canManageCompliance,
    canManageMarketplace,
    can,
    hasEnterpriseTier
  } = useAuth.getState();
  
  return {
    // Core admin functions
    isAdmin: isAdmin(),
    canManageUsers: canManageUsers(),
    canManageSystem: canManageSystem(),
    canViewAnalytics: canViewAnalytics(),
    
    // Enterprise-specific admin functions
    canManageEnterprise: canManageEnterprise(),
    canManageDAO: canManageDAO(),
    canManageCompliance: canManageCompliance(),
    canManageMarketplace: canManageMarketplace(),
    
    // Enterprise tier checks
    hasBusinessTier: hasEnterpriseTier('Business'),
    hasEnterpriseTier: hasEnterpriseTier('Enterprise'),
    hasWhaleTier: hasEnterpriseTier('Whale'),
    
    // Specific Web3 admin permissions
    hasUserManagement: can('admin:enterprise:users:manage'),
    hasSystemManagement: can('admin:enterprise:system:manage'),
    hasDAOManagement: can('admin:dao:manage'),
    hasComplianceManagement: can('admin:compliance:manage'),
    hasMarketplaceManagement: can('admin:marketplace:manage'),
    hasAnalyticsAccess: can('admin:enterprise:analytics:view'),
    
    // Helper function for checking any permission
    can,
  };
}

// Utility Functions for Web3 Enterprise Admin UI
export function getAdminDisplayName(user: EnterpriseAdminUser | null): string {
  if (!user) return 'Unknown Admin';
  return user.wallet_address ? 
    `${user.wallet_address.slice(0, 6)}...${user.wallet_address.slice(-4)}` : 
    'Enterprise Admin';
}

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

// Check if user has Web3 enterprise admin access
export function hasEnterpriseAdminAccess(): boolean {
  const { user, hasAnyPermission } = useAuth.getState();
  if (!user || !user.is_admin) return false;
  
  // Check if user has any enterprise admin permissions
  return hasAnyPermission([
    'admin:*:*',
    'admin:enterprise:*:*',
    'admin:dao:*:*',
    'admin:compliance:*:*',
    'admin:marketplace:*:*'
  ]);
}

// Enterprise tier utility functions
export function getEnterpriseTierDisplayName(tier: string): string {
  const tierNames: Record<string, string> = {
    'Starter': 'Starter ($1K+ tokens)',
    'Business': 'Business ($10K+ tokens)',
    'Enterprise': 'Enterprise ($100K+ tokens)', 
    'Whale': 'Whale ($1M+ tokens)'
  };
  
  return tierNames[tier] || tier;
}

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
export async function connectAdminWallet() {
  const { connectWallet } = useAuth.getState();
  return await connectWallet();
}

// Web3 admin authentication helper
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

// Helper function to check if current user is enterprise admin
export function isEnterpriseAdmin(): boolean {
  const { user } = useAuth.getState();
  return user?.is_admin === true && hasEnterpriseAdminAccess();
}