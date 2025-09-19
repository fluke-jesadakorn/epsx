// ============================================================================
// SHARED PERMISSION AUDIT TYPES
// ============================================================================
// Audit, logging, and admin-specific permission types

import { PermissionSource } from './core'

// ============================================================================
// AUDIT ENTRY TYPES
// ============================================================================

export interface PermissionAuditEntry {
  id: string
  user_id: string
  permission: string
  action: 'grant' | 'revoke' | 'extend' | 'cleanup' | 'migrate' | 'bulk_grant' | 'bulk_revoke'
  performed_by: string
  performed_at: number
  expires_at?: number
  reason?: string
  metadata?: Record<string, any>
  source?: PermissionSource
  batch_id?: string // For bulk operations
}

export interface BulkAuditEntry {
  id: string
  batch_id: string
  operation: string
  initiated_by: string
  initiated_at: number
  completed_at?: number
  total_operations: number
  successful_operations: number
  failed_operations: number
  permission_pattern?: string
  user_count: number
  metadata?: Record<string, any>
}

// ============================================================================
// ADMIN DASHBOARD TYPES
// ============================================================================
// Note: AdminPermissionDashboard is defined in api.ts to avoid circular dependencies

export interface CacheStatistics {
  total_cached_entries: number
  cache_hit_rate: number
  average_cache_age_minutes: number
  stale_entries: number
  last_cleanup: number
}

export interface PermissionDistribution {
  by_platform: Record<string, number>
  by_source: Record<PermissionSource, number>
  by_expiry_status: {
    permanent: number
    temporary: number
    expired: number
    expiring_soon: number
  }
}

// ============================================================================
// TEMPLATE TYPES
// ============================================================================

export interface PermissionTemplate {
  id: string
  name: string
  description: string
  permissions: string[]
  default_expiry_hours?: number
  source: PermissionSource
  created_by: string
  created_at: number
  updated_at?: number
  updated_by?: string
  usage_count: number
  is_active: boolean
  tags?: string[]
}

export interface PermissionTemplateUsage {
  template_id: string
  template_name: string
  applied_by: string
  applied_at: number
  user_ids: string[]
  permissions_granted: number
  success_rate: number
}

// ============================================================================
// MONITORING AND HEALTH TYPES
// ============================================================================

export interface SystemHealthReport {
  overall_health_score: number
  timestamp: number
  components: {
    permissions: ComponentHealth
    cache: ComponentHealth
    database: ComponentHealth
    authentication: ComponentHealth
  }
  alerts: SystemAlert[]
  performance_metrics: PerformanceMetrics
}

export interface ComponentHealth {
  status: 'healthy' | 'warning' | 'critical' | 'unknown'
  score: number
  message?: string
  last_check: number
  metrics?: Record<string, any>
}

export interface SystemAlert {
  id: string
  severity: 'info' | 'warning' | 'error' | 'critical'
  title: string
  message: string
  component: string
  created_at: number
  acknowledged?: boolean
  acknowledged_by?: string
  resolved?: boolean
  resolved_at?: number
}

export interface PerformanceMetrics {
  permission_check_avg_ms: number
  cache_lookup_avg_ms: number
  database_query_avg_ms: number
  api_response_avg_ms: number
  throughput_per_second: number
  error_rate_percentage: number
}

// ============================================================================
// SEARCH AND FILTERING TYPES
// ============================================================================

export interface AuditSearchFilters {
  user_id?: string
  performed_by?: string
  action?: PermissionAuditEntry['action']
  permission_pattern?: string
  date_from?: number
  date_to?: number
  source?: PermissionSource
  batch_id?: string
  limit?: number
  offset?: number
  sort_by?: 'performed_at' | 'user_id' | 'permission' | 'action'
  sort_order?: 'asc' | 'desc'
}

export interface AdminSearchFilters {
  email?: string
  permission_pattern?: string
  source?: PermissionSource
  expires_within_hours?: number
  is_expired?: boolean
  has_expiring_soon?: boolean
  has_admin_permissions?: boolean
  inactive_for_days?: number
  created_after?: number
  created_before?: number
  limit?: number
  offset?: number
}

// ============================================================================
// MIGRATION AND MAINTENANCE TYPES
// ============================================================================

export interface MigrationPlan {
  phase: 'analysis' | 'migration' | 'validation' | 'cleanup'
  total_steps: number
  completed_steps: number
  current_step_description: string
  estimated_completion?: number // Unix timestamp
  errors: string[]
  warnings: string[]
}

export interface MigrationStatus {
  total_users: number
  migrated_users: number
  failed_migrations: number
  legacy_permissions_remaining: number
  migration_started: number
  migration_completed?: number
  current_phase: MigrationPlan['phase']
}

export interface MaintenanceTask {
  id: string
  name: string
  description: string
  type: 'cleanup' | 'migration' | 'optimization' | 'audit'
  status: 'pending' | 'running' | 'completed' | 'failed'
  scheduled_at?: number
  started_at?: number
  completed_at?: number
  progress_percentage: number
  details?: Record<string, any>
  error_message?: string
}

// ============================================================================
// REPORTING TYPES
// ============================================================================

export interface PermissionUsageReport {
  report_id: string
  generated_at: number
  period_start: number
  period_end: number
  total_permission_checks: number
  unique_users: number
  most_checked_permissions: Array<{
    permission: string
    count: number
    unique_users: number
  }>
  permission_grants_by_day: Array<{
    date: string
    grants: number
    revocations: number
  }>
  user_activity_summary: {
    active_users: number
    inactive_users: number
    new_users: number
  }
}

export interface SecurityReport {
  report_id: string
  generated_at: number
  period_start: number
  period_end: number
  security_events: Array<{
    event_type: string
    count: number
    severity: 'low' | 'medium' | 'high' | 'critical'
  }>
  permission_violations: Array<{
    user_id: string
    permission: string
    attempted_at: number
    violation_type: string
  }>
  admin_activities: Array<{
    admin_id: string
    action: string
    target_user?: string
    performed_at: number
  }>
  recommendations: string[]
}