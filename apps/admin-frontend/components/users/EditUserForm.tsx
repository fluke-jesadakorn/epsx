/**
 * Edit User Form Component
 * Extracted from UserForms.tsx for better maintainability
 * Follows zero animation policy and strict typing
 */

'use client';

import { useState, useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { useRouter } from 'next/navigation';
import { 
  Edit3, 
  Mail,
  Shield,
  Clock,
  Unlock,
  Lock,
  AlertCircle
} from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Checkbox } from '@/components/ui/checkbox';
import { Switch } from '@/components/ui/switch';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';

import { adminClient } from '@/lib/api/unified-admin-client';
import { logger } from '@/lib/logger';
import {
  editUserSchema,
  type EditUserForm as EditUserFormData,
  type EditUserFormProps
} from './shared/user-schemas';
import {
  showSuccessToast,
  showErrorToast,
  updateUserFromFormData
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

export function EditUserForm({
  editUser,
  availableRoles = [],
  availablePackageTiers = [],
  availablePermissions = [],
  onUserUpdated,
  className = ''
}: EditUserFormProps) {
  const router = useRouter();
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [activeTab, setActiveTab] = useState('basic');

  const form = useForm<EditUserFormData>({
    resolver: zodResolver(editUserSchema),
    defaultValues: {
      id: '',
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

  // Initialize form with user data
  useEffect(() => {
    if (editUser) {
      form.reset({
        id: editUser.id,
        email: editUser.email,
        firstName: editUser.firstName,
        lastName: editUser.lastName,
        displayName: editUser.displayName,
        role: editUser.role,
        packageTier: editUser.packageTier,
        permissions: editUser.permissions || [],
        isActive: editUser.isActive
      });
    }
  }, [editUser, form]);

  const handleSubmit = async (data: EditUserFormData) => {
    setIsSubmitting(true);
    try {
      const updateData = {
        email: data.email,
        display_name: data.displayName,
        first_name: data.firstName,
        last_name: data.lastName,
        role: data.role,
        package_tier: data.packageTier,
        is_active: data.isActive,
        permissions: data.permissions || []
      };
      
      const response = await adminClient.updateUser(data.id!, updateData);

      if (response.success) {
        const updatedUser = updateUserFromFormData(editUser, data);
        
        showSuccessToast(
          "User updated successfully",
          `${data.email || editUser.email} has been updated.`
        );
        
        onUserUpdated?.(updatedUser);
        
        // Navigate to user profile
        router.push(`/users/${data.id}`);
      } else {
        showErrorToast(
          "Error updating user",
          response.error || "Failed to update user. Please try again."
        );
      }
    } catch (error) {
      logger.error('Edit user error', { userId: editUser.id, updateData: data, error });
      showErrorToast(
        "Error",
        "Network error. Please check your connection and try again."
      );
    } finally {
      setIsSubmitting(false);
    }
  };

  if (!editUser) {
    return (
      <div className={`space-y-4 ${className}`}>
        <Card className="p-6">
          <div className="text-center">
            <p className="text-gray-500">No user selected for editing.</p>
          </div>
        </Card>
      </div>
    );
  }

  return (
    <div className={`space-y-4 ${className}`}>
      <Card className="p-6">
        <form onSubmit={form.handleSubmit(handleSubmit)} className="space-y-6">
          <div className="flex items-center gap-2 mb-4">
            <Edit3 className="w-5 h-5 text-blue-500" />
            <h3 className="text-lg font-medium">Edit User Profile</h3>
          </div>

          <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
            <TabsList className="grid w-full grid-cols-2">
              <TabsTrigger value="basic">Basic Information</TabsTrigger>
              <TabsTrigger value="access">Access Control</TabsTrigger>
            </TabsList>

            <TabsContent value="basic" className="space-y-6">
              {/* Basic Information */}
              <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                <div className="space-y-2">
                  <Label htmlFor="email" className="text-sm font-medium">
                    Email Address <span className="text-red-500">*</span>
                  </Label>
                  <div className="relative">
                    <Mail className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-4 h-4" />
                    <Input
                      {...form.register('email')}
                      placeholder="user@example.com"
                      className="pl-10"
                    />
                  </div>
                  {form.formState.errors.email && (
                    <p className="text-sm text-red-500 flex items-center gap-1">
                      <AlertCircle className="w-4 h-4" />
                      {form.formState.errors.email.message}
                    </p>
                  )}
                </div>

                <div className="space-y-2">
                  <Label htmlFor="displayName" className="text-sm font-medium">
                    Display Name
                  </Label>
                  <Input
                    {...form.register('displayName')}
                    placeholder="John Doe"
                  />
                </div>
              </div>

              <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                <div className="space-y-2">
                  <Label htmlFor="firstName" className="text-sm font-medium">
                    First Name
                  </Label>
                  <Input
                    {...form.register('firstName')}
                    placeholder="John"
                  />
                </div>

                <div className="space-y-2">
                  <Label htmlFor="lastName" className="text-sm font-medium">
                    Last Name
                  </Label>
                  <Input
                    {...form.register('lastName')}
                    placeholder="Doe"
                  />
                </div>
              </div>
            </TabsContent>

            <TabsContent value="access" className="space-y-6">
              {/* Access Control */}
              <div className="flex items-center gap-2 mb-4">
                <Shield className="w-5 h-5 text-indigo-500" />
                <h4 className="text-md font-medium">Access Control</h4>
              </div>

              <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                <div className="space-y-2">
                  <Label htmlFor="role" className="text-sm font-medium">
                    User Role <span className="text-red-500">*</span>
                  </Label>
                  <Select
                    value={form.watch('role') || ''}
                    onValueChange={(value: 'admin' | 'user' | 'premium_user') => form.setValue('role', value)}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select user role" />
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
                    <p className="text-sm text-red-500 flex items-center gap-1">
                      <AlertCircle className="w-4 h-4" />
                      {form.formState.errors.role.message}
                    </p>
                  )}
                </div>

                <div className="space-y-2">
                  <Label htmlFor="packageTier" className="text-sm font-medium">
                    Package Tier
                  </Label>
                  <Select
                    value={form.watch('packageTier') || ''}
                    onValueChange={(value) => form.setValue('packageTier', value)}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select package tier" />
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
                <Label className="text-sm font-medium">User Permissions</Label>
                <div className="border rounded-md p-4 max-h-64 overflow-y-auto">
                  {availablePermissions.length > 0 ? (
                    <div className="space-y-2">
                      {availablePermissions.map(permission => (
                        <label key={permission.id} className="flex items-start gap-3 p-2 hover:bg-gray-100 dark:hover:bg-gray-800 rounded cursor-pointer">
                          <Checkbox
                            checked={(form.watch('permissions') || []).includes(permission.id)}
                            onCheckedChange={(checked) => {
                              const currentPermissions = form.watch('permissions') || [];
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
                  ) : (
                    <div className="text-center py-8">
                      <Shield className="w-12 h-12 text-gray-400 mx-auto mb-4" />
                      <p className="text-gray-500">No permissions available to assign</p>
                    </div>
                  )}
                </div>
              </div>

              {/* Account Status */}
              <div className="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800 rounded-md">
                <div className="flex items-center gap-3">
                  {form.watch('isActive') ? (
                    <Unlock className="w-5 h-5 text-green-500" />
                  ) : (
                    <Lock className="w-5 h-5 text-red-500" />
                  )}
                  <div>
                    <Label className="text-sm font-medium">Account Status</Label>
                    <p className="text-xs text-gray-500">
                      {form.watch('isActive') ? 'Account is active and can access the system' : 'Account is disabled and cannot access the system'}
                    </p>
                  </div>
                </div>
                <Switch
                  checked={form.watch('isActive')}
                  onCheckedChange={(checked) => form.setValue('isActive', checked)}
                />
              </div>
            </TabsContent>
          </Tabs>

          {/* Submit Buttons */}
          <div className="flex justify-end gap-3">
            <Button
              type="button"
              variant="outline"
              onClick={() => router.push(`/users/${editUser.id}`)}
            >
              Cancel
            </Button>
            <Button
              type="submit"
              disabled={isSubmitting}
            >
              {isSubmitting ? (
                <>
                  <Clock className="w-4 h-4 mr-2" />
                  Updating...
                </>
              ) : (
                <>
                  <Edit3 className="w-4 h-4 mr-2" />
                  Update User
                </>
              )}
            </Button>
          </div>
        </form>
      </Card>
    </div>
  );
}