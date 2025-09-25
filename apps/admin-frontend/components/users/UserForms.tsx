/**
 * User Forms - LEGACY FILE (REFACTORED)
 * 
 * This massive 1,769-line file has been split into focused components:
 * - CreateUserForm.tsx (create user functionality)
 * - EditUserForm.tsx (edit user functionality) 
 * - BulkUserOperations.tsx (bulk operations)
 * - UserImportExport.tsx (import/export functionality)
 * - UserFormsContainer.tsx (unified container)
 * 
 * This file now serves as a backward compatibility layer.
 * 
 * REFACTORING BENEFITS:
 * ✅ Reduced from 1,769 lines to ~20 lines  
 * ✅ Eliminated code duplication
 * ✅ Removed animation violations (animate-spin, animate-pulse)
 * ✅ Fixed 'any' type usage
 * ✅ Better separation of concerns
 * ✅ Easier testing and maintenance
 */

// Re-export the new modular components for backward compatibility
export { UserFormsContainer as UserForms } from './UserFormsContainer';

// Additional exports for direct component access
export { CreateUserForm } from './CreateUserForm';
export { EditUserForm } from './EditUserForm'; 
export { BulkUserOperations } from './BulkUserOperations';
export { UserImportExport } from './UserImportExport';