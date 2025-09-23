'use client';

/**
 * Cross-Tab Synchronization Hook
 * 
 * Provides comprehensive cross-tab state synchronization for wallet and auth state.
 * Ensures consistent state across all browser tabs and handles various sync scenarios.
 */

import { useEffect, useCallback, useRef } from 'react';
import { useAccount } from 'wagmi';
import { useWeb3AuthContext } from '@/providers/Web3AuthProvider';
import { useWeb3Context } from '@/providers/Web3Provider';
import { resetWalletState } from '@/lib/utils/wallet-state-reset';
import { toast } from 'sonner';

interface SyncMessage {
  type: string;
  timestamp: number;
  data?: any;
  source?: string;
  tabId?: string;
}

interface CrossTabSyncOptions {
  enableWalletSync?: boolean;
  enableAuthSync?: boolean;
  enableAutoRecovery?: boolean;
  showNotifications?: boolean;
}

const DEFAULT_OPTIONS: CrossTabSyncOptions = {
  enableWalletSync: true,
  enableAuthSync: true,
  enableAutoRecovery: true,
  showNotifications: false,
};

export function useCrossTabSync(options: CrossTabSyncOptions = {}) {
  const opts = { ...DEFAULT_OPTIONS, ...options };
  const { address, isConnected } = useAccount();
  const { isAuthenticated, disconnect: authDisconnect } = useWeb3AuthContext();
  const { resetWalletConnection } = useWeb3Context();
  
  // Generate unique tab ID for this session
  const tabIdRef = useRef<string>(() => 
    `tab_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
  );
  
  // Keep track of channels for cleanup
  const channelsRef = useRef<BroadcastChannel[]>([]);

  // Send sync message to other tabs
  const sendSyncMessage = useCallback((message: Omit<SyncMessage, 'timestamp' | 'tabId'>) => {
    if (typeof window === 'undefined' || !window.BroadcastChannel) return;

    try {
      const channel = new BroadcastChannel('cross_tab_sync');
      const fullMessage: SyncMessage = {
        ...message,
        timestamp: Date.now(),
        tabId: tabIdRef.current,
      };
      
      channel.postMessage(fullMessage);
      channel.close();
      
      console.log('📡 Sent cross-tab sync message:', fullMessage.type);
    } catch (error) {
      console.warn('Failed to send cross-tab sync message:', error);
    }
  }, []);

  // Handle incoming sync messages
  const handleSyncMessage = useCallback(async (event: MessageEvent<SyncMessage>) => {
    const message = event.data;
    
    // Ignore messages from this tab
    if (message.tabId === tabIdRef.current) return;
    
    console.log('📡 Received cross-tab sync message:', message.type, message.data);

    try {
      switch (message.type) {
        case 'WALLET_DISCONNECTED':
          if (opts.enableWalletSync && isConnected) {
            console.log('🔄 Syncing wallet disconnect from another tab');
            await authDisconnect();
            if (opts.showNotifications) {
              toast.info('Wallet disconnected in another tab');
            }
          }
          break;

        case 'AUTH_LOGOUT':
          if (opts.enableAuthSync && isAuthenticated) {
            console.log('🔄 Syncing auth logout from another tab');
            await authDisconnect();
            if (opts.showNotifications) {
              toast.info('Logged out in another tab');
            }
          }
          break;

        case 'WALLET_STATE_RESET':
          if (opts.enableWalletSync && opts.enableAutoRecovery) {
            console.log('🔄 Syncing wallet state reset from another tab');
            await resetWalletConnection();
            if (opts.showNotifications) {
              toast.info('Wallet state reset in another tab');
            }
          }
          break;

        case 'CONNECTION_RECOVERY':
          if (opts.enableAutoRecovery) {
            console.log('🔄 Connection recovery initiated in another tab');
            // Reset local state to ensure consistency
            resetWalletState({
              clearCookies: false, // Don't clear cookies as other tab is handling it
              clearBroadcastState: false, // Don't broadcast to avoid loops
              showToast: false,
              preserveTheme: true,
            });
          }
          break;

        case 'STATE_CORRUPTION_DETECTED':
          if (opts.enableAutoRecovery) {
            console.warn('⚠️ State corruption detected in another tab, checking local state...');
            // Trigger local state validation
            await resetWalletConnection();
          }
          break;

        case 'TAB_HEARTBEAT':
          // Respond to heartbeat to indicate this tab is active
          sendSyncMessage({
            type: 'TAB_HEARTBEAT_RESPONSE',
            data: {
              address,
              isConnected,
              isAuthenticated,
            },
          });
          break;

        case 'REQUEST_STATE_SYNC':
          // Another tab is requesting current state
          sendSyncMessage({
            type: 'STATE_SYNC_RESPONSE',
            data: {
              address,
              isConnected,
              isAuthenticated,
              timestamp: Date.now(),
            },
          });
          break;

        default:
          console.log('Unknown cross-tab sync message type:', message.type);
      }
    } catch (error) {
      console.error('Error handling cross-tab sync message:', error);
    }
  }, [opts, isConnected, isAuthenticated, authDisconnect, resetWalletConnection, address, sendSyncMessage]);

  // Set up cross-tab listeners
  useEffect(() => {
    if (typeof window === 'undefined' || !window.BroadcastChannel) return;

    const channels: BroadcastChannel[] = [];

    // Main sync channel
    const syncChannel = new BroadcastChannel('cross_tab_sync');
    syncChannel.addEventListener('message', handleSyncMessage);
    channels.push(syncChannel);

    // Legacy channels for backward compatibility
    const authChannel = new BroadcastChannel('auth_session');
    authChannel.addEventListener('message', (event) => {
      if (event.data?.type === 'SESSION_INVALIDATED') {
        handleSyncMessage({
          data: {
            type: 'AUTH_LOGOUT',
            timestamp: Date.now(),
            source: event.data.source,
          }
        } as MessageEvent<SyncMessage>);
      }
    });
    channels.push(authChannel);

    const walletResetChannel = new BroadcastChannel('wallet_state_reset');
    walletResetChannel.addEventListener('message', (event) => {
      if (event.data?.type === 'COMPLETE_WALLET_RESET') {
        handleSyncMessage({
          data: {
            type: 'WALLET_STATE_RESET',
            timestamp: Date.now(),
            source: event.data.source,
          }
        } as MessageEvent<SyncMessage>);
      }
    });
    channels.push(walletResetChannel);

    channelsRef.current = channels;

    // Cleanup function
    return () => {
      channels.forEach(channel => {
        try {
          channel.close();
        } catch (error) {
          console.warn('Error closing broadcast channel:', error);
        }
      });
      channelsRef.current = [];
    };
  }, [handleSyncMessage]);

  // Send heartbeat to detect other active tabs
  const sendHeartbeat = useCallback(() => {
    sendSyncMessage({
      type: 'TAB_HEARTBEAT',
      data: {
        address,
        isConnected,
        isAuthenticated,
      },
    });
  }, [sendSyncMessage, address, isConnected, isAuthenticated]);

  // Request state sync from other tabs
  const requestStateSync = useCallback(() => {
    sendSyncMessage({
      type: 'REQUEST_STATE_SYNC',
    });
  }, [sendSyncMessage]);

  // Notify other tabs of wallet disconnection
  const notifyWalletDisconnected = useCallback(() => {
    sendSyncMessage({
      type: 'WALLET_DISCONNECTED',
      data: { address, timestamp: Date.now() },
    });
  }, [sendSyncMessage, address]);

  // Notify other tabs of auth logout
  const notifyAuthLogout = useCallback(() => {
    sendSyncMessage({
      type: 'AUTH_LOGOUT',
      data: { address, timestamp: Date.now() },
    });
  }, [sendSyncMessage, address]);

  // Notify other tabs of state corruption
  const notifyStateCorruption = useCallback((details?: string) => {
    sendSyncMessage({
      type: 'STATE_CORRUPTION_DETECTED',
      data: { details, address, timestamp: Date.now() },
    });
  }, [sendSyncMessage, address]);

  // Notify other tabs of connection recovery
  const notifyConnectionRecovery = useCallback(() => {
    sendSyncMessage({
      type: 'CONNECTION_RECOVERY',
      data: { address, timestamp: Date.now() },
    });
  }, [sendSyncMessage, address]);

  // Clean up channels on unmount
  useEffect(() => {
    return () => {
      channelsRef.current.forEach(channel => {
        try {
          channel.close();
        } catch (error) {
          console.warn('Error closing broadcast channel on unmount:', error);
        }
      });
    };
  }, []);

  return {
    // Sync notification functions
    sendHeartbeat,
    requestStateSync,
    notifyWalletDisconnected,
    notifyAuthLogout,
    notifyStateCorruption,
    notifyConnectionRecovery,
    
    // Current tab ID
    tabId: tabIdRef.current,
    
    // Manual sync trigger
    sendSyncMessage,
  };
}