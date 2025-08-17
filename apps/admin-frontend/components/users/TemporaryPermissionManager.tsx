'use client';

import { useState, useEffect, useMemo } from 'react';
import { 
  Clock, 
  Plus, 
  Filter, 
  RefreshCw, 
  AlertTriangle, 
  CheckCircle,
  XCircle,
  Trash2,
  Calendar,
  User,
  Shield,
  MoreHorizontal,
  Edit,
  Ban,
  Trash,
  FileText
} from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Input } from '@epsx/ui';
import { Label } from '@epsx/ui';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@epsx/ui';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@epsx/ui';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Separator } from '@/components/ui/separator';
import { Textarea } from '@/components/ui/textarea';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@epsx/ui';
import { 
  DropdownMenu, 
  DropdownMenuContent, 
  DropdownMenuItem, 
  DropdownMenuSeparator, 
  DropdownMenuTrigger 
} from '@/components/ui/dropdown-menu';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@epsx/ui';
import { useToast } from '@/components/ui/use-toast';
import { format, formatDistance, isPast, addDays, addHours } from 'date-fns';

import {
  TemporaryPermission,
  CreateTemporaryPermissionData,
  UpdateTemporaryPermissionData,
  ListTemporaryPermissionsParams,
  createTemporaryPermission,
  getUserTemporaryPermissions,
  updateTemporaryPermission,
  revokeTemporaryPermission,
  deleteTemporaryPermission,
  cleanupExpiredPermissions,
} from '@/lib/actions/temporary-permission-actions';

interface TemporaryPermissionManagerProps {
  userId: string;
}

const STATUS_COLORS = {
  active: 'bg-green-100 text-green-800',
  suspended: 'bg-yellow-100 text-yellow-800',
  expired: 'bg-gray-100 text-gray-800',
  revoked: 'bg-red-100 text-red-800',
};

const STATUS_ICONS = {
  active: CheckCircle,
  suspended: AlertTriangle,
  expired: Clock,
  revoked: XCircle,
};

export function TemporaryPermissionManager({ userId }: TemporaryPermissionManagerProps) {
  const { toast } = useToast();
  const [permissions, setPermissions] = useState<TemporaryPermission[]>([]);
  const [loading, setLoading] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [filter, setFilter] = useState<'all' | 'active' | 'expired' | 'revoked'>('all');
  const [searchTerm, setSearchTerm] = useState('');
  
  // Form state
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [editingPermission, setEditingPermission] = useState<TemporaryPermission | null>(null);
  const [formData, setFormData] = useState<CreateTemporaryPermissionData>({
    user_id: userId,
    permission: '',
    resource: '',
    action: '',
    expires_at: '',
    reason: '',
  });

  const loadPermissions = async () => {
    try {
      setRefreshing(true);
      const result = await getUserTemporaryPermissions(userId);
      
      if (result.success && result.data) {
        setPermissions(result.data);
      } else {
        toast({
          title: 'Error',
          description: result.error?.message || 'Failed to load temporary permissions',
          variant: 'destructive',
        });
      }
    } catch (error) {
      console.error('Failed to load temporary permissions:', error);
      toast({
        title: 'Error',
        description: 'An unexpected error occurred',
        variant: 'destructive',
      });
    } finally {
      setRefreshing(false);
    }
  };

  useEffect(() => {
    loadPermissions();
  }, [userId]);

  const filteredPermissions = useMemo(() => {
    return permissions.filter(permission => {
      // Status filter
      if (filter !== 'all' && permission.status !== filter) {
        return false;
      }
      
      // Search filter
      if (searchTerm) {
        const searchLower = searchTerm.toLowerCase();
        return (
          permission.permission.toLowerCase().includes(searchLower) ||
          permission.resource.toLowerCase().includes(searchLower) ||
          permission.action.toLowerCase().includes(searchLower) ||
          (permission.reason && permission.reason.toLowerCase().includes(searchLower))
        );
      }
      
      return true;
    });
  }, [permissions, filter, searchTerm]);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!formData.permission || !formData.resource || !formData.action || !formData.expires_at) {
      toast({
        title: 'Validation Error',
        description: 'Please fill in all required fields',
        variant: 'destructive',
      });
      return;
    }

    try {
      setLoading(true);
      
      const result = await createTemporaryPermission(formData);
      
      if (result.success) {
        toast({
          title: 'Success',
          description: 'Temporary permission created successfully',
        });
        
        setShowCreateForm(false);
        setFormData({
          user_id: userId,
          permission: '',
          resource: '',
          action: '',
          expires_at: '',
          reason: '',
        });
        
        await loadPermissions();
      } else {
        toast({
          title: 'Error',
          description: result.error?.message || 'Failed to create temporary permission',
          variant: 'destructive',
        });
      }
    } catch (error) {
      console.error('Failed to create temporary permission:', error);
      toast({
        title: 'Error',
        description: 'An unexpected error occurred',
        variant: 'destructive',
      });
    } finally {
      setLoading(false);
    }
  };

  const handleRevoke = async (permission: TemporaryPermission, reason?: string) => {
    try {
      const result = await revokeTemporaryPermission(permission.id, reason);
      
      if (result.success) {
        toast({
          title: 'Success',
          description: 'Permission revoked successfully',
        });
        await loadPermissions();
      } else {
        toast({
          title: 'Error',
          description: result.error?.message || 'Failed to revoke permission',
          variant: 'destructive',
        });
      }
    } catch (error) {
      console.error('Failed to revoke permission:', error);
      toast({
        title: 'Error',
        description: 'An unexpected error occurred',
        variant: 'destructive',
      });
    }
  };

  const handleDelete = async (permission: TemporaryPermission) => {
    try {
      const result = await deleteTemporaryPermission(permission.id);
      
      if (result.success) {
        toast({
          title: 'Success',
          description: 'Permission deleted successfully',
        });
        await loadPermissions();
      } else {
        toast({
          title: 'Error',
          description: result.error?.message || 'Failed to delete permission',
          variant: 'destructive',
        });
      }
    } catch (error) {
      console.error('Failed to delete permission:', error);
      toast({
        title: 'Error',
        description: 'An unexpected error occurred',
        variant: 'destructive',
      });
    }
  };

  const handleCleanup = async () => {
    try {
      const result = await cleanupExpiredPermissions();
      
      if (result.success && result.data) {
        toast({
          title: 'Success',
          description: `Cleaned up ${result.data.cleaned_count} expired permissions`,
        });
        await loadPermissions();
      } else {
        toast({
          title: 'Error',
          description: result.error?.message || 'Failed to cleanup expired permissions',
          variant: 'destructive',
        });
      }
    } catch (error) {
      console.error('Failed to cleanup expired permissions:', error);
      toast({
        title: 'Error',
        description: 'An unexpected error occurred',
        variant: 'destructive',
      });
    }
  };

  const activePermissions = permissions.filter(p => p.status === 'active' && !p.is_expired);
  const expiredPermissions = permissions.filter(p => p.is_expired || p.status === 'expired');
  const revokedPermissions = permissions.filter(p => p.status === 'revoked');

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-semibold">Temporary Permissions</h3>
          <p className="text-sm text-muted-foreground">
            Manage time-limited access permissions for this user
          </p>
        </div>
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={loadPermissions}
            disabled={refreshing}
          >
            <RefreshCw className={`h-4 w-4 ${refreshing ? 'animate-spin' : ''}`} />
          </Button>
          <Button
            size="sm"
            onClick={() => setShowCreateForm(true)}
          >
            <Plus className="h-4 w-4 mr-2" />
            Grant Permission
          </Button>
        </div>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2">
              <CheckCircle className="h-5 w-5 text-green-500" />
              <div>
                <p className="text-2xl font-bold">{activePermissions.length}</p>
                <p className="text-sm text-muted-foreground">Active</p>
              </div>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2">
              <Clock className="h-5 w-5 text-gray-500" />
              <div>
                <p className="text-2xl font-bold">{expiredPermissions.length}</p>
                <p className="text-sm text-muted-foreground">Expired</p>
              </div>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2">
              <XCircle className="h-5 w-5 text-red-500" />
              <div>
                <p className="text-2xl font-bold">{revokedPermissions.length}</p>
                <p className="text-sm text-muted-foreground">Revoked</p>
              </div>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-4">
            <div className="flex items-center gap-2">
              <Shield className="h-5 w-5 text-blue-500" />
              <div>
                <p className="text-2xl font-bold">{permissions.length}</p>
                <p className="text-sm text-muted-foreground">Total</p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Filters */}
      <div className="flex flex-col sm:flex-row gap-4">
        <div className="flex-1">
          <Input
            placeholder="Search permissions, resources, actions..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="max-w-sm"
          />
        </div>
        <Select value={filter} onValueChange={(value: any) => setFilter(value)}>
          <SelectTrigger className="w-[180px]">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Permissions</SelectItem>
            <SelectItem value="active">Active Only</SelectItem>
            <SelectItem value="expired">Expired Only</SelectItem>
            <SelectItem value="revoked">Revoked Only</SelectItem>
          </SelectContent>
        </Select>
        {expiredPermissions.length > 0 && (
          <Button
            variant="outline"
            size="sm"
            onClick={handleCleanup}
          >
            <Trash2 className="h-4 w-4 mr-2" />
            Cleanup Expired
          </Button>
        )}
      </div>

      {/* Permissions List */}
      <div className="space-y-4">
        {filteredPermissions.length === 0 ? (
          <Card>
            <CardContent className="p-8 text-center">
              <Clock className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
              <h3 className="text-lg font-medium mb-2">No temporary permissions found</h3>
              <p className="text-muted-foreground mb-4">
                {filter === 'all' 
                  ? 'This user has no temporary permissions yet.'
                  : `No ${filter} temporary permissions found.`
                }
              </p>
              <Button
                size="sm"
                onClick={() => setShowCreateForm(true)}
              >
                <Plus className="h-4 w-4 mr-2" />
                Grant First Permission
              </Button>
            </CardContent>
          </Card>
        ) : (
          filteredPermissions.map((permission) => {
            const StatusIcon = STATUS_ICONS[permission.status];
            const isExpiringSoon = !permission.is_expired && 
              new Date(permission.expires_at) < addHours(new Date(), 24);
            
            return (
              <Card key={permission.id}>
                <CardContent className="p-4">
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-2">
                        <Badge className={STATUS_COLORS[permission.status]}>
                          <StatusIcon className="h-3 w-3 mr-1" />
                          {permission.status}
                        </Badge>
                        {isExpiringSoon && (
                          <Badge variant="destructive" className="text-xs">
                            <AlertTriangle className="h-3 w-3 mr-1" />
                            Expiring Soon
                          </Badge>
                        )}
                      </div>
                      
                      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-3">
                        <div>
                          <p className="text-xs text-muted-foreground">Permission</p>
                          <p className="font-medium">{permission.permission}</p>
                        </div>
                        <div>
                          <p className="text-xs text-muted-foreground">Resource</p>
                          <p className="font-medium">{permission.resource}</p>
                        </div>
                        <div>
                          <p className="text-xs text-muted-foreground">Action</p>
                          <p className="font-medium">{permission.action}</p>
                        </div>
                      </div>

                      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm text-muted-foreground">
                        <div>
                          <p className="flex items-center gap-1">
                            <Calendar className="h-3 w-3" />
                            Granted: {format(new Date(permission.granted_at), 'PPp')}
                          </p>
                          <p className="flex items-center gap-1 mt-1">
                            <Clock className="h-3 w-3" />
                            Expires: {format(new Date(permission.expires_at), 'PPp')} 
                            ({formatDistance(new Date(permission.expires_at), new Date(), { addSuffix: true })})
                          </p>
                        </div>
                        {permission.reason && (
                          <div>
                            <p className="flex items-center gap-1">
                              <FileText className="h-3 w-3" />
                              Reason: {permission.reason}
                            </p>
                          </div>
                        )}
                      </div>

                      {permission.status === 'revoked' && permission.revoked_at && (
                        <div className="mt-2 p-2 bg-red-50 rounded text-sm">
                          <p className="text-red-700">
                            Revoked on {format(new Date(permission.revoked_at), 'PPp')}
                            {permission.revocation_reason && ` - ${permission.revocation_reason}`}
                          </p>
                        </div>
                      )}
                    </div>
                    
                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <Button variant="ghost" size="sm">
                          <MoreHorizontal className="h-4 w-4" />
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end">
                        {permission.status === 'active' && !permission.is_expired && (
                          <>
                            <DropdownMenuItem onClick={() => setEditingPermission(permission)}>
                              <Edit className="h-4 w-4 mr-2" />
                              Edit
                            </DropdownMenuItem>
                            <DropdownMenuItem 
                              onClick={() => handleRevoke(permission)}
                              className="text-destructive"
                            >
                              <Ban className="h-4 w-4 mr-2" />
                              Revoke
                            </DropdownMenuItem>
                            <DropdownMenuSeparator />
                          </>
                        )}
                        <DropdownMenuItem 
                          onClick={() => handleDelete(permission)}
                          className="text-destructive"
                        >
                          <Trash className="h-4 w-4 mr-2" />
                          Delete
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </div>
                </CardContent>
              </Card>
            );
          })
        )}
      </div>

      {/* Create Permission Dialog */}
      <Dialog open={showCreateForm} onOpenChange={setShowCreateForm}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>Grant Temporary Permission</DialogTitle>
            <DialogDescription>
              Assign a time-limited permission to this user
            </DialogDescription>
          </DialogHeader>
          
          <form onSubmit={handleCreate} className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="permission">Permission</Label>
              <Input
                id="permission"
                value={formData.permission}
                onChange={(e) => setFormData(prev => ({ ...prev, permission: e.target.value }))}
                placeholder="e.g., admin.users"
                required
              />
            </div>
            
            <div className="space-y-2">
              <Label htmlFor="resource">Resource</Label>
              <Input
                id="resource"
                value={formData.resource}
                onChange={(e) => setFormData(prev => ({ ...prev, resource: e.target.value }))}
                placeholder="e.g., user_accounts"
                required
              />
            </div>
            
            <div className="space-y-2">
              <Label htmlFor="action">Action</Label>
              <Input
                id="action"
                value={formData.action}
                onChange={(e) => setFormData(prev => ({ ...prev, action: e.target.value }))}
                placeholder="e.g., read, write, manage"
                required
              />
            </div>
            
            <div className="space-y-2">
              <Label htmlFor="expires_at">Expires At</Label>
              <Input
                id="expires_at"
                type="datetime-local"
                value={formData.expires_at}
                onChange={(e) => setFormData(prev => ({ ...prev, expires_at: e.target.value }))}
                min={new Date().toISOString().slice(0, 16)}
                required
              />
            </div>
            
            <div className="space-y-2">
              <Label htmlFor="reason">Reason (Optional)</Label>
              <Textarea
                id="reason"
                value={formData.reason}
                onChange={(e) => setFormData(prev => ({ ...prev, reason: e.target.value }))}
                placeholder="Why is this permission needed?"
                rows={2}
              />
            </div>
            
            <div className="flex gap-2 pt-4">
              <Button type="submit" disabled={loading} className="flex-1">
                {loading ? 'Creating...' : 'Grant Permission'}
              </Button>
              <Button
                type="button"
                variant="outline"
                onClick={() => setShowCreateForm(false)}
              >
                Cancel
              </Button>
            </div>
          </form>
        </DialogContent>
      </Dialog>
    </div>
  );
}