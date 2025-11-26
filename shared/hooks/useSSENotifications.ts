import { useState, useEffect, useCallback, useRef } from 'react'
import { createNotificationsClient, SSENotification } from '../api/notifications'
import type { UnifiedApiClient } from '../utils/api-client'

interface UseSSENotificationsOptions {
  apiClient: UnifiedApiClient
  walletAddress?: string
  types?: string[]
  autoConnect?: boolean
  onNotification?: (notification: SSENotification) => void
  onError?: (error: string) => void
  onConnect?: () => void
}

interface UseSSENotificationsReturn {
  notifications: SSENotification[]
  isConnected: boolean
  error: string | null
  reconnect: () => void
  disconnect: () => void
  addNotification: (notification: SSENotification) => void
}

// Connection states to prevent race conditions
enum ConnectionState {
  DISCONNECTED = 'DISCONNECTED',
  CONNECTING = 'CONNECTING',
  CONNECTED = 'CONNECTED',
  DISCONNECTING = 'DISCONNECTING',
}

export function useSSENotifications(
  options: UseSSENotificationsOptions
): UseSSENotificationsReturn {
  const {
    apiClient,
    walletAddress,
    types,
    autoConnect = true,
    onNotification,
    onError,
    onConnect,
  } = options

  const [notifications, setNotifications] = useState<SSENotification[]>([])
  const [isConnected, setIsConnected] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Connection management with atomic state transitions
  const connectionStateRef = useRef<ConnectionState>(ConnectionState.DISCONNECTED)
  const disconnectRef = useRef<(() => void) | null>(null)
  const reconnectAttempts = useRef(0)
  const maxReconnectAttempts = 10
  const isMounted = useRef(true)
  const connectionIdRef = useRef(0) // Track connection attempts to identify stale ones

  // Store callbacks in refs to avoid dependency changes
  const onNotificationRef = useRef(onNotification)
  const onErrorRef = useRef(onError)
  const onConnectRef = useRef(onConnect)

  // Update refs when callbacks change
  useEffect(() => {
    onNotificationRef.current = onNotification
    onErrorRef.current = onError
    onConnectRef.current = onConnect
  }, [onNotification, onError, onConnect])

  const connect = useCallback(() => {
    // Atomic state check - prevent race conditions
    if (connectionStateRef.current !== ConnectionState.DISCONNECTED) {
      console.log(`⏭️ SSE: Already ${connectionStateRef.current}, skipping connection attempt`)
      return
    }

    // Atomically transition to CONNECTING state
    connectionStateRef.current = ConnectionState.CONNECTING
    const currentConnectionId = ++connectionIdRef.current
    console.log(`🔌 SSE: Initiating connection #${currentConnectionId}...`)

    try {
      const client = createNotificationsClient(apiClient)

      const disconnectFn = client.connectToSSE(
        {
          wallet_address: walletAddress,
          types: types as any,
          auto_reconnect: false, // We handle reconnection ourselves to avoid conflicts
          reconnect_interval: Math.min(5000 * (reconnectAttempts.current + 1), 30000),
        },
        (notification) => {
          // Verify this callback is from the current connection
          if (!isMounted.current || connectionIdRef.current !== currentConnectionId) {
            console.log(`⚠️ SSE: Stale notification from connection #${currentConnectionId}, ignoring`)
            return
          }

          setNotifications((prev) => [notification, ...prev].slice(0, 100))
          onNotificationRef.current?.(notification)
          reconnectAttempts.current = 0
        },
        (err) => {
          // Verify this callback is from the current connection
          if (!isMounted.current || connectionIdRef.current !== currentConnectionId) {
            console.log(`⚠️ SSE: Stale error from connection #${currentConnectionId}, ignoring`)
            return
          }

          // Only update state if we're still in CONNECTED state
          if (connectionStateRef.current === ConnectionState.CONNECTED) {
            connectionStateRef.current = ConnectionState.DISCONNECTED
            setIsConnected(false)
            const errorMsg = 'Connection lost. Reconnecting...'
            setError(errorMsg)
            onErrorRef.current?.(errorMsg)

            reconnectAttempts.current++
            if (reconnectAttempts.current < maxReconnectAttempts) {
              // Attempt reconnection with exponential backoff
              const delay = Math.min(5000 * reconnectAttempts.current, 30000)
              console.log(`🔄 SSE: Scheduling reconnect attempt ${reconnectAttempts.current} in ${delay}ms`)
              setTimeout(() => {
                if (isMounted.current && connectionStateRef.current === ConnectionState.DISCONNECTED) {
                  connect()
                }
              }, delay)
            } else {
              setError('Failed to connect after multiple attempts')
              onErrorRef.current?.('Failed to connect after multiple attempts')
            }
          }
        },
        () => {
          // Verify this callback is from the current connection
          if (!isMounted.current || connectionIdRef.current !== currentConnectionId) {
            console.log(`⚠️ SSE: Stale open event from connection #${currentConnectionId}, ignoring`)
            return
          }

          console.log(`✅ SSE: Connection #${currentConnectionId} established`)
          connectionStateRef.current = ConnectionState.CONNECTED
          setIsConnected(true)
          setError(null)
          reconnectAttempts.current = 0
          onConnectRef.current?.()
        }
      )

      disconnectRef.current = disconnectFn
    } catch (err) {
      // Reset state on connection failure
      connectionStateRef.current = ConnectionState.DISCONNECTED
      const errorMsg = err instanceof Error ? err.message : 'Failed to connect to notification stream'

      if (isMounted.current) {
        setError(errorMsg)
      }
      onErrorRef.current?.(errorMsg)
      console.error(`❌ SSE: Connection #${currentConnectionId} failed:`, errorMsg)
    }
  }, [apiClient, walletAddress, types])

  const disconnect = useCallback(() => {
    // Atomic state transition to DISCONNECTING
    const prevState = connectionStateRef.current
    if (prevState === ConnectionState.DISCONNECTED) {
      console.log('⏭️ SSE: Already disconnected, skipping')
      return
    }

    console.log(`🔴 SSE: Disconnecting from ${prevState} state...`)
    connectionStateRef.current = ConnectionState.DISCONNECTING

    // Increment connection ID to invalidate any pending callbacks
    connectionIdRef.current++

    if (disconnectRef.current) {
      try {
        disconnectRef.current()
      } catch (err) {
        console.error('❌ SSE: Error during disconnect:', err)
      }
      disconnectRef.current = null
    }

    // Final state transition
    connectionStateRef.current = ConnectionState.DISCONNECTED
    setIsConnected(false)
    console.log('✅ SSE: Disconnected successfully')
  }, [])

  const addNotification = useCallback((notification: SSENotification) => {
    setNotifications((prev) => [notification, ...prev].slice(0, 100))
  }, [])

  useEffect(() => {
    isMounted.current = true

    if (autoConnect && connectionStateRef.current === ConnectionState.DISCONNECTED) {
      console.log('🎬 SSE: Auto-connect triggered')
      // Add a small delay to prevent race conditions during component mounting
      const connectTimer = setTimeout(() => {
        if (isMounted.current && connectionStateRef.current === ConnectionState.DISCONNECTED) {
          connect()
        }
      }, 100)

      return () => {
        clearTimeout(connectTimer)
      }
    }

    return () => {
      console.log('🧹 SSE: Cleanup triggered')
      isMounted.current = false
      disconnect()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [autoConnect, connect, disconnect])

  return {
    notifications,
    isConnected,
    error,
    reconnect: connect,
    disconnect,
    addNotification,
  }
}
