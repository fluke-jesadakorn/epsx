/**
 * Bulk User Operations Component
 * Extracted from UserForms.tsx for better maintainability
 * Follows zero animation policy and strict typing
 */

'use client';

import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { 
  Users, 
  Clock
} from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Checkbox } from '@/components/ui/checkbox';
import { Textarea } from '@/components/ui/textarea';

import { logger } from '@/lib/logger';
import {
  bulkUserSchema,
  type BulkUserForm as BulkUserFormData,
  type BulkUserFormProps
} from './shared/user-schemas';
import {
  showSuccessToast,
  showErrorToast
} from './shared/user-form-utils';

const defaultRoles = [
  { label: 'Admin', value: 'admin' },
  { label: 'User', value: 'user' },
  { label: 'Premium User', value: 'premium_user' }
];

const defaultTiers = [
  { label: 'Basic', value: 'basic' },
  { label: 'Premium', value: 'premium' },
  { label: 'Pro', value: 'pro' },
  { label: 'Enterprise', value: 'enterprise' }
];

type BulkOperation = 'update_role' | 'update_tier' | 'assign_permissions' | 'activate' | 'deactivate' | 'delete';

export function BulkUserOperations({
  users,
  availableRoles = [],
  availablePackageTiers = [],
  onBulkOperationComplete,
  className = ''
}: BulkUserFormProps) {
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [selectedUsers, setSelectedUsers] = useState<string[]>([]);
  const [bulkOperation, setBulkOperation] = useState<BulkOperation | ''>('');

  const form = useForm<BulkUserFormData>({
    resolver: zodResolver(bulkUserSchema),
    defaultValues: {
      userIds: [],
      operation: 'update_role',
      reason: ''
    }
  });

  const roles = availableRoles.length > 0 ? availableRoles : defaultRoles;
  const tiers = availablePackageTiers.length > 0 ? availablePackageTiers : defaultTiers;

  const handleUserSelection = (userId: string, checked: boolean) => {
    if (checked) {
      const updated = [...selectedUsers, userId];
      setSelectedUsers(updated);
      form.setValue('userIds', updated);
    } else {
      const updated = selectedUsers.filter(id => id !== userId);
      setSelectedUsers(updated);
      form.setValue('userIds', updated);
    }
  };

  const handleOperationChange = (operation: BulkOperation) => {
    form.setValue('operation', operation);
    setBulkOperation(operation);
  };

  const handleSubmit = async (data: BulkUserFormData) => {
    setIsSubmitting(true);
    try {
      // Mock bulk operation - replace with actual API call
      const result = {
        operation: data.operation,
        userIds: data.userIds,
        successful: data.userIds.length,
        failed: 0,
        reason: data.reason
      };

      showSuccessToast(
        "Bulk operation completed",
        `Successfully applied ${data.operation} to ${data.userIds.length} user(s).`
      );

      onBulkOperationComplete?.(result);
      handleReset();
    } catch (error) {
      logger.error('Bulk operation error', { 
        operation: data.operation, 
        userIds: data.userIds, 
        error 
      });
      showErrorToast(
        "Bulk operation failed",
        "An error occurred while processing the bulk operation."
      );
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleReset = () => {
    form.reset();
    setSelectedUsers([]);
    setBulkOperation('');
  };

  return (
    <div className={`space-y-4 ${className}`}>
      <Card className="p-6">
        <form onSubmit={form.handleSubmit(handleSubmit)} className="space-y-6">
          <div className="flex items-center gap-2 mb-4">
            <Users className="w-5 h-5 text-purple-500" />
            <h3 className="text-lg font-medium">Bulk User Operations</h3>
          </div>

          {/* User Selection */}
          <div className="space-y-2">
            <Label>Select Users</Label>
            <div className="border rounded-md p-3 max-h-48 overflow-y-auto">
              <div className="space-y-2">
                {users.map(user => (
                  <label key={user.id} className="flex items-center gap-3 p-2 hover:bg-gray-100 dark:hover:bg-gray-800 rounded cursor-pointer">
                    <Checkbox
                      checked={selectedUsers.includes(user.id)}
                      onCheckedChange={(checked) => handleUserSelection(user.id, !!checked)}
                    />
                    <div className="flex items-center gap-2 flex-1">
                      <div className="w-6 h-6 rounded-full bg-blue-500/20 flex items-center justify-center text-xs">
                        {user.email?.[0]?.toUpperCase() || 'U'}
                      </div>
                      <div>
                        <div className="text-sm font-medium">{user.email}</div>
                        <div className="text-xs text-gray-500">{user.name || 'No name'}</div>
                      </div>
                    </div>
                    <Badge variant={user.isActive ? 'default' : 'secondary'} className="text-xs">
                      {user.role}
                    </Badge>
                  </label>
                ))}
              </div>
            </div>
            {selectedUsers.length > 0 && (
              <p className="text-sm text-blue-600 dark:text-blue-400">
                {selectedUsers.length} user(s) selected
              </p>
            )}
          </div>

          {/* Operation Selection */}
          <div className="space-y-2">
            <Label htmlFor="operation">Operation</Label>
            <Select
              value={bulkOperation}
              onValueChange={handleOperationChange}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select operation" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="update_role">Update Role</SelectItem>
                <SelectItem value="update_tier">Update Package Tier</SelectItem>
                <SelectItem value="assign_permissions">Assign Permissions</SelectItem>
                <SelectItem value="activate">Activate Users</SelectItem>
                <SelectItem value="deactivate">Deactivate Users</SelectItem>
                <SelectItem value="delete">Delete Users</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Conditional Fields Based on Operation */}
          {bulkOperation === 'update_role' && (
            <div className="space-y-2">
              <Label htmlFor="bulkRole">New Role</Label>
              <Select
                value={form.watch('role') || ''}
                onValueChange={(value) => form.setValue('role', value)}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select role" />
                </SelectTrigger>
                <SelectContent>
                  {roles.map(role => (
                    <SelectItem key={role.value} value={role.value}>
                      {role.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          )}

          {bulkOperation === 'update_tier' && (
            <div className="space-y-2">
              <Label htmlFor="bulkTier">New Package Tier</Label>
              <Select
                value={form.watch('packageTier') || ''}
                onValueChange={(value) => form.setValue('packageTier', value)}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select tier" />
                </SelectTrigger>
                <SelectContent>
                  {tiers.map(tier => (
                    <SelectItem key={tier.value} value={tier.value}>
                      {tier.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          )}

          {bulkOperation === 'assign_permissions' && (
            <div className="space-y-2">
              <Label>Permissions to Assign</Label>
              <div className="text-sm text-gray-500 mb-2">
                Permission assignment functionality will be available when connected to the permissions system.
              </div>
            </div>
          )}

          {/* Reason */}
          <div className="space-y-2">
            <Label htmlFor="bulkReason">Reason <span className="text-red-500">*</span></Label>
            <Textarea
              {...form.register('reason')}
              placeholder="Reason for bulk operation..."
              rows={3}
            />
            {form.formState.errors.reason && (
              <p className="text-sm text-red-500">{form.formState.errors.reason.message}</p>
            )}
          </div>

          {/* Submit Button */}
          <div className="flex justify-end gap-3">
            <Button
              type="button"
              variant="outline"
              onClick={handleReset}
            >
              Reset
            </Button>
            <Button
              type="submit"
              disabled={isSubmitting || selectedUsers.length === 0 || !bulkOperation}
              variant={bulkOperation === 'delete' ? 'destructive' : 'default'}
            >
              {isSubmitting ? (
                <>
                  <Clock className="w-4 h-4 mr-2" />
                  Processing...
                </>
              ) : (
                <>
                  <Users className="w-4 h-4 mr-2" />
                  Apply to {selectedUsers.length} User(s)
                </>
              )}
            </Button>
          </div>
        </form>
      </Card>
    </div>
  );
}