import { AbilityBuilder, createMongoAbility, MongoAbility } from '@casl/ability'

/**
 * CASL permission management for EPSX
 * Simple, flexible permissions that replace complex Casbin policies
 */

export type Subjects = 
  | 'User' 
  | 'Analytics' 
  | 'Stock' 
  | 'Payment' 
  | 'Admin' 
  | 'Module' 
  | 'System'
  | 'all'

export type Actions = 
  | 'manage' 
  | 'create' 
  | 'read' 
  | 'update' 
  | 'delete' 
  | 'export' 
  | 'import'
  | 'assign'
  | 'revoke'

export type AppAbility = MongoAbility<[Actions, Subjects]>

/**
 * Create CASL ability based on user permissions from JWT
 */
export function createUserAbility(user: {
  permissions: string[]
  admin_modules: string[]
  package_tier: string
  role: string
}): AppAbility {
  const { can, build } = new AbilityBuilder<AppAbility>(createMongoAbility)

  // Basic user permissions (everyone gets these)
  can('read', 'User') // Can read their own user data
  can('update', 'User') // Can update their own profile

  // Package tier based permissions
  if (user.package_tier !== 'FREE') {
    can('read', 'Analytics') // Premium users can read analytics
    can('export', 'Analytics') // Premium users can export data
  }

  if (['SILVER', 'GOLD', 'PLATINUM', 'ENTERPRISE'].includes(user.package_tier)) {
    can('read', 'Stock') // Advanced users can read stock data
    can('export', 'Stock') // Advanced users can export stock data
  }

  if (user.package_tier === 'ENTERPRISE') {
    can('manage', 'all') // Enterprise users can do everything (except admin)
  }

  // Permission string based rules
  user.permissions.forEach(permission => {
    const [action, subject] = permission.split(':')
    
    switch (permission) {
      // Analytics permissions
      case 'analytics:read':
        can('read', 'Analytics')
        break
      case 'analytics:advanced':
        can(['read', 'export'], 'Analytics')
        break
      case 'analytics:premium':
        can(['read', 'export', 'create'], 'Analytics')
        break

      // Stock permissions
      case 'stock:read':
        can('read', 'Stock')
        break
      case 'stock:write':
        can(['read', 'update'], 'Stock')
        break

      // Payment permissions
      case 'payment:read':
        can('read', 'Payment')
        break
      case 'payment:write':
        can(['read', 'create', 'update'], 'Payment')
        break

      // User permissions
      case 'user:read':
        can('read', 'User')
        break
      case 'user:write':
        can(['read', 'update'], 'User')
        break

      // Wildcard permissions
      case '*':
        can('manage', 'all')
        break
      
      // Dynamic permission parsing
      default:
        if (action && subject) {
          const caslAction = mapActionToCasl(action)
          const caslSubject = mapSubjectToCasl(subject)
          if (caslAction && caslSubject) {
            can(caslAction, caslSubject)
          }
        }
        break
    }
  })

  // Admin module based permissions
  user.admin_modules.forEach(module => {
    switch (module) {
      case 'user_operations':
        can('manage', 'User') // Can manage all users
        break
      case 'system_admin':
        can('manage', 'all') // Can manage everything
        break
      case 'billing_admin':
        can('manage', 'Payment') // Can manage all payments
        break
      case 'analytics_specialist':
        can('manage', 'Analytics') // Can manage analytics
        break
      case 'module_coordinator':
        can('manage', 'Module') // Can manage modules
        break
    }
  })

  // Role-based permissions
  if (user.role === 'admin' || user.role === 'super_admin') {
    can('manage', 'Admin') // Can access admin features
  }

  if (user.role === 'super_admin') {
    can('manage', 'all') // Super admin can do everything
  }

  return build()
}

/**
 * Map permission action strings to CASL actions
 */
function mapActionToCasl(action: string): Actions | null {
  const actionMap: Record<string, Actions> = {
    'read': 'read',
    'write': 'update',
    'create': 'create',
    'update': 'update',
    'delete': 'delete',
    'manage': 'manage',
    'export': 'export',
    'import': 'import',
    'assign': 'assign',
    'revoke': 'revoke',
  }
  
  return actionMap[action] || null
}

/**
 * Map permission subject strings to CASL subjects
 */
function mapSubjectToCasl(subject: string): Subjects | null {
  const subjectMap: Record<string, Subjects> = {
    'user': 'User',
    'users': 'User',
    'analytics': 'Analytics',
    'stock': 'Stock',
    'stocks': 'Stock',
    'payment': 'Payment',
    'payments': 'Payment',
    'admin': 'Admin',
    'module': 'Module',
    'modules': 'Module',
    'system': 'System',
    '*': 'all',
  }
  
  return subjectMap[subject.toLowerCase()] || null
}

/**
 * Check if user can perform action on subject
 */
export function can(
  ability: AppAbility, 
  action: Actions, 
  subject: Subjects
): boolean {
  return ability.can(action, subject)
}

/**
 * Check if user cannot perform action on subject
 */
export function cannot(
  ability: AppAbility, 
  action: Actions, 
  subject: Subjects
): boolean {
  return ability.cannot(action, subject)
}

/**
 * Get all rules for debugging
 */
export function getRules(ability: AppAbility) {
  return ability.rules
}

/**
 * Common permission checks
 */
export const PermissionChecks = {
  canReadAnalytics: (ability: AppAbility) => can(ability, 'read', 'Analytics'),
  canExportData: (ability: AppAbility) => can(ability, 'export', 'Analytics'),
  canManageUsers: (ability: AppAbility) => can(ability, 'manage', 'User'),
  canAccessAdmin: (ability: AppAbility) => can(ability, 'read', 'Admin'),
  canManagePayments: (ability: AppAbility) => can(ability, 'manage', 'Payment'),
  canReadStock: (ability: AppAbility) => can(ability, 'read', 'Stock'),
  isSystemAdmin: (ability: AppAbility) => can(ability, 'manage', 'System'),
}

/**
 * Package tier permission helpers
 */
export const PackagePermissions = {
  requiresPremium: (ability: AppAbility) => can(ability, 'read', 'Analytics'),
  requiresAdvanced: (ability: AppAbility) => can(ability, 'read', 'Stock'),
  requiresEnterprise: (ability: AppAbility) => can(ability, 'manage', 'all'),
}