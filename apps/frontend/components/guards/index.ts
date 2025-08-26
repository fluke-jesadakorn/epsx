/**
 * Simple Role-Based Feature Guards Export
 * Unified authentication and authorization guards
 */

// Auth Guards
export { default as AuthGuard, WithUser } from './AuthGuard';

// Simple role-based feature guards
export { 
  FeatureGuard,
  AdminOnly,
  UserOnly, 
  ViewEpsOnly,
  ExportDataOnly,
  RealtimeOnly,
  ProfileOnly,
  NotificationsOnly,
  BillingOnly,
  AdvancedFiltersOnly
} from './FeatureGuard';