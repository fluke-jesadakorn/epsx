// React hook for security monitoring integration
import { useState, useEffect, useCallback, useMemo, useRef } from 'react'
import { 
  SecurityMonitoringService, 
  SecurityEvent, 
  SecurityAlert, 
  ThreatMetrics,
  SecurityReport,
  globalSecurityMonitoring
} from '@/lib/security/security-monitoring-service'

export interface UseSecurityMonitoringOptions {
  autoRefresh?: boolean
  refreshInterval?: number
  realTimeUpdates?: boolean
  eventFilter?: {
    eventType?: string
    severity?: string
    timeRange?: { start: string; end: string }
    userId?: string
    component?: string
  }
  alertFilter?: {
    type?: string
    priority?: string
    resolved?: boolean
    acknowledged?: boolean
  }
  maxEvents?: number
  maxAlerts?: number
}

export interface SecurityMonitoringState {
  events: SecurityEvent[]
  alerts: SecurityAlert[]
  metrics: ThreatMetrics | null
  loading: boolean
  error: string | null
  lastUpdate: Date | null
  isRealTime: boolean
}

export interface SecurityMonitoringActions {
  refreshData: () => Promise<void>
  recordEvent: (event: Partial<SecurityEvent>) => SecurityEvent
  acknowledgeAlert: (alertId: string) => Promise<void>
  resolveAlert: (alertId: string) => Promise<void>
  generateReport: (type: 'daily' | 'weekly' | 'monthly') => SecurityReport
  clearError: () => void
  toggleRealTime: () => void
  updateEventFilter: (filter: UseSecurityMonitoringOptions['eventFilter']) => void
  updateAlertFilter: (filter: UseSecurityMonitoringOptions['alertFilter']) => void
  exportSecurityData: () => void
}

export interface UseSecurityMonitoringReturn {
  state: SecurityMonitoringState
  actions: SecurityMonitoringActions
  statistics: {
    totalEvents: number
    criticalEvents: number
    unresolvedAlerts: number
    riskScore: number
    complianceScore: number
    threatTrends: Array<{
      period: string
      count: number
      severity: string
    }>
  }
}

export const useSecurityMonitoring = (
  options: UseSecurityMonitoringOptions = {}
): UseSecurityMonitoringReturn => {
  const {
    autoRefresh = true,
    refreshInterval = 30000,
    realTimeUpdates = true,
    eventFilter,
    alertFilter,
    maxEvents = 1000,
    maxAlerts = 100
  } = options

  const [state, setState] = useState<SecurityMonitoringState>({
    events: [],
    alerts: [],
    metrics: null,
    loading: true,
    error: null,
    lastUpdate: null,
    isRealTime: realTimeUpdates
  })

  const [currentEventFilter, setCurrentEventFilter] = useState(eventFilter)
  const [currentAlertFilter, setCurrentAlertFilter] = useState(alertFilter)

  // MEMORY LEAK PROTECTION: Track component mount state
  const isMountedRef = useRef(true)

  useEffect(() => {
    isMountedRef.current = true
    return () => {
      isMountedRef.current = false
    }
  }, [])

  // Load security data from monitoring service
  const loadSecurityData = useCallback(async () => {
    try {
      if (!isMountedRef.current) return
      
      setState(prev => ({ ...prev, loading: true, error: null }))

      // Get events with filtering
      const events = globalSecurityMonitoring.getEvents(currentEventFilter)
        .slice(0, maxEvents)

      // Get alerts with filtering
      const alerts = globalSecurityMonitoring.getAlerts(currentAlertFilter)
        .slice(0, maxAlerts)

      // Get metrics
      const metrics = globalSecurityMonitoring.getSecurityMetrics()

      if (isMountedRef.current) {
        setState(prev => ({
          ...prev,
          events,
          alerts,
          metrics,
          loading: false,
          lastUpdate: new Date()
        }))
      }

    } catch (error) {
      if (isMountedRef.current) {
        setState(prev => ({
          ...prev,
          loading: false,
          error: error instanceof Error ? error.message : 'Failed to load security data'
        }))
      }
    }
  }, [currentEventFilter, currentAlertFilter, maxEvents, maxAlerts])

  // Initialize and setup auto-refresh
  useEffect(() => {
    loadSecurityData()

    if (autoRefresh || state.isRealTime) {
      const interval = setInterval(() => {
        if (isMountedRef.current) {
          loadSecurityData()
        }
      }, refreshInterval)
      return () => clearInterval(interval)
    }
  }, [loadSecurityData, autoRefresh, state.isRealTime, refreshInterval])

  // Setup real-time event listeners
  useEffect(() => {
    if (!state.isRealTime) return

    const handleNewEvent = (event: SecurityEvent) => {
      if (!isMountedRef.current) return
      
      setState(prev => {
        // Check if event matches current filter
        const matchesFilter = !currentEventFilter || (
          (!currentEventFilter.eventType || event.eventType === currentEventFilter.eventType) &&
          (!currentEventFilter.severity || event.severity === currentEventFilter.severity) &&
          (!currentEventFilter.userId || event.userId === currentEventFilter.userId) &&
          (!currentEventFilter.component || event.component === currentEventFilter.component)
        )

        if (matchesFilter) {
          const updatedEvents = [event, ...prev.events].slice(0, maxEvents)
          return {
            ...prev,
            events: updatedEvents,
            lastUpdate: new Date()
          }
        }

        return prev
      })
    }

    const handleNewAlert = (alert: SecurityAlert) => {
      if (!isMountedRef.current) return
      
      setState(prev => {
        // Check if alert matches current filter
        const matchesFilter = !currentAlertFilter || (
          (!currentAlertFilter.type || alert.type === currentAlertFilter.type) &&
          (!currentAlertFilter.priority || alert.priority === currentAlertFilter.priority) &&
          (currentAlertFilter.resolved === undefined || alert.resolved === currentAlertFilter.resolved) &&
          (currentAlertFilter.acknowledged === undefined || alert.acknowledged === currentAlertFilter.acknowledged)
        )

        if (matchesFilter) {
          const updatedAlerts = [alert, ...prev.alerts].slice(0, maxAlerts)
          return {
            ...prev,
            alerts: updatedAlerts,
            lastUpdate: new Date()
          }
        }

        return prev
      })
    }

    // Add listeners for all event types
    globalSecurityMonitoring.addEventListener('*', handleNewEvent)
    globalSecurityMonitoring.addAlertListener(handleNewAlert)

    // Cleanup function would need to be implemented in the service
    // For now, we'll handle cleanup manually
    return () => {
      // Note: Real cleanup would require removeListener methods in the service
    }
  }, [state.isRealTime, currentEventFilter, currentAlertFilter, maxEvents, maxAlerts])

  // Actions
  const actions: SecurityMonitoringActions = {
    refreshData: useCallback(async () => {
      await loadSecurityData()
    }, [loadSecurityData]),

    recordEvent: useCallback((eventData: Partial<SecurityEvent>) => {
      const event = globalSecurityMonitoring.recordSecurityEvent(eventData)
      
      // Immediately add to state if it matches the current filter
      if (isMountedRef.current) {
        setState(prev => {
          const matchesFilter = !currentEventFilter || (
            (!currentEventFilter.eventType || event.eventType === currentEventFilter.eventType) &&
            (!currentEventFilter.severity || event.severity === currentEventFilter.severity) &&
            (!currentEventFilter.userId || event.userId === currentEventFilter.userId) &&
            (!currentEventFilter.component || event.component === currentEventFilter.component)
          )

          if (matchesFilter) {
            return {
              ...prev,
              events: [event, ...prev.events].slice(0, maxEvents),
              lastUpdate: new Date()
            }
          }

          return prev
        })
      }

      return event
    }, [currentEventFilter, maxEvents]),

    acknowledgeAlert: useCallback(async (alertId: string) => {
      if (isMountedRef.current) {
        setState(prev => ({
          ...prev,
          alerts: prev.alerts.map(alert =>
            alert.id === alertId ? { ...alert, acknowledged: true } : alert
          )
        }))
      }
    }, []),

    resolveAlert: useCallback(async (alertId: string) => {
      if (isMountedRef.current) {
        setState(prev => ({
          ...prev,
          alerts: prev.alerts.map(alert =>
            alert.id === alertId 
              ? { ...alert, resolved: true, acknowledged: true } 
              : alert
          )
        }))
      }
    }, []),

    generateReport: useCallback((type: 'daily' | 'weekly' | 'monthly') => {
      return globalSecurityMonitoring.generateSecurityReport(type)
    }, []),

    clearError: useCallback(() => {
      if (isMountedRef.current) {
        setState(prev => ({ ...prev, error: null }))
      }
    }, []),

    toggleRealTime: useCallback(() => {
      if (isMountedRef.current) {
        setState(prev => ({ ...prev, isRealTime: !prev.isRealTime }))
      }
    }, []),

    updateEventFilter: useCallback((filter: UseSecurityMonitoringOptions['eventFilter']) => {
      setCurrentEventFilter(filter)
    }, []),

    updateAlertFilter: useCallback((filter: UseSecurityMonitoringOptions['alertFilter']) => {
      setCurrentAlertFilter(filter)
    }, []),

    exportSecurityData: useCallback(() => {
      const data = {
        exportTime: new Date().toISOString(),
        events: state.events,
        alerts: state.alerts,
        metrics: state.metrics,
        filters: {
          events: currentEventFilter,
          alerts: currentAlertFilter
        }
      }

      const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `security-monitoring-export-${new Date().toISOString().split('T')[0]}.json`
      a.click()
      URL.revokeObjectURL(url)
    }, [state.events, state.alerts, state.metrics, currentEventFilter, currentAlertFilter])
  }

  // Calculate statistics
  const statistics = useMemo(() => {
    const criticalEvents = state.events.filter(e => e.severity === 'critical').length
    const unresolvedAlerts = state.alerts.filter(a => !a.resolved).length
    
    return {
      totalEvents: state.events.length,
      criticalEvents,
      unresolvedAlerts,
      riskScore: state.metrics?.riskScore || 0,
      complianceScore: state.metrics?.complianceScore || 0,
      threatTrends: state.metrics?.threatTrends || []
    }
  }, [state.events, state.alerts, state.metrics])

  return {
    state,
    actions,
    statistics
  }
}

// Specialized hooks for specific use cases
export const useSecurityAlerts = (options?: {
  priorityFilter?: 'urgent' | 'high' | 'medium' | 'low'
  unresolvedOnly?: boolean
  maxAlerts?: number
}) => {
  const alertFilter = useMemo(() => ({
    priority: options?.priorityFilter,
    resolved: options?.unresolvedOnly ? false : undefined
  }), [options?.priorityFilter, options?.unresolvedOnly])

  const { state, actions, statistics } = useSecurityMonitoring({
    alertFilter,
    maxAlerts: options?.maxAlerts || 50,
    maxEvents: 0 // Don't load events for alerts-only hook
  })

  return {
    alerts: state.alerts,
    loading: state.loading,
    error: state.error,
    unresolvedCount: statistics.unresolvedAlerts,
    acknowledgeAlert: actions.acknowledgeAlert,
    resolveAlert: actions.resolveAlert,
    refreshAlerts: actions.refreshData,
    clearError: actions.clearError
  }
}

export const useSecurityMetrics = (options?: {
  refreshInterval?: number
}) => {
  const { state, actions, statistics } = useSecurityMonitoring({
    refreshInterval: options?.refreshInterval || 60000,
    maxEvents: 0,
    maxAlerts: 0
  })

  return {
    metrics: state.metrics,
    statistics,
    loading: state.loading,
    error: state.error,
    lastUpdate: state.lastUpdate,
    refreshMetrics: actions.refreshData,
    clearError: actions.clearError
  }
}

export const useSecurityEvents = (options?: {
  eventType?: string
  severity?: string
  component?: string
  maxEvents?: number
}) => {
  const eventFilter = useMemo(() => ({
    eventType: options?.eventType,
    severity: options?.severity,
    component: options?.component
  }), [options?.eventType, options?.severity, options?.component])

  const { state, actions } = useSecurityMonitoring({
    eventFilter,
    maxEvents: options?.maxEvents || 100,
    maxAlerts: 0 // Don't load alerts for events-only hook
  })

  return {
    events: state.events,
    loading: state.loading,
    error: state.error,
    lastUpdate: state.lastUpdate,
    recordEvent: actions.recordEvent,
    refreshEvents: actions.refreshData,
    updateFilter: actions.updateEventFilter,
    clearError: actions.clearError
  }
}

// Legacy compatibility hooks (keep existing API for backward compatibility)
export const useSecurityNotifications = () => {
  const [notifications, setNotifications] = useState<Array<{
    id: string
    type: 'success' | 'error' | 'warning' | 'info'
    title: string
    message: string
    timestamp: Date
  }>>([])

  // MEMORY LEAK PROTECTION: Track component mount state and active timers
  const isMountedRef = useRef(true)
  const activeTimersRef = useRef<Set<number>>(new Set())

  // MEMORY LEAK PROTECTION: Cleanup on unmount
  useEffect(() => {
    isMountedRef.current = true
    
    return () => {
      isMountedRef.current = false
      
      // Clear all active timers
      activeTimersRef.current.forEach(timer => {
        clearTimeout(timer)
      })
      activeTimersRef.current.clear()
    }
  }, [])

  const addNotification = useCallback((notification: {
    type: 'success' | 'error' | 'warning' | 'info'
    title: string
    message: string
  }) => {
    const id = Date.now().toString()
    
    // Only update state if component is mounted
    if (isMountedRef.current) {
      setNotifications(prev => [...prev, {
        ...notification,
        id,
        timestamp: new Date(),
      }])

      // MEMORY LEAK FIX: Safe timeout with cleanup tracking
      const timer = setTimeout(() => {
        activeTimersRef.current.delete(timer as any)
        if (isMountedRef.current) {
          setNotifications(prev => prev.filter(n => n.id !== id))
        }
      }, 10000)
      
      activeTimersRef.current.add(timer as any)
    }
  }, [])

  const removeNotification = useCallback((id: string) => {
    if (isMountedRef.current) {
      setNotifications(prev => prev.filter(n => n.id !== id))
    }
  }, [])

  const clearNotifications = useCallback(() => {
    if (isMountedRef.current) {
      setNotifications([])
    }
  }, [])

  return {
    notifications,
    addNotification,
    removeNotification,
    clearNotifications,
  }
}