/**
 * Create User Form Component
 * Extracted from UserForms.tsx for better maintainability
 * Follows zero animation policy and strict typing
 */

'use client';

import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { useRouter } from 'next/navigation';
import { 
  UserPlus, 
  Mail,
  Clock
} from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Checkbox } from '@/components/ui/checkbox';
import { Switch } from '@/components/ui/switch';

import { adminClient } from '@/lib/api/unified-admin-client';
import { logger } from '@/lib/logger';
import {
  createUserSchema,
  type CreateUserForm as CreateUserFormData,
  type CreateUserFormProps
} from './shared/user-schemas';
import {
  showSuccessToast,
  showErrorToast,
  createUserFromFormData
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

export function CreateUserForm({
  availableRoles = [],
  availablePackageTiers = [],
  availablePermissions = [],
  onUserCreated,
  className = ''
}: CreateUserFormProps) {
  const router = useRouter();
  const [isSubmitting, setIsSubmitting] = useState(false);

  const form = useForm<CreateUserFormData>({
    resolver: zodResolver(createUserSchema),
    defaultValues: {
      email: '',
      firstName: '',
      lastName: '',
      displayName: '',
      role: 'user',
      packageTier: 'basic',
      permissions: [],
      isActive: true
    }
  });

  const roles = availableRoles.length > 0 ? availableRoles : defaultRoles;
  const tiers = availablePackageTiers.length > 0 ? availablePackageTiers : defaultTiers;

  const handleSubmit = async (data: CreateUserFormData) => {
    setIsSubmitting(true);
    try {
      const response = await adminClient.createUser({
        email: data.email,
        permissions: data.permissions,
        display_name: data.displayName,
      });

      if (response.success && response.data) {
        const newUser = createUserFromFormData({
          ...data,
          id: (response.data as any)?.userId || 'temp-id'
        });

        showSuccessToast(
          "User created successfully",
          `${data.email} has been added to the system.`
        );

        onUserCreated?.(newUser);
        form.reset();
        
        // Navigate back to users list
        router.push('/users');
      } else {
        showErrorToast(
          "Error creating user",
          response.error || "Failed to create user. Please try again."
        );
      }
    } catch (error) {
      logger.error('Create user error', { userData: data, error });
      showErrorToast(
        "Error",
        "Network error. Please check your connection and try again."
      );
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className={`space-y-4 ${className}`}>
      <Card className="p-6">
        <form onSubmit={form.handleSubmit(handleSubmit)} className="space-y-6">
          <div className="flex items-center gap-2 mb-4">
            <UserPlus className="w-5 h-5 text-green-500" />
            <h3 className="text-lg font-medium">Create New User</h3>
          </div>

          {/* Basic Information */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="email">Email <span className="text-red-500">*</span></Label>
              <div className="relative">
                <Mail className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-4 h-4" />
                <Input
                  {...form.register('email')}
                  placeholder="user@example.com"
                  className="pl-10"
                />
              </div>
              {form.formState.errors.email && (
                <p className="text-sm text-red-500">{form.formState.errors.email.message}</p>
              )}
            </div>

            <div className="space-y-2">
              <Label htmlFor="displayName">Display Name</Label>
              <Input
                {...form.register('displayName')}
                placeholder="John Doe"
              />
            </div>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="firstName">First Name</Label>
              <Input
                {...form.register('firstName')}
                placeholder="John"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="lastName">Last Name</Label>
              <Input
                {...form.register('lastName')}
                placeholder="Doe"
              />
            </div>
          </div>

          {/* Role and Tier */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="role">Role</Label>
              <Select
                value={form.watch('role')}
                onValueChange={(value: 'admin' | 'user' | 'premium_user') => form.setValue('role', value)}
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
              {form.formState.errors.role && (
                <p className="text-sm text-red-500">{form.formState.errors.role.message}</p>
              )}
            </div>

            <div className="space-y-2">
              <Label htmlFor="packageTier">Package Tier</Label>
              <Select
                value={form.watch('packageTier')}
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
          </div>

          {/* Permissions */}
          <div className="space-y-2">
            <Label>Initial Permissions (Optional)</Label>
            <div className="border rounded-md p-3 max-h-48 overflow-y-auto">
              <div className="space-y-2">
                {availablePermissions.map(permission => (
                  <label key={permission.id} className="flex items-start gap-3 p-2 hover:bg-gray-100 dark:hover:bg-gray-800 rounded cursor-pointer">
                    <Checkbox
                      checked={form.watch('permissions').includes(permission.id)}
                      onCheckedChange={(checked) => {
                        const currentPermissions = form.watch('permissions');
                        if (checked) {
                          form.setValue('permissions', [...currentPermissions, permission.id]);
                        } else {
                          form.setValue('permissions', currentPermissions.filter(p => p !== permission.id));
                        }
                      }}
                    />
                    <div className="flex-1">
                      <div className="font-medium text-sm">{permission.name}</div>
                      <div className="text-xs text-gray-500">{permission.description}</div>
                      <Badge variant="secondary" className="mt-1 text-xs">
                        {permission.category}
                      </Badge>
                    </div>
                  </label>
                ))}
              </div>
            </div>
          </div>

          {/* Account Status */}
          <div className="flex items-center space-x-2">
            <Switch
              checked={form.watch('isActive')}
              onCheckedChange={(checked) => form.setValue('isActive', checked)}
            />
            <Label>Account Active</Label>
          </div>

          {/* Submit Buttons */}
          <div className="flex justify-end gap-3">
            <Button
              type="button"
              variant="outline"
              onClick={() => form.reset()}
            >
              Reset
            </Button>
            <Button
              type="submit"
              disabled={isSubmitting}
            >
              {isSubmitting ? (
                <>
                  <Clock className="w-4 h-4 mr-2" />
                  Creating...
                </>
              ) : (
                <>
                  <UserPlus className="w-4 h-4 mr-2" />
                  Create User
                </>
              )}
            </Button>
          </div>
        </form>
      </Card>
    </div>
  );
}