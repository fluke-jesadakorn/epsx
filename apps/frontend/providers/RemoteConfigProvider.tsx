'use client'

/**
 * Remote Config Context Provider
 * Provides Remote Config settings throughout the application
 */

import { createContext, useContext, useEffect, useState, ReactNode } from 'react'
import { 
  fetchRemoteConfig, 
  getAllRemoteSettings, 
  getRemoteConfigStatus,
  defaultConfig,
  type RemoteUserSettings 
} from '@/lib/remote-config'
import { logger, devLog, safeError } from '@/lib/logger'

// ============================================================================
// Context Definition
// ============================================================================

interface RemoteConfigContextType {
  settings: RemoteUserSettings
  isLoading: boolean
  error: string | null
  lastFetchTime: Date | null
  isRefreshing: boolean
  refreshSettings: () => Promise<void>
  status: {
    isReady: boolean
    lastFetchTime: Date | null
    lastFetchStatus: string
    activeConfig: boolean
  }
}

const RemoteConfigContext = createContext<RemoteConfigContextType | null>(null)

// ============================================================================
// Provider Component
// ============================================================================

interface RemoteConfigProviderProps {
  children: ReactNode
  /**
   * Auto-refresh interval in milliseconds
   * Set to 0 to disable auto-refresh
   * Default: 5 minutes (300000ms)
   */
  autoRefreshInterval?: number
  /**
   * Whether to fetch config immediately on mount
   * Default: true
   */
  fetchOnMount?: boolean
}

export function RemoteConfigProvider({ 
  children, 
  autoRefreshInterval = 300000, // 5 minutes
  fetchOnMount = true 
}: RemoteConfigProviderProps) {
  const [settings, setSettings] = useState<RemoteUserSettings>(defaultConfig)
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [lastFetchTime, setLastFetchTime] = useState<Date | null>(null)
  const [isRefreshing, setIsRefreshing] = useState(false)
  const [status, setStatus] = useState({
    isReady: false,
    lastFetchTime: null as Date | null,
    lastFetchStatus: 'unknown',
    activeConfig: false
  })

  // Update status
  const updateStatus = () => {
    const configStatus = getRemoteConfigStatus()
    setStatus(configStatus)
  }

  // Refresh settings function
  const refreshSettings = async () => {
    try {
      setIsRefreshing(true)
      setError(null)
      
      devLog('RemoteConfigProvider: Fetching Remote Config...')
      
      const activated = await fetchRemoteConfig()
      const newSettings = getAllRemoteSettings()
      
      setSettings(newSettings)
      
      // Update status and timestamp
      updateStatus()
      const configStatus = getRemoteConfigStatus()
      setLastFetchTime(configStatus.lastFetchTime)
      
      devLog('RemoteConfigProvider: Settings updated', {
        activated,
        settingsCount: Object.keys(newSettings).length,
        lastFetch: configStatus.lastFetchTime
      })
      
    } catch (err) {
      logger.error('RemoteConfigProvider: Error refreshing settings', err)
      setError(err instanceof Error ? err.message : 'Failed to refresh Remote Config')
    } finally {
      setIsRefreshing(false)
    }
  }

  // Initial setup
  useEffect(() => {
    const initialize = async () => {
      try {
        setIsLoading(true)
        
        // Load cached settings immediately
        const currentSettings = getAllRemoteSettings()
        setSettings(currentSettings)
        updateStatus()
        
        devLog('RemoteConfigProvider: Initialized with cached settings')
        
        // Fetch fresh settings if enabled
        if (fetchOnMount) {
          await refreshSettings()
        }
        
      } catch (err) {
        logger.error('RemoteConfigProvider: Initialization error', err)
        setError(err instanceof Error ? err.message : 'Failed to initialize Remote Config')
      } finally {
        setIsLoading(false)
      }
    }

    initialize()
  }, [fetchOnMount])

  // Auto-refresh setup
  useEffect(() => {
    if (autoRefreshInterval > 0) {
      devLog(`RemoteConfigProvider: Setting up auto-refresh every ${autoRefreshInterval / 1000}s`)
      
      const interval = setInterval(() => {
        if (!isRefreshing) {
          devLog('RemoteConfigProvider: Auto-refresh triggered')
          refreshSettings()
        }
      }, autoRefreshInterval)

      return () => {
        devLog('RemoteConfigProvider: Cleaning up auto-refresh interval')
        clearInterval(interval)
      }
    }
  }, [autoRefreshInterval, isRefreshing])

  // Status update interval
  useEffect(() => {
    const statusInterval = setInterval(updateStatus, 30000) // Update status every 30 seconds

    return () => clearInterval(statusInterval)
  }, [])

  // Context value
  const contextValue: RemoteConfigContextType = {
    settings,
    isLoading,
    error,
    lastFetchTime,
    isRefreshing,
    refreshSettings,
    status
  }

  return (
    <RemoteConfigContext.Provider value={contextValue}>
      {children}
    </RemoteConfigContext.Provider>
  )
}

// ============================================================================
// Context Hook
// ============================================================================

/**
 * Hook to use Remote Config context
 * Must be used within RemoteConfigProvider
 */
export function useRemoteConfigContext(): RemoteConfigContextType {
  const context = useContext(RemoteConfigContext)
  
  if (!context) {
    throw new Error('useRemoteConfigContext must be used within RemoteConfigProvider')
  }
  
  return context
}

// ============================================================================
// Convenience Context Hooks
// ============================================================================

/**
 * Hook for UX settings from context
 */
export function useUXSettingsContext() {
  const { settings, isLoading, refreshSettings } = useRemoteConfigContext()
  
  return {
    uxSettings: settings.ux,
    isLoading,
    refreshSettings,
    theme: settings.ux.theme,
    compactMode: settings.ux.compactMode,
    animationsEnabled: settings.ux.animationsEnabled,
    mobileOptimized: settings.ux.mobileOptimized,
    accessibilityMode: settings.ux.accessibilityMode
  }
}

/**
 * Hook for performance settings from context
 */
export function usePerformanceSettingsContext() {
  const { settings, isLoading, refreshSettings } = useRemoteConfigContext()
  
  return {
    performanceSettings: settings.performance,
    isLoading,
    refreshSettings,
    refreshInterval: settings.performance.refreshInterval,
    realTimeUpdates: settings.performance.realTimeUpdates,
    dataCacheMinutes: settings.performance.dataCacheMinutes,
    lazyLoadImages: settings.performance.lazyLoadImages,
    preloadNextPage: settings.performance.preloadNextPage
  }
}

/**
 * Hook for feature flags from context
 */
export function useFeatureFlagsContext() {
  const { settings, isLoading, refreshSettings } = useRemoteConfigContext()
  
  return {
    featureFlags: settings.features,
    isLoading,
    refreshSettings,
    advancedChartsEnabled: settings.features.advancedChartsEnabled,
    betaAnalyticsEnabled: settings.features.betaAnalyticsEnabled,
    experimentalUIEnabled: settings.features.experimentalUIEnabled,
    mobileAppPromotion: settings.features.mobileAppPromotion,
    newNotificationSystem: settings.features.newNotificationSystem
  }
}

/**
 * Hook for business settings from context
 */
export function useBusinessSettingsContext() {
  const { settings, isLoading, refreshSettings } = useRemoteConfigContext()
  
  return {
    businessSettings: settings.business,
    isLoading,
    refreshSettings,
    stocksPerPage: settings.business.stocksPerPage,
    defaultChartType: settings.business.defaultChartType,
    maxWatchlistItems: settings.business.maxWatchlistItems,
    premiumFeaturesEnabled: settings.business.premiumFeaturesEnabled,
    dataExportEnabled: settings.business.dataExportEnabled
  }
}

/**
 * Hook for A/B test settings from context
 */
export function useABTestSettingsContext() {
  const { settings, isLoading, refreshSettings } = useRemoteConfigContext()
  
  return {
    abTestSettings: settings.abTests,
    isLoading,
    refreshSettings,
    homepageLayout: settings.abTests.homepageLayout,
    paymentFlow: settings.abTests.paymentFlow,
    onboardingFlow: settings.abTests.onboardingFlow,
    dashboardStyle: settings.abTests.dashboardStyle
  }
}

/**
 * Hook for checking specific feature flag from context
 */
export function useFeatureFlagContext(flagName: keyof RemoteUserSettings['features']) {
  const { settings, isLoading } = useRemoteConfigContext()
  
  return {
    isEnabled: settings.features[flagName],
    isLoading
  }
}