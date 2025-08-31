'use client';

import { useEffect, useState, useCallback, useRef } from 'react';
import { useAuth } from '@/lib/auth';
import { usePermissionExpiry } from './usePermissionExpiry';

// Types for real-time permission events
export interface PermissionUpdateEvent {
  type: 'permission_granted' | 'permission_revoked' | 'permission_expired' | 'permission_expiring_soon' | 'permission_extended';
  userId: string;
  permission: string;
  basePermission: string;
  platform: string;
  resource: string;
  action: string;
  expiryTimestamp?: number;
  previousExpiry?: number;
  newExpiry?: number;
  reason?: string;
  grantedBy?: string;
  timestamp: number;
  metadata?: Record<string, any>;
}

export interface RealtimePermissionOptions {
  enabled?: boolean;
  reconnectAttempts?: number;
  reconnectDelay?: number;
  onPermissionUpdated?: (event: PermissionUpdateEvent) => void;
  onPermissionExpired?: (event: PermissionUpdateEvent) => void;
  onPermissionExpiring?: (event: PermissionUpdateEvent) => void;
  onConnectionStateChange?: (state: 'connecting' | 'connected' | 'disconnected' | 'error') => void;
}

export interface RealtimePermissionReturn {
  isConnected: boolean;
  connectionState: 'connecting' | 'connected' | 'disconnected' | 'error';
  lastUpdate: PermissionUpdateEvent | null;
  reconnectAttempts: number;
  connect: () => void;
  disconnect: () => void;
  forceRefresh: () => Promise<void>;
}

const DEFAULT_OPTIONS: Required<Omit<RealtimePermissionOptions, 'onPermissionUpdated' | 'onPermissionExpired' | 'onPermissionExpiring' | 'onConnectionStateChange'>> = {
  enabled: true,
  reconnectAttempts: 5,
  reconnectDelay: 3000,
};

export function useRealtimePermissions(options: RealtimePermissionOptions = {}): RealtimePermissionReturn {
  const { user, refreshUser } = useAuth();
  const { refresh: refreshExpiry } = usePermissionExpiry();
  
  const [isConnected, setIsConnected] = useState(false);
  const [connectionState, setConnectionState] = useState<'connecting' | 'connected' | 'disconnected' | 'error'>('disconnected');
  const [lastUpdate, setLastUpdate] = useState<PermissionUpdateEvent | null>(null);
  const [reconnectAttempts, setReconnectAttempts] = useState(0);
  
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const heartbeatIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const mountedRef = useRef(true);
  
  const config = { ...DEFAULT_OPTIONS, ...options };

  // Cleanup function
  const cleanup = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }
    if (heartbeatIntervalRef.current) {
      clearInterval(heartbeatIntervalRef.current);
      heartbeatIntervalRef.current = null;
    }
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }
  }, []);

  // Handle connection state changes
  const updateConnectionState = useCallback((state: 'connecting' | 'connected' | 'disconnected' | 'error') => {
    if (!mountedRef.current) return;
    
    setConnectionState(state);
    setIsConnected(state === 'connected');
    options.onConnectionStateChange?.(state);
  }, [options]);

  // Handle incoming WebSocket messages
  const handleMessage = useCallback(async (event: MessageEvent) => {
    try {
      const data: PermissionUpdateEvent = JSON.parse(event.data);
      
      if (!mountedRef.current) return;
      
      // Verify the event is for the current user
      if (data.userId !== user?.id) {
        return;
      }

      setLastUpdate(data);

      // Refresh user permissions and expiry data
      await Promise.all([
        refreshUser(),
        refreshExpiry()
      ]);

      // Call specific event handlers
      switch (data.type) {
        case 'permission_expired':
          options.onPermissionExpired?.(data);
          break;
        case 'permission_expiring_soon':
          options.onPermissionExpiring?.(data);
          break;
        case 'permission_granted':
        case 'permission_revoked':
        case 'permission_extended':
          options.onPermissionUpdated?.(data);
          break;
      }
    } catch (error) {
      console.error('Error parsing WebSocket message:', error);
    }
  }, [user?.id, refreshUser, refreshExpiry, options]);

  // Start heartbeat to keep connection alive
  const startHeartbeat = useCallback(() => {
    if (heartbeatIntervalRef.current) {
      clearInterval(heartbeatIntervalRef.current);
    }
    
    heartbeatIntervalRef.current = setInterval(() => {
      if (wsRef.current?.readyState === WebSocket.OPEN) {
        wsRef.current.send(JSON.stringify({ type: 'ping' }));
      }
    }, 30000); // Send ping every 30 seconds
  }, []);

  // Connect to WebSocket
  const connect = useCallback(() => {
    if (!config.enabled || !user?.id) return;
    if (wsRef.current?.readyState === WebSocket.CONNECTING || wsRef.current?.readyState === WebSocket.OPEN) {
      return; // Already connecting or connected
    }

    updateConnectionState('connecting');
    
    try {
      const wsUrl = `${process.env.NEXT_PUBLIC_WS_URL || 'wss://api.epsx.io'}/ws/permissions/${user.id}`;
      const ws = new WebSocket(wsUrl);
      wsRef.current = ws;

      ws.onopen = () => {
        if (!mountedRef.current) return;
        
        console.log('Permission WebSocket connected');
        updateConnectionState('connected');
        setReconnectAttempts(0);
        startHeartbeat();
        
        // Send authentication message
        ws.send(JSON.stringify({
          type: 'auth',
          userId: user.id,
          token: (user as any)?.accessToken
        }));
      };

      ws.onmessage = handleMessage;

      ws.onclose = (event) => {
        if (!mountedRef.current) return;
        
        console.log('Permission WebSocket closed', event.code, event.reason);
        updateConnectionState('disconnected');
        
        if (heartbeatIntervalRef.current) {
          clearInterval(heartbeatIntervalRef.current);
          heartbeatIntervalRef.current = null;
        }
        
        // Attempt to reconnect unless it was a normal closure
        if (event.code !== 1000 && reconnectAttempts < config.reconnectAttempts) {
          setReconnectAttempts(prev => prev + 1);
          reconnectTimeoutRef.current = setTimeout(() => {
            if (mountedRef.current) {
              connect();
            }
          }, config.reconnectDelay);
        }
      };

      ws.onerror = (error) => {
        if (!mountedRef.current) return;
        
        console.error('Permission WebSocket error:', error);
        updateConnectionState('error');
      };
    } catch (error) {
      console.error('Failed to create WebSocket connection:', error);
      updateConnectionState('error');
    }
  }, [config.enabled, config.reconnectAttempts, config.reconnectDelay, user?.id, reconnectAttempts, updateConnectionState, handleMessage, startHeartbeat]);

  // Disconnect from WebSocket
  const disconnect = useCallback(() => {
    cleanup();
    updateConnectionState('disconnected');
    setReconnectAttempts(0);
  }, [cleanup, updateConnectionState]);

  // Force refresh permissions
  const forceRefresh = useCallback(async () => {
    try {
      await Promise.all([
        refreshUser(),
        refreshExpiry()
      ]);
    } catch (error) {
      console.error('Error refreshing permissions:', error);
    }
  }, [refreshUser, refreshExpiry]);

  // Auto-connect when user is available and enabled
  useEffect(() => {
    if (config.enabled && user?.id) {
      connect();
    }
    return () => {
      cleanup();
    };
  }, [config.enabled, user?.id, connect, cleanup]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      mountedRef.current = false;
      cleanup();
    };
  }, [cleanup]);

  return {
    isConnected,
    connectionState,
    lastUpdate,
    reconnectAttempts,
    connect,
    disconnect,
    forceRefresh,
  };
}