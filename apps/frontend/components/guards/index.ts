/**
 * Simple Role-Based Feature Guards Export
 * Unified authentication and authorization guards
 */

// Auth Guards - removed unused AuthGuard and WithUser

// Simple role-based feature guards
export { 
  PermissionGuard as FeatureGuard,
  AdminOnly,
  ViewEpsOnly,
  ExportDataOnly,
  RealtimeOnly,
  ProfileOnly,
  NotificationsOnly,
  BillingOnly,
  AdvancedFiltersOnly
} from './FeatureGuard';