'use client'

import { useEffect, useRef, useState, useCallback } from 'react'
import { useOptimisticUpdates } from '@/lib/hooks/use-optimistic-updates'
import type { UnifiedUserData } from '@/lib/types/unified-user'

export interface RealtimeEvent {
  type: string
  data: any
  metadata?: {
    event_id: string
    timestamp: string
    source: string
    user_id?: string
    session_id?: string
  }
}

export interface UserStatusUpdate {
  user_id: string
  status: 'active' | 'disabled' | 'pending' | 'suspended'
  timestamp: string
  reason?: string
}

export interface UserProfileUpdate {
  user_id: string
  field: string
  old_value: any
  new_value: any
  timestamp: string
}

export interface UserRoleUpdate {
  user_id: string
  role_id: string
  role_name: string
  action: 'added' | 'removed' | 'updated'
  timestamp: string
}

export interface RealtimeHookOptions {
  events?: string[]
  autoConnect?: boolean
  reconnectAttempts?: number
  reconnectDelay?: number
  onConnect?: () => void
  onDisconnect?: () => void
  onError?: (error: Error) => void
  onUserStatusUpdate?: (update: UserStatusUpdate) => void
  onUserProfileUpdate?: (update: UserProfileUpdate) => void
  onUserRoleUpdate?: (update: UserRoleUpdate) => void
}

export function useRealtimeUpdates(
  users: UnifiedUserData[],
  options: RealtimeHookOptions = {}
) {
  const {
    events = ['user', 'notification'],
    autoConnect = true,
    reconnectAttempts = 5,
    reconnectDelay = 3000,
    onConnect,
    onDisconnect,
    onError,
    onUserStatusUpdate,
    onUserProfileUpdate,
    onUserRoleUpdate,
  } = options

  const [isConnected, setIsConnected] = useState(false)
  const [connectionStatus, setConnectionStatus] = useState<'disconnected' | 'connecting' | 'connected' | 'error'>('disconnected')
  const [lastUpdate, setLastUpdate] = useState<Date | null>(null)
  
  const wsRef = useRef<WebSocket | null>(null)
  const reconnectTimeoutRef = useRef<NodeJS.Timeout>()
  const reconnectCountRef = useRef(0)
  const mountedRef = useRef(true)

  // Optimistic updates for user data
  const {
    pendingUpdates,
    addOptimisticUpdate,
    confirmUpdate,
    rollbackUpdate,
    applyOptimisticUpdates
  } = useOptimisticUpdates<UnifiedUserData>({
    maxRetries: 3,
    retryDelay: 2000,
    rollbackDelay: 10000
  })

  // Apply optimistic updates to user list
  const optimisticUsers = applyOptimisticUpdates(users, pendingUpdates)

  const getWebSocketUrl = useCallback(() => {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    const host = process.env.NODE_ENV === 'development' 
      ? 'localhost:8080' 
      : window.location.host.replace(':3001', ':8080')
    
    const eventParams = events.join(',')
    return `${protocol}//${host}/api/v1/realtime/ws?events=${eventParams}`
  }, [events])

  const connect = useCallback(() => {
    if (!mountedRef.current || wsRef.current?.readyState === WebSocket.OPEN) {
      return
    }

    setConnectionStatus('connecting')
    
    try {
      const wsUrl = getWebSocketUrl()
      console.log('Connecting to WebSocket:', wsUrl)
      
      const ws = new WebSocket(wsUrl)
      wsRef.current = ws

      ws.onopen = () => {
        console.log('WebSocket connected')
        setIsConnected(true)
        setConnectionStatus('connected')
        reconnectCountRef.current = 0
        onConnect?.()
      }

      ws.onclose = (event) => {
        console.log('WebSocket disconnected:', event.code, event.reason)
        setIsConnected(false)
        setConnectionStatus('disconnected')
        wsRef.current = null
        onDisconnect?.()

        // Attempt reconnection if not manually closed
        if (event.code !== 1000 && reconnectCountRef.current < reconnectAttempts && mountedRef.current) {
          reconnectCountRef.current++
          console.log(`Attempting reconnection ${reconnectCountRef.current}/${reconnectAttempts}`)
          
          reconnectTimeoutRef.current = setTimeout(() => {
            if (mountedRef.current) {
              connect()
            }
          }, reconnectDelay * Math.pow(1.5, reconnectCountRef.current - 1)) // Exponential backoff
        }
      }

      ws.onerror = (error) => {
        console.error('WebSocket error:', error)
        setConnectionStatus('error')
        onError?.(new Error('WebSocket connection error'))
      }

      ws.onmessage = (event) => {
        try {
          const message: RealtimeEvent = JSON.parse(event.data)
          setLastUpdate(new Date())
          
          handleRealtimeEvent(message)
        } catch (error) {
          console.error('Failed to parse WebSocket message:', error)
        }
      }

    } catch (error) {
      console.error('Failed to create WebSocket connection:', error)
      setConnectionStatus('error')
      onError?.(error as Error)
    }
  }, [getWebSocketUrl, onConnect, onDisconnect, onError, reconnectAttempts, reconnectDelay])

  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current)
    }
    
    if (wsRef.current) {
      wsRef.current.close(1000, 'Manual disconnect')
    }
    
    setIsConnected(false)
    setConnectionStatus('disconnected')
  }, [])

  const handleRealtimeEvent = useCallback((event: RealtimeEvent) => {
    console.log('Received realtime event:', event.type, event.data)

    switch (event.type) {
      case 'UserStatusChanged':
        const statusUpdate: UserStatusUpdate = event.data
        onUserStatusUpdate?.(statusUpdate)
        
        // Apply optimistic update for status change
        const statusUpdateId = `status_${statusUpdate.user_id}_${Date.now()}`
        const user = users.find(u => u.id === statusUpdate.user_id)
        if (user) {
          const updatedUser = { ...user, status: statusUpdate.status }
          addOptimisticUpdate(statusUpdateId, 'update', updatedUser, user)
          
          // Confirm update after a delay (simulating server confirmation)
          setTimeout(() => confirmUpdate(statusUpdateId), 1000)
        }
        break

      case 'UserProfileUpdated':
        const profileUpdate: UserProfileUpdate = event.data
        onUserProfileUpdate?.(profileUpdate)
        
        // Apply optimistic update for profile change
        const profileUpdateId = `profile_${profileUpdate.user_id}_${Date.now()}`
        const profileUser = users.find(u => u.id === profileUpdate.user_id)
        if (profileUser) {
          const updatedProfileUser = { 
            ...profileUser, 
            [profileUpdate.field]: profileUpdate.new_value 
          }
          addOptimisticUpdate(profileUpdateId, 'update', updatedProfileUser, profileUser)
          setTimeout(() => confirmUpdate(profileUpdateId), 1000)
        }
        break

      case 'UserRoleUpdated':
        const roleUpdate: UserRoleUpdate = event.data
        onUserRoleUpdate?.(roleUpdate)
        
        // Apply optimistic update for role change
        const roleUpdateId = `role_${roleUpdate.user_id}_${Date.now()}`
        const roleUser = users.find(u => u.id === roleUpdate.user_id)
        if (roleUser) {
          let updatedRoles = [...(roleUser.roles || [])]
          
          if (roleUpdate.action === 'added') {
            updatedRoles.push({
              id: roleUpdate.role_id,
              name: roleUpdate.role_name,
              description: '',
              isActive: true,
              assignedAt: new Date()
            })
          } else if (roleUpdate.action === 'removed') {
            updatedRoles = updatedRoles.filter(r => r.id !== roleUpdate.role_id)
          }
          
          const updatedRoleUser = { ...roleUser, roles: updatedRoles }
          addOptimisticUpdate(roleUpdateId, 'update', updatedRoleUser, roleUser)
          setTimeout(() => confirmUpdate(roleUpdateId), 1000)
        }
        break

      case 'UserCreated':
        const newUser: UnifiedUserData = event.data
        const createId = `create_${newUser.id}_${Date.now()}`
        addOptimisticUpdate(createId, 'create', newUser)
        setTimeout(() => confirmUpdate(createId), 1000)
        break

      case 'UserDeleted':
        const deletedUserId = event.data.user_id
        const deleteId = `delete_${deletedUserId}_${Date.now()}`
        const deletedUser = users.find(u => u.id === deletedUserId)
        if (deletedUser) {
          addOptimisticUpdate(deleteId, 'delete', deletedUser)
          setTimeout(() => confirmUpdate(deleteId), 1000)
        }
        break

      case 'connection_established':
        console.log('WebSocket connection established:', event.data)
        break

      default:
        console.log('Unhandled realtime event:', event.type, event.data)
    }
  }, [users, onUserStatusUpdate, onUserProfileUpdate, onUserRoleUpdate, addOptimisticUpdate, confirmUpdate])

  const sendMessage = useCallback((message: any) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(message))
    } else {
      console.warn('WebSocket not connected, cannot send message:', message)
    }
  }, [])

  const ping = useCallback(() => {
    sendMessage({ type: 'ping', timestamp: new Date().toISOString() })
  }, [sendMessage])

  // Auto-connect on mount
  useEffect(() => {
    mountedRef.current = true
    
    if (autoConnect) {
      connect()
    }

    return () => {
      mountedRef.current = false
      disconnect()
    }
  }, [autoConnect, connect, disconnect])

  // Periodic ping to keep connection alive
  useEffect(() => {
    if (!isConnected) return

    const pingInterval = setInterval(ping, 30000) // Ping every 30 seconds

    return () => clearInterval(pingInterval)
  }, [isConnected, ping])

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current)
      }
      disconnect()
    }
  }, [disconnect])

  return {
    // Connection state
    isConnected,
    connectionStatus,
    lastUpdate,
    
    // Data with optimistic updates applied
    users: optimisticUsers,
    pendingUpdates: Array.from(pendingUpdates.values()),
    
    // Control methods
    connect,
    disconnect,
    sendMessage,
    ping,
    
    // Optimistic update controls
    addOptimisticUpdate,
    confirmUpdate,
    rollbackUpdate,
  }
}