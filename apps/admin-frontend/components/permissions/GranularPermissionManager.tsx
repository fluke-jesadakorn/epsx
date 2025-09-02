'use client';

import React, { useState, useEffect, useCallback } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Badge } from '@/components/ui/badge';
import { Textarea } from '@/components/ui/textarea';
import { Separator } from '@/components/ui/separator';
import { 
  Dialog, 
  DialogContent, 
  DialogDescription, 
  DialogFooter, 
  DialogHeader, 
  DialogTitle 
} from '@/components/ui/dialog';
import { 
  Table, 
  TableBody, 
  TableCell, 
  TableHead, 
  TableHeader, 
  TableRow 
} from '@/components/ui/table';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { 
  Clock, 
  Shield, 
  ShieldAlert, 
  ShieldCheck, 
  Trash2, 
  Plus, 
  RotateCcw,
  AlertTriangle,
  CheckCircle,
  XCircle
} from 'lucide-react';
import { useUserPermissionManagement } from '@/hooks/useGranularPermissions';
import { 
  PermissionSource,
  GranularPermissionClaim,
  PermissionExpiryDetails,
  GrantPermissionRequest,
  RevokePermissionRequest,
  ExtendPermissionRequest
} from '@/types/granular-permissions';
import { formatDistanceToNow } from 'date-fns';

interface GranularPermissionManagerProps {
  userId: string;
  userEmail?: string;
  userName?: string;
  onPermissionChange?: (permissions: Record<string, GranularPermissionClaim>) => void;
  readOnly?: boolean;
  showAuditLog?: boolean;
  className?: string;
}

interface PermissionRow {
  permission: string;
  platform: string;
  resource: string;
  action: string;
  claim: GranularPermissionClaim;
  is_expired: boolean;
  expires_in_human?: string;
  health_status: 'healthy' | 'expiring' | 'expired';
}

export function GranularPermissionManager({ 
  userId, 
  userEmail,
  userName,
  onPermissionChange,
  readOnly = false,
  showAuditLog = true,
  className = ''
}: GranularPermissionManagerProps) {
  const { 
    userPermissions, 
    refreshUserPermissions, 
    grantPermission, 
    revokePermission, 
    extendPermission,
    getPermissionAudit,
    loading, 
    error 
  } = useUserPermissionManagement(userId);

  const [permissionRows, setPermissionRows] = useState<PermissionRow[]>([]);
  const [isGrantDialogOpen, setIsGrantDialogOpen] = useState(false);
  const [isExtendDialogOpen, setIsExtendDialogOpen] = useState(false);
  const [selectedPermission, setSelectedPermission] = useState<string>('');
  const [newPermission, setNewPermission] = useState({
    platform: 'epsx',
    resource: '',
    action: '',
    expires_at: '',
    source: 'Admin' as PermissionSource,
    reason: ''
  });
  const [extensionHours, setExtensionHours] = useState('24');
  const [extensionReason, setExtensionReason] = useState('');

  // Parse permissions into table rows
  useEffect(() => {
    if (userPermissions?.permissions) {
      const now = Date.now();
      const twentyFourHoursFromNow = now + (24 * 60 * 60 * 1000);

      const rows: PermissionRow[] = Object.entries(userPermissions.permissions).map(([permission, claim]) => {
        const parts = permission.split(':');
        const platform = parts[0] || '';
        const resource = parts[1] || '';
        const action = parts[2] || '';
        
        const isExpired = claim.expires_at ? (claim.expires_at * 1000) <= now : false;
        const expiresInMs = claim.expires_at ? (claim.expires_at * 1000) - now : undefined;
        const isExpiringSoon = claim.expires_at && !isExpired && (claim.expires_at * 1000) <= twentyFourHoursFromNow;

        let expiresInHuman: string | undefined;
        if (claim.expires_at) {
          expiresInHuman = formatDistanceToNow(new Date(claim.expires_at * 1000), { addSuffix: true });
        }

        let healthStatus: 'healthy' | 'expiring' | 'expired' = 'healthy';
        if (isExpired) healthStatus = 'expired';
        else if (isExpiringSoon) healthStatus = 'expiring';

        return {
          permission,
          platform,
          resource,
          action,
          claim,
          is_expired: isExpired,
          expires_in_human: expiresInHuman,
          health_status: healthStatus
        };
      });

      // Sort by health status (expired first, then expiring, then healthy)
      rows.sort((a, b) => {
        const statusOrder = { expired: 0, expiring: 1, healthy: 2 };
        return statusOrder[a.health_status] - statusOrder[b.health_status];
      });

      setPermissionRows(rows);
    }
  }, [userPermissions]);

  // Handle permission grant
  const handleGrantPermission = useCallback(async () => {
    if (!newPermission.resource || !newPermission.action) return;

    const permission = `${newPermission.platform}:${newPermission.resource}:${newPermission.action}`;
    const expiresAt = newPermission.expires_at ? 
      Math.floor(new Date(newPermission.expires_at).getTime() / 1000) : undefined;

    const request: GrantPermissionRequest = {
      user_id: userId,
      permission,
      expires_at: expiresAt,
      source: newPermission.source,
      reason: newPermission.reason || undefined
    };

    try {
      await grantPermission(request);
      await refreshUserPermissions();
      onPermissionChange?.(userPermissions?.permissions || {});
      
      // Reset form
      setNewPermission({
        platform: 'epsx',
        resource: '',
        action: '',
        expires_at: '',
        source: 'Admin',
        reason: ''
      });
      setIsGrantDialogOpen(false);
    } catch (err) {
      console.error('Failed to grant permission:', err);
    }
  }, [newPermission, userId, grantPermission, refreshUserPermissions, onPermissionChange, userPermissions]);

  // Handle permission revocation
  const handleRevokePermission = useCallback(async (permission: string, reason?: string) => {
    const request: RevokePermissionRequest = {
      user_id: userId,
      permission,
      reason
    };

    try {
      await revokePermission(request);
      await refreshUserPermissions();
      onPermissionChange?.(userPermissions?.permissions || {});
    } catch (err) {
      console.error('Failed to revoke permission:', err);
    }
  }, [userId, revokePermission, refreshUserPermissions, onPermissionChange, userPermissions]);

  // Handle permission extension
  const handleExtendPermission = useCallback(async () => {
    if (!selectedPermission || !extensionHours) return;

    const hoursToAdd = parseInt(extensionHours, 10);
    const newExpiresAt = Math.floor((Date.now() + (hoursToAdd * 60 * 60 * 1000)) / 1000);

    const request: ExtendPermissionRequest = {
      user_id: userId,
      permission: selectedPermission,
      new_expires_at: newExpiresAt,
      reason: extensionReason || undefined
    };

    try {
      await extendPermission(request);
      await refreshUserPermissions();
      onPermissionChange?.(userPermissions?.permissions || {});
      
      setIsExtendDialogOpen(false);
      setSelectedPermission('');
      setExtensionHours('24');
      setExtensionReason('');
    } catch (err) {
      console.error('Failed to extend permission:', err);
    }
  }, [selectedPermission, extensionHours, extensionReason, userId, extendPermission, refreshUserPermissions, onPermissionChange, userPermissions]);

  const getHealthIcon = (status: 'healthy' | 'expiring' | 'expired') => {
    switch (status) {
      case 'healthy': return <ShieldCheck className="h-4 w-4 text-green-500" />;
      case 'expiring': return <ShieldAlert className="h-4 w-4 text-yellow-500" />;
      case 'expired': return <XCircle className="h-4 w-4 text-red-500" />;
    }
  };

  const getHealthBadge = (status: 'healthy' | 'expiring' | 'expired') => {
    switch (status) {
      case 'healthy': return <Badge variant="default" className="bg-green-100 text-green-800">Active</Badge>;
      case 'expiring': return <Badge variant="destructive" className="bg-yellow-100 text-yellow-800">Expiring Soon</Badge>;
      case 'expired': return <Badge variant="destructive">Expired</Badge>;
    }
  };

  const getSourceBadge = (source: PermissionSource) => {
    const colors = {
      Admin: 'bg-blue-100 text-blue-800',
      Subscription: 'bg-green-100 text-green-800',
      Trial: 'bg-purple-100 text-purple-800',
      Legacy: 'bg-gray-100 text-gray-800',
      System: 'bg-indigo-100 text-indigo-800'
    };
    
    return (
      <Badge variant="outline" className={colors[source]}>
        {source}
      </Badge>
    );
  };

  if (loading && !userPermissions) {
    return (
      <Card className={className}>
        <CardContent className="flex items-center justify-center p-6">
          <div className="text-center">
            <RotateCcw className="h-8 w-8 animate-spin mx-auto mb-2" />
            <p>Loading permissions...</p>
          </div>
        </CardContent>
      </Card>
    );
  }

  if (error) {
    return (
      <Card className={className}>
        <CardContent className="p-6">
          <Alert variant="destructive">
            <AlertTriangle className="h-4 w-4" />
            <AlertDescription>
              Failed to load permissions: {error.message}
            </AlertDescription>
          </Alert>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <Shield className="h-5 w-5" />
                Permission Management
              </CardTitle>
              <CardDescription>
                Manage granular permissions for {userName || userEmail || userId}
              </CardDescription>
            </div>
            <div className="flex gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => refreshUserPermissions()}
                disabled={loading}
              >
                <RotateCcw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
                Refresh
              </Button>
              {!readOnly && (
                <Button
                  size="sm"
                  onClick={() => setIsGrantDialogOpen(true)}
                >
                  <Plus className="h-4 w-4" />
                  Grant Permission
                </Button>
              )}
            </div>
          </div>
        </CardHeader>
        
        {userPermissions?.health && (
          <CardContent>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div className="text-center">
                <div className="text-2xl font-bold text-green-600">
                  {userPermissions.health.active_permissions}
                </div>
                <div className="text-sm text-muted-foreground">Active</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-yellow-600">
                  {userPermissions.health.expiring_soon_permissions}
                </div>
                <div className="text-sm text-muted-foreground">Expiring Soon</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-red-600">
                  {userPermissions.health.expired_permissions}
                </div>
                <div className="text-sm text-muted-foreground">Expired</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-blue-600">
                  {userPermissions.health.health_score}%
                </div>
                <div className="text-sm text-muted-foreground">Health Score</div>
              </div>
            </div>
          </CardContent>
        )}
      </Card>

      {/* Permissions Table */}
      <Card>
        <CardHeader>
          <CardTitle>Current Permissions ({permissionRows.length})</CardTitle>
        </CardHeader>
        <CardContent>
          {permissionRows.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              No permissions found for this user.
            </div>
          ) : (
            <div className="rounded-md border">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Status</TableHead>
                    <TableHead>Permission</TableHead>
                    <TableHead>Source</TableHead>
                    <TableHead>Granted</TableHead>
                    <TableHead>Expires</TableHead>
                    {!readOnly && <TableHead>Actions</TableHead>}
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {permissionRows.map((row) => (
                    <TableRow key={row.permission}>
                      <TableCell>
                        <div className="flex items-center gap-2">
                          {getHealthIcon(row.health_status)}
                          {getHealthBadge(row.health_status)}
                        </div>
                      </TableCell>
                      <TableCell>
                        <div>
                          <div className="font-mono text-sm">{row.permission}</div>
                          <div className="text-xs text-muted-foreground">
                            {row.platform} • {row.resource} • {row.action}
                          </div>
                        </div>
                      </TableCell>
                      <TableCell>
                        {getSourceBadge(row.claim.source)}
                      </TableCell>
                      <TableCell>
                        <div className="text-sm">
                          {formatDistanceToNow(new Date(row.claim.granted_at * 1000), { addSuffix: true })}
                          {row.claim.granted_by && (
                            <div className="text-xs text-muted-foreground">
                              by {row.claim.granted_by}
                            </div>
                          )}
                        </div>
                      </TableCell>
                      <TableCell>
                        {row.claim.expires_at ? (
                          <div className="text-sm">
                            <div className={row.is_expired ? 'text-red-600' : row.health_status === 'expiring' ? 'text-yellow-600' : ''}>
                              {row.expires_in_human}
                            </div>
                            <div className="text-xs text-muted-foreground">
                              {new Date(row.claim.expires_at * 1000).toLocaleDateString()}
                            </div>
                          </div>
                        ) : (
                          <Badge variant="outline">Permanent</Badge>
                        )}
                      </TableCell>
                      {!readOnly && (
                        <TableCell>
                          <div className="flex gap-1">
                            {row.claim.expires_at && !row.is_expired && (
                              <Button
                                size="sm"
                                variant="outline"
                                onClick={() => {
                                  setSelectedPermission(row.permission);
                                  setIsExtendDialogOpen(true);
                                }}
                              >
                                <Clock className="h-3 w-3" />
                              </Button>
                            )}
                            <Button
                              size="sm"
                              variant="destructive"
                              onClick={() => handleRevokePermission(row.permission, 'Revoked via admin panel')}
                            >
                              <Trash2 className="h-3 w-3" />
                            </Button>
                          </div>
                        </TableCell>
                      )}
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Grant Permission Dialog */}
      <Dialog open={isGrantDialogOpen} onOpenChange={setIsGrantDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Grant New Permission</DialogTitle>
            <DialogDescription>
              Grant a new permission to {userName || userEmail || userId}
            </DialogDescription>
          </DialogHeader>
          
          <div className="space-y-4">
            <div className="grid grid-cols-3 gap-2">
              <div>
                <Label htmlFor="platform">Platform</Label>
                <Select 
                  value={newPermission.platform} 
                  onValueChange={(value) => setNewPermission(prev => ({ ...prev, platform: value }))}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="epsx">EPSX</SelectItem>
                    <SelectItem value="epsx-pay">EPSX Pay</SelectItem>
                    <SelectItem value="epsx-token">EPSX Token</SelectItem>
                    <SelectItem value="admin">Admin</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div>
                <Label htmlFor="resource">Resource</Label>
                <Input
                  id="resource"
                  value={newPermission.resource}
                  onChange={(e) => setNewPermission(prev => ({ ...prev, resource: e.target.value }))}
                  placeholder="users, analytics, etc."
                />
              </div>
              <div>
                <Label htmlFor="action">Action</Label>
                <Input
                  id="action"
                  value={newPermission.action}
                  onChange={(e) => setNewPermission(prev => ({ ...prev, action: e.target.value }))}
                  placeholder="view, manage, *, etc."
                />
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label htmlFor="source">Source</Label>
                <Select 
                  value={newPermission.source} 
                  onValueChange={(value) => setNewPermission(prev => ({ ...prev, source: value as PermissionSource }))}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="Admin">Admin</SelectItem>
                    <SelectItem value="Subscription">Subscription</SelectItem>
                    <SelectItem value="Trial">Trial</SelectItem>
                    <SelectItem value="System">System</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div>
                <Label htmlFor="expires_at">Expires At (Optional)</Label>
                <Input
                  id="expires_at"
                  type="datetime-local"
                  value={newPermission.expires_at}
                  onChange={(e) => setNewPermission(prev => ({ ...prev, expires_at: e.target.value }))}
                />
              </div>
            </div>

            <div>
              <Label htmlFor="reason">Reason (Optional)</Label>
              <Textarea
                id="reason"
                value={newPermission.reason}
                onChange={(e) => setNewPermission(prev => ({ ...prev, reason: e.target.value }))}
                placeholder="Reason for granting this permission..."
                rows={2}
              />
            </div>

            <div className="p-3 bg-muted rounded">
              <div className="text-sm font-medium">Permission Preview:</div>
              <div className="font-mono text-sm">
                {newPermission.platform}:{newPermission.resource || '*'}:{newPermission.action || '*'}
              </div>
            </div>
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setIsGrantDialogOpen(false)}>
              Cancel
            </Button>
            <Button 
              onClick={handleGrantPermission}
              disabled={!newPermission.resource || !newPermission.action || loading}
            >
              Grant Permission
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Extend Permission Dialog */}
      <Dialog open={isExtendDialogOpen} onOpenChange={setIsExtendDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Extend Permission</DialogTitle>
            <DialogDescription>
              Extend the expiry time for permission: {selectedPermission}
            </DialogDescription>
          </DialogHeader>
          
          <div className="space-y-4">
            <div>
              <Label htmlFor="extension_hours">Extend by (hours)</Label>
              <Select 
                value={extensionHours} 
                onValueChange={setExtensionHours}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="1">1 hour</SelectItem>
                  <SelectItem value="24">24 hours</SelectItem>
                  <SelectItem value="168">1 week</SelectItem>
                  <SelectItem value="720">1 month</SelectItem>
                  <SelectItem value="8760">1 year</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div>
              <Label htmlFor="extension_reason">Reason (Optional)</Label>
              <Textarea
                id="extension_reason"
                value={extensionReason}
                onChange={(e) => setExtensionReason(e.target.value)}
                placeholder="Reason for extending this permission..."
                rows={2}
              />
            </div>
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setIsExtendDialogOpen(false)}>
              Cancel
            </Button>
            <Button 
              onClick={handleExtendPermission}
              disabled={loading}
            >
              Extend Permission
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

export default GranularPermissionManager;