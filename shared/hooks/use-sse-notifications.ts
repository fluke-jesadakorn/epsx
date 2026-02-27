import type { Dispatch, MutableRefObject, SetStateAction } from 'react';
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

const MAX_RECONNECT_ATTEMPTS = 10

interface SSEStateCtx {
  connectionStateRef: MutableRefObject<ConnectionState>
  connectionIdRef: MutableRefObject<number>
  isMounted: MutableRefObject<boolean>
  reconnectAttempts: MutableRefObject<number>
  optionsRef: MutableRefObject<UseSSENotificationsOptions>
  setIsConnected: Dispatch<SetStateAction<boolean>>
  setError: Dispatch<SetStateAction<string | null>>
}

function handleSSEError(
  ctx: SSEStateCtx,
  currentId: number,
  scheduleReconnect: (delay: number) => void
): void {
  if (!ctx.isMounted.current || ctx.connectionIdRef.current !== currentId) {
    logger.warn(`⚠️ SSE: Stale error from connection #${currentId}, ignoring`)
    return
  }

  if (ctx.connectionStateRef.current === ConnectionState.CONNECTING) {
    ctx.connectionStateRef.current = ConnectionState.DISCONNECTED
    ctx.setIsConnected(false)
    ctx.setError('Connection failed. Retrying...')
    ctx.reconnectAttempts.current++
    scheduleReconnect(Math.min(3000 * ctx.reconnectAttempts.current, 30000))
    return
  }

  if (ctx.connectionStateRef.current === ConnectionState.CONNECTED) {
    ctx.connectionStateRef.current = ConnectionState.DISCONNECTED
    ctx.setIsConnected(false)
    ctx.setError('Connection lost. Reconnecting...')
    ctx.optionsRef.current.onError?.('Connection lost. Reconnecting...')
    ctx.reconnectAttempts.current++
    if (ctx.reconnectAttempts.current < MAX_RECONNECT_ATTEMPTS) {
      const delay = Math.min(5000 * ctx.reconnectAttempts.current, 30000)
      logger.debug(`🔄 SSE: Scheduling reconnect attempt ${ctx.reconnectAttempts.current} in ${delay}ms`)
      scheduleReconnect(delay)
    } else {
      ctx.setError('Failed to connect after multiple attempts')
      ctx.optionsRef.current.onError?.('Failed to connect after multiple attempts')
    }
  }
}

interface UseSSEConnectCtx extends SSEStateCtx {
  disconnectRef: MutableRefObject<(() => void) | null>
  setNotifications: Dispatch<SetStateAction<SSENotification[]>>
}

function useSSEConnect(ctx: UseSSEConnectCtx): { connect: () => void; disconnect: () => void; reconnect: () => void } {
  // Store ctx in a ref so connect/disconnect callbacks don't need ctx as a dependency
  const ctxRef = useRef(ctx)
  ctxRef.current = ctx

  const connectRef = useRef<() => void>(() => undefined)

  const connect = useCallback(() => {
    const { connectionStateRef, connectionIdRef, isMounted, reconnectAttempts, optionsRef, setIsConnected, setError, disconnectRef, setNotifications } = ctxRef.current
    if (connectionStateRef.current !== ConnectionState.DISCONNECTED) {
      logger.debug(`⏭️ SSE: Already ${connectionStateRef.current}, skipping connection attempt`)
      return
    }
    connectionStateRef.current = ConnectionState.CONNECTING
    const currentId = ++connectionIdRef.current
    logger.debug(`🔌 SSE: Initiating connection #${currentId}...`)

    try {
      const { apiClient, walletAddress, types } = optionsRef.current
      if (apiClient === undefined) { throw new Error('API client not available') }
      const client = createNotificationsClient(apiClient)

      const stateCtx: SSEStateCtx = { connectionStateRef, connectionIdRef, isMounted, reconnectAttempts, optionsRef, setIsConnected, setError }
      const scheduleReconnect = (delay: number): void => {
        setTimeout(() => {
          if (isMounted.current && connectionStateRef.current === ConnectionState.DISCONNECTED) {
            connectRef.current()
          }
        }, delay)
      }

      const disconnectFn = client.connectToSSE(
        { wallet_address: walletAddress, types, auto_reconnect: false, reconnect_interval: Math.min(5000 * (reconnectAttempts.current + 1), 30000) },
        {
          onNotification: (notification) => {
            if (!isMounted.current || connectionIdRef.current !== currentId) {
              logger.warn(`⚠️ SSE: Stale notification from connection #${currentId}, ignoring`)
              return
            }
            setNotifications((prev) => [notification, ...prev].slice(0, 50))
            optionsRef.current.onNotification?.(notification)
            reconnectAttempts.current = 0
          },
          onError: (_err) => { handleSSEError(stateCtx, currentId, scheduleReconnect) },
          onOpen: () => {
            if (!isMounted.current || connectionIdRef.current !== currentId) {
              logger.warn(`⚠️ SSE: Stale open event from connection #${currentId}, ignoring`)
              return
            }
            logger.info(`✅ SSE: Connection #${currentId} established`)
            connectionStateRef.current = ConnectionState.CONNECTED
            setIsConnected(true)
            setError(null)
            reconnectAttempts.current = 0
            optionsRef.current.onConnect?.()
          },
        }
      )
      disconnectRef.current = disconnectFn
    } catch (err) {
      connectionStateRef.current = ConnectionState.DISCONNECTED
      const errorMsg = err instanceof Error ? err.message : 'Failed to connect to notification stream'
      setError(errorMsg)
      optionsRef.current.onError?.(errorMsg)
      logger.error(`❌ SSE: Connection #${connectionIdRef.current} failed:`, errorMsg)
    }
  }, []) // ctxRef is stable; all values accessed via ctxRef.current

  connectRef.current = connect

  const disconnect = useCallback(() => {
    const { connectionStateRef, connectionIdRef, disconnectRef, setIsConnected } = ctxRef.current
    const prevState = connectionStateRef.current
    if (prevState === ConnectionState.DISCONNECTED) {
      logger.debug('⏭️ SSE: Already disconnected, skipping')
      return
    }
    logger.info(`🔴 SSE: Disconnecting from ${prevState} state...`)
    connectionStateRef.current = ConnectionState.DISCONNECTING
    connectionIdRef.current++

    if (disconnectRef.current !== null) {
      try { disconnectRef.current() } catch (err) { logger.error('❌ SSE: Error during disconnect:', err) }
      disconnectRef.current = null
    }

    connectionStateRef.current = ConnectionState.DISCONNECTED
    setIsConnected(false)
    logger.info('✅ SSE: Disconnected successfully')
  }, []) // ctxRef is stable

  const reconnect = useCallback(() => { connect() }, [connect])

  return { connect, disconnect, reconnect }
}

export function useSSENotifications(
  options: UseSSENotificationsOptions
): UseSSENotificationsReturn {
  const { autoConnect = true } = options

  const [notifications, setNotifications] = useState<SSENotification[]>([])
  const [isConnected, setIsConnected] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const connectionStateRef = useRef<ConnectionState>(ConnectionState.DISCONNECTED)
  const disconnectRef = useRef<(() => void) | null>(null)
  const reconnectAttempts = useRef(0)
  const isMounted = useRef<boolean>(true)
  const connectionIdRef = useRef(0)
  const optionsRef = useRef(options)
  optionsRef.current = options

  const { connect, disconnect, reconnect } = useSSEConnect({
    connectionStateRef, disconnectRef, reconnectAttempts, isMounted,
    connectionIdRef, optionsRef, setIsConnected, setError, setNotifications,
  })

  const addNotification = useCallback((notification: SSENotification) => {
    setNotifications((prev) => [notification, ...prev].slice(0, 50))
  }, [])

  useEffect(() => {
    isMounted.current = true
    // Capture ref objects (not .current) for use in cleanup
    const idRef = connectionIdRef
    const dRef = disconnectRef
    const stateRef = connectionStateRef

    if (autoConnect && stateRef.current === ConnectionState.DISCONNECTED) {
      logger.debug('🎬 SSE: Auto-connect triggered')
      const connectTimer = setTimeout(() => {
        if (isMounted.current && stateRef.current === ConnectionState.DISCONNECTED) {
          connect()
        }
      }, 100)
      return () => { clearTimeout(connectTimer) }
    }

    return () => {
      logger.debug('🧹 SSE: Cleanup triggered')
      idRef.current++
      if (dRef.current !== null) {
        try { dRef.current() } catch (err) { logger.error('❌ SSE: Error during disconnect:', err) }
        dRef.current = null
      }
      stateRef.current = ConnectionState.DISCONNECTED
      logger.info('✅ SSE: Disconnected successfully')
      isMounted.current = false
    }
  }, [autoConnect, connect])

  return { notifications, isConnected, error, reconnect, disconnect, addNotification }
}
