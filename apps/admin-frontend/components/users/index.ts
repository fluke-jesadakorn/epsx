/**
 * User Components Barrel Export
 * Clean interface for importing user-related components
 */

// Individual focused components
export { CreateUserForm } from './CreateUserForm';
export { EditUserForm } from './EditUserForm';
export { BulkUserOperations } from './BulkUserOperations';
export { UserImportExport } from './UserImportExport';

// Container component (replaces massive UserForms.tsx)
export { UserFormsContainer } from './UserFormsContainer';

// Shared utilities
export * from './shared/user-schemas';
export * from './shared/user-form-utils';

// Legacy exports for backward compatibility
export { UserFormsContainer as UserForms } from './UserFormsContainer';