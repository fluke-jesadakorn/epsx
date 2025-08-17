'use client';

import { useState, useEffect, useMemo } from 'react';
import { 
  Users, 
  Plus, 
  Trash2, 
  RefreshCw, 
  Clock,
  CheckCircle,
  XCircle,
  AlertTriangle,
  Upload,
  Download,
  Play,
  Pause,
  Settings,
  Filter,
  Search,
  Calendar,
  Shield,
  Key,
  Ban
} from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Input } from '@epsx/ui';
import { Label } from '@epsx/ui';
import { Textarea } from '@/components/ui/textarea';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@epsx/ui';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@epsx/ui';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Progress } from '@/components/ui/progress';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@epsx/ui';
import { Checkbox } from '@/components/ui/form-components';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
  DialogFooter,
} from '@epsx/ui';
import { useToast } from '@/components/ui/use-toast';
import { format } from 'date-fns';

import {
  bulkCreateTemporaryPermissions,
  bulkRevokeTemporaryPermissions,
  bulkUpdateTemporaryPermissions,
  BulkCreateTemporaryPermissionsData,
  BulkRevokeTemporaryPermissionsData,
  BulkUpdateTemporaryPermissionsData,
  CreateTemporaryPermissionData,
} from '@/lib/actions/temporary-permission-actions';

interface BulkOperationsInterfaceProps {
  selectedUserIds?: string[];
  onOperationComplete?: () => void;
}

interface BulkOperationResult {
  operation: string;
  timestamp: string;
  totalRequested: number;
  successful: number;
  failed: number;
  executionTimeMs: number;
  errors: Array<{ id?: string; error: string; details?: string }>;
}

interface PermissionTemplate {
  id: string;
  name: string;
  permission: string;
  resource: string;
  action: string;
  defaultDurationHours: number;
}

const PERMISSION_TEMPLATES: PermissionTemplate[] = [
  {
    id: 'admin-temp-access',
    name: 'Temporary Admin Access',
    permission: 'admin.*',
    resource: 'system',
    action: 'manage',
    defaultDurationHours: 4,
  },
  {
    id: 'user-management',
    name: 'User Management',
    permission: 'users',
    resource: 'user_accounts',
    action: 'manage',
    defaultDurationHours: 8,
  },
  {
    id: 'trading-override',
    name: 'Trading Override',
    permission: 'trading.override',
    resource: 'trades',
    action: 'manage',
    defaultDurationHours: 2,
  },
  {
    id: 'analytics-access',
    name: 'Analytics Access',
    permission: 'analytics',
    resource: 'reports',
    action: 'read',
    defaultDurationHours: 24,
  },
];

export function BulkOperationsInterface({ 
  selectedUserIds = [], 
  onOperationComplete 
}: BulkOperationsInterfaceProps) {
  const { toast } = useToast();
  
  // State
  const [activeTab, setActiveTab] = useState<'create' | 'revoke' | 'update' | 'history'>('create');
  const [loading, setLoading] = useState(false);
  const [progress, setProgress] = useState(0);
  const [operationResults, setOperationResults] = useState<BulkOperationResult[]>([]);
  const [userIdInput, setUserIdInput] = useState('');
  const [currentUserIds, setCurrentUserIds] = useState<string[]>(selectedUserIds);
  const [permissionIds, setPermissionIds] = useState<string[]>([]);
  const [permissionIdInput, setPermissionIdInput] = useState('');

  // Bulk Create State
  const [bulkCreateData, setBulkCreateData] = useState<{
    template: string;
    customPermission: string;
    customResource: string;
    customAction: string;
    duration: number;
    reason: string;
  }>({
    template: '',
    customPermission: '',
    customResource: '',
    customAction: '',
    duration: 8,
    reason: '',
  });

  // Bulk Revoke State
  const [revokeReason, setRevokeReason] = useState('');

  // Bulk Update State
  const [updateData, setUpdateData] = useState<{
    permission?: string;
    resource?: string;
    action?: string;
    extendHours?: number;
    reason?: string;
    status?: 'active' | 'suspended' | 'revoked';
  }>({});

  useEffect(() => {
    setCurrentUserIds(selectedUserIds);
  }, [selectedUserIds]);

  const addUserId = () => {
    if (userIdInput.trim() && !currentUserIds.includes(userIdInput.trim())) {
      setCurrentUserIds([...currentUserIds, userIdInput.trim()]);
      setUserIdInput('');
    }
  };

  const removeUserId = (userId: string) => {
    setCurrentUserIds(currentUserIds.filter(id => id !== userId));
  };

  const addPermissionId = () => {
    if (permissionIdInput.trim() && !permissionIds.includes(permissionIdInput.trim())) {
      setPermissionIds([...permissionIds, permissionIdInput.trim()]);
      setPermissionIdInput('');
    }
  };

  const removePermissionId = (permissionId: string) => {
    setPermissionIds(permissionIds.filter(id => id !== permissionId));
  };

  const handleBulkCreate = async () => {
    if (currentUserIds.length === 0) {
      toast({
        title: 'Validation Error',
        description: 'Please select at least one user',
        variant: 'destructive',
      });
      return;
    }

    let permission, resource, action;
    
    if (bulkCreateData.template) {
      const template = PERMISSION_TEMPLATES.find(t => t.id === bulkCreateData.template);
      if (!template) {
        toast({
          title: 'Error',
          description: 'Invalid template selected',
          variant: 'destructive',
        });
        return;
      }
      permission = template.permission;
      resource = template.resource;
      action = template.action;
    } else {
      permission = bulkCreateData.customPermission;
      resource = bulkCreateData.customResource;
      action = bulkCreateData.customAction;
    }

    if (!permission || !resource || !action) {
      toast({
        title: 'Validation Error',
        description: 'Please fill in all permission fields',
        variant: 'destructive',
      });
      return;
    }

    try {
      setLoading(true);
      setProgress(0);

      const expiresAt = new Date();
      expiresAt.setHours(expiresAt.getHours() + bulkCreateData.duration);

      const permissions: CreateTemporaryPermissionData[] = currentUserIds.map(userId => ({
        user_id: userId,
        permission,
        resource,
        action,
        expires_at: expiresAt.toISOString(),
        reason: bulkCreateData.reason || undefined,
      }));

      const startTime = Date.now();
      
      // Simulate progress updates
      const progressInterval = setInterval(() => {
        setProgress(prev => Math.min(prev + 10, 90));
      }, 200);

      const result = await bulkCreateTemporaryPermissions({ permissions });
      
      clearInterval(progressInterval);
      setProgress(100);

      if (result.success && result.data) {
        const operationResult: BulkOperationResult = {
          operation: 'Bulk Create Permissions',
          timestamp: new Date().toISOString(),
          totalRequested: result.data.summary.total_requested,
          successful: result.data.summary.successful,
          failed: result.data.summary.failed,
          executionTimeMs: result.data.summary.execution_time_ms,
          errors: result.data.failed,
        };

        setOperationResults(prev => [operationResult, ...prev.slice(0, 9)]);

        toast({
          title: 'Bulk Operation Complete',
          description: `Created ${result.data.summary.successful}/${result.data.summary.total_requested} permissions successfully`,
          variant: result.data.summary.failed > 0 ? 'destructive' : 'default',
        });

        if (result.data.summary.failed === 0) {
          // Reset form on complete success
          setBulkCreateData({
            template: '',
            customPermission: '',
            customResource: '',
            customAction: '',
            duration: 8,
            reason: '',
          });
        }

        onOperationComplete?.();
      } else {
        toast({
          title: 'Error',
          description: result.error?.message || 'Bulk create operation failed',
          variant: 'destructive',
        });
      }
    } catch (error) {
      console.error('Bulk create failed:', error);
      toast({
        title: 'Error',
        description: 'An unexpected error occurred',
        variant: 'destructive',
      });
    } finally {
      setLoading(false);
      setProgress(0);
    }
  };

  const handleBulkRevoke = async () => {
    if (permissionIds.length === 0) {
      toast({
        title: 'Validation Error',
        description: 'Please add at least one permission ID to revoke',
        variant: 'destructive',
      });
      return;
    }

    try {
      setLoading(true);
      setProgress(0);

      const progressInterval = setInterval(() => {
        setProgress(prev => Math.min(prev + 15, 90));
      }, 200);

      const data: BulkRevokeTemporaryPermissionsData = {
        permission_ids: permissionIds,
        reason: revokeReason || undefined,
      };

      const result = await bulkRevokeTemporaryPermissions(data);
      
      clearInterval(progressInterval);
      setProgress(100);

      if (result.success && result.data) {
        const operationResult: BulkOperationResult = {
          operation: 'Bulk Revoke Permissions',
          timestamp: new Date().toISOString(),
          totalRequested: result.data.summary.total_requested,
          successful: result.data.summary.successful,
          failed: result.data.summary.failed,
          executionTimeMs: result.data.summary.execution_time_ms,
          errors: result.data.failed,
        };

        setOperationResults(prev => [operationResult, ...prev.slice(0, 9)]);

        toast({
          title: 'Bulk Operation Complete',
          description: `Revoked ${result.data.summary.successful}/${result.data.summary.total_requested} permissions successfully`,
          variant: result.data.summary.failed > 0 ? 'destructive' : 'default',
        });

        if (result.data.summary.failed === 0) {
          setPermissionIds([]);
          setRevokeReason('');
        }

        onOperationComplete?.();
      } else {
        toast({
          title: 'Error',
          description: result.error?.message || 'Bulk revoke operation failed',
          variant: 'destructive',
        });
      }
    } catch (error) {
      console.error('Bulk revoke failed:', error);
      toast({
        title: 'Error',
        description: 'An unexpected error occurred',
        variant: 'destructive',
      });
    } finally {
      setLoading(false);
      setProgress(0);
    }
  };

  const handleBulkUpdate = async () => {
    if (permissionIds.length === 0) {
      toast({
        title: 'Validation Error',
        description: 'Please add at least one permission ID to update',
        variant: 'destructive',
      });
      return;
    }

    const hasUpdates = Object.values(updateData).some(value => 
      value !== undefined && value !== '' && value !== null
    );

    if (!hasUpdates) {
      toast({
        title: 'Validation Error',
        description: 'Please specify at least one field to update',
        variant: 'destructive',
      });
      return;
    }

    try {
      setLoading(true);
      setProgress(0);

      const progressInterval = setInterval(() => {
        setProgress(prev => Math.min(prev + 12, 90));
      }, 250);

      // Prepare update data with expiry extension
      const updatePayload: {
        permission?: string;
        resource?: string;
        action?: string;
        extendHours?: number;
        reason?: string;
        status?: 'active' | 'suspended' | 'revoked';
        expires_at?: string;
      } = { ...updateData };
      if (updateData.extendHours) {
        const newExpiresAt = new Date();
        newExpiresAt.setHours(newExpiresAt.getHours() + updateData.extendHours);
        updatePayload.expires_at = newExpiresAt.toISOString();
        delete updatePayload.extendHours;
      }

      const data: BulkUpdateTemporaryPermissionsData = {
        updates: permissionIds.map(id => ({
          id,
          updates: updatePayload,
        })),
      };

      const result = await bulkUpdateTemporaryPermissions(data);
      
      clearInterval(progressInterval);
      setProgress(100);

      if (result.success && result.data) {
        const operationResult: BulkOperationResult = {
          operation: 'Bulk Update Permissions',
          timestamp: new Date().toISOString(),
          totalRequested: result.data.summary.total_requested,
          successful: result.data.summary.successful,
          failed: result.data.summary.failed,
          executionTimeMs: result.data.summary.execution_time_ms,
          errors: result.data.failed,
        };

        setOperationResults(prev => [operationResult, ...prev.slice(0, 9)]);

        toast({
          title: 'Bulk Operation Complete',
          description: `Updated ${result.data.summary.successful}/${result.data.summary.total_requested} permissions successfully`,
          variant: result.data.summary.failed > 0 ? 'destructive' : 'default',
        });

        if (result.data.summary.failed === 0) {
          setPermissionIds([]);
          setUpdateData({});
        }

        onOperationComplete?.();
      } else {
        toast({
          title: 'Error',
          description: result.error?.message || 'Bulk update operation failed',
          variant: 'destructive',
        });
      }
    } catch (error) {
      console.error('Bulk update failed:', error);
      toast({
        title: 'Error',
        description: 'An unexpected error occurred',
        variant: 'destructive',
      });
    } finally {
      setLoading(false);
      setProgress(0);
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-semibold flex items-center gap-2">
            <Users className="h-5 w-5" />
            Bulk Operations
          </h3>
          <p className="text-sm text-muted-foreground">
            Manage permissions for multiple users simultaneously
          </p>
        </div>
      </div>

      {/* Progress Bar */}
      {loading && (
        <div className="space-y-2">
          <div className="flex items-center justify-between text-sm">
            <span>Processing bulk operation...</span>
            <span>{progress}%</span>
          </div>
          <Progress value={progress} className="w-full" />
        </div>
      )}

      {/* Operation Tabs */}
      <Tabs value={activeTab} onValueChange={(value: any) => setActiveTab(value)}>
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="create" className="flex items-center gap-2">
            <Plus className="h-4 w-4" />
            Create
          </TabsTrigger>
          <TabsTrigger value="revoke" className="flex items-center gap-2">
            <Ban className="h-4 w-4" />
            Revoke
          </TabsTrigger>
          <TabsTrigger value="update" className="flex items-center gap-2">
            <RefreshCw className="h-4 w-4" />
            Update
          </TabsTrigger>
          <TabsTrigger value="history" className="flex items-center gap-2">
            <Clock className="h-4 w-4" />
            History
          </TabsTrigger>
        </TabsList>

        {/* Bulk Create Tab */}
        <TabsContent value="create" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Plus className="h-5 w-5" />
                Bulk Create Temporary Permissions
              </CardTitle>
              <CardDescription>
                Grant temporary permissions to multiple users at once
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              {/* User Selection */}
              <div className="space-y-4">
                <Label>Target Users ({currentUserIds.length} selected)</Label>
                <div className="flex gap-2">
                  <Input
                    placeholder="Enter user ID"
                    value={userIdInput}
                    onChange={(e) => setUserIdInput(e.target.value)}
                    onKeyPress={(e) => e.key === 'Enter' && addUserId()}
                  />
                  <Button onClick={addUserId} size="sm">
                    Add
                  </Button>
                </div>
                {currentUserIds.length > 0 && (
                  <div className="flex flex-wrap gap-2">
                    {currentUserIds.map((userId) => (
                      <Badge key={userId} variant="secondary" className="flex items-center gap-1">
                        {userId}
                        <button onClick={() => removeUserId(userId)}>
                          <XCircle className="h-3 w-3" />
                        </button>
                      </Badge>
                    ))}
                  </div>
                )}
              </div>

              {/* Permission Configuration */}
              <div className="space-y-4">
                <Label>Permission Configuration</Label>
                
                {/* Template Selection */}
                <div className="space-y-2">
                  <Label htmlFor="template">Use Template</Label>
                  <Select
                    value={bulkCreateData.template}
                    onValueChange={(value) => setBulkCreateData(prev => ({ 
                      ...prev, 
                      template: value,
                      customPermission: '',
                      customResource: '',
                      customAction: ''
                    }))}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select a permission template (optional)" />
                    </SelectTrigger>
                    <SelectContent>
                      {PERMISSION_TEMPLATES.map((template) => (
                        <SelectItem key={template.id} value={template.id}>
                          <div>
                            <div className="font-medium">{template.name}</div>
                            <div className="text-xs text-muted-foreground">
                              {template.permission} on {template.resource}
                            </div>
                          </div>
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>

                {/* Custom Permission Fields */}
                {!bulkCreateData.template && (
                  <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <div className="space-y-2">
                      <Label htmlFor="permission">Permission</Label>
                      <Input
                        id="permission"
                        value={bulkCreateData.customPermission}
                        onChange={(e) => setBulkCreateData(prev => ({ 
                          ...prev, 
                          customPermission: e.target.value 
                        }))}
                        placeholder="e.g., admin.users"
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="resource">Resource</Label>
                      <Input
                        id="resource"
                        value={bulkCreateData.customResource}
                        onChange={(e) => setBulkCreateData(prev => ({ 
                          ...prev, 
                          customResource: e.target.value 
                        }))}
                        placeholder="e.g., user_accounts"
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="action">Action</Label>
                      <Input
                        id="action"
                        value={bulkCreateData.customAction}
                        onChange={(e) => setBulkCreateData(prev => ({ 
                          ...prev, 
                          customAction: e.target.value 
                        }))}
                        placeholder="e.g., manage"
                      />
                    </div>
                  </div>
                )}

                {/* Duration and Reason */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label htmlFor="duration">Duration (hours)</Label>
                    <Input
                      id="duration"
                      type="number"
                      min="1"
                      max="168"
                      value={bulkCreateData.duration}
                      onChange={(e) => setBulkCreateData(prev => ({ 
                        ...prev, 
                        duration: parseInt(e.target.value) || 8 
                      }))}
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="reason">Reason</Label>
                    <Input
                      id="reason"
                      value={bulkCreateData.reason}
                      onChange={(e) => setBulkCreateData(prev => ({ 
                        ...prev, 
                        reason: e.target.value 
                      }))}
                      placeholder="Why are these permissions needed?"
                    />
                  </div>
                </div>
              </div>

              <Button 
                onClick={handleBulkCreate} 
                disabled={loading || currentUserIds.length === 0}
                className="w-full"
              >
                {loading ? 'Creating Permissions...' : `Create Permissions for ${currentUserIds.length} Users`}
              </Button>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Bulk Revoke Tab */}
        <TabsContent value="revoke" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Ban className="h-5 w-5" />
                Bulk Revoke Permissions
              </CardTitle>
              <CardDescription>
                Revoke multiple temporary permissions at once
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              {/* Permission ID Selection */}
              <div className="space-y-4">
                <Label>Permission IDs to Revoke ({permissionIds.length} selected)</Label>
                <div className="flex gap-2">
                  <Input
                    placeholder="Enter permission ID"
                    value={permissionIdInput}
                    onChange={(e) => setPermissionIdInput(e.target.value)}
                    onKeyPress={(e) => e.key === 'Enter' && addPermissionId()}
                  />
                  <Button onClick={addPermissionId} size="sm">
                    Add
                  </Button>
                </div>
                {permissionIds.length > 0 && (
                  <div className="flex flex-wrap gap-2">
                    {permissionIds.map((permissionId) => (
                      <Badge key={permissionId} variant="destructive" className="flex items-center gap-1">
                        {permissionId.slice(0, 8)}...
                        <button onClick={() => removePermissionId(permissionId)}>
                          <XCircle className="h-3 w-3" />
                        </button>
                      </Badge>
                    ))}
                  </div>
                )}
              </div>

              {/* Revoke Reason */}
              <div className="space-y-2">
                <Label htmlFor="revokeReason">Revocation Reason (Optional)</Label>
                <Textarea
                  id="revokeReason"
                  value={revokeReason}
                  onChange={(e) => setRevokeReason(e.target.value)}
                  placeholder="Why are these permissions being revoked?"
                  rows={3}
                />
              </div>

              <Button 
                onClick={handleBulkRevoke} 
                disabled={loading || permissionIds.length === 0}
                variant="destructive"
                className="w-full"
              >
                {loading ? 'Revoking Permissions...' : `Revoke ${permissionIds.length} Permissions`}
              </Button>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Bulk Update Tab */}
        <TabsContent value="update" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <RefreshCw className="h-5 w-5" />
                Bulk Update Permissions
              </CardTitle>
              <CardDescription>
                Update multiple permissions with new values
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              {/* Permission ID Selection (Reused) */}
              <div className="space-y-4">
                <Label>Permission IDs to Update ({permissionIds.length} selected)</Label>
                <div className="flex gap-2">
                  <Input
                    placeholder="Enter permission ID"
                    value={permissionIdInput}
                    onChange={(e) => setPermissionIdInput(e.target.value)}
                    onKeyPress={(e) => e.key === 'Enter' && addPermissionId()}
                  />
                  <Button onClick={addPermissionId} size="sm">
                    Add
                  </Button>
                </div>
                {permissionIds.length > 0 && (
                  <div className="flex flex-wrap gap-2">
                    {permissionIds.map((permissionId) => (
                      <Badge key={permissionId} variant="secondary" className="flex items-center gap-1">
                        {permissionId.slice(0, 8)}...
                        <button onClick={() => removePermissionId(permissionId)}>
                          <XCircle className="h-3 w-3" />
                        </button>
                      </Badge>
                    ))}
                  </div>
                )}
              </div>

              {/* Update Fields */}
              <div className="space-y-4">
                <Label>Fields to Update (leave empty to keep unchanged)</Label>
                
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                  <div className="space-y-2">
                    <Label htmlFor="updatePermission">Permission</Label>
                    <Input
                      id="updatePermission"
                      value={updateData.permission || ''}
                      onChange={(e) => setUpdateData(prev => ({ 
                        ...prev, 
                        permission: e.target.value || undefined 
                      }))}
                      placeholder="New permission value"
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="updateResource">Resource</Label>
                    <Input
                      id="updateResource"
                      value={updateData.resource || ''}
                      onChange={(e) => setUpdateData(prev => ({ 
                        ...prev, 
                        resource: e.target.value || undefined 
                      }))}
                      placeholder="New resource value"
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="updateAction">Action</Label>
                    <Input
                      id="updateAction"
                      value={updateData.action || ''}
                      onChange={(e) => setUpdateData(prev => ({ 
                        ...prev, 
                        action: e.target.value || undefined 
                      }))}
                      placeholder="New action value"
                    />
                  </div>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label htmlFor="extendHours">Extend Expiry (hours)</Label>
                    <Input
                      id="extendHours"
                      type="number"
                      min="1"
                      value={updateData.extendHours || ''}
                      onChange={(e) => setUpdateData(prev => ({ 
                        ...prev, 
                        extendHours: parseInt(e.target.value) || undefined 
                      }))}
                      placeholder="Hours to extend from now"
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="updateStatus">Status</Label>
                    <Select
                      value={updateData.status || ''}
                      onValueChange={(value: any) => setUpdateData(prev => ({ 
                        ...prev, 
                        status: value || undefined 
                      }))}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="Keep current status" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="active">Active</SelectItem>
                        <SelectItem value="suspended">Suspended</SelectItem>
                        <SelectItem value="revoked">Revoked</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="updateReason">Update Reason</Label>
                  <Input
                    id="updateReason"
                    value={updateData.reason || ''}
                    onChange={(e) => setUpdateData(prev => ({ 
                      ...prev, 
                      reason: e.target.value || undefined 
                    }))}
                    placeholder="Reason for these updates"
                  />
                </div>
              </div>

              <Button 
                onClick={handleBulkUpdate} 
                disabled={loading || permissionIds.length === 0}
                className="w-full"
              >
                {loading ? 'Updating Permissions...' : `Update ${permissionIds.length} Permissions`}
              </Button>
            </CardContent>
          </Card>
        </TabsContent>

        {/* History Tab */}
        <TabsContent value="history" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Clock className="h-5 w-5" />
                Operation History
              </CardTitle>
              <CardDescription>
                Recent bulk operations and their results
              </CardDescription>
            </CardHeader>
            <CardContent>
              {operationResults.length === 0 ? (
                <div className="text-center py-8 text-muted-foreground">
                  <Clock className="h-12 w-12 mx-auto mb-4 opacity-50" />
                  <p>No bulk operations performed yet</p>
                </div>
              ) : (
                <div className="space-y-4">
                  {operationResults.map((result, index) => (
                    <Card key={index} className="border">
                      <CardContent className="p-4">
                        <div className="flex items-start justify-between mb-2">
                          <div>
                            <h4 className="font-medium">{result.operation}</h4>
                            <p className="text-sm text-muted-foreground">
                              {format(new Date(result.timestamp), 'PPp')}
                            </p>
                          </div>
                          <Badge variant={result.failed > 0 ? 'destructive' : 'default'}>
                            {result.successful}/{result.totalRequested} successful
                          </Badge>
                        </div>
                        
                        <div className="grid grid-cols-4 gap-4 text-sm">
                          <div>
                            <p className="text-muted-foreground">Total</p>
                            <p className="font-medium">{result.totalRequested}</p>
                          </div>
                          <div>
                            <p className="text-muted-foreground">Successful</p>
                            <p className="font-medium text-green-600">{result.successful}</p>
                          </div>
                          <div>
                            <p className="text-muted-foreground">Failed</p>
                            <p className="font-medium text-red-600">{result.failed}</p>
                          </div>
                          <div>
                            <p className="text-muted-foreground">Duration</p>
                            <p className="font-medium">{result.executionTimeMs}ms</p>
                          </div>
                        </div>

                        {result.errors.length > 0 && (
                          <div className="mt-4">
                            <details className="text-sm">
                              <summary className="cursor-pointer text-red-600 hover:text-red-700">
                                View {result.errors.length} errors
                              </summary>
                              <div className="mt-2 space-y-2">
                                {result.errors.slice(0, 5).map((error, errorIndex) => (
                                  <div key={errorIndex} className="p-2 bg-red-50 rounded text-red-700">
                                    <p className="font-medium">{error.error}</p>
                                    {error.details && (
                                      <p className="text-xs mt-1">{error.details}</p>
                                    )}
                                  </div>
                                ))}
                                {result.errors.length > 5 && (
                                  <p className="text-xs text-muted-foreground">
                                    ... and {result.errors.length - 5} more errors
                                  </p>
                                )}
                              </div>
                            </details>
                          </div>
                        )}
                      </CardContent>
                    </Card>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}