// ============================================================================
// SHARED PERMISSION CONSTANTS
// ============================================================================

export const PLATFORMS = {
  EPSX: 'epsx',
  EPSX_PAY: 'epsx-pay',
  EPSX_TOKEN: 'epsx-token',
  ADMIN: 'admin'
} as const

export const PERMISSION_SOURCES = {
  SUBSCRIPTION: 'Subscription',
  ADMIN: 'Admin',
  TRIAL: 'Trial',
  LEGACY: 'Legacy',
  SYSTEM: 'System'
} as const

export const WILDCARDS = {
  ALL_RESOURCES: '*',
  ALL_ACTIONS: '*',
  FULL_ACCESS: '*:*'
} as const

export const EXPIRY_THRESHOLDS = {
  EXPIRING_SOON_HOURS: 24,
  WARNING_HOURS: 72,
  CRITICAL_HOURS: 2
} as const

export const HEALTH_SCORES = {
  EXCELLENT: 95,
  GOOD: 80,
  WARNING: 60,
  CRITICAL: 40
} as const