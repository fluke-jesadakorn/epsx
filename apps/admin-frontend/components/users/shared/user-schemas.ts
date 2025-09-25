/**
 * Shared User Form Schemas and Types
 * Now imports from shared validators to eliminate duplication
 */

import type { User, Permission } from '@/types/core';
import type { SelectOption } from '@/types/ui';

// Import shared validation schemas
export {
  createUserSchema,
  editUserSchema,
  bulkUserSchema,
  type CreateUserForm,
  type EditUserForm,
  type BulkUserForm
} from '../../../../../shared/validators/schemas';

// Shared interfaces
export interface BaseUserFormProps {
  currentUser?: User;
  availableRoles?: SelectOption[];
  availablePackageTiers?: SelectOption[];
  availablePermissions?: Array<{
    id: string;
    name: string;
    description: string;
    category: string;
  }>;
  className?: string;
}

export interface CreateUserFormProps extends BaseUserFormProps {
  onUserCreated?: (user: User) => void;
}

export interface EditUserFormProps extends BaseUserFormProps {
  editUser: User;
  onUserUpdated?: (user: User) => void;
}

export interface BulkUserFormProps extends BaseUserFormProps {
  users: User[];
  onBulkOperationComplete?: (result: any) => void;
}