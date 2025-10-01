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
// PERMISSION GROUP DERIVATION UTILITIES
// ============================================================================

/**
 * Derive permission group from permissions (matches backend logic)
 */
export function derivePermissionGroupFromPermissions(permissions: string[]): string {
  if (hasEnterprisePermissions(permissions)) {
    return 'Enterprise Access Group'
  } else if (hasPlatinumPermissions(permissions)) {
    return 'Professional Access Group'
  } else if (hasGoldPermissions(permissions)) {
    return 'Premium Access Group'
  } else if (hasSilverPermissions(permissions)) {
    return 'Standard Access Group'
  } else if (hasBronzePermissions(permissions)) {
    return 'Basic Access Group'
  } else {
    return 'Basic Access Group'
  }
}

/**
 * @deprecated Use derivePermissionGroupFromPermissions instead
 * Legacy function for backward compatibility during migration
 */
export function derivePackageTierFromPermissions(permissions: string[]): string {
  // Map permission groups back to legacy tier names for compatibility
  const group = derivePermissionGroupFromPermissions(permissions)
  switch (group) {
    case 'Enterprise Access Group': return 'ENTERPRISE'
    case 'Professional Access Group': return 'PLATINUM'
    case 'Premium Access Group': return 'GOLD'
    case 'Standard Access Group': return 'SILVER'
    case 'Basic Access Group': return 'BRONZE'
    default: return 'FREE'
  }
}

// ============================================================================
// PERMISSION LEVEL DETECTION HELPERS (Private)
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
 * Get permission group information from permissions (NEW)
 */
export function getPermissionGroupFromPermissions(permissions: string[]): {
  permissionGroup: string
  platforms: string[]
  primaryPlatform: string
} {
  return {
    permissionGroup: derivePermissionGroupFromPermissions(permissions),
    platforms: deriveAccessiblePlatformsFromPermissions(permissions),
    primaryPlatform: derivePrimaryPlatformFromPermissions(permissions)
  }
}

/**
 * Get package information from permissions
 * @deprecated Use getPermissionGroupFromPermissions instead
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

// ============================================================================
// PERMISSION GROUP UTILITIES (NEW)
// ============================================================================

/**
 * Check if user has a specific permission group level or higher
 */
export function hasMinimumPermissionGroup(permissions: string[], requiredGroup: string): boolean {
  const userGroup = derivePermissionGroupFromPermissions(permissions)
  const groupHierarchy: Record<string, number> = {
    'Basic Access Group': 1,
    'Standard Access Group': 2,
    'Premium Access Group': 3,
    'Professional Access Group': 4,
    'Enterprise Access Group': 5,
  }
  
  const userLevel = groupHierarchy[userGroup] || 0
  const requiredLevel = groupHierarchy[requiredGroup] || 1
  
  return userLevel >= requiredLevel
}

/**
 * Get permission group display name with icon
 */
export function getPermissionGroupDisplayName(group: string): string {
  const displayNames: Record<string, string> = {
    'Basic Access Group': '🥉 Basic',
    'Standard Access Group': '⚪ Standard', 
    'Premium Access Group': '🥇 Premium',
    'Professional Access Group': '💎 Professional',
    'Enterprise Access Group': '🏢 Enterprise',
  }
  
  return displayNames[group] || group
}

/**
 * Get the permission group level (1-5)
 */
export function getPermissionGroupLevel(group: string): number {
  const levels: Record<string, number> = {
    'Basic Access Group': 1,
    'Standard Access Group': 2,
    'Premium Access Group': 3,
    'Professional Access Group': 4,
    'Enterprise Access Group': 5,
  }
  
  return levels[group] || 0
}