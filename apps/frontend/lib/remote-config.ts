'use client'

/**
 * Firebase Remote Config Service
 * Handles dynamic user settings and feature flags
 */

import { remoteConfig } from './firebase'
import { fetchAndActivate, getValue, getAll } from 'firebase/remote-config'
import { logger, devLog, safeError } from '@/lib/logger'

// ============================================================================
// User Settings Types & Interfaces
// ============================================================================

export interface UXSettings {
  theme: 'light' | 'dark' | 'pancake' | 'cyberpunk' | 'auto'
  compactMode: boolean
  animationsEnabled: boolean
  mobileOptimized: boolean
  accessibilityMode: boolean
}

export interface PerformanceSettings {
  refreshInterval: 15000 | 30000 | 60000
  realTimeUpdates: boolean
  dataCacheMinutes: 5 | 10 | 15 | 30
  lazyLoadImages: boolean
  preloadNextPage: boolean
}

export interface FeatureFlags {
  advancedChartsEnabled: boolean
  betaAnalyticsEnabled: boolean
  experimentalUIEnabled: boolean
  mobileAppPromotion: boolean
  newNotificationSystem: boolean
}

export interface BusinessSettings {
  stocksPerPage: 10 | 25 | 50 | 100
  defaultChartType: 'candlestick' | 'line' | 'area'
  maxWatchlistItems: number
  premiumFeaturesEnabled: boolean
  dataExportEnabled: boolean
}

export interface ABTestSettings {
  homepageLayout: 'original' | 'variant_a' | 'variant_b'
  paymentFlow: 'standard' | 'simplified' | 'premium'
  onboardingFlow: 'standard' | 'gamified' | 'minimal'
  dashboardStyle: 'cards' | 'table' | 'hybrid'
}

export interface RemoteUserSettings {
  ux: UXSettings
  performance: PerformanceSettings
  features: FeatureFlags
  business: BusinessSettings
  abTests: ABTestSettings
}

// ============================================================================
// Default Configuration Values
// ============================================================================

export const defaultConfig: RemoteUserSettings = {
  ux: {
    theme: 'pancake',
    compactMode: false,
    animationsEnabled: true,
    mobileOptimized: true,
    accessibilityMode: false
  },
  performance: {
    refreshInterval: 30000,
    realTimeUpdates: true,
    dataCacheMinutes: 10,
    lazyLoadImages: true,
    preloadNextPage: false
  },
  features: {
    advancedChartsEnabled: false,
    betaAnalyticsEnabled: false,
    experimentalUIEnabled: false,
    mobileAppPromotion: true,
    newNotificationSystem: false
  },
  business: {
    stocksPerPage: 25,
    defaultChartType: 'candlestick',
    maxWatchlistItems: 50,
    premiumFeaturesEnabled: true,
    dataExportEnabled: true
  },
  abTests: {
    homepageLayout: 'original',
    paymentFlow: 'standard',
    onboardingFlow: 'standard',
    dashboardStyle: 'cards'
  }
}

/**
 * Initialize Remote Config defaults (lazy initialization)
 * Called only when Remote Config is actually used to prevent startup errors
 */
function initializeRemoteConfigDefaults(): boolean {
  try {
    // Check if Remote Config is available and not already initialized
    if (!remoteConfig || typeof remoteConfig !== 'object') {
      if (process.env.NODE_ENV === 'development') {
        console.warn('⚠️ Remote Config not available - using fallback defaults')
      }
      return false
    }

    // Check if defaults are already set to avoid overwriting
    if (remoteConfig.defaultConfig && Object.keys(remoteConfig.defaultConfig).length > 0) {
      return true
    }

    // Set default values for Remote Config
    remoteConfig.defaultConfig = {
      // UX Settings
      'ux_theme': defaultConfig.ux.theme,
      'ux_compact_mode': defaultConfig.ux.compactMode,
      'ux_animations_enabled': defaultConfig.ux.animationsEnabled,
      'ux_mobile_optimized': defaultConfig.ux.mobileOptimized,
      'ux_accessibility_mode': defaultConfig.ux.accessibilityMode,
      
      // Performance Settings
      'performance_refresh_interval': defaultConfig.performance.refreshInterval,
      'performance_realtime_updates': defaultConfig.performance.realTimeUpdates,
      'performance_cache_minutes': defaultConfig.performance.dataCacheMinutes,
      'performance_lazy_load': defaultConfig.performance.lazyLoadImages,
      'performance_preload_next': defaultConfig.performance.preloadNextPage,
      
      // Feature Flags
      'features_advanced_charts': defaultConfig.features.advancedChartsEnabled,
      'features_beta_analytics': defaultConfig.features.betaAnalyticsEnabled,
      'features_experimental_ui': defaultConfig.features.experimentalUIEnabled,
      'features_mobile_app_promo': defaultConfig.features.mobileAppPromotion,
      'features_new_notifications': defaultConfig.features.newNotificationSystem,
      
      // Business Settings
      'business_stocks_per_page': defaultConfig.business.stocksPerPage,
      'business_chart_type': defaultConfig.business.defaultChartType,
      'business_max_watchlist': defaultConfig.business.maxWatchlistItems,
      'business_premium_features': defaultConfig.business.premiumFeaturesEnabled,
      'business_data_export': defaultConfig.business.dataExportEnabled,
      
      // A/B Testing
      'ab_homepage_layout': defaultConfig.abTests.homepageLayout,
      'ab_payment_flow': defaultConfig.abTests.paymentFlow,
      'ab_onboarding_flow': defaultConfig.abTests.onboardingFlow,
      'ab_dashboard_style': defaultConfig.abTests.dashboardStyle
    }
    
    if (process.env.NODE_ENV === 'development') {
      console.log('✅ Firebase Remote Config default values set successfully')
    }
    return true
  } catch (error) {
    console.warn('⚠️ Failed to set Remote Config default values:', error)
    return false
  }
}

// ============================================================================
// Remote Config Service Functions
// ============================================================================

/**
 * Fetch and activate remote configuration
 */
export async function fetchRemoteConfig(): Promise<boolean> {
  try {
    if (!remoteConfig) {
      devLog('Remote Config not initialized - skipping fetch')
      return false
    }
    
    // Initialize defaults before fetching
    initializeRemoteConfigDefaults()
    
    devLog('Fetching Firebase Remote Config...')
    
    const activated = await fetchAndActivate(remoteConfig)
    
    if (activated) {
      devLog('Remote Config fetched and activated successfully')
    } else {
      devLog('Remote Config fetched but not activated (no changes)')
    }
    
    return activated
  } catch (error) {
    logger.error('Failed to fetch Remote Config', error)
    return false
  }
}

/**
 * Get all remote configuration values as structured settings
 */
export function getAllRemoteSettings(): RemoteUserSettings {
  try {
    if (!remoteConfig) {
      // Remote Config not initialized, return default config
      return defaultConfig
    }
    
    // Initialize defaults before getting values
    initializeRemoteConfigDefaults()
    
    const allValues = getAll(remoteConfig)
    
    return {
      ux: {
        theme: allValues.ux_theme?.asString() as UXSettings['theme'] || defaultConfig.ux.theme,
        compactMode: allValues.ux_compact_mode?.asBoolean() ?? defaultConfig.ux.compactMode,
        animationsEnabled: allValues.ux_animations_enabled?.asBoolean() ?? defaultConfig.ux.animationsEnabled,
        mobileOptimized: allValues.ux_mobile_optimized?.asBoolean() ?? defaultConfig.ux.mobileOptimized,
        accessibilityMode: allValues.ux_accessibility_mode?.asBoolean() ?? defaultConfig.ux.accessibilityMode
      },
      performance: {
        refreshInterval: allValues.performance_refresh_interval?.asNumber() as PerformanceSettings['refreshInterval'] || defaultConfig.performance.refreshInterval,
        realTimeUpdates: allValues.performance_realtime_updates?.asBoolean() ?? defaultConfig.performance.realTimeUpdates,
        dataCacheMinutes: allValues.performance_cache_minutes?.asNumber() as PerformanceSettings['dataCacheMinutes'] || defaultConfig.performance.dataCacheMinutes,
        lazyLoadImages: allValues.performance_lazy_load?.asBoolean() ?? defaultConfig.performance.lazyLoadImages,
        preloadNextPage: allValues.performance_preload_next?.asBoolean() ?? defaultConfig.performance.preloadNextPage
      },
      features: {
        advancedChartsEnabled: allValues.features_advanced_charts?.asBoolean() ?? defaultConfig.features.advancedChartsEnabled,
        betaAnalyticsEnabled: allValues.features_beta_analytics?.asBoolean() ?? defaultConfig.features.betaAnalyticsEnabled,
        experimentalUIEnabled: allValues.features_experimental_ui?.asBoolean() ?? defaultConfig.features.experimentalUIEnabled,
        mobileAppPromotion: allValues.features_mobile_app_promo?.asBoolean() ?? defaultConfig.features.mobileAppPromotion,
        newNotificationSystem: allValues.features_new_notifications?.asBoolean() ?? defaultConfig.features.newNotificationSystem
      },
      business: {
        stocksPerPage: allValues.business_stocks_per_page?.asNumber() as BusinessSettings['stocksPerPage'] || defaultConfig.business.stocksPerPage,
        defaultChartType: allValues.business_chart_type?.asString() as BusinessSettings['defaultChartType'] || defaultConfig.business.defaultChartType,
        maxWatchlistItems: allValues.business_max_watchlist?.asNumber() ?? defaultConfig.business.maxWatchlistItems,
        premiumFeaturesEnabled: allValues.business_premium_features?.asBoolean() ?? defaultConfig.business.premiumFeaturesEnabled,
        dataExportEnabled: allValues.business_data_export?.asBoolean() ?? defaultConfig.business.dataExportEnabled
      },
      abTests: {
        homepageLayout: allValues.ab_homepage_layout?.asString() as ABTestSettings['homepageLayout'] || defaultConfig.abTests.homepageLayout,
        paymentFlow: allValues.ab_payment_flow?.asString() as ABTestSettings['paymentFlow'] || defaultConfig.abTests.paymentFlow,
        onboardingFlow: allValues.ab_onboarding_flow?.asString() as ABTestSettings['onboardingFlow'] || defaultConfig.abTests.onboardingFlow,
        dashboardStyle: allValues.ab_dashboard_style?.asString() as ABTestSettings['dashboardStyle'] || defaultConfig.abTests.dashboardStyle
      }
    }
  } catch (error) {
    logger.error('Failed to get Remote Config values', error)
    return defaultConfig
  }
}

/**
 * Get specific remote config value by key
 */
export function getRemoteConfigValue(key: string): string | boolean | number {
  try {
    if (!remoteConfig) {
      // Remote Config not initialized, return empty string
      return ''
    }
    
    // Initialize defaults before getting value
    initializeRemoteConfigDefaults()
    
    const value = getValue(remoteConfig, key)
    
    // Try to parse as boolean first
    if (value.asString().toLowerCase() === 'true') return true
    if (value.asString().toLowerCase() === 'false') return false
    
    // Try to parse as number
    const numValue = value.asNumber()
    if (!isNaN(numValue) && isFinite(numValue)) return numValue
    
    // Return as string
    return value.asString()
  } catch (error) {
    logger.error(`Failed to get Remote Config value for key "${key}"`, error)
    return ''
  }
}

/**
 * Check if Remote Config is initialized and ready
 */
export function isRemoteConfigReady(): boolean {
  try {
    return remoteConfig !== null && remoteConfig !== undefined
  } catch (error) {
    return false
  }
}

/**
 * Get Remote Config fetch timestamp
 */
export function getLastFetchTime(): Date | null {
  try {
    if (!remoteConfig) {
      return null
    }
    
    // Initialize defaults safely
    initializeRemoteConfigDefaults()
    
    const timestamp = remoteConfig.fetchTimeMillis
    return timestamp ? new Date(timestamp) : null
  } catch (error) {
    logger.error('Failed to get Remote Config fetch time', error)
    return null
  }
}

/**
 * Get Remote Config status information
 */
export function getRemoteConfigStatus(): {
  isReady: boolean
  lastFetchTime: Date | null
  lastFetchStatus: string
  activeConfig: boolean
} {
  try {
    if (!remoteConfig) {
      return {
        isReady: false,
        lastFetchTime: null,
        lastFetchStatus: 'not_initialized',
        activeConfig: false
      }
    }
    
    // Initialize defaults safely
    initializeRemoteConfigDefaults()
    
    return {
      isReady: isRemoteConfigReady(),
      lastFetchTime: getLastFetchTime(),
      lastFetchStatus: remoteConfig.lastFetchStatus,
      activeConfig: Object.keys(getAll(remoteConfig)).length > 0
    }
  } catch (error) {
    logger.error('Failed to get Remote Config status', error)
    return {
      isReady: false,
      lastFetchTime: null,
      lastFetchStatus: 'error',
      activeConfig: false
    }
  }
}