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
  const disconnectRef = useRef<(() => void) | null>(null)
  const reconnectAttempts = useRef(0)
  const maxReconnectAttempts = 10
  const isConnecting = useRef(false)
  const isMounted = useRef(true)

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
    // Prevent multiple simultaneous connections
    if (isConnecting.current || disconnectRef.current) {
      console.log('⏭️ SSE: Already connected or connecting, skipping...')
      return
    }

    isConnecting.current = true
    console.log('🔌 SSE: Initiating connection...')

    try {
      const client = createNotificationsClient(apiClient)

      const disconnectFn = client.connectToSSE(
        {
          wallet_address: walletAddress,
          types: types as any,
          auto_reconnect: true,
          reconnect_interval: Math.min(5000 * (reconnectAttempts.current + 1), 30000),
        },
        (notification) => {
          if (!isMounted.current) return
          setNotifications((prev) => [notification, ...prev].slice(0, 100))
          onNotificationRef.current?.(notification)
          reconnectAttempts.current = 0
        },
        (err) => {
          if (!isMounted.current) return
          setIsConnected(false)
          const errorMsg = 'Connection lost. Reconnecting...'
          setError(errorMsg)
          onErrorRef.current?.(errorMsg)

          reconnectAttempts.current++
          if (reconnectAttempts.current >= maxReconnectAttempts) {
            setError('Failed to connect after multiple attempts')
            onErrorRef.current?.('Failed to connect after multiple attempts')
          }
        },
        () => {
          if (!isMounted.current) return
          console.log('✅ SSE: Connection established')
          setIsConnected(true)
          setError(null)
          reconnectAttempts.current = 0
          isConnecting.current = false
          onConnectRef.current?.()
        }
      )

      disconnectRef.current = disconnectFn
    } catch (err) {
      isConnecting.current = false
      const errorMsg = err instanceof Error ? err.message : 'Failed to connect to notification stream'
      setError(errorMsg)
      onErrorRef.current?.(errorMsg)
    }
  }, [apiClient, walletAddress, types])

  const disconnect = useCallback(() => {
    console.log('🔴 SSE: Disconnecting...')
    if (disconnectRef.current) {
      disconnectRef.current()
      disconnectRef.current = null
    }
    isConnecting.current = false
    setIsConnected(false)
  }, [])

  const addNotification = useCallback((notification: SSENotification) => {
    setNotifications((prev) => [notification, ...prev].slice(0, 100))
  }, [])

  useEffect(() => {
    isMounted.current = true

    if (autoConnect && !isConnecting.current && !disconnectRef.current) {
      console.log('🎬 SSE: Auto-connect triggered')
      connect()
    }

    return () => {
      console.log('🧹 SSE: Cleanup triggered')
      isMounted.current = false
      disconnect()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [autoConnect])

  return {
    notifications,
    isConnected,
    error,
    reconnect: connect,
    disconnect,
    addNotification,
  }
}
