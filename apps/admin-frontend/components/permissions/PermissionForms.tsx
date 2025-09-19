/**
 * Permission Forms - All Grant/Request/Bulk Operations
 * Consolidates: GrantPermissionForm, GrantPermissionModal, PermissionRequestForm,
 * RequestPermissionModal, BulkPermissionAssignment, BulkPermissionManager,
 * BulkRoleManager, TemporaryPermissionForm, TemporaryPermissionManager
 */

'use client';

import { useState, useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { 
  Users, 
  Shield, 
  Calendar, 
  Plus, 
  Trash2, 
  Clock,
  CheckCircle,
  XCircle,
  AlertCircle
} from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Checkbox } from '@/components/ui/checkbox';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';

import type { User, Permission, Platform } from '@/types/core';
import { adminClient } from '@/lib/api/unified-admin-client';

// Form validation schemas
const grantPermissionSchema = z.object({
  userId: z.string().min(1, 'User is required'),
  permissions: z.array(z.string()).min(1, 'At least one permission is required'),
  expiresAt: z.string().optional(),
  reason: z.string().min(3, 'Reason must be at least 3 characters').optional()
});

const bulkPermissionSchema = z.object({
  userIds: z.array(z.string()).min(1, 'At least one user is required'),
  permissions: z.array(z.string()).min(1, 'At least one permission is required'),
  expiresAt: z.string().optional(),
  reason: z.string().min(3, 'Reason must be at least 3 characters').optional()
});

const requestPermissionSchema = z.object({
  permissions: z.array(z.string()).min(1, 'At least one permission is required'),
  justification: z.string().min(10, 'Justification must be at least 10 characters'),
  urgency: z.enum(['low', 'medium', 'high', 'critical']),
  duration: z.enum(['temporary', 'permanent']),
  expiresAt: z.string().optional()
});

type GrantPermissionForm = z.infer<typeof grantPermissionSchema>;
type BulkPermissionForm = z.infer<typeof bulkPermissionSchema>;
type RequestPermissionForm = z.infer<typeof requestPermissionSchema>;

interface PermissionFormsProps {
  users?: User[];
  availablePermissions?: Array<{
    id: string;
    name: string;
    description: string;
    platform: Platform;
    category: string;
  }>;
  onPermissionGranted?: (data: any) => void;
  onPermissionRequested?: (data: any) => void;
  onBulkOperationComplete?: (result: any) => void;
  className?: string;
}

interface PermissionSelectorProps {
  permissions: Array<{
    id: string;
    name: string;
    description: string;
    platform: Platform;
    category: string;
  }>;
  selectedPermissions: string[];
  onSelectionChange: (permissions: string[]) => void;
  onFormUpdate: (permissions: string[]) => void;
  searchQuery: string;
  onSearchChange: (query: string) => void;
  filterPlatform: string;
  onFilterChange: (platform: string) => void;
}

export function PermissionForms({
  users = [],
  availablePermissions = [],
  onPermissionGranted,
  onPermissionRequested,
  onBulkOperationComplete,
  className = ''
}: PermissionFormsProps) {
  const [activeTab, setActiveTab] = useState('grant');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [selectedUsers, setSelectedUsers] = useState<string[]>([]);
  const [selectedPermissions, setSelectedPermissions] = useState<string[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [filterPlatform, setFilterPlatform] = useState<string>('all');
  
  // Grant Permission Form
  const grantForm = useForm<GrantPermissionForm>({
    resolver: zodResolver(grantPermissionSchema),
    defaultValues: {
      userId: '',
      permissions: [],
      expiresAt: '',
      reason: ''
    }
  });

  // Bulk Permission Form
  const bulkForm = useForm<BulkPermissionForm>({
    resolver: zodResolver(bulkPermissionSchema),
    defaultValues: {
      userIds: [],
      permissions: [],
      expiresAt: '',
      reason: ''
    }
  });

  // Request Permission Form
  const requestForm = useForm<RequestPermissionForm>({
    resolver: zodResolver(requestPermissionSchema),
    defaultValues: {
      permissions: [],
      justification: '',
      urgency: 'medium',
      duration: 'temporary',
      expiresAt: ''
    }
  });

  // Filtered permissions based on search and platform
  const filteredPermissions = availablePermissions.filter(permission => {
    const matchesSearch = searchQuery === '' || 
      permission.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      permission.description.toLowerCase().includes(searchQuery.toLowerCase());
    
    const matchesPlatform = filterPlatform === 'all' || 
      permission.platform === filterPlatform;
    
    return matchesSearch && matchesPlatform;
  });

  // Grouped permissions by category
  const permissionsByCategory = filteredPermissions.reduce((acc, permission) => {
    if (!acc[permission.category]) {
      acc[permission.category] = [];
    }
    acc[permission.category].push(permission);
    return acc;
  }, {} as Record<string, typeof availablePermissions>);

  // Grant permission handler
  const handleGrantPermission = async (data: GrantPermissionForm) => {
    setIsSubmitting(true);
    try {
      for (const permission of data.permissions) {
        const response = await adminClient.grantPermission(
          data.userId,
          permission,
          data.expiresAt
        );
        
        if (!response.success) {
          throw new Error(response.error || 'Failed to grant permission');
        }
      }
      
      onPermissionGranted?.({
        userId: data.userId,
        permissions: data.permissions,
        expiresAt: data.expiresAt,
        reason: data.reason
      });
      
      grantForm.reset();
      setSelectedPermissions([]);
      
    } catch (error) {
      console.error('Grant permission error:', error);
    } finally {
      setIsSubmitting(false);
    }
  };

  // Bulk grant permission handler
  const handleBulkGrantPermission = async (data: BulkPermissionForm) => {
    setIsSubmitting(true);
    try {
      const response = await adminClient.bulkGrantPermissions(
        data.userIds,
        data.permissions,
        data.expiresAt
      );
      
      if (!response.success) {
        throw new Error(response.error || 'Bulk operation failed');
      }
      
      onBulkOperationComplete?.(response.data);
      
      bulkForm.reset();
      setSelectedUsers([]);
      setSelectedPermissions([]);
      
    } catch (error) {
      console.error('Bulk grant error:', error);
    } finally {
      setIsSubmitting(false);
    }
  };

  // Request permission handler
  const handleRequestPermission = async (data: RequestPermissionForm) => {
    setIsSubmitting(true);
    try {
      // In a real implementation, this would create a permission request
      // that admins can approve/deny
      const requestData = {
        permissions: data.permissions,
        justification: data.justification,
        urgency: data.urgency,
        duration: data.duration,
        expiresAt: data.expiresAt,
        requestedAt: new Date().toISOString()
      };
      
      onPermissionRequested?.(requestData);
      
      requestForm.reset();
      setSelectedPermissions([]);
      
    } catch (error) {
      console.error('Request permission error:', error);
    } finally {
      setIsSubmitting(false);
    }
  };

  // Permission Selector Component
  function PermissionSelector({
    permissions,
    selectedPermissions,
    onSelectionChange,
    onFormUpdate,
    searchQuery,
    onSearchChange,
    filterPlatform,
    onFilterChange
  }: PermissionSelectorProps) {
    const handlePermissionToggle = (permissionId: string) => {
      const updated = selectedPermissions.includes(permissionId)
        ? selectedPermissions.filter(id => id !== permissionId)
        : [...selectedPermissions, permissionId];
      
      onSelectionChange(updated);
      onFormUpdate(updated);
    };

    const permissionsByCategory = permissions.reduce((acc, permission) => {
      if (!acc[permission.category]) {
        acc[permission.category] = [];
      }
      acc[permission.category].push(permission);
      return acc;
    }, {} as Record<string, typeof permissions>);

    return (
      <div className="space-y-4">
        <div>
          <Label>Select Permissions</Label>
          <p className="text-sm text-gray-500">Choose which permissions to grant</p>
        </div>

        {/* Permission Filters */}
        <div className="flex flex-col sm:flex-row gap-3">
          <Input
            placeholder="Search permissions..."
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            className="flex-1"
          />
          <Select value={filterPlatform} onValueChange={onFilterChange}>
            <SelectTrigger className="w-full sm:w-48">
              <SelectValue placeholder="All Platforms" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Platforms</SelectItem>
              <SelectItem value="admin">Admin</SelectItem>
              <SelectItem value="epsx">EPSX</SelectItem>
              <SelectItem value="epsx-pay">EPSX Pay</SelectItem>
              <SelectItem value="epsx-token">EPSX Token</SelectItem>
            </SelectContent>
          </Select>
        </div>

        {/* Permission Categories */}
        <div className="space-y-4 max-h-64 overflow-y-auto">
          {Object.entries(permissionsByCategory).map(([category, categoryPermissions]) => {
            const filteredPermissions = categoryPermissions.filter(permission => {
              const matchesSearch = permission.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                                  permission.description.toLowerCase().includes(searchQuery.toLowerCase());
              const matchesPlatform = filterPlatform === 'all' || permission.platform === filterPlatform;
              return matchesSearch && matchesPlatform;
            });

            if (filteredPermissions.length === 0) return null;

            return (
              <div key={category} className="space-y-2">
                <h4 className="font-semibold text-sm text-gray-700 dark:text-gray-300 capitalize">
                  {category.replace(/([A-Z])/g, ' $1').trim()}
                </h4>
                <div className="grid grid-cols-1 gap-2">
                  {filteredPermissions.map(permission => (
                    <div
                      key={permission.id}
                      className="flex items-center space-x-3 p-3 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800"
                    >
                      <Checkbox
                        id={permission.id}
                        checked={selectedPermissions.includes(permission.id)}
                        onCheckedChange={() => handlePermissionToggle(permission.id)}
                      />
                      <div className="flex-1">
                        <div className="flex items-center gap-2">
                          <Label htmlFor={permission.id} className="font-medium cursor-pointer">
                            {permission.name}
                          </Label>
                          <Badge variant="outline" className="text-xs">
                            {permission.platform}
                          </Badge>
                        </div>
                        <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                          {permission.description}
                        </p>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            );
          })}
        </div>

        {Object.keys(permissionsByCategory).length === 0 && (
          <div className="text-center py-8 text-gray-500 dark:text-gray-400">
            No permissions found matching your criteria.
          </div>
        )}

        {selectedPermissions.length > 0 && (
          <div className="bg-blue-500/10 border border-blue-500/20 rounded-md p-3">
            <p className="text-sm text-blue-400">
              {selectedPermissions.length} permission(s) selected
            </p>
          </div>
        )}
      </div>
    );
  }

  return (
    <div className={`space-y-6 sm:space-y-8 ${className}`}>
      {/* Background Decorations */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-yellow-400/20 to-orange-500/20 rounded-full blur-xl"></div>
        <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-pink-400/20 to-purple-500/20 rounded-full blur-lg"></div>
        <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-orange-400/15 to-yellow-500/15 rounded-full blur-xl"></div>
      </div>

      {/* Header */}
      <div className="relative text-center mb-8 sm:mb-12">
        <div className="relative inline-block">
          <h1 className="text-3xl sm:text-4xl lg:text-5xl font-bold bg-gradient-to-r from-yellow-600 via-orange-600 via-pink-600 to-purple-600 bg-clip-text text-transparent mb-4">
            🔍 Permission Operations
          </h1>
          <div className="absolute -top-2 -right-2 w-4 h-4 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-full"></div>
        </div>
        <p className="text-base sm:text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
          Grant, request, and manage permissions across the EPSX platform
        </p>
      </div>

      {/* Main Form Tabs */}
      <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-purple-400/20 via-blue-400/20 to-green-400/20 p-0.5">
        <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4">
          <Tabs value={activeTab} onValueChange={setActiveTab}>
            <TabsList className="grid w-full grid-cols-1 sm:grid-cols-3 gap-1 p-1 bg-gray-100 dark:bg-gray-800 rounded-2xl">
              <TabsTrigger value="grant" className="rounded-xl min-h-[44px] text-sm font-medium">Grant Permissions</TabsTrigger>
              <TabsTrigger value="bulk" className="rounded-xl min-h-[44px] text-sm font-medium">Bulk Operations</TabsTrigger>
              <TabsTrigger value="request" className="rounded-xl min-h-[44px] text-sm font-medium">Request Permissions</TabsTrigger>
            </TabsList>

            {/* Grant Permissions Tab */}
            <TabsContent value="grant" className="space-y-4 sm:space-y-6 mt-6">
              <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-blue-400/20 via-indigo-400/20 to-purple-400/20 p-0.5">
                <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6">
                  <form onSubmit={grantForm.handleSubmit(handleGrantPermission)} className="space-y-4 sm:space-y-6">
                    <div className="flex items-center gap-3 mb-4">
                      <div className="h-10 w-10 bg-gradient-to-br from-blue-500 to-indigo-500 rounded-2xl flex items-center justify-center">
                        <Shield className="w-5 h-5 text-white" />
                      </div>
                      <h3 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-blue-600 via-indigo-600 to-purple-600 bg-clip-text text-transparent">Grant Permissions to User</h3>
                    </div>

                    {/* User Selection */}
                    <div className="space-y-2">
                      <Label htmlFor="userId" className="text-sm font-semibold text-gray-700 dark:text-gray-300">Select User</Label>
                      <Select
                        value={grantForm.watch('userId')}
                        onValueChange={(value) => grantForm.setValue('userId', value)}
                      >
                        <SelectTrigger className="min-h-[44px] rounded-2xl border-2">
                          <SelectValue placeholder="Choose a user..." />
                        </SelectTrigger>
                        <SelectContent>
                          {users.map(user => (
                            <SelectItem key={user.id} value={user.id}>
                              <div className="flex items-center gap-2">
                                <div className="w-6 h-6 rounded-full bg-blue-500/20 flex items-center justify-center text-xs">
                                  {user.email[0].toUpperCase()}
                                </div>
                                <div>
                                  <div className="font-medium">{user.email}</div>
                                  <div className="text-xs text-gray-500">{user.name}</div>
                                </div>
                              </div>
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                      {grantForm.formState.errors.userId && (
                        <p className="text-sm text-red-500">{grantForm.formState.errors.userId.message}</p>
                      )}
                    </div>

              {/* Permission Selection */}
              <PermissionSelector
                permissions={filteredPermissions}
                selectedPermissions={selectedPermissions}
                onSelectionChange={setSelectedPermissions}
                onFormUpdate={(permissions) => grantForm.setValue('permissions', permissions)}
                searchQuery={searchQuery}
                onSearchChange={setSearchQuery}
                filterPlatform={filterPlatform}
                onFilterChange={setFilterPlatform}
              />
              
              {grantForm.formState.errors.permissions && (
                <p className="text-sm text-red-500">{grantForm.formState.errors.permissions.message}</p>
              )}

                    {/* Expiry Date */}
                    <div className="space-y-2">
                      <Label htmlFor="expiresAt" className="text-sm font-semibold text-gray-700 dark:text-gray-300">Expiry Date (Optional)</Label>
                      <div className="flex items-center gap-2">
                        <Calendar className="w-4 h-4 text-gray-400" />
                        <Input
                          type="datetime-local"
                          {...grantForm.register('expiresAt')}
                          min={new Date().toISOString().slice(0, 16)}
                          className="rounded-2xl border-2 min-h-[44px]"
                        />
                      </div>
                      <p className="text-xs text-gray-500 dark:text-gray-400">
                        Leave empty for permanent permissions
                      </p>
                    </div>

                    {/* Reason */}
                    <div className="space-y-2">
                      <Label htmlFor="reason" className="text-sm font-semibold text-gray-700 dark:text-gray-300">Reason (Optional)</Label>
                      <Textarea
                        placeholder="Reason for granting these permissions..."
                        {...grantForm.register('reason')}
                        rows={3}
                        className="rounded-2xl border-2"
                      />
                      {grantForm.formState.errors.reason && (
                        <p className="text-sm text-red-500">{grantForm.formState.errors.reason.message}</p>
                      )}
                    </div>

                    {/* Submit Button */}
                    <div className="flex flex-col sm:flex-row justify-end gap-3">
                      <Button
                        type="button"
                        variant="outline"
                        onClick={() => {
                          grantForm.reset();
                          setSelectedPermissions([]);
                        }}
                        className="min-h-[44px] rounded-2xl border-2"
                      >
                        Reset
                      </Button>
                      <Button
                        type="submit"
                        disabled={isSubmitting || selectedPermissions.length === 0}
                        className="min-h-[44px] rounded-2xl bg-gradient-to-r from-green-500 to-emerald-500 hover:from-green-600 hover:to-emerald-600"
                      >
                        {isSubmitting ? (
                          <>
                            <Clock className="w-4 h-4 mr-2" />
                            Granting...
                          </>
                        ) : (
                          <>
                            <Plus className="w-4 h-4 mr-2" />
                            Grant Permissions
                          </>
                        )}
                      </Button>
                    </div>
                  </form>
                </div>
              </div>
            </TabsContent>

            {/* Bulk Operations Tab */}
            <TabsContent value="bulk" className="space-y-4 sm:space-y-6 mt-6">
              <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-green-400/20 via-emerald-400/20 to-teal-400/20 p-0.5">
                <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6">
                  <form onSubmit={bulkForm.handleSubmit(handleBulkGrantPermission)} className="space-y-4 sm:space-y-6">
                    <div className="flex items-center gap-3 mb-4">
                      <div className="h-10 w-10 bg-gradient-to-br from-green-500 to-emerald-500 rounded-2xl flex items-center justify-center">
                        <Users className="w-5 h-5 text-white" />
                      </div>
                      <h3 className="text-lg sm:text-xl font-bold bg-gradient-to-r from-green-600 via-emerald-600 to-teal-600 bg-clip-text text-transparent">Bulk Permission Operations</h3>
                    </div>

                    {/* Multiple User Selection */}
                    <div className="space-y-2">
                      <Label className="text-sm font-semibold text-gray-700 dark:text-gray-300">Select Users</Label>
                      <div className="border-2 border-gray-300 dark:border-gray-600 rounded-2xl p-3 max-h-64 overflow-y-auto bg-gray-50 dark:bg-gray-800">
                        <div className="space-y-3">
                          {users.map(user => (
                            <label key={user.id} className="flex items-center gap-3 p-3 hover:bg-white dark:hover:bg-gray-700 rounded-xl cursor-pointer">
                              <Checkbox
                                checked={selectedUsers.includes(user.id)}
                                onCheckedChange={(checked) => {
                                  if (checked) {
                                    setSelectedUsers([...selectedUsers, user.id]);
                                    bulkForm.setValue('userIds', [...selectedUsers, user.id]);
                                  } else {
                                    const updated = selectedUsers.filter(id => id !== user.id);
                                    setSelectedUsers(updated);
                                    bulkForm.setValue('userIds', updated);
                                  }
                                }}
                                className="h-5 w-5 rounded border-gray-300 text-green-600 focus:ring-green-500"
                              />
                              <div className="flex items-center gap-3 flex-1">
                                <div className="w-8 h-8 rounded-full bg-gradient-to-br from-blue-500 to-indigo-500 flex items-center justify-center text-xs text-white font-semibold">
                                  {user.email[0].toUpperCase()}
                                </div>
                                <div className="flex-1">
                                  <div className="text-sm font-semibold text-gray-900 dark:text-white">{user.email}</div>
                                  <div className="text-xs text-gray-500 dark:text-gray-400">{user.name || 'No name'}</div>
                                </div>
                              </div>
                            </label>
                          ))}
                        </div>
                      </div>
                      {selectedUsers.length > 0 && (
                        <p className="text-sm text-green-600 dark:text-green-400 font-medium">
                          ✓ {selectedUsers.length} user(s) selected
                        </p>
                      )}
                      {bulkForm.formState.errors.userIds && (
                        <p className="text-sm text-red-500">{bulkForm.formState.errors.userIds.message}</p>
                      )}
                    </div>

              {/* Permission Selection for Bulk */}
              <PermissionSelector
                permissions={filteredPermissions}
                selectedPermissions={selectedPermissions}
                onSelectionChange={setSelectedPermissions}
                onFormUpdate={(permissions) => bulkForm.setValue('permissions', permissions)}
                searchQuery={searchQuery}
                onSearchChange={setSearchQuery}
                filterPlatform={filterPlatform}
                onFilterChange={setFilterPlatform}
              />
              
              {bulkForm.formState.errors.permissions && (
                <p className="text-sm text-red-500">{bulkForm.formState.errors.permissions.message}</p>
              )}

                    {/* Bulk Expiry Date */}
                    <div className="space-y-2">
                      <Label htmlFor="bulkExpiresAt" className="text-sm font-semibold text-gray-700 dark:text-gray-300">Expiry Date (Optional)</Label>
                      <div className="flex items-center gap-2">
                        <Calendar className="w-4 h-4 text-gray-400" />
                        <Input
                          type="datetime-local"
                          {...bulkForm.register('expiresAt')}
                          min={new Date().toISOString().slice(0, 16)}
                          className="rounded-2xl border-2 min-h-[44px]"
                        />
                      </div>
                    </div>

                    {/* Bulk Reason */}
                    <div className="space-y-2">
                      <Label htmlFor="bulkReason" className="text-sm font-semibold text-gray-700 dark:text-gray-300">Reason</Label>
                      <Textarea
                        placeholder="Reason for bulk permission grant..."
                        {...bulkForm.register('reason')}
                        rows={3}
                        className="rounded-2xl border-2"
                      />
                      {bulkForm.formState.errors.reason && (
                        <p className="text-sm text-red-500">{bulkForm.formState.errors.reason.message}</p>
                      )}
                    </div>

                    {/* Submit Button */}
                    <div className="flex flex-col sm:flex-row justify-end gap-3">
                      <Button
                        type="button"
                        variant="outline"
                        onClick={() => {
                          bulkForm.reset();
                          setSelectedUsers([]);
                          setSelectedPermissions([]);
                        }}
                        className="min-h-[44px] rounded-2xl border-2"
                      >
                        Reset
                      </Button>
                      <Button
                        type="submit"
                        disabled={isSubmitting || selectedUsers.length === 0 || selectedPermissions.length === 0}
                        className="min-h-[44px] rounded-2xl bg-gradient-to-r from-green-500 to-emerald-500 hover:from-green-600 hover:to-emerald-600"
                      >
                        {isSubmitting ? (
                          <>
                            <Clock className="w-4 h-4 mr-2" />
                            Processing...
                          </>
                        ) : (
                          <>
                            <Users className="w-4 h-4 mr-2" />
                            Grant to {selectedUsers.length} User(s)
                          </>
                        )}
                      </Button>
                    </div>
                  </form>
                </div>
              </div>
            </TabsContent>

            {/* Request Permissions Tab */}
            <TabsContent value="request" className="space-y-4">
              <Card className="p-6">
                <form onSubmit={requestForm.handleSubmit(handleRequestPermission)} className="space-y-6">
              <div className="flex items-center gap-2 mb-4">
                <AlertCircle className="w-5 h-5 text-yellow-500" />
                <h3 className="text-lg font-medium text-white">Request Permissions</h3>
              </div>

              {/* Permission Selection for Request */}
              <PermissionSelector
                permissions={filteredPermissions}
                selectedPermissions={selectedPermissions}
                onSelectionChange={setSelectedPermissions}
                onFormUpdate={(permissions) => requestForm.setValue('permissions', permissions)}
                searchQuery={searchQuery}
                onSearchChange={setSearchQuery}
                filterPlatform={filterPlatform}
                onFilterChange={setFilterPlatform}
              />
              
              {requestForm.formState.errors.permissions && (
                <p className="text-sm text-red-500">{requestForm.formState.errors.permissions.message}</p>
              )}

              {/* Justification */}
              <div className="space-y-2">
                <Label htmlFor="justification">Justification <span className="text-red-500">*</span></Label>
                <Textarea
                  placeholder="Please explain why you need these permissions..."
                  {...requestForm.register('justification')}
                  rows={4}
                />
                {requestForm.formState.errors.justification && (
                  <p className="text-sm text-red-500">{requestForm.formState.errors.justification.message}</p>
                )}
              </div>

              {/* Urgency and Duration */}
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="urgency">Urgency</Label>
                  <Select
                    value={requestForm.watch('urgency')}
                    onValueChange={(value: any) => requestForm.setValue('urgency', value)}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="low">Low</SelectItem>
                      <SelectItem value="medium">Medium</SelectItem>
                      <SelectItem value="high">High</SelectItem>
                      <SelectItem value="critical">Critical</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="duration">Duration</Label>
                  <Select
                    value={requestForm.watch('duration')}
                    onValueChange={(value: any) => requestForm.setValue('duration', value)}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="temporary">Temporary</SelectItem>
                      <SelectItem value="permanent">Permanent</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>

              {/* Conditional Expiry Date */}
              {requestForm.watch('duration') === 'temporary' && (
                <div className="space-y-2">
                  <Label htmlFor="requestExpiresAt">Requested Expiry Date</Label>
                  <div className="flex items-center gap-2">
                    <Calendar className="w-4 h-4 text-gray-400" />
                    <Input
                      type="datetime-local"
                      {...requestForm.register('expiresAt')}
                      min={new Date().toISOString().slice(0, 16)}
                    />
                  </div>
                </div>
              )}

              {/* Submit Button */}
              <div className="flex justify-end gap-3">
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => {
                    requestForm.reset();
                    setSelectedPermissions([]);
                  }}
                >
                  Reset
                </Button>
                <Button
                  type="submit"
                  disabled={isSubmitting || selectedPermissions.length === 0}
                >
                  {isSubmitting ? (
                    <>
                      <Clock className="w-4 h-4 mr-2" />
                      Submitting...
                    </>
                  ) : (
                    <>
                      <AlertCircle className="w-4 h-4 mr-2" />
                      Submit Request
                    </>
                  )}
                </Button>
              </div>
                </form>
              </Card>
            </TabsContent>
          </Tabs>
        </div>
      </div>
    </div>
  );
}

export default PermissionForms;
