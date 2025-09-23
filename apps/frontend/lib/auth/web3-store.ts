'use client';

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { useAccount, useDisconnect, useSignMessage } from 'wagmi';
import { toast } from 'sonner';

// Types
export interface Web3Permission {
  permission: string;
  source: 'manual' | 'nft' | 'token' | 'dao';
  expires_at?: string;
  metadata?: {
    nft_collection?: string;
    token_contract?: string;
    dao_name?: string;
    required_amount?: string;
    [key: string]: any;
  };
}

export interface Web3AuthState {
  // Connection state
  isConnected: boolean;
  isAuthenticated: boolean;
  isAuthenticating: boolean;
  isLoading: boolean;
  hasInitialized: boolean;
  
  // User data
  walletAddress?: string;
  permissions: Web3Permission[];
  userTier: 'free' | 'nft' | 'token' | 'dao' | 'enterprise';
  hasApiAccess: boolean;
  error?: string;
}

export interface Web3AuthActions {
  // State management
  setConnected: (connected: boolean) => void;
  setAuthenticated: (authenticated: boolean) => void;
  setAuthenticating: (authenticating: boolean) => void;
  setLoading: (loading: boolean) => void;
  setInitialized: (initialized: boolean) => void;
  setWalletAddress: (address?: string) => void;
  setPermissions: (permissions: Web3Permission[]) => void;
  setUserTier: (tier: Web3AuthState['userTier']) => void;
  setApiAccess: (hasAccess: boolean) => void;
  setError: (error?: string) => void;
  
  // Auth actions
  authenticate: () => Promise<void>;
  disconnect: () => Promise<void>;
  checkAuthStatus: () => Promise<boolean | undefined>;
  refreshPermissions: () => Promise<boolean>;
  linkEmail: (email: string, password: string) => Promise<void>;
  generateApiKey: (name: string) => Promise<string>;
  resetAuthState: () => void;
  
  // Internal state management
  initializeAuth: () => Promise<void>;
}

export type Web3AuthStore = Web3AuthState & Web3AuthActions;

// Store
export const useWeb3AuthStore = create<Web3AuthStore>()(
  persist(
    (set, get) => ({
      // Initial state
      isConnected: false,
      isAuthenticated: false,
      isAuthenticating: false,
      isLoading: true,
      hasInitialized: false,
      permissions: [],
      userTier: 'free',
      hasApiAccess: false,

      // State setters
      setConnected: (connected) => set({ isConnected: connected }),
      setAuthenticated: (authenticated) => set({ isAuthenticated: authenticated }),
      setAuthenticating: (authenticating) => set({ isAuthenticating: authenticating }),
      setLoading: (loading) => set({ isLoading: loading }),
      setInitialized: (initialized) => set({ hasInitialized: initialized }),
      setWalletAddress: (address) => set({ walletAddress: address }),
      setPermissions: (permissions) => set({ permissions }),
      setUserTier: (tier) => set({ userTier: tier }),
      setApiAccess: (hasAccess) => set({ hasApiAccess: hasAccess }),
      setError: (error) => set({ error }),

      // Initialize authentication state
      initializeAuth: async () => {
        const state = get();
        
        // Prevent multiple initialization attempts
        if (state.hasInitialized) return;
        
        try {
          console.log('🚀 Initializing Web3 auth store');
          set({ isLoading: true, error: undefined });
          
          // Check authentication status
          await get().checkAuthStatus();
          
          set({ hasInitialized: true });
          console.log('✅ Web3 auth store initialization completed');
        } catch (error) {
          console.error('❌ Failed to initialize Web3 auth store:', error);
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
              
              // Refresh permissions
              await get().refreshPermissions();
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

      // Refresh user permissions
      refreshPermissions: async () => {
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
            const { permissions, user_tier, has_api_access } = await response.json();
            set({
              permissions: permissions || [],
              userTier: user_tier || 'free',
              hasApiAccess: has_api_access || false,
            });
            return true;
          } else if (response.status === 405) {
            // Set default values when endpoint is not available
            set({
              permissions: [],
              userTier: 'free',
              hasApiAccess: false,
            });
            return true;
          }
          return false;
        } catch (error) {
          console.warn('Failed to fetch permissions:', error);
          set({
            permissions: [],
            userTier: 'free',
            hasApiAccess: false,
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
          // Get challenge from backend
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

          // Verify signature with backend
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

          // Success
          set({
            isAuthenticated: true,
            isAuthenticating: false,
          });

          // Mark session for future checks
          if (typeof window !== 'undefined') {
            try {
              window.localStorage.setItem('oidc_session', '1');
              document.cookie = 'oidc_session=1; path=/; SameSite=Lax';
            } catch {}
          }

          // Refresh permissions after successful authentication
          await get().refreshPermissions();
          toast.success('Successfully authenticated with Web3 wallet');
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
        console.log('🔄 Starting Web3 wallet disconnect...');

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
              console.log('✅ Server-side session invalidated');
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
                    key.includes('oidc'))
                ) {
                  keysToRemove.push(key);
                }
              }

              keysToRemove.forEach(key => {
                window.localStorage.removeItem(key);
              });

              // Clear cookies
              const cookiesToClear = ['oidc_session', 'access_token', 'id_token', 'refresh_token'];
              cookiesToClear.forEach(cookieName => {
                document.cookie = `${cookieName}=; Max-Age=0; path=/; SameSite=Lax`;
                document.cookie = `${cookieName}=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path=/`;
              });
            } catch (error) {
              console.error('Storage cleanup error:', error);
            }
          }

          // Reset store state
          set({
            isConnected: false,
            isAuthenticated: false,
            isAuthenticating: false,
            permissions: [],
            userTier: 'free',
            hasApiAccess: false,
            walletAddress: undefined,
            error: undefined,
          });

          toast.success('Wallet disconnected successfully');
          console.log('✅ Web3 wallet disconnect completed');

          // Refresh page for complete state reset
          window.location.reload();
        } catch (error) {
          console.error('❌ Error during wallet disconnect:', error);
          toast.error('Wallet disconnected with errors - refreshing page...');
          window.location.reload();
        }
      },

      // Link email to wallet
      linkEmail: async (email: string, password: string) => {
        const state = get();
        if (!state.walletAddress) {
          throw new Error('Wallet not connected');
        }

        const response = await fetch('/api/auth/web3/link-email', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            wallet_address: state.walletAddress,
            email,
            password,
          }),
          credentials: 'include',
        });

        if (!response.ok) {
          const error = await response.json();
          throw new Error(error.message || 'Failed to link email');
        }

        toast.success('Email linked successfully');
        await get().refreshPermissions();
      },

      // Generate API key
      generateApiKey: async (name: string): Promise<string> => {
        const state = get();
        if (!state.walletAddress || !state.hasApiAccess) {
          throw new Error('API access not available');
        }

        const response = await fetch('/api/auth/web3/api-keys', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            wallet_address: state.walletAddress,
            name,
          }),
          credentials: 'include',
        });

        if (!response.ok) {
          const error = await response.json();
          throw new Error(error.message || 'Failed to generate API key');
        }

        const { api_key } = await response.json();
        toast.success('API key generated successfully');
        return api_key;
      },

      // Reset authentication state
      resetAuthState: () => {
        console.log('🔄 Resetting Web3 authentication state...');
        const state = get();
        set({
          isAuthenticated: false,
          isAuthenticating: false,
          permissions: [],
          userTier: 'free',
          hasApiAccess: false,
          error: undefined,
        });
        console.log('✅ Web3 authentication state reset');
      },
    }),
    {
      name: 'web3-auth-storage',
      partialize: (state) => ({
        // Only persist essential connection state, not auth state
        walletAddress: state.walletAddress,
        hasInitialized: state.hasInitialized,
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
  userTier: state.userTier,
  hasApiAccess: state.hasApiAccess,
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

export function getTierDescription(tier: Web3AuthState['userTier']): string {
  switch (tier) {
    case 'free': return 'Basic access to platform features';
    case 'nft': return 'Enhanced access via NFT ownership';
    case 'token': return 'Token-gated premium features';
    case 'dao': return 'DAO governance access and voting';
    case 'enterprise': return 'Full API access and team management';
    default: return 'Standard user access';
  }
}

export function isPermissionExpired(permission: Web3Permission): boolean {
  if (!permission.expires_at) return false;
  return new Date(permission.expires_at) < new Date();
}

export function formatAddress(address: string): string {
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
}