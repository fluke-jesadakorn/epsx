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

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div>
        <h2 className="text-xl font-bold text-white mb-2">Permission Operations</h2>
        <p className="text-gray-400">Grant, request, and manage permissions</p>
      </div>

      {/* Main Form Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="grant">Grant Permissions</TabsTrigger>
          <TabsTrigger value="bulk">Bulk Operations</TabsTrigger>
          <TabsTrigger value="request">Request Permissions</TabsTrigger>
        </TabsList>

        {/* Grant Permissions Tab */}
        <TabsContent value="grant" className="space-y-4">
          <Card className="p-6">
            <form onSubmit={grantForm.handleSubmit(handleGrantPermission)} className="space-y-6">
              <div className="flex items-center gap-2 mb-4">
                <Shield className="w-5 h-5 text-blue-500" />
                <h3 className="text-lg font-medium text-white">Grant Permissions to User</h3>
              </div>

              {/* User Selection */}
              <div className="space-y-2">
                <Label htmlFor="userId">Select User</Label>
                <Select
                  value={grantForm.watch('userId')}
                  onValueChange={(value) => grantForm.setValue('userId', value)}
                >
                  <SelectTrigger>
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
                <Label htmlFor="expiresAt">Expiry Date (Optional)</Label>
                <div className="flex items-center gap-2">
                  <Calendar className="w-4 h-4 text-gray-400" />
                  <Input
                    type="datetime-local"
                    {...grantForm.register('expiresAt')}
                    min={new Date().toISOString().slice(0, 16)}
                  />
                </div>
                <p className="text-xs text-gray-500">
                  Leave empty for permanent permissions
                </p>
              </div>

              {/* Reason */}
              <div className="space-y-2">
                <Label htmlFor="reason">Reason (Optional)</Label>
                <Textarea
                  placeholder="Reason for granting these permissions..."
                  {...grantForm.register('reason')}
                  rows={3}
                />
                {grantForm.formState.errors.reason && (
                  <p className="text-sm text-red-500">{grantForm.formState.errors.reason.message}</p>
                )}
              </div>

              {/* Submit Button */}
              <div className="flex justify-end gap-3">
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => {
                    grantForm.reset();
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
                      <Clock className="w-4 h-4 mr-2 animate-spin" />
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
          </Card>
        </TabsContent>

        {/* Bulk Operations Tab */}
        <TabsContent value="bulk" className="space-y-4">
          <Card className="p-6">
            <form onSubmit={bulkForm.handleSubmit(handleBulkGrantPermission)} className="space-y-6">
              <div className="flex items-center gap-2 mb-4">
                <Users className="w-5 h-5 text-green-500" />
                <h3 className="text-lg font-medium text-white">Bulk Permission Operations</h3>
              </div>

              {/* Multiple User Selection */}
              <div className="space-y-2">
                <Label>Select Users</Label>
                <div className="border border-gray-700 rounded-md p-3 max-h-48 overflow-y-auto">
                  <div className="space-y-2">
                    {users.map(user => (
                      <label key={user.id} className="flex items-center gap-3 p-2 hover:bg-gray-800 rounded">
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
                        />
                        <div className="flex items-center gap-2 flex-1">
                          <div className="w-6 h-6 rounded-full bg-blue-500/20 flex items-center justify-center text-xs">
                            {user.email[0].toUpperCase()}
                          </div>
                          <div className="flex-1">
                            <div className="text-sm font-medium">{user.email}</div>
                            <div className="text-xs text-gray-500">{user.name || 'No name'}</div>
                          </div>
                        </div>
                      </label>
                    ))}
                  </div>
                </div>
                {selectedUsers.length > 0 && (
                  <p className="text-sm text-blue-400">
                    {selectedUsers.length} user(s) selected
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
                <Label htmlFor="bulkExpiresAt">Expiry Date (Optional)</Label>
                <div className="flex items-center gap-2">
                  <Calendar className="w-4 h-4 text-gray-400" />
                  <Input
                    type="datetime-local"
                    {...bulkForm.register('expiresAt')}
                    min={new Date().toISOString().slice(0, 16)}
                  />
                </div>
              </div>

              {/* Bulk Reason */}
              <div className="space-y-2">
                <Label htmlFor="bulkReason">Reason</Label>
                <Textarea
                  placeholder="Reason for bulk permission grant..."
                  {...bulkForm.register('reason')}
                  rows={3}
                />
                {bulkForm.formState.errors.reason && (
                  <p className="text-sm text-red-500">{bulkForm.formState.errors.reason.message}</p>
                )}
              </div>

              {/* Submit Button */}
              <div className="flex justify-end gap-3">
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => {
                    bulkForm.reset();
                    setSelectedUsers([]);
                    setSelectedPermissions([]);
                  }}
                >
                  Reset
                </Button>
                <Button
                  type="submit"
                  disabled={isSubmitting || selectedUsers.length === 0 || selectedPermissions.length === 0}
                >
                  {isSubmitting ? (
                    <>
                      <Clock className="w-4 h-4 mr-2 animate-spin" />
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
          </Card>
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
                      <Clock className="w-4 h-4 mr-2 animate-spin" />
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
  );
}

// Permission Selector Component
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

      {/* Permissions List */}
      <div className="border border-gray-700 rounded-md p-4 max-h-80 overflow-y-auto">
        {Object.entries(permissionsByCategory).map(([category, categoryPermissions]) => (
          <div key={category} className="mb-6 last:mb-0">
            <h4 className="font-medium text-white mb-3 text-sm uppercase tracking-wide">
              {category}
            </h4>
            <div className="space-y-2">
              {categoryPermissions.map(permission => (
                <label
                  key={permission.id}
                  className="flex items-start gap-3 p-3 hover:bg-gray-800 rounded-md cursor-pointer"
                >
                  <Checkbox
                    checked={selectedPermissions.includes(permission.id)}
                    onCheckedChange={() => handlePermissionToggle(permission.id)}
                  />
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-1">
                      <span className="font-medium text-sm">{permission.name}</span>
                      <Badge variant="secondary" size="sm">
                        {permission.platform}
                      </Badge>
                    </div>
                    <p className="text-xs text-gray-400 leading-relaxed">
                      {permission.description}
                    </p>
                  </div>
                </label>
              ))}
            </div>
          </div>
        ))}

        {permissions.length === 0 && (
          <div className="text-center py-6 text-gray-400">
            No permissions found matching your criteria.
          </div>
        )}
      </div>

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

export default PermissionForms;