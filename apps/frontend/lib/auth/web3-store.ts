'use client';

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { toast } from 'sonner';
import { createWeb3FrontendClient } from '@/shared/utils/web3-api-client';
import type {
  WalletAuthState,
  PermissionInfo,
  GroupMembership,
  PermissionStats,
  Web3Permission,
  Web3PermissionType,
  Web3AuthError,
  BatchPermissionResult
} from '@/shared/types/wallet-auth';

// Enhanced Web3 Auth State using comprehensive backend types
export interface EnhancedWeb3AuthState {
  // Connection state
  isConnected: boolean;
  isAuthenticated: boolean;
  isAuthenticating: boolean;
  isLoading: boolean;
  hasInitialized: boolean;
  
  // User data
  walletAddress?: string;
  permissions: string[]; // Simple permission strings
  permissionInfo: PermissionInfo[]; // Detailed permission info
  groupMemberships: GroupMembership[]; // Permission group memberships
  permissionStats?: PermissionStats; // Permission statistics
  web3Permissions: Web3Permission[]; // Web3-specific permissions
  
  // Tier/Plan info
  tier?: string;
  userTier?: string;
  
  // Legacy enterprise fields (for backward compatibility)
  enterpriseTier: 'Starter' | 'Business' | 'Enterprise' | 'Whale';
  hasApiAccess: boolean;
  verifiedTokensUsd: number;
  nftCollections: string[];
  daoMemberships: string[];
  
  // Error handling
  error?: string | Web3AuthError;
  
  // Session management
  expiresAt?: number;
  accessToken?: string;
  isNewUser?: boolean;
}

export interface EnhancedWeb3AuthActions {
  // State management
  setConnected: (connected: boolean) => void;
  setAuthenticated: (authenticated: boolean) => void;
  setAuthenticating: (authenticating: boolean) => void;
  setLoading: (loading: boolean) => void;
  setInitialized: (initialized: boolean) => void;
  setWalletAddress: (address?: string) => void;
  setPermissions: (permissions: string[]) => void;
  setPermissionInfo: (permissionInfo: PermissionInfo[]) => void;
  setGroupMemberships: (memberships: GroupMembership[]) => void;
  setPermissionStats: (stats?: PermissionStats) => void;
  setWeb3Permissions: (permissions: Web3Permission[]) => void;
  setTier: (tier?: string) => void;
  setEnterpriseTier: (tier: EnhancedWeb3AuthState['enterpriseTier']) => void;
  setApiAccess: (hasAccess: boolean) => void;
  setVerifiedTokensUsd: (amount: number) => void;
  setNftCollections: (collections: string[]) => void;
  setDaoMemberships: (memberships: string[]) => void;
  setError: (error?: string | Web3AuthError) => void;
  setAccessToken: (token?: string) => void;
  setExpiresAt: (expiresAt?: number) => void;
  setIsNewUser: (isNewUser?: boolean) => void;
  
  // Core authentication actions
  authenticate: () => Promise<void>;
  disconnect: () => Promise<void>;
  logout: () => Promise<void>;
  checkAuthStatus: () => Promise<boolean>;
  
  // Permission management actions
  refreshPermissions: () => Promise<void>;
  getPermissionStats: () => Promise<PermissionStats>;
  getGroupMemberships: () => Promise<GroupMembership[]>;
  checkPermissions: (permissions: string[]) => Promise<BatchPermissionResult>;
  hasPermission: (permission: string) => boolean;
  hasAnyPermission: (permissions: string[]) => boolean;
  hasAllPermissions: (permissions: string[]) => boolean;
  
  // Legacy enterprise actions (for backward compatibility)
  refreshEnterpriseData: () => Promise<boolean>;
  generateApiKey: (name: string) => Promise<string>;
  
  // Internal state management
  initializeAuth: () => Promise<void>;
  resetAuthState: () => void;
}

export type EnhancedWeb3AuthStore = EnhancedWeb3AuthState & EnhancedWeb3AuthActions;

// Initialize Web3 API client
const web3ApiClient = createWeb3FrontendClient();

// Enhanced Store
export const useWeb3AuthStore = create<EnhancedWeb3AuthStore>()(
  persist(
    (set, get) => ({
      // Initial state
      isConnected: false,
      isAuthenticated: false,
      isAuthenticating: false,
      isLoading: true,
      hasInitialized: false,
      
      // User data
      walletAddress: undefined,
      permissions: [],
      permissionInfo: [],
      groupMemberships: [],
      permissionStats: undefined,
      web3Permissions: [],
      
      // Tier/Plan info
      tier: undefined,
      userTier: undefined,
      
      // Legacy enterprise fields
      enterpriseTier: 'Starter',
      hasApiAccess: false,
      verifiedTokensUsd: 0,
      nftCollections: [],
      daoMemberships: [],
      
      // Error and session
      error: undefined,
      expiresAt: undefined,
      accessToken: undefined,
      isNewUser: undefined,

      // Enhanced state setters
      setConnected: (connected) => set({ isConnected: connected }),
      setAuthenticated: (authenticated) => set({ isAuthenticated: authenticated }),
      setAuthenticating: (authenticating) => set({ isAuthenticating: authenticating }),
      setLoading: (loading) => set({ isLoading: loading }),
      setInitialized: (initialized) => set({ hasInitialized: initialized }),
      setWalletAddress: (address) => set({ walletAddress: address }),
      setPermissions: (permissions) => set({ permissions }),
      setPermissionInfo: (permissionInfo) => set({ permissionInfo }),
      setGroupMemberships: (memberships) => set({ groupMemberships: memberships }),
      setPermissionStats: (stats) => set({ permissionStats: stats }),
      setWeb3Permissions: (permissions) => set({ web3Permissions: permissions }),
      setTier: (tier) => set({ tier }),
      setEnterpriseTier: (tier) => set({ enterpriseTier: tier }),
      setApiAccess: (hasAccess) => set({ hasApiAccess: hasAccess }),
      setVerifiedTokensUsd: (amount) => set({ verifiedTokensUsd: amount }),
      setNftCollections: (collections) => set({ nftCollections: collections }),
      setDaoMemberships: (memberships) => set({ daoMemberships: memberships }),
      setError: (error) => set({ error }),
      setAccessToken: (token) => set({ accessToken: token }),
      setExpiresAt: (expiresAt) => set({ expiresAt }),
      setIsNewUser: (isNewUser) => set({ isNewUser }),

      // Initialize authentication state
      initializeAuth: async () => {
        const state = get();
        
        // Prevent multiple initialization attempts
        if (state.hasInitialized) return;
        
        try {
          set({ isLoading: true, error: undefined });
          
          // Check authentication status
          await get().checkAuthStatus();
          
          set({ hasInitialized: true });
        } catch (error) {
          console.error('❌ Failed to initialize Web3 enterprise auth store:', error);
          set({ error: error instanceof Error ? error.message : 'Initialization failed' });
        } finally {
          set({ isLoading: false });
        }
      },

      // Check current authentication status
      checkAuthStatus: async () => {
        const state = get();
        if (!state.walletAddress) return false;

        try {
          const response = await fetch('/api/auth/session', {
            credentials: 'include',
            cache: 'no-cache',
          });

          if (response.ok) {
            const session = await response.json();
            if (
              session.isAuthenticated &&
              session.user?.wallet_address === state.walletAddress
            ) {
              set({
                isAuthenticated: true,
                error: undefined,
              });
              
              // Refresh enterprise data
              await get().refreshEnterpriseData();
              return true;
            }
          }

          set({ isAuthenticated: false, error: undefined });
          return false;
        } catch (error) {
          console.warn('Auth status check failed:', error);
          return false;
        }
      },

      // Refresh enterprise data and permissions
      refreshEnterpriseData: async () => {
        const state = get();
        if (!state.walletAddress) return false;

        try {
          const response = await fetch(
            `/api/auth/web3/permissions?wallet_address=${encodeURIComponent(state.walletAddress)}`,
            {
              method: 'GET',
              credentials: 'include',
            }
          );

          if (response.ok) {
            const data = await response.json();
            set({
              permissions: data.permissions || [],
              enterpriseTier: data.enterprise_tier || 'Starter',
              hasApiAccess: data.has_api_access || false,
              verifiedTokensUsd: data.verified_tokens_usd || 0,
              nftCollections: data.nft_collections || [],
              daoMemberships: data.dao_memberships || []
            });
            return true;
          } else if (response.status === 405) {
            // Set default values when endpoint is not available
            set({
              permissions: [],
              enterpriseTier: 'Starter',
              hasApiAccess: false,
              verifiedTokensUsd: 0,
              nftCollections: [],
              daoMemberships: []
            });
            return true;
          }
          return false;
        } catch (error) {
          console.warn('Failed to fetch enterprise data:', error);
          set({
            permissions: [],
            enterpriseTier: 'Starter',
            hasApiAccess: false,
            verifiedTokensUsd: 0,
            nftCollections: [],
            daoMemberships: []
          });
          return false;
        }
      },

      // Authenticate with wallet
      authenticate: async () => {
        const state = get();
        if (!state.walletAddress) {
          toast.error('Please connect your wallet first');
          return;
        }

        if (state.isAuthenticating) {
          toast.error('Authentication already in progress. Please wait.');
          return;
        }

        set({ isAuthenticating: true, error: undefined });

        try {
          // Get challenge from Web3 auth API
          const challengeResponse = await fetch(
            `${process.env.NEXT_PUBLIC_BACKEND_URL}/api/auth/web3/challenge`,
            {
              method: 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({
                wallet_address: state.walletAddress,
              }),
            }
          );

          if (!challengeResponse.ok) {
            throw new Error(
              `Failed to get authentication challenge: ${challengeResponse.status}`
            );
          }

          const challenge = await challengeResponse.json();
          const messageString = challenge.message;

          // Note: signMessageAsync needs to be passed from component using Wagmi hooks
          // This will be handled by the useWeb3Auth hook that wraps this store
          const signResult = await (window as any).__web3Auth_signMessage?.(messageString);
          if (!signResult) {
            throw new Error('Wallet signing function not available');
          }

          // Verify signature with Web3 auth API
          const authResponse = await fetch(
            `${process.env.NEXT_PUBLIC_BACKEND_URL}/api/auth/web3/verify`,
            {
              method: 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({
                wallet_address: state.walletAddress,
                signature: signResult,
                message: messageString,
                nonce: challenge.nonce,
              }),
            }
          );

          if (!authResponse.ok) {
            const errorText = await authResponse.text();
            let errorData;
            try {
              errorData = JSON.parse(errorText);
            } catch {
              errorData = { error: `Authentication failed: ${authResponse.status}` };
            }
            throw new Error(errorData.error || 'Authentication failed');
          }

          // Success - extract enterprise data
          const authData = await authResponse.json();
          
          set({
            isAuthenticated: true,
            isAuthenticating: false,
            enterpriseTier: authData.enterprise_tier || 'Starter',
            hasApiAccess: authData.has_api_access || false,
            verifiedTokensUsd: authData.verified_tokens_usd || 0,
            permissions: authData.permissions || [],
          });

          // Mark session for future checks
          if (typeof window !== 'undefined') {
            try {
              window.localStorage.setItem('web3_session', '1');
              document.cookie = 'web3_session=1; path=/; SameSite=Lax';
            } catch {}
          }

          // Refresh enterprise data after successful authentication
          await get().refreshEnterpriseData();
          toast.success('Successfully authenticated with Web3 enterprise wallet');
        } catch (error: any) {
          let errorMessage = 'Authentication failed';
          if (
            error.message?.includes('User rejected') ||
            error.message?.includes('cancelled')
          ) {
            errorMessage = 'Wallet signature was cancelled';
          } else if (error.message?.includes('timeout')) {
            errorMessage = 'Request timeout. Please try again.';
          } else if (error.message?.includes('expired')) {
            errorMessage = 'Authentication expired - please try again';
          } else if (error.message) {
            errorMessage = error.message;
          }

          set({
            isAuthenticating: false,
            isAuthenticated: false,
            error: errorMessage,
          });

          toast.error(errorMessage);
        }
      },

      // Disconnect wallet
      disconnect: async () => {

        try {
          const state = get();

          // Server-side session invalidation
          if (state.isAuthenticated && state.walletAddress) {
            try {
              await fetch('/api/auth/web3/logout', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                credentials: 'include',
                body: JSON.stringify({
                  wallet_address: state.walletAddress,
                  logout_reason: 'user_initiated_disconnect',
                }),
              });
            } catch (error) {
              console.warn('⚠️ Server-side session invalidation failed:', error);
            }
          }

          // Clear storage
          if (typeof window !== 'undefined') {
            try {
              const keysToRemove = [];
              for (let i = 0; i < window.localStorage.length; i++) {
                const key = window.localStorage.key(i);
                if (
                  key &&
                  (key.startsWith('wagmi.') ||
                    key.startsWith('rk-') ||
                    key.startsWith('rainbow') ||
                    key.includes('wallet') ||
                    key.includes('auth') ||
                    key.includes('web3'))
                ) {
                  keysToRemove.push(key);
                }
              }

              keysToRemove.forEach(key => {
                window.localStorage.removeItem(key);
              });

              // Clear enterprise session cookies
              const cookiesToClear = ['web3_session', 'access_token', 'id_token', 'refresh_token'];
              cookiesToClear.forEach(cookieName => {
                document.cookie = `${cookieName}=; Max-Age=0; path=/; SameSite=Lax`;
                document.cookie = `${cookieName}=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path=/`;
              });
            } catch (error) {
              console.error('Storage cleanup error:', error);
            }
          }

          // Reset store state completely to enterprise defaults
          set({
            isConnected: false,
            isAuthenticated: false,
            isAuthenticating: false,
            isLoading: false,
            hasInitialized: false, // Reset initialization flag
            permissions: [],
            enterpriseTier: 'Starter',
            hasApiAccess: false,
            verifiedTokensUsd: 0,
            nftCollections: [],
            daoMemberships: [],
            walletAddress: undefined,
            error: undefined,
          });

          // Clear any persisted storage to prevent conflicts
          if (typeof window !== 'undefined') {
            try {
              window.localStorage.removeItem('web3-auth-storage');
            } catch (error) {
              console.warn('Failed to clear persisted storage:', error);
            }
          }

          toast.success('Enterprise wallet disconnected successfully');

          // Broadcast disconnect to other tabs
          if (typeof window !== 'undefined' && window.BroadcastChannel) {
            try {
              const channel = new BroadcastChannel('auth_session');
              channel.postMessage({
                type: 'SESSION_INVALIDATED',
                source: 'web3_enterprise_disconnect',
                walletAddress: state.walletAddress,
                timestamp: Date.now(),
              });
              channel.close();
            } catch (broadcastError) {
              console.warn('Failed to broadcast disconnect:', broadcastError);
            }
          }
        } catch (error) {
          console.error('❌ Error during enterprise wallet disconnect:', error);
          toast.error('Enterprise wallet disconnected with errors');
        }
      },

      // Generate enterprise API key
      generateApiKey: async (name: string): Promise<string> => {
        const state = get();
        if (!state.walletAddress || !state.hasApiAccess) {
          throw new Error('Enterprise API access not available for current tier');
        }

        const response = await fetch(`${process.env.NEXT_PUBLIC_BACKEND_URL}/api/v1/enterprise/billing/api-keys`, {
          method: 'POST',
          headers: { 
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${state.walletAddress}` // Temporary - should use proper Bearer token
          },
          body: JSON.stringify({
            wallet_address: state.walletAddress,
            name,
            enterprise_tier: state.enterpriseTier,
          }),
          credentials: 'include',
        });

        if (!response.ok) {
          const error = await response.json();
          throw new Error(error.message || 'Failed to generate enterprise API key');
        }

        const { api_key } = await response.json();
        toast.success('Enterprise API key generated successfully');
        return api_key;
      },

      // Reset authentication state to enterprise defaults
      resetAuthState: () => {
        set({
          isAuthenticated: false,
          isAuthenticating: false,
          isLoading: false,
          hasInitialized: false, // Reset to allow fresh initialization
          permissions: [],
          enterpriseTier: 'Starter',
          hasApiAccess: false,
          verifiedTokensUsd: 0,
          nftCollections: [],
          daoMemberships: [],
          error: undefined,
        });
      },
    }),
    {
      name: 'web3-auth-storage',
      partialize: (state) => ({
        // Don't persist connection-related state to prevent conflicts
        // Only persist non-connection preferences if needed
        // walletAddress and hasInitialized removed to fix reconnection issues
      }),
    }
  )
);

// Selector hooks for better performance
export const useWeb3ConnectedState = () => useWeb3AuthStore(state => ({
  isConnected: state.isConnected,
  walletAddress: state.walletAddress,
}));

export const useWeb3AuthenticatedState = () => useWeb3AuthStore(state => ({
  isAuthenticated: state.isAuthenticated,
  isAuthenticating: state.isAuthenticating,
  permissions: state.permissions,
  enterpriseTier: state.enterpriseTier,
  hasApiAccess: state.hasApiAccess,
  verifiedTokensUsd: state.verifiedTokensUsd,
  nftCollections: state.nftCollections,
  daoMemberships: state.daoMemberships,
}));

export const useWeb3LoadingState = () => useWeb3AuthStore(state => ({
  isLoading: state.isLoading,
  hasInitialized: state.hasInitialized,
  error: state.error,
}));

// Utility functions
export function getPermissionIcon(source: Web3Permission['source']): string {
  switch (source) {
    case 'nft': return '🎨';
    case 'token': return '🪙';
    case 'dao': return '🗳️';
    case 'manual': return '👤';
    default: return '🔑';
  }
}

export function getPermissionBadgeColor(source: Web3Permission['source']): string {
  switch (source) {
    case 'nft': return 'bg-purple-100 text-purple-800 dark:bg-purple-900/20 dark:text-purple-300';
    case 'token': return 'bg-orange-100 text-orange-800 dark:bg-orange-900/20 dark:text-orange-300';
    case 'dao': return 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-300';
    case 'manual': return 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-300';
    default: return 'bg-gray-100 text-gray-800 dark:bg-gray-900/20 dark:text-gray-300';
  }
}

export function getEnterpriseTierDescription(tier: Web3AuthState['enterpriseTier']): string {
  switch (tier) {
    case 'Starter': return 'Basic enterprise features - $1,000+ in verified tokens';
    case 'Business': return 'Advanced features - $10,000+ in tokens OR enterprise NFT';
    case 'Enterprise': return 'Full enterprise features - $100,000+ in tokens OR DAO membership';
    case 'Whale': return 'Unlimited access - $1,000,000+ in tokens with custom infrastructure';
    default: return 'Enterprise user access';
  }
}

export function getEnterpriseTierColor(tier: Web3AuthState['enterpriseTier']): string {
  switch (tier) {
    case 'Starter': return 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-300';
    case 'Business': return 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-300';
    case 'Enterprise': return 'bg-purple-100 text-purple-800 dark:bg-purple-900/20 dark:text-purple-300';
    case 'Whale': return 'bg-gold-100 text-gold-800 dark:bg-gold-900/20 dark:text-gold-300';
    default: return 'bg-gray-100 text-gray-800 dark:bg-gray-900/20 dark:text-gray-300';
  }
}

export function getEnterpriseTierIcon(tier: Web3AuthState['enterpriseTier']): string {
  switch (tier) {
    case 'Starter': return '🚀';
    case 'Business': return '💼';
    case 'Enterprise': return '🏢';
    case 'Whale': return '🐋';
    default: return '⭐';
  }
}

export function isPermissionExpired(permission: Web3Permission): boolean {
  if (!permission.expires_at) return false;
  return new Date(permission.expires_at) < new Date();
}

export function formatAddress(address: string): string {
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
}