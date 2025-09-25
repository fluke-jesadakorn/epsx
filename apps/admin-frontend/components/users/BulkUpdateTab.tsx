'use client';

import { memo, useState, useCallback } from 'react';
import { RefreshCw, XCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { useToast } from '@/components/ui/use-toast';

interface UpdateData {
  permission?: string;
  resource?: string;
  action?: string;
  extendHours?: number;
  reason?: string;
  status?: 'active' | 'suspended' | 'revoked';
}

interface BulkUpdateTabProps {
  permissionIds: string[];
  onPermissionIdsChange: (permissionIds: string[]) => void;
  loading: boolean;
  onBulkUpdate: (updateData: UpdateData) => void;
}

function BulkUpdateTab({
  permissionIds,
  onPermissionIdsChange,
  loading,
  onBulkUpdate,
}: BulkUpdateTabProps) {
  const { toast } = useToast();
  const [permissionIdInput, setPermissionIdInput] = useState('');
  
  // Bulk Update State
  const [updateData, setUpdateData] = useState<UpdateData>({});

  const addPermissionId = useCallback(() => {
    const trimmedId = permissionIdInput.trim();
    
    // SECURITY: Validate permission ID format
    if (!trimmedId) return;
    
    // SECURITY: Basic UUID format validation to prevent injection
    const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
    
    if (!uuidRegex.test(trimmedId)) {
      toast({
        title: 'Invalid Permission ID',
        description: 'Permission ID must be a valid UUID format',
        variant: 'destructive',
      });
      return;
    }
    
    // SECURITY: Prevent duplicate entries
    if (permissionIds.includes(trimmedId)) {
      toast({
        title: 'Duplicate Entry',
        description: 'Permission ID already added',
        variant: 'destructive',
      });
      return;
    }
    
    // SECURITY: Limit total entries to prevent DoS
    if (permissionIds.length >= 100) {
      toast({
        title: 'Limit Exceeded',
        description: 'Maximum 100 permissions allowed per bulk operation',
        variant: 'destructive',
      });
      return;
    }
    
    onPermissionIdsChange([...permissionIds, trimmedId]);
    setPermissionIdInput('');
  }, [permissionIdInput, permissionIds, onPermissionIdsChange, toast]);

  const removePermissionId = useCallback((permissionId: string) => {
    onPermissionIdsChange(permissionIds.filter(id => id !== permissionId));
  }, [permissionIds, onPermissionIdsChange]);

  const handleUpdate = useCallback(() => {
    onBulkUpdate(updateData);
  }, [updateData, onBulkUpdate]);

  return (
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
        {/* Permission ID Selection */}
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
          onClick={handleUpdate} 
          disabled={loading || permissionIds.length === 0}
          className="w-full"
        >
          {loading ? 'Updating Permissions...' : `Update ${permissionIds.length} Permissions`}
        </Button>
      </CardContent>
    </Card>
  );
}

export default memo(BulkUpdateTab);