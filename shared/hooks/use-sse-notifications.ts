/* eslint-disable max-lines-per-function */
/* eslint-disable @typescript-eslint/no-unnecessary-condition */
import { useCallback, useEffect, useRef, useState } from 'react';
import type { NotificationType, SSENotification } from '../api/notifications';
import { createNotificationsClient } from '../api/notifications';
import type { UnifiedApiClient } from '../utils/api-client';
import { logger } from '../utils/logger';

interface UseSSENotificationsOptions {
  apiClient?: UnifiedApiClient
  walletAddress?: string
  types?: NotificationType[]
  autoConnect?: boolean
  onNotification?: (notification: SSENotification) => void
  onError?: (error: string) => void
  onConnect?: () => void
  refreshSession?: () => Promise<boolean>
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

const checkSessionExpiry = async (refreshSession: () => Promise<boolean>): Promise<boolean> => {
  try {
    logger.info('SSE: Proactively refreshing session before connection...')
    const refreshed = await refreshSession()
    if (refreshed) {
      logger.info('SSE: Session refreshed, proceeding with connection')
      return true
    }
    logger.warn('SSE: Session refresh failed, aborting connection')
    return false
  } catch (e) {
    logger.error('SSE: Error during session refresh', e)
    return false
  }
}

export function useSSENotifications(
  options: UseSSENotificationsOptions
): UseSSENotificationsReturn {
  const { autoConnect = true } = options;

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

  // Store options in refs to avoid dependency changes
  const optionsRef = useRef(options)
  optionsRef.current = options

  const connect = useCallback(async () => {
    // Atomic state check - prevent race conditions
    if (connectionStateRef.current !== ConnectionState.DISCONNECTED) {
      logger.debug(`⏭️ SSE: Already ${connectionStateRef.current}, skipping connection attempt`)
      return
    }

    // Atomically transition to CONNECTING state
    connectionStateRef.current = ConnectionState.CONNECTING
    const currentConnectionId = ++connectionIdRef.current
    logger.debug(`🔌 SSE: Initiating connection #${currentConnectionId}...`)

    // Check for token expiry and refresh if needed
    if (optionsRef.current.refreshSession !== undefined) {
      const sessionOk = await checkSessionExpiry(optionsRef.current.refreshSession)
      if (!sessionOk) {
        connectionStateRef.current = ConnectionState.DISCONNECTED
        if (isMounted.current) {
          setError('Session expired')
        }
        return
      }
    }

    try {
      const { apiClient, walletAddress, types } = optionsRef.current
      if (apiClient === undefined) {
        throw new Error('API client not available')
      }
      const client = createNotificationsClient(apiClient)

      const disconnectFn = client.connectToSSE(
        {
          wallet_address: walletAddress,
          types,
          auto_reconnect: false, // We handle reconnection ourselves to avoid conflicts
          reconnect_interval: Math.min(5000 * (reconnectAttempts.current + 1), 30000),
        },
        {
          onNotification: (notification) => {
            // Verify this callback is from the current connection
            if (!isMounted.current || connectionIdRef.current !== currentConnectionId) {
              logger.warn(`⚠️ SSE: Stale notification from connection #${currentConnectionId}, ignoring`)
              return
            }

            if (isMounted.current) {
              setNotifications((prev) => {
                const newArray = [notification, ...prev]
                // Keep only last 50 notifications to prevent memory issues
                return newArray.slice(0, 50)
              })
            }
            optionsRef.current.onNotification?.(notification)
            reconnectAttempts.current = 0
          },
          onError: (_err) => {
            const handleError = async () => {
              // Verify this callback is from the current connection
              if (!isMounted.current || connectionIdRef.current !== currentConnectionId) {
                logger.warn(`⚠️ SSE: Stale error from connection #${currentConnectionId}, ignoring`)
                return
              }

              // Only update state if we're still in CONNECTED state
              if (connectionStateRef.current === ConnectionState.CONNECTED) {
                connectionStateRef.current = ConnectionState.DISCONNECTED
                if (isMounted.current) {
                  setIsConnected(false)
                  const errorMsg = 'Connection lost. Reconnecting...'
                  setError(errorMsg)
                }
                optionsRef.current.onError?.('Connection lost. Reconnecting...')

                // This handles the edge case where token expires but cookie is still valid
                let refreshOk = true
                if (optionsRef.current.refreshSession !== undefined) {
                  logger.info('🔄 SSE: Connection dropped, attempting proactive session refresh...')
                  try {
                    const refreshed = await optionsRef.current.refreshSession()
                    if (refreshed) {
                      logger.info('✅ SSE: Session refreshed successfully after drop')
                      reconnectAttempts.current = 0
                    } else {
                      logger.warn('⛔ SSE: Session refresh returned false (logged out), stopping reconnect')
                      refreshOk = false
                    }
                  } catch (e) {
                    logger.warn('⛔ SSE: Session refresh failed during reconnect logic, stopping reconnect', e)
                    refreshOk = false
                  }
                }

                // Don't reconnect if session refresh failed (user logged out)
                if (!refreshOk) {
                  if (isMounted.current) {
                    setError('Session expired')
                  }
                  return
                }

                reconnectAttempts.current++
                if (reconnectAttempts.current < maxReconnectAttempts) {
                  // Attempt reconnection with exponential backoff
                  const delay = Math.min(5000 * reconnectAttempts.current, 30000)
                  logger.debug(`🔄 SSE: Scheduling reconnect attempt ${reconnectAttempts.current} in ${delay}ms`)
                  setTimeout(() => {
                    if (isMounted.current && connectionStateRef.current === ConnectionState.DISCONNECTED) {
                      void connect()
                    }
                  }, delay)
                } else {
                  if (isMounted.current) {
                    setError('Failed to connect after multiple attempts')
                  }
                  optionsRef.current.onError?.('Failed to connect after multiple attempts')
                }
              }
            }
            void handleError()
          },
          onOpen: () => {
            // Verify this callback is from the current connection
            if (!isMounted.current || connectionIdRef.current !== currentConnectionId) {
              logger.warn(`⚠️ SSE: Stale open event from connection #${currentConnectionId}, ignoring`)
              return
            }

            logger.info(`✅ SSE: Connection #${currentConnectionId} established`)
            connectionStateRef.current = ConnectionState.CONNECTED
            if (isMounted.current) {
              setIsConnected(true)
              setError(null)
            }
            reconnectAttempts.current = 0
            optionsRef.current.onConnect?.()
          }
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
      optionsRef.current.onError?.(errorMsg)
      logger.error(`❌ SSE: Connection #${currentConnectionId} failed:`, errorMsg)
    }
  }, []) // Empty dependency array since we use refs for all values

  const disconnect = useCallback(() => {
    // Atomic state transition to DISCONNECTING
    const prevState = connectionStateRef.current
    if (prevState === ConnectionState.DISCONNECTED) {
      logger.debug('⏭️ SSE: Already disconnected, skipping')
      return
    }

    logger.info(`🔴 SSE: Disconnecting from ${prevState} state...`)
    connectionStateRef.current = ConnectionState.DISCONNECTING

    // Increment connection ID to invalidate any pending callbacks
    connectionIdRef.current++

    if (disconnectRef.current) {
      try {
        disconnectRef.current()
      } catch (err) {
        logger.error('❌ SSE: Error during disconnect:', err)
      }
      disconnectRef.current = null
    }

    // Final state transition
    connectionStateRef.current = ConnectionState.DISCONNECTED
    setIsConnected(false)
    logger.info('✅ SSE: Disconnected successfully')
  }, [])

  const addNotification = useCallback((notification: SSENotification) => {
    setNotifications((prev) => {
      const newArray = [notification, ...prev]
      // Keep only last 50 notifications to prevent memory issues
      return newArray.slice(0, 50)
    })
  }, [])

  useEffect(() => {
    isMounted.current = true

    if (autoConnect && connectionStateRef.current === ConnectionState.DISCONNECTED) {
      logger.debug('🎬 SSE: Auto-connect triggered')
      // Add a small delay to prevent race conditions during component mounting
      const connectTimer = setTimeout(() => {
        if (isMounted.current && connectionStateRef.current === ConnectionState.DISCONNECTED) {
          void connect()
        }
      }, 100)

      return () => {
        clearTimeout(connectTimer)
      }
    }

    return () => {
      logger.debug('🧹 SSE: Cleanup triggered')

      // Cancel any pending operations without setting state
      if (connectionStateRef.current !== ConnectionState.DISCONNECTED) {
        // eslint-disable-next-line react-hooks/exhaustive-deps
        connectionIdRef.current++ // Invalidate any pending callbacks

        if (disconnectRef.current) {
          try {
            disconnectRef.current()
          } catch (err) {
            logger.error('❌ SSE: Error during disconnect:', err)
          }
          disconnectRef.current = null
        }

        connectionStateRef.current = ConnectionState.DISCONNECTED
        // Remove setIsConnected call to prevent state updates during cleanup
        logger.info('✅ SSE: Disconnected successfully')
      }

      isMounted.current = false
    }
  }, [autoConnect, connect])

  return {
    notifications,
    isConnected,
    error,
    reconnect: () => { void connect() },
    disconnect,
    addNotification,
  }
}
