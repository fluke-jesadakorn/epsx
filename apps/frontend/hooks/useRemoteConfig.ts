'use client'

/**
 * React Hooks for Firebase Remote Config
 * Provides easy-to-use hooks for accessing remote configuration
 */

import { useState, useEffect, useCallback } from 'react'
import { 
  fetchRemoteConfig, 
  getAllRemoteSettings, 
  getRemoteConfigValue, 
  getRemoteConfigStatus,
  defaultConfig,
  type RemoteUserSettings,
  type UXSettings,
  type PerformanceSettings,
  type FeatureFlags,
  type BusinessSettings,
  type ABTestSettings
} from '@/lib/remote-config'

// ============================================================================
// Main Remote Config Hook
// ============================================================================

/**
 * Main hook for accessing all Remote Config functionality
 */
export function useRemoteConfig() {
  const [settings, setSettings] = useState<RemoteUserSettings>(defaultConfig)
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [lastFetchTime, setLastFetchTime] = useState<Date | null>(null)
  const [isRefreshing, setIsRefreshing] = useState(false)

  // Fetch and update settings
  const refreshSettings = useCallback(async () => {
    try {
      setIsRefreshing(true)
      setError(null)
      
      const activated = await fetchRemoteConfig()
      const newSettings = getAllRemoteSettings()
      const status = getRemoteConfigStatus()
      
      setSettings(newSettings)
      setLastFetchTime(status.lastFetchTime)
      
      console.log('🔄 Remote Config refreshed:', {
        activated,
        settingsUpdated: activated,
        lastFetch: status.lastFetchTime
      })
      
    } catch (err) {
      console.error('❌ Error refreshing Remote Config:', err)
      setError(err instanceof Error ? err.message : 'Failed to refresh settings')
    } finally {
      setIsRefreshing(false)
    }
  }, [])

  // Initial load
  useEffect(() => {
    const loadInitialSettings = async () => {
      try {
        setIsLoading(true)
        
        // Load current settings immediately (cached values)
        const currentSettings = getAllRemoteSettings()
        setSettings(currentSettings)
        
        const status = getRemoteConfigStatus()
        setLastFetchTime(status.lastFetchTime)
        
        // Fetch fresh settings in background
        await refreshSettings()
        
      } catch (err) {
        console.error('❌ Error loading initial Remote Config:', err)
        setError(err instanceof Error ? err.message : 'Failed to load settings')
      } finally {
        setIsLoading(false)
      }
    }

    loadInitialSettings()
  }, [refreshSettings])

  return {
    settings,
    isLoading,
    error,
    lastFetchTime,
    isRefreshing,
    refreshSettings
  }
}

// ============================================================================
// Specialized Hooks for Different Setting Categories
// ============================================================================

/**
 * Hook for UX/UI settings
 */
export function useUXSettings() {
  const { settings, isLoading, refreshSettings } = useRemoteConfig()
  
  return {
    uxSettings: settings.ux,
    isLoading,
    refreshSettings,
    // Convenience getters
    theme: settings.ux.theme,
    compactMode: settings.ux.compactMode,
    animationsEnabled: settings.ux.animationsEnabled,
    mobileOptimized: settings.ux.mobileOptimized,
    accessibilityMode: settings.ux.accessibilityMode
  }
}

/**
 * Hook for performance settings
 */
export function usePerformanceSettings() {
  const { settings, isLoading, refreshSettings } = useRemoteConfig()
  
  return {
    performanceSettings: settings.performance,
    isLoading,
    refreshSettings,
    // Convenience getters
    refreshInterval: settings.performance.refreshInterval,
    realTimeUpdates: settings.performance.realTimeUpdates,
    dataCacheMinutes: settings.performance.dataCacheMinutes,
    lazyLoadImages: settings.performance.lazyLoadImages,
    preloadNextPage: settings.performance.preloadNextPage
  }
}

/**
 * Hook for feature flags
 */
export function useFeatureFlags() {
  const { settings, isLoading, refreshSettings } = useRemoteConfig()
  
  return {
    featureFlags: settings.features,
    isLoading,
    refreshSettings,
    // Convenience getters
    advancedChartsEnabled: settings.features.advancedChartsEnabled,
    betaAnalyticsEnabled: settings.features.betaAnalyticsEnabled,
    experimentalUIEnabled: settings.features.experimentalUIEnabled,
    mobileAppPromotion: settings.features.mobileAppPromotion,
    newNotificationSystem: settings.features.newNotificationSystem
  }
}

/**
 * Hook for business settings
 */
export function useBusinessSettings() {
  const { settings, isLoading, refreshSettings } = useRemoteConfig()
  
  return {
    businessSettings: settings.business,
    isLoading,
    refreshSettings,
    // Convenience getters
    stocksPerPage: settings.business.stocksPerPage,
    defaultChartType: settings.business.defaultChartType,
    maxWatchlistItems: settings.business.maxWatchlistItems,
    premiumFeaturesEnabled: settings.business.premiumFeaturesEnabled,
    dataExportEnabled: settings.business.dataExportEnabled
  }
}

/**
 * Hook for A/B testing settings
 */
export function useABTestSettings() {
  const { settings, isLoading, refreshSettings } = useRemoteConfig()
  
  return {
    abTestSettings: settings.abTests,
    isLoading,
    refreshSettings,
    // Convenience getters
    homepageLayout: settings.abTests.homepageLayout,
    paymentFlow: settings.abTests.paymentFlow,
    onboardingFlow: settings.abTests.onboardingFlow,
    dashboardStyle: settings.abTests.dashboardStyle
  }
}

// ============================================================================
// Utility Hooks
// ============================================================================

/**
 * Hook for checking if a specific feature is enabled
 */
export function useFeatureFlag(flagName: keyof FeatureFlags) {
  const { featureFlags, isLoading } = useFeatureFlags()
  
  return {
    isEnabled: featureFlags[flagName],
    isLoading
  }
}

/**
 * Hook for getting a specific remote config value
 */
export function useRemoteConfigValue(key: string) {
  const [value, setValue] = useState<string | boolean | number>('')
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    try {
      const configValue = getRemoteConfigValue(key)
      setValue(configValue)
    } catch (error) {
      console.error(`Error getting remote config value for "${key}":`, error)
    } finally {
      setIsLoading(false)
    }
  }, [key])

  return { value, isLoading }
}

/**
 * Hook for Remote Config status and debugging
 */
export function useRemoteConfigStatus() {
  const [status, setStatus] = useState({
    isReady: false,
    lastFetchTime: null as Date | null,
    lastFetchStatus: 'unknown',
    activeConfig: false
  })

  useEffect(() => {
    const updateStatus = () => {
      const configStatus = getRemoteConfigStatus()
      setStatus(configStatus)
    }

    updateStatus()

    // Update status every 30 seconds
    const interval = setInterval(updateStatus, 30000)

    return () => clearInterval(interval)
  }, [])

  return status
}

// ============================================================================
// Theme Integration Hook
// ============================================================================

/**
 * Hook for integrating Remote Config theme with next-themes
 */
export function useRemoteTheme() {
  const { theme, isLoading } = useUXSettings()
  const [effectiveTheme, setEffectiveTheme] = useState(theme)

  useEffect(() => {
    if (!isLoading) {
      if (theme === 'auto') {
        // Auto theme: use system preference
        const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
        setEffectiveTheme(mediaQuery.matches ? 'dark' : 'light')
        
        const handleChange = (e: MediaQueryListEvent) => {
          setEffectiveTheme(e.matches ? 'dark' : 'light')
        }
        
        mediaQuery.addEventListener('change', handleChange)
        return () => mediaQuery.removeEventListener('change', handleChange)
      } else {
        setEffectiveTheme(theme)
      }
    }
  }, [theme, isLoading])

  return {
    theme: effectiveTheme,
    originalTheme: theme,
    isLoading
  }
}