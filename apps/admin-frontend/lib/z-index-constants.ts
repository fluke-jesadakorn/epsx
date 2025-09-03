/**
 * Standardized Z-Index Hierarchy for Admin Frontend
 * 
 * This document defines the z-index values used across all modal and overlay components
 * to prevent stacking conflicts and ensure consistent layering behavior.
 * 
 * Updated: 2025-08-07
 */

export const Z_INDEX_LAYERS = {
  // Base content layer: z-0 to z-10
  BASE: {
    CONTENT: 'z-0',
    ELEVATED_CONTENT: 'z-10',
  },
  
  // Dropdowns and tooltips: z-[9999] to z-[10000]
  DROPDOWNS: {
    DROPDOWN: 'z-[10000]',
    TOOLTIP: 'z-[9999]',
  },
  
  // Sidebar and navigation overlays: z-40 to z-50
  NAVIGATION: {
    OVERLAY_BACKDROP: 'z-40',  // Mobile sidebar backdrop
    SIDEBAR: 'z-50',           // Sidebar panels and navigation bars
  },
  
  // Modal dialogs and overlays: z-60 to z-70
  MODALS: {
    MODAL_BACKDROP: 'z-60',    // Modal overlay backgrounds
    MODAL_CONTENT: 'z-70',     // Modal content (if needed to stack above backdrop)
  },
  
  // Toast notifications: z-80 to z-90
  NOTIFICATIONS: {
    TOAST: 'z-80',             // Toast notification containers
    CRITICAL_TOAST: 'z-90',    // Critical/urgent notifications
  },
} as const;

/**
 * Files updated with standardized z-index values:
 * 
 * Modal Components (z-60):
 * - Modals converted to pages/inline forms (UserCreate, Permissions)
 * - ModuleManagementClient.tsx (was z-50)
 * - ConfirmDialog.tsx (was z-50)  
 * - DeveloperPortal.tsx (was z-50)
 * - UserManagement.tsx (was z-[9999])
 * 
 * Toast Components (z-80):
 * - toast.tsx in admin-frontend (was z-[9998])
 * - toast.tsx in frontend (was z-[100])
 * 
 * Navigation Components:
 * - AdminLayout.tsx sidebar backdrop (z-40, was z-30)
 * - AdminLayout.tsx sidebar panel (z-50, was z-40)
 * - nav.tsx navigation bars (z-50, unchanged)
 */

/**
 * Usage Guidelines:
 * 
 * 1. Always use the constants defined in this file rather than hardcoded values
 * 2. When creating new components, choose the appropriate layer based on function
 * 3. Modal components should use Z_INDEX_LAYERS.MODALS.MODAL_BACKDROP
 * 4. Toast notifications should use Z_INDEX_LAYERS.NOTIFICATIONS.TOAST
 * 5. Navigation overlays should use Z_INDEX_LAYERS.NAVIGATION values
 * 6. Avoid arbitrary z-index values like z-[9999] or z-[100]
 */