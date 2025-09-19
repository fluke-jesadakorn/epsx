/**
 * Firebase Admin Remote Config Service
 * Provides administrative control over Remote Config parameters
 * 
 * Note: This uses the Firebase Admin SDK and requires server-side execution
 * or proper authentication with Firebase project admin privileges
 */

// ============================================================================
// Types & Interfaces
// ============================================================================

export interface RemoteConfigParameter {
  key: string
  value: string | number | boolean
  valueType: 'string' | 'number' | 'boolean' | 'json'
  description: string
  conditionalValues?: ConditionalValue[]
  lastModified?: Date
  modifiedBy?: string
}

export interface ConditionalValue {
  conditionName: string
  value: string | number | boolean
}

export interface UserCondition {
  name: string
  expression: string
  description: string
  tagColor?: string
}

export interface RemoteConfigTemplate {
  etag: string
  parameters: Record<string, RemoteConfigParameter>
  conditions: UserCondition[]
  version?: {
    versionNumber: string
    updateTime: string
    updateUser: {
      email: string
    }
    description: string
    updateOrigin: string
    updateType: string
  }
}

export interface ParameterGroup {
  name: string
  description: string
  parameters: string[]
  color?: string
}

// ============================================================================
// Parameter Categories
// ============================================================================

export const PARAMETER_CATEGORIES = {
  UX: {
    name: 'User Experience',
    description: 'Appearance, theme, and UI behavior settings',
    color: 'purple',
    parameters: [
      'ux_theme',
      'ux_compact_mode', 
      'ux_animations_enabled',
      'ux_mobile_optimized',
      'ux_accessibility_mode'
    ]
  },
  PERFORMANCE: {
    name: 'Performance & Speed',
    description: 'Data refresh rates, caching, and optimization settings',
    color: 'blue',
    parameters: [
      'performance_refresh_interval',
      'performance_realtime_updates',
      'performance_cache_minutes',
      'performance_lazy_load',
      'performance_preload_next'
    ]
  },
  FEATURES: {
    name: 'Feature Flags',
    description: 'Enable/disable features and experimental functionality',
    color: 'green',
    parameters: [
      'features_advanced_charts',
      'features_beta_analytics',
      'features_experimental_ui',
      'features_mobile_app_promo',
      'features_new_notifications'
    ]
  },
  BUSINESS: {
    name: 'Business Logic',
    description: 'Display limits, data presentation, and business rules',
    color: 'orange',
    parameters: [
      'business_stocks_per_page',
      'business_chart_type',
      'business_max_watchlist',
      'business_premium_features',
      'business_data_export'
    ]
  },
  AB_TESTING: {
    name: 'A/B Testing',
    description: 'Experimental variants and user experience tests',
    color: 'pink',
    parameters: [
      'ab_homepage_layout',
      'ab_payment_flow', 
      'ab_onboarding_flow',
      'ab_dashboard_style'
    ]
  },
  USER_TARGETING: {
    name: 'User Targeting',
    description: 'User segmentation and targeting configurations',
    color: 'indigo',
    parameters: [
      'targeting_enable_segments',
      'targeting_rollout_percentage',
      'targeting_premium_features',
      'targeting_mobile_features'
    ]
  }
} as const

// ============================================================================
// Default Parameters Configuration
// ============================================================================

export const DEFAULT_PARAMETERS: Record<string, RemoteConfigParameter> = {
  // UX Settings
  ux_theme: {
    key: 'ux_theme',
    value: 'pancake',
    valueType: 'string',
    description: 'Default UI theme for new users'
  },
  ux_compact_mode: {
    key: 'ux_compact_mode',
    value: false,
    valueType: 'boolean',
    description: 'Enable compact layout by default'
  },
  ux_animations_enabled: {
    key: 'ux_animations_enabled',
    value: false,
    valueType: 'boolean',
    description: 'Enable UI animations and transitions (disabled for performance)'
  },
  ux_mobile_optimized: {
    key: 'ux_mobile_optimized',
    value: true,
    valueType: 'boolean',
    description: 'Apply mobile-specific optimizations'
  },
  ux_accessibility_mode: {
    key: 'ux_accessibility_mode',
    value: false,
    valueType: 'boolean',
    description: 'Enhanced accessibility features'
  },

  // Performance Settings
  performance_refresh_interval: {
    key: 'performance_refresh_interval',
    value: 30000,
    valueType: 'number',
    description: 'Data refresh interval in milliseconds'
  },
  performance_realtime_updates: {
    key: 'performance_realtime_updates',
    value: true,
    valueType: 'boolean',
    description: 'Enable real-time data streaming'
  },
  performance_cache_minutes: {
    key: 'performance_cache_minutes',
    value: 10,
    valueType: 'number',
    description: 'Local data cache duration in minutes'
  },
  performance_lazy_load: {
    key: 'performance_lazy_load',
    value: true,
    valueType: 'boolean',
    description: 'Enable lazy loading for images and components'
  },
  performance_preload_next: {
    key: 'performance_preload_next',
    value: false,
    valueType: 'boolean',
    description: 'Preload next page data'
  },

  // Feature Flags
  features_advanced_charts: {
    key: 'features_advanced_charts',
    value: false,
    valueType: 'boolean',
    description: 'Enable advanced charting features'
  },
  features_beta_analytics: {
    key: 'features_beta_analytics',
    value: false,
    valueType: 'boolean',
    description: 'Enable beta analytics features'
  },
  features_experimental_ui: {
    key: 'features_experimental_ui',
    value: false,
    valueType: 'boolean',
    description: 'Enable experimental UI components'
  },
  features_mobile_app_promo: {
    key: 'features_mobile_app_promo',
    value: true,
    valueType: 'boolean',
    description: 'Show mobile app promotion banners'
  },
  features_new_notifications: {
    key: 'features_new_notifications',
    value: false,
    valueType: 'boolean',
    description: 'Enable new notification system'
  },

  // Business Settings
  business_stocks_per_page: {
    key: 'business_stocks_per_page',
    value: 25,
    valueType: 'number',
    description: 'Default number of stocks displayed per page'
  },
  business_chart_type: {
    key: 'business_chart_type',
    value: 'candlestick',
    valueType: 'string',
    description: 'Default chart type for stock visualization'
  },
  business_max_watchlist: {
    key: 'business_max_watchlist',
    value: 50,
    valueType: 'number',
    description: 'Maximum number of items in user watchlists'
  },
  business_premium_features: {
    key: 'business_premium_features',
    value: true,
    valueType: 'boolean',
    description: 'Enable premium features for eligible users'
  },
  business_data_export: {
    key: 'business_data_export',
    value: true,
    valueType: 'boolean',
    description: 'Allow users to export data'
  },

  // A/B Testing
  ab_homepage_layout: {
    key: 'ab_homepage_layout',
    value: 'original',
    valueType: 'string',
    description: 'Homepage layout variant for A/B testing'
  },
  ab_payment_flow: {
    key: 'ab_payment_flow',
    value: 'standard',
    valueType: 'string',
    description: 'Payment flow variant for A/B testing'
  },
  ab_onboarding_flow: {
    key: 'ab_onboarding_flow',
    value: 'standard',
    valueType: 'string',
    description: 'User onboarding flow variant'
  },
  ab_dashboard_style: {
    key: 'ab_dashboard_style',
    value: 'cards',
    valueType: 'string',
    description: 'Dashboard layout style variant'
  },

  // User Targeting
  targeting_enable_segments: {
    key: 'targeting_enable_segments',
    value: true,
    valueType: 'boolean',
    description: 'Enable user segmentation for targeted configs'
  },
  targeting_rollout_percentage: {
    key: 'targeting_rollout_percentage',
    value: 100,
    valueType: 'number',
    description: 'Percentage of users to receive new features'
  },
  targeting_premium_features: {
    key: 'targeting_premium_features',
    value: true,
    valueType: 'boolean',
    description: 'Enable premium feature targeting'
  },
  targeting_mobile_features: {
    key: 'targeting_mobile_features',
    value: true,
    valueType: 'boolean',
    description: 'Enable mobile-specific feature targeting'
  }
}

// ============================================================================
// Common User Conditions
// ============================================================================

export const COMMON_CONDITIONS: UserCondition[] = [
  {
    name: 'premium_users',
    expression: 'user.level in ["GOLD", "PLATINUM"]',
    description: 'Premium tier users (Gold and Platinum)',
    tagColor: 'gold'
  },
  {
    name: 'mobile_users',
    expression: 'device.type == "mobile"',
    description: 'Users on mobile devices',
    tagColor: 'blue'
  },
  {
    name: 'beta_testers',
    expression: 'user.betaTester == true',
    description: 'Opted-in beta testing users',
    tagColor: 'purple'
  },
  {
    name: 'new_users',
    expression: 'user.accountAge < 30',
    description: 'Users with accounts less than 30 days old',
    tagColor: 'green'
  },
  {
    name: 'high_volume_users', 
    expression: 'user.dailyRequests > 1000',
    description: 'Users making more than 1000 requests per day',
    tagColor: 'red'
  },
  {
    name: 'us_users',
    expression: 'user.country == "US"',
    description: 'Users located in the United States',
    tagColor: 'blue'
  }
]

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Get parameters by category
 */
export function getParametersByCategory(category: keyof typeof PARAMETER_CATEGORIES): RemoteConfigParameter[] {
  const categoryInfo = PARAMETER_CATEGORIES[category]
  return categoryInfo.parameters.map(key => DEFAULT_PARAMETERS[key]).filter(Boolean)
}

/**
 * Get all parameters grouped by category
 */
export function getAllParametersByCategory(): Record<string, RemoteConfigParameter[]> {
  const result: Record<string, RemoteConfigParameter[]> = {}
  
  for (const [categoryKey, categoryInfo] of Object.entries(PARAMETER_CATEGORIES)) {
    result[categoryKey] = getParametersByCategory(categoryKey as keyof typeof PARAMETER_CATEGORIES)
  }
  
  return result
}

/**
 * Validate parameter value based on type
 */
export function validateParameterValue(parameter: RemoteConfigParameter, value: any): { valid: boolean; error?: string } {
  try {
    switch (parameter.valueType) {
      case 'boolean':
        if (typeof value !== 'boolean' && value !== 'true' && value !== 'false') {
          return { valid: false, error: 'Value must be a boolean (true/false)' }
        }
        break
        
      case 'number':
        const numValue = Number(value)
        if (isNaN(numValue) || !isFinite(numValue)) {
          return { valid: false, error: 'Value must be a valid number' }
        }
        break
        
      case 'string':
        if (typeof value !== 'string') {
          return { valid: false, error: 'Value must be a string' }
        }
        break
        
      case 'json':
        if (typeof value === 'string') {
          try {
            JSON.parse(value)
          } catch {
            return { valid: false, error: 'Value must be valid JSON' }
          }
        }
        break
        
      default:
        return { valid: false, error: 'Unknown value type' }
    }
    
    return { valid: true }
  } catch (error) {
    return { valid: false, error: 'Validation error occurred' }
  }
}

/**
 * Format parameter value for display
 */
export function formatParameterValue(parameter: RemoteConfigParameter): string {
  const value = parameter.value
  
  switch (parameter.valueType) {
    case 'boolean':
      return value ? 'Enabled' : 'Disabled'
    case 'number':
      return value.toString()
    case 'json':
      return typeof value === 'string' ? value : JSON.stringify(value, null, 2)
    default:
      return String(value)
  }
}

/**
 * Get parameter category info
 */
export function getParameterCategory(parameterKey: string): { category: keyof typeof PARAMETER_CATEGORIES; info: typeof PARAMETER_CATEGORIES[keyof typeof PARAMETER_CATEGORIES] } | null {
  for (const [categoryKey, categoryInfo] of Object.entries(PARAMETER_CATEGORIES)) {
    if (categoryInfo.parameters.includes(parameterKey)) {
      return {
        category: categoryKey as keyof typeof PARAMETER_CATEGORIES,
        info: categoryInfo
      }
    }
  }
  return null
}

// ============================================================================
// User Targeting Integration
// ============================================================================

export interface UserSegment {
  id: string
  name: string
  description: string
  condition: string
  userCount: number
  isActive: boolean
  color?: string
  parameters?: Record<string, any>
}

/**
 * Get user targeting related parameters
 */
export function getUserTargetingParameters(): RemoteConfigParameter[] {
  return getParametersByCategory('USER_TARGETING')
}

/**
 * Create user condition for specific user properties
 */
export function createUserCondition(
  name: string,
  userProperty: string,
  operator: 'equals' | 'contains' | 'in' | 'greater_than' | 'less_than',
  value: any,
  description?: string
): UserCondition {
  let expression = ''
  
  switch (operator) {
    case 'equals':
      expression = `user.${userProperty} == "${value}"`
      break
    case 'contains':
      expression = `user.${userProperty}.contains("${value}")`
      break
    case 'in':
      expression = `user.${userProperty} in ${JSON.stringify(Array.isArray(value) ? value : [value])}`
      break
    case 'greater_than':
      expression = `user.${userProperty} > ${value}`
      break
    case 'less_than':
      expression = `user.${userProperty} < ${value}`
      break
  }
  
  return {
    name,
    expression,
    description: description || `Users where ${userProperty} ${operator} ${value}`,
    tagColor: 'blue'
  }
}

/**
 * Get common user segments for targeting
 */
export function getCommonUserSegments(): UserSegment[] {
  return [
    {
      id: 'premium_users',
      name: 'Premium Users',
      description: 'Gold and Platinum tier users',
      condition: 'user.level in ["GOLD", "PLATINUM"]',
      userCount: 1250,
      isActive: true,
      color: 'gold',
      parameters: {
        'features_advanced_charts': true,
        'business_premium_features': true
      }
    },
    {
      id: 'mobile_users',
      name: 'Mobile Users',
      description: 'Users accessing from mobile devices',
      condition: 'device.type == "mobile"',
      userCount: 3400,
      isActive: true,
      color: 'blue',
      parameters: {
        'ux_mobile_optimized': true,
        'performance_lazy_load': true
      }
    },
    {
      id: 'beta_testers',
      name: 'Beta Testers',
      description: 'Users opted into beta testing',
      condition: 'user.betaTester == true',
      userCount: 156,
      isActive: false,
      color: 'purple',
      parameters: {
        'features_experimental_ui': true,
        'features_beta_analytics': true
      }
    },
    {
      id: 'new_users',
      name: 'New Users',
      description: 'Users with accounts less than 30 days old',
      condition: 'user.accountAge < 30',
      userCount: 890,
      isActive: true,
      color: 'green',
      parameters: {
        'ab_onboarding_flow': 'enhanced',
        'features_mobile_app_promo': true
      }
    }
  ]
}

/**
 * Calculate user segment overlap
 */
export function calculateSegmentOverlap(segments: UserSegment[]): { overlap: number; conflicts: string[] } {
  // This would implement logic to calculate how many users might be in multiple segments
  // For now, return mock data
  return {
    overlap: Math.floor(Math.random() * 500),
    conflicts: []
  }
}