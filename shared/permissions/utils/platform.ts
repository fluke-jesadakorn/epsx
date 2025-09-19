// ============================================================================
// SHARED PERMISSION PLATFORM UTILITIES
// ============================================================================
// Platform, tier, and package derivation utilities

// ============================================================================
// PLATFORM UTILITIES
// ============================================================================

/**
 * Derive accessible platforms from permissions
 */
export function deriveAccessiblePlatformsFromPermissions(permissions: string[]): string[] {
  const platforms = new Set<string>()
  
  for (const permission of permissions) {
    const platform = permission.split(':')[0]
    if (platform) {
      platforms.add(platform)
    }
  }
  
  return platforms.size > 0 ? Array.from(platforms) : ['epsx']
}

/**
 * Derive primary platform from permissions (priority: admin > epsx > epsx-pay > epsx-token)
 */
export function derivePrimaryPlatformFromPermissions(permissions: string[]): string {
  if (permissions.some(p => p.startsWith('admin:'))) {
    return 'admin'
  } else if (permissions.some(p => p.startsWith('epsx:'))) {
    return 'epsx'
  } else if (permissions.some(p => p.startsWith('epsx-pay:'))) {
    return 'epsx-pay'
  } else if (permissions.some(p => p.startsWith('epsx-token:'))) {
    return 'epsx-token'
  } else {
    return 'epsx'
  }
}

// ============================================================================
// TIER DERIVATION UTILITIES
// ============================================================================

/**
 * Derive package tier from permissions (matches backend logic)
 */
export function derivePackageTierFromPermissions(permissions: string[]): string {
  if (hasEnterprisePermissions(permissions)) {
    return 'ENTERPRISE'
  } else if (hasPlatinumPermissions(permissions)) {
    return 'PLATINUM'
  } else if (hasGoldPermissions(permissions)) {
    return 'GOLD'
  } else if (hasSilverPermissions(permissions)) {
    return 'SILVER'
  } else if (hasBronzePermissions(permissions)) {
    return 'BRONZE'
  } else {
    return 'FREE'
  }
}

// ============================================================================
// TIER DETECTION HELPERS
// ============================================================================

/**
 * Check if user has enterprise-level permissions
 */
function hasEnterprisePermissions(permissions: string[]): boolean {
  return permissions.some(p => 
    p.startsWith('enterprise:') || 
    p === 'admin:*:*' ||
    p.includes('enterprise') ||
    permissions.some(perm => perm.startsWith('admin:'))
  )
}

/**
 * Check if user has platinum-level permissions
 */
function hasPlatinumPermissions(permissions: string[]): boolean {
  return permissions.some(p => 
    p.startsWith('platinum:') ||
    p.includes('platinum') ||
    permissions.length >= 10 // Many permissions indicate higher tier
  )
}

/**
 * Check if user has gold-level permissions
 */
function hasGoldPermissions(permissions: string[]): boolean {
  return permissions.some(p => 
    p.startsWith('gold:') ||
    p.includes('gold') ||
    permissions.some(perm => perm.includes('export') || perm.includes('advanced'))
  )
}

/**
 * Check if user has silver-level permissions
 */
function hasSilverPermissions(permissions: string[]): boolean {
  return permissions.some(p => 
    p.startsWith('silver:') ||
    p.includes('silver') ||
    permissions.length >= 5 // Several permissions indicate silver tier
  )
}

/**
 * Check if user has bronze-level permissions
 */
function hasBronzePermissions(permissions: string[]): boolean {
  return permissions.some(p => 
    p.startsWith('bronze:') ||
    p.includes('bronze') ||
    permissions.length >= 3 // Few permissions indicate bronze tier
  )
}

// ============================================================================
// PACKAGE INFORMATION UTILITIES
// ============================================================================

/**
 * Get package information from permissions
 */
export function getPackageFromPermissions(permissions: string[]): {
  tier: string
  platforms: string[]
  primaryPlatform: string
} {
  return {
    tier: derivePackageTierFromPermissions(permissions),
    platforms: deriveAccessiblePlatformsFromPermissions(permissions),
    primaryPlatform: derivePrimaryPlatformFromPermissions(permissions)
  }
}

/**
 * Check if permissions indicate admin access
 */
export function hasAdminAccess(permissions: string[]): boolean {
  return permissions.some(p => p.startsWith('admin:'))
}

/**
 * Check if permissions indicate enterprise-level access
 */
export function hasEnterpriseAccess(permissions: string[]): boolean {
  return hasEnterprisePermissions(permissions)
}