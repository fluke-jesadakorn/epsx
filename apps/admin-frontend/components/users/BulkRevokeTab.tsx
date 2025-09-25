'use client';

import { memo, useState, useCallback } from 'react';
import { Ban, XCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { useToast } from '@/components/ui/use-toast';

interface BulkRevokeTabProps {
  permissionIds: string[];
  onPermissionIdsChange: (permissionIds: string[]) => void;
  loading: boolean;
  onBulkRevoke: (reason: string) => void;
}

function BulkRevokeTab({
  permissionIds,
  onPermissionIdsChange,
  loading,
  onBulkRevoke,
}: BulkRevokeTabProps) {
  const { toast } = useToast();
  const [permissionIdInput, setPermissionIdInput] = useState('');
  const [revokeReason, setRevokeReason] = useState('');

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

  const handleRevoke = useCallback(() => {
    onBulkRevoke(revokeReason);
  }, [revokeReason, onBulkRevoke]);

  return (
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
          onClick={handleRevoke} 
          disabled={loading || permissionIds.length === 0}
          variant="destructive"
          className="w-full"
        >
          {loading ? 'Revoking Permissions...' : `Revoke ${permissionIds.length} Permissions`}
        </Button>
      </CardContent>
    </Card>
  );
}

export default memo(BulkRevokeTab);