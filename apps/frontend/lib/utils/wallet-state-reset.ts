'use client';

/**
 * Comprehensive Wallet State Reset Utility
 * 
 * This utility provides a complete reset of all wallet-related state including:
 * - Wagmi cache and storage
 * - Custom application storage
 * - QueryClient cache invalidation
 * - Session and authentication state
 */

import { QueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';

interface WalletStateResetOptions {
  queryClient?: QueryClient;
  clearCookies?: boolean;
  clearBroadcastState?: boolean;
  showToast?: boolean;
  preserveTheme?: boolean;
}

/**
 * All known storage keys used by wagmi, RainbowKit, and the application
 */
const WAGMI_STORAGE_KEYS = [
  'wagmi.cache',
  'wagmi.store',
  'wagmi.recent-connectors',
  'wagmi.wallet',
  'wagmi.connected',
  'wagmi.recentConnector',
  'wagmi.accountIndex',
] as const;

const RAINBOWKIT_STORAGE_KEYS = [
  'rk-recent',
  'rk-wallet-data',
  'rainbow-wallet-data',
  'rainbowkit-recent-wallet',
] as const;

const APPLICATION_STORAGE_KEYS = [
  'oidc_session',
  'web3_auth_state',
  'auth_state',
  'wallet_state',
  'connection_state',
  'user_preferences', // Wallet-related preferences only
] as const;

const SESSION_STORAGE_KEYS = [
  'oidc_session',
  'web3_auth_state',
  'wagmi_temp',
  'wallet_temp',
] as const;

const AUTHENTICATION_COOKIES = [
  'oidc_session',
  'access_token',
  'id_token', 
  'refresh_token',
  'epsx_frontend_jwt',
  'next-auth.session-token',
  'next-auth.csrf-token',
  'auth_session',
  'wallet_session',
] as const;

/**
 * Clear all localStorage keys related to wallet state
 */
function clearLocalStorage(preserveTheme = true): void {
  if (typeof window === 'undefined') return;

  try {
    // Get theme before clearing if preservation is enabled
    let themeValue: string | null = null;
    if (preserveTheme) {
      themeValue = window.localStorage.getItem('theme');
    }

    // Clear all wagmi-related keys
    WAGMI_STORAGE_KEYS.forEach(key => {
      try {
        window.localStorage.removeItem(key);
      } catch (error) {
        console.warn(`Failed to remove localStorage key: ${key}`, error);
      }
    });

    // Clear all RainbowKit keys
    RAINBOWKIT_STORAGE_KEYS.forEach(key => {
      try {
        window.localStorage.removeItem(key);
      } catch (error) {
        console.warn(`Failed to remove localStorage key: ${key}`, error);
      }
    });

    // Clear application-specific keys
    APPLICATION_STORAGE_KEYS.forEach(key => {
      try {
        window.localStorage.removeItem(key);
      } catch (error) {
        console.warn(`Failed to remove localStorage key: ${key}`, error);
      }
    });

    // Clear any keys that start with known prefixes
    const keysToRemove: string[] = [];
    for (let i = 0; i < window.localStorage.length; i++) {
      const key = window.localStorage.key(i);
      if (key && (
        key.startsWith('wagmi.') ||
        key.startsWith('rk-') ||
        key.startsWith('rainbow') ||
        key.startsWith('wallet') ||
        key.startsWith('auth') ||
        key.startsWith('oidc')
      )) {
        keysToRemove.push(key);
      }
    }

    keysToRemove.forEach(key => {
      try {
        window.localStorage.removeItem(key);
      } catch (error) {
        console.warn(`Failed to remove localStorage key: ${key}`, error);
      }
    });

    // Restore theme if preservation is enabled
    if (preserveTheme && themeValue) {
      window.localStorage.setItem('theme', themeValue);
    }

    console.log('✅ localStorage cleared successfully');
  } catch (error) {
    console.error('❌ Failed to clear localStorage:', error);
  }
}

/**
 * Clear all sessionStorage keys related to wallet state
 */
function clearSessionStorage(): void {
  if (typeof window === 'undefined') return;

  try {
    SESSION_STORAGE_KEYS.forEach(key => {
      try {
        window.sessionStorage.removeItem(key);
      } catch (error) {
        console.warn(`Failed to remove sessionStorage key: ${key}`, error);
      }
    });

    // Clear any keys that start with known prefixes
    const keysToRemove: string[] = [];
    for (let i = 0; i < window.sessionStorage.length; i++) {
      const key = window.sessionStorage.key(i);
      if (key && (
        key.startsWith('wagmi') ||
        key.startsWith('wallet') ||
        key.startsWith('auth') ||
        key.startsWith('oidc')
      )) {
        keysToRemove.push(key);
      }
    }

    keysToRemove.forEach(key => {
      try {
        window.sessionStorage.removeItem(key);
      } catch (error) {
        console.warn(`Failed to remove sessionStorage key: ${key}`, error);
      }
    });

    console.log('✅ sessionStorage cleared successfully');
  } catch (error) {
    console.error('❌ Failed to clear sessionStorage:', error);
  }
}

/**
 * Clear authentication cookies
 */
function clearAuthCookies(): void {
  if (typeof window === 'undefined') return;

  try {
    AUTHENTICATION_COOKIES.forEach(cookieName => {
      // Clear with different path and domain combinations to ensure complete removal
      const cookieVariations = [
        `${cookieName}=; Max-Age=0; path=/; SameSite=Lax`,
        `${cookieName}=; Max-Age=0; path=/; domain=${window.location.hostname}; SameSite=Lax`,
        `${cookieName}=; Max-Age=0; path=/; SameSite=Lax; Secure`,
        `${cookieName}=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path=/`,
        `${cookieName}=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path=/; domain=${window.location.hostname}`,
      ];

      cookieVariations.forEach(cookieString => {
        try {
          document.cookie = cookieString;
        } catch (error) {
          console.warn(`Failed to clear cookie: ${cookieName}`, error);
        }
      });
    });

    console.log('✅ Authentication cookies cleared successfully');
  } catch (error) {
    console.error('❌ Failed to clear authentication cookies:', error);
  }
}

/**
 * Invalidate QueryClient cache for wallet-related queries
 */
function invalidateQueryClientCache(queryClient?: QueryClient): void {
  if (!queryClient) return;

  try {
    console.log('🗑️ Invalidating QueryClient cache for wallet data...');
    
    // First, invalidate specific wagmi queries
    queryClient.invalidateQueries({
      predicate: (query) => {
        const queryKey = query.queryKey;
        if (Array.isArray(queryKey)) {
          const keyString = queryKey.join('.').toLowerCase();
          return keyString.includes('wagmi') || 
                 keyString.includes('connector') || 
                 keyString.includes('account') ||
                 keyString.includes('balance') ||
                 keyString.includes('chain') ||
                 keyString.includes('wallet');
        }
        return false;
      }
    });
    
    // Remove queries that might be stale after disconnect
    queryClient.removeQueries({
      predicate: (query) => {
        const queryKey = query.queryKey;
        if (Array.isArray(queryKey)) {
          const keyString = queryKey.join('.').toLowerCase();
          return keyString.includes('wallet') || 
                 keyString.includes('connect') || 
                 keyString.includes('auth') ||
                 keyString.includes('wagmi') ||
                 keyString.includes('metamask') ||
                 keyString.includes('ethereum');
        }
        return false;
      }
    });

    // Clear specific wagmi-related queries that could cause reconnection issues
    const wagmiQueryPatterns = [
      ['wagmi', 'account'],
      ['wagmi', 'balance'],
      ['wagmi', 'chain'],
      ['wagmi', 'connector'],
      ['wagmi', 'connection'],
      ['connect'],
      ['disconnect'],
      ['ethereum'],
      ['metamask']
    ];

    wagmiQueryPatterns.forEach(pattern => {
      queryClient.removeQueries({ queryKey: pattern });
    });

    console.log('✅ QueryClient cache invalidated successfully');
  } catch (error) {
    console.error('❌ Failed to invalidate QueryClient cache:', error);
  }
}

/**
 * Send cross-tab broadcast to reset state in other tabs
 */
function broadcastStateReset(): void {
  if (typeof window === 'undefined' || !window.BroadcastChannel) return;

  try {
    const channel = new BroadcastChannel('wallet_state_reset');
    channel.postMessage({
      type: 'COMPLETE_WALLET_RESET',
      timestamp: Date.now(),
      source: 'wallet_disconnect',
    });
    
    // Safe close with null check
    if (channel && typeof channel.close === 'function') {
      try {
        channel.close();
      } catch (closeError) {
        console.warn('Channel close error:', closeError);
      }
    }
    
    console.log('✅ Cross-tab state reset broadcast sent');
  } catch (error) {
    console.warn('❌ Failed to broadcast state reset:', error);
  }
}

/**
 * Complete wallet state reset function
 * 
 * This function performs a comprehensive reset of all wallet-related state:
 * - Clears localStorage and sessionStorage
 * - Removes authentication cookies
 * - Invalidates QueryClient cache
 * - Broadcasts reset to other tabs
 * 
 * @param options Configuration options for the reset operation
 */
export function resetWalletState(options: WalletStateResetOptions = {}): void {
  const {
    queryClient,
    clearCookies = true,
    clearBroadcastState = true,
    showToast = false,
    preserveTheme = true,
  } = options;

  console.log('🔄 Starting comprehensive wallet state reset...');

  try {
    // Clear browser storage
    clearLocalStorage(preserveTheme);
    clearSessionStorage();

    // Clear authentication cookies if requested
    if (clearCookies) {
      clearAuthCookies();
    }

    // Invalidate QueryClient cache if provided
    if (queryClient) {
      invalidateQueryClientCache(queryClient);
    }

    // Broadcast reset to other tabs if requested
    if (clearBroadcastState) {
      broadcastStateReset();
    }

    console.log('✅ Comprehensive wallet state reset completed successfully');

    if (showToast) {
      toast.success('Wallet state reset successfully');
    }

  } catch (error) {
    console.error('❌ Error during wallet state reset:', error);
    
    if (showToast) {
      toast.error('Failed to reset wallet state completely');
    }
  }
}

/**
 * Listen for cross-tab state reset broadcasts
 * 
 * @param onReset Callback function to execute when a reset is received
 * @returns Cleanup function to remove the listener
 */
export function listenForStateReset(onReset: () => void): () => void {
  if (typeof window === 'undefined' || !window.BroadcastChannel) {
    return () => {}; // No-op cleanup function
  }

  let channel: BroadcastChannel | null = null;
  
  try {
    channel = new BroadcastChannel('wallet_state_reset');
    
    const handleMessage = (event: MessageEvent) => {
      try {
        if (event.data?.type === 'COMPLETE_WALLET_RESET') {
          console.log('📡 Received cross-tab wallet reset broadcast');
          onReset();
        }
      } catch (error) {
        console.warn('Error handling cross-tab message:', error);
      }
    };

    channel.addEventListener('message', handleMessage);

    return () => {
      try {
        if (channel) {
          channel.removeEventListener('message', handleMessage);
          if (typeof channel.close === 'function') {
            channel.close();
          }
        }
      } catch (error) {
        console.warn('Error cleaning up cross-tab listener:', error);
      }
    };
  } catch (error) {
    console.warn('Failed to create cross-tab listener:', error);
    return () => {}; // No-op cleanup function
  }
}

/**
 * Check if the wallet state appears to be corrupted
 * 
 * @returns true if state corruption is detected
 */
export function detectStateCorruption(): boolean {
  if (typeof window === 'undefined') return false;

  try {
    // Check for conflicting state in localStorage
    const wagmiKeys = WAGMI_STORAGE_KEYS.filter(key => 
      window.localStorage.getItem(key) !== null
    );

    // Check for orphaned authentication state
    const hasOidcSession = window.localStorage.getItem('oidc_session') !== null;
    const hasAuthState = window.localStorage.getItem('web3_auth_state') !== null;
    
    // Detect potential corruption patterns
    const hasWagmiData = wagmiKeys.length > 0;
    const hasAuthData = hasOidcSession || hasAuthState;
    
    // If we have auth data but no wagmi data, or vice versa, it might be corrupted
    if ((hasAuthData && !hasWagmiData) || (!hasAuthData && hasWagmiData)) {
      console.warn('⚠️ Potential wallet state corruption detected');
      return true;
    }

    return false;
  } catch (error) {
    console.warn('Failed to detect state corruption:', error);
    return false;
  }
}

/**
 * Auto-recovery function for corrupted wallet state
 * 
 * @param queryClient Optional QueryClient instance for cache invalidation
 */
export function autoRecoverWalletState(queryClient?: QueryClient): void {
  if (detectStateCorruption()) {
    console.log('🔧 Auto-recovering from detected wallet state corruption...');
    resetWalletState({
      queryClient,
      clearCookies: true,
      clearBroadcastState: true,
      showToast: true,
      preserveTheme: true,
    });
  }
}