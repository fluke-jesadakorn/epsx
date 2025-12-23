/**
 * UNIFIED NOTIFICATION - Re-export from Shared
 */
export * from '@/shared/components/notifications/UnifiedNotification';

// Re-export specialized hooks for backward compatibility
export {
  MetroNotification, ProfessionalAlert, ProfessionalNotification, useAdminToast,
  useAnalyticsToast, useMetroToast, usePancakeToast, useProfessionalToast
} from '@/shared/components/notifications/UnifiedNotification';
