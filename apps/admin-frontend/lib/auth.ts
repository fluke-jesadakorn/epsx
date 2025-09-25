'use client'

/**
 * Web3 Enterprise Admin Authentication
 * Pure Web3 wallet-based authentication for admin dashboard
 */

import { create } from 'zustand'
import { config } from '@/config/env'

// Web3 Enterprise Admin User interface
export interface EnterpriseAdminUser {
  wallet_address: string;
  enterprise_tier: 'Starter' | 'Business' | 'Enterprise' | 'Whale';
  permissions: string[];
  has_api_access: boolean;
  verified_tokens_usd: number;
  nft_collections: string[];
  dao_memberships: string[];
  is_admin: boolean;
  admin_permissions: string[];
}

export interface Web3AdminAuthState {
  user: EnterpriseAdminUser | null;
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

// Create Web3 enterprise admin auth store
export const useAuth = create<Web3AdminAuthState & {
  // Web3 Admin Actions
  connectWallet: () => Promise<void>;
  authenticateAdmin: () => Promise<void>;
  disconnectWallet: () => Promise<void>;
  getAdminUser: () => Promise<EnterpriseAdminUser | null>;
  refreshSession: () => Promise<void>;
  clearError: () => void;
  // Permission helpers
  can: (permission: string) => boolean;
  hasAnyPermission: (permissions: string[]) => boolean;
  hasAllPermissions: (permissions: string[]) => boolean;
  isAdmin: () => boolean;
  canManageUsers: () => boolean;
  canManageSystem: () => boolean;
  canViewAnalytics: () => boolean;
}>((set, get) => ({
  user: null,
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
      // This would integrate with wallet connection logic
      // For now, we'll simulate the wallet connection
      console.log('🔄 Admin: Connecting Web3 wallet...');
      
      // Note: This should integrate with actual wallet connection library
      // such as wagmi/rainbowkit for the admin frontend
      const walletAddress = await (window as any).__admin_wallet_connect?.();
      
      if (walletAddress) {
        set({ walletAddress, isConnecting: false });
        console.log('✅ Admin: Wallet connected:', walletAddress.slice(0, 8) + '...');
      } else {
        throw new Error('Wallet connection cancelled');
      }
    } catch (error) {
      console.error('❌ Admin: Wallet connection failed:', error);
      set({ 
        error: error instanceof Error ? error.message : 'Wallet connection failed',
        isConnecting: false 
      });
    }
  },

  // Authenticate admin with Web3 signature
  authenticateAdmin: async () => {
    const { walletAddress } = get();
    if (!walletAddress) {
      set({ error: 'Please connect wallet first' });
      return;
    }

    set({ isAuthenticating: true, error: null });

    try {
      console.log('🔄 Admin: Starting Web3 enterprise authentication...');
      
      // Get challenge from enterprise API
      const challengeResponse = await fetch('/api/auth/web3/challenge', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ wallet_address: walletAddress }),
      });

      if (!challengeResponse.ok) {
        throw new Error('Failed to get authentication challenge');
      }

      const challenge = await challengeResponse.json();
      
      // Sign message with wallet
      const signature = await (window as any).__admin_wallet_sign?.(challenge.message);
      if (!signature) {
        throw new Error('Signature cancelled');
      }

      // Verify signature with enterprise API
      const verifyResponse = await fetch('/api/auth/web3/verify', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          wallet_address: walletAddress,
          signature,
          message: challenge.message,
          nonce: challenge.nonce,
        }),
      });

      if (!verifyResponse.ok) {
        throw new Error('Authentication verification failed');
      }

      const adminData = await verifyResponse.json();
      
      // Check if user has admin permissions
      if (!adminData.admin_permissions?.length && !adminData.permissions?.some((p: string) => p.startsWith('admin:'))) {
        throw new Error('Insufficient admin permissions');
      }

      const adminUser: EnterpriseAdminUser = {
        wallet_address: adminData.wallet_address,
        enterprise_tier: adminData.enterprise_tier || 'Starter',
        permissions: adminData.permissions || [],
        has_api_access: adminData.has_api_access || false,
        verified_tokens_usd: adminData.verified_tokens_usd || 0,
        nft_collections: adminData.nft_collections || [],
        dao_memberships: adminData.dao_memberships || [],
        is_admin: true,
        admin_permissions: adminData.admin_permissions || [],
      };

      set({
        user: adminUser,
        isAuthenticated: true,
        isAuthenticating: false,
        expiresAt: Date.now() + (24 * 60 * 60 * 1000), // 24 hours
      });

      console.log('✅ Admin: Web3 enterprise authentication successful');
    } catch (error) {
      console.error('❌ Admin: Authentication failed:', error);
      set({
        error: error instanceof Error ? error.message : 'Authentication failed',
        isAuthenticating: false,
        isAuthenticated: false,
      });
    }
  },

  // Disconnect wallet and clear session
  disconnectWallet: async () => {
    set({ isLoading: true });
    
    try {
      // Notify backend of logout
      if (get().isAuthenticated) {
        await fetch('/api/auth/web3/logout', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ wallet_address: get().walletAddress }),
          credentials: 'include',
        });
      }

      // Clear wallet connection
      await (window as any).__admin_wallet_disconnect?.();

      set({
        user: null,
        isAuthenticated: false,
        expiresAt: null,
        walletAddress: undefined,
        isLoading: false,
        error: null,
      });

      console.log('✅ Admin: Wallet disconnected and session cleared');
      window.location.href = '/login';
    } catch (error) {
      console.error('❌ Admin: Disconnect failed:', error);
      set({ 
        error: 'Disconnect failed. Please try again.',
        isLoading: false 
      });
    }
  },

  // Get current admin user
  getAdminUser: async () => {
    const { user, expiresAt } = get();
    if (user && expiresAt && Date.now() < expiresAt) {
      return user;
    }

    set({ isLoading: true, error: null });

    try {
      const response = await fetch('/api/auth/session', {
        credentials: 'include'
      });

      if (!response.ok) {
        if (response.status === 401) {
          set({ 
            user: null,
            isAuthenticated: false,
            expiresAt: null,
            isLoading: false
          });
          return null;
        }
        throw new Error(`Admin session fetch failed: ${response.status}`);
      }

      const data = await response.json();
      
      if (!data.isAuthenticated || !data.user) {
        set({ 
          user: null,
          isAuthenticated: false,
          expiresAt: null,
          isLoading: false
        });
        return null;
      }

      // Validate admin permissions
      const hasAdminAccess = data.user.admin_permissions?.length > 0 || 
        data.user.permissions?.some((p: string) => p.startsWith('admin:'));

      if (!hasAdminAccess) {
        set({ 
          user: null,
          isAuthenticated: false,
          expiresAt: null,
          isLoading: false,
          error: 'Insufficient admin permissions'
        });
        return null;
      }

      const adminUser: EnterpriseAdminUser = {
        wallet_address: data.user.wallet_address,
        enterprise_tier: data.user.enterprise_tier || 'Starter',
        permissions: data.user.permissions || [],
        has_api_access: data.user.has_api_access || false,
        verified_tokens_usd: data.user.verified_tokens_usd || 0,
        nft_collections: data.user.nft_collections || [],
        dao_memberships: data.user.dao_memberships || [],
        is_admin: true,
        admin_permissions: data.user.admin_permissions || [],
      };

      set({ 
        user: adminUser,
        isAuthenticated: true,
        expiresAt: data.expiresAt,
        isLoading: false
      });

      return adminUser;

    } catch (error) {
      console.error('❌ Get admin user failed:', error);
      set({ 
        user: null,
        isAuthenticated: false,
        expiresAt: null,
        error: 'Failed to load admin session. Please authenticate again.',
        isLoading: false
      });
      return null;
    }
  },

  // Refresh admin session
  refreshSession: async () => {
    try {
      await get().getAdminUser();
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

  // Web3 enterprise tier checking
  hasEnterpriseTier: (tier: 'Starter' | 'Business' | 'Enterprise' | 'Whale') => {
    const { user } = get();
    if (!user) return false;
    
    const tierHierarchy = { 'Starter': 1, 'Business': 2, 'Enterprise': 3, 'Whale': 4 };
    const userLevel = tierHierarchy[user.enterprise_tier] || 0;
    const requiredLevel = tierHierarchy[tier] || 1;
    
    return userLevel >= requiredLevel;
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

// Initialize Web3 admin auth on client mount
if (typeof window !== 'undefined') {
  // Auto-fetch admin user on app start
  useAuth.getState().getAdminUser();
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