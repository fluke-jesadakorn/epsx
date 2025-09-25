/**
 * User Forms Container
 * Replaces the massive UserForms.tsx with focused, maintainable components
 * Reduced from 1,769 lines to ~150 lines with better separation of concerns
 */

'use client';

import { useState } from 'react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';

import { CreateUserForm } from './CreateUserForm';
import { EditUserForm } from './EditUserForm';
import { BulkUserOperations } from './BulkUserOperations';
import { UserImportExport } from './UserImportExport';

import type { User } from '@/types/core';
import type { SelectOption } from '@/types/ui';

interface UserFormsContainerProps {
  mode?: 'create' | 'edit' | 'bulk' | 'import' | 'tabs';
  currentUser?: User;
  users?: User[];
  editUser?: User;
  availableRoles?: SelectOption[];
  availablePackageTiers?: SelectOption[];
  availablePermissions?: Array<{
    id: string;
    name: string;
    description: string;
    category: string;
  }>;
  onUserCreated?: (user: User) => void;
  onUserUpdated?: (user: User) => void;
  onBulkOperationComplete?: (result: any) => void;
  onImportComplete?: (users: User[]) => void;
  className?: string;
}

export function UserFormsContainer({
  mode = 'tabs',
  currentUser,
  users = [],
  editUser,
  availableRoles = [],
  availablePackageTiers = [],
  availablePermissions = [],
  onUserCreated,
  onUserUpdated,
  onBulkOperationComplete,
  onImportComplete,
  className = ''
}: UserFormsContainerProps) {
  const [activeTab, setActiveTab] = useState(mode === 'tabs' ? 'create' : mode);

  // Single form mode - return specific component
  if (mode === 'create') {
    return (
      <CreateUserForm
        currentUser={currentUser}
        availableRoles={availableRoles}
        availablePackageTiers={availablePackageTiers}
        availablePermissions={availablePermissions}
        onUserCreated={onUserCreated}
        className={className}
      />
    );
  }

  if (mode === 'edit') {
    if (!editUser) {
      return (
        <div className={`p-6 text-center ${className}`}>
          <p className="text-gray-500">No user selected for editing.</p>
        </div>
      );
    }
    
    return (
      <EditUserForm
        editUser={editUser}
        currentUser={currentUser}
        availableRoles={availableRoles}
        availablePackageTiers={availablePackageTiers}
        availablePermissions={availablePermissions}
        onUserUpdated={onUserUpdated}
        className={className}
      />
    );
  }

  if (mode === 'bulk') {
    return (
      <BulkUserOperations
        users={users}
        currentUser={currentUser}
        availableRoles={availableRoles}
        availablePackageTiers={availablePackageTiers}
        availablePermissions={availablePermissions}
        onBulkOperationComplete={onBulkOperationComplete}
        className={className}
      />
    );
  }

  if (mode === 'import') {
    return (
      <UserImportExport
        users={users}
        onImportComplete={onImportComplete}
        className={className}
      />
    );
  }

  // Tabbed interface mode
  return (
    <div className={`space-y-6 ${className}`}>
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-4">
          User Management
        </h1>
        <p className="text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
          Create, edit, and manage users with comprehensive permission control
        </p>
      </div>

      <Tabs value={activeTab} onValueChange={(value) => setActiveTab(value as typeof activeTab)} className="w-full">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="create">Create User</TabsTrigger>
          <TabsTrigger value="edit">Edit User</TabsTrigger>
          <TabsTrigger value="bulk">Bulk Operations</TabsTrigger>
          <TabsTrigger value="import">Import/Export</TabsTrigger>
        </TabsList>

        <TabsContent value="create" className="mt-6">
          <CreateUserForm
            currentUser={currentUser}
            availableRoles={availableRoles}
            availablePackageTiers={availablePackageTiers}
            availablePermissions={availablePermissions}
            onUserCreated={onUserCreated}
          />
        </TabsContent>

        <TabsContent value="edit" className="mt-6">
          {editUser ? (
            <EditUserForm
              editUser={editUser}
              currentUser={currentUser}
              availableRoles={availableRoles}
              availablePackageTiers={availablePackageTiers}
              availablePermissions={availablePermissions}
              onUserUpdated={onUserUpdated}
            />
          ) : (
            <div className="text-center p-8">
              <p className="text-gray-500">
                Please select a user from the users list to edit their profile.
              </p>
            </div>
          )}
        </TabsContent>

        <TabsContent value="bulk" className="mt-6">
          <BulkUserOperations
            users={users}
            currentUser={currentUser}
            availableRoles={availableRoles}
            availablePackageTiers={availablePackageTiers}
            availablePermissions={availablePermissions}
            onBulkOperationComplete={onBulkOperationComplete}
          />
        </TabsContent>

        <TabsContent value="import" className="mt-6">
          <UserImportExport
            users={users}
            onImportComplete={onImportComplete}
          />
        </TabsContent>
      </Tabs>
    </div>
  );
}

// Export as default for backward compatibility
export { UserFormsContainer as UserForms };