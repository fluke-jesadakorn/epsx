// ============================================================================
// ADMIN FRONTEND PERMISSION TYPES
// ============================================================================
// Admin-specific permission types that extend shared permission system

// Re-export all shared types
export * from '@/shared/permissions/types'

// Admin-specific extensions
import { EnhancedUserClaims } from '@/shared/permissions/types'

export interface AdminUserClaims extends EnhancedUserClaims {
  // Admin-specific claim extensions
  admin_level?: 'basic' | 'advanced' | 'super'
  last_admin_login?: number
  admin_session_id?: string
}

export interface AdminPermissionContext {
  userClaims: AdminUserClaims | null
  isAdmin: boolean
  canManagePermissions: boolean
  canManageUsers: boolean
  canViewAuditLogs: boolean
  canManageSystem: boolean
  loading: boolean
  error: any
  refreshAdminSession: () => Promise<void>
}

export interface AdminGuardOptions {
  strictMode?: boolean
  requireElevatedPrivileges?: boolean
  adminAction?: 'manage' | 'read' | 'write' | 'delete'
  showAdminWarnings?: boolean
  auditTrail?: boolean
}