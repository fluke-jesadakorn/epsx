'use client';

import { useState, useEffect, useMemo, memo, useCallback } from 'react';
import { RefreshCw, Plus } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { useToast } from '@/components/ui/use-toast';

// Extracted components
import PermissionStatsCards from './PermissionStatsCards';
import PermissionFilters from './PermissionFilters';
import EmbeddedPermissionForm from './EmbeddedPermissionForm';
import PermissionListItem from './PermissionListItem';

// Actions and types
import {
  TemporaryPermission,
  getUserTemporaryPermissions,
  revokeTemporaryPermission,
  grantTemporaryPermission,
} from '@/lib/actions/consolidated-permission-actions';

interface EmbeddedPermissionData {
  basePermission: string;
  expiryTimestamp: number;
  reason?: string;
}

interface TemporaryPermissionManagerProps {
  userId: string;
  enableEmbeddedTimestamps?: boolean;
  showTimeline?: boolean;
  allowQuickActions?: boolean;
}

function TemporaryPermissionManagerRefactored({ 
  userId, 
  enableEmbeddedTimestamps = true,
  showTimeline = true,
  allowQuickActions = true 
}: TemporaryPermissionManagerProps) {
  const { toast } = useToast();
  
  // Core state
  const [permissions, setPermissions] = useState<TemporaryPermission[]>([]);
  const [loading, setLoading] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  
  // Filter state
  const [filter, setFilter] = useState<'all' | 'active' | 'expired' | 'revoked'>('all');
  const [searchTerm, setSearchTerm] = useState('');
  
  // Form state
  const [showEmbeddedForm, setShowEmbeddedForm] = useState(false);

  // Load permissions
  const loadPermissions = useCallback(async () => {
    try {
      setRefreshing(true);
      const result = await getUserTemporaryPermissions(userId);
      
      if (result.success && result.data) {
        setPermissions(result.data);
      } else {
        toast({
          title: 'Error',
          description: result.error || 'Failed to load temporary permissions',
          variant: 'destructive',
        });
      }
    } catch (error) {
      console.error('Error loading permissions:', error);
      toast({
        title: 'Error',
        description: 'Failed to load temporary permissions',
        variant: 'destructive',
      });
    } finally {
      setRefreshing(false);
    }
  }, [userId, toast]);

  // Load permissions on mount
  useEffect(() => {
    loadPermissions();
  }, [loadPermissions]);

  // Filter permissions
  const filteredPermissions = useMemo(() => {
    return permissions.filter(permission => {
      // Status filter based on isActive and expiry
      if (filter !== 'all') {
        const isExpired = new Date(permission.expiresAt) <= new Date();
        if (filter === 'active' && (!permission.isActive || isExpired)) return false;
        if (filter === 'expired' && !isExpired) return false;
        if (filter === 'revoked' && permission.isActive) return false;
      }
      
      // Search filter
      if (searchTerm) {
        const searchLower = searchTerm.toLowerCase();
        return (
          permission.permission.toLowerCase().includes(searchLower) ||
          (permission.reason && permission.reason.toLowerCase().includes(searchLower))
        );
      }
      
      return true;
    });
  }, [permissions, filter, searchTerm]);

  // Calculate stats and transform to expected format
  const permissionStats = useMemo(() => {
    const transformPermission = (p: TemporaryPermission) => ({
      id: p.id,
      status: !p.isActive ? 'revoked' : new Date(p.expiresAt) <= new Date() ? 'expired' : 'active',
      is_expired: new Date(p.expiresAt) <= new Date()
    });
    
    return {
      activePermissions: permissions
        .filter(p => p.isActive && new Date(p.expiresAt) > new Date())
        .map(transformPermission),
      expiredPermissions: permissions
        .filter(p => new Date(p.expiresAt) <= new Date())
        .map(transformPermission),
      revokedPermissions: permissions
        .filter(p => !p.isActive)
        .map(transformPermission)
    };
  }, [permissions]);

  // Handle embedded permission creation
  const handleCreateEmbeddedPermission = useCallback(async (data: EmbeddedPermissionData) => {
    try {
      setLoading(true);
      
      // Create embedded permission string: basePermission:timestamp
      const embeddedPermission = `${data.basePermission}:${data.expiryTimestamp}`;
      
      // Use the existing grant permission API (you might need to adjust this)
      const result = await grantTemporaryPermission({
        userId: userId,
        permission: embeddedPermission,
        expiresAt: new Date(data.expiryTimestamp * 1000).toISOString(),
        reason: data.reason || 'Embedded timestamp permission',
      });

      if (result.success) {
        toast({
          title: 'Success',
          description: 'Embedded permission granted successfully',
        });
        setShowEmbeddedForm(false);
        loadPermissions(); // Refresh the list
      } else {
        throw new Error(result.error || 'Failed to grant permission');
      }
    } catch (error) {
      console.error('Error creating embedded permission:', error);
      toast({
        title: 'Error',
        description: error instanceof Error ? error.message : 'Failed to create embedded permission',
        variant: 'destructive',
      });
      throw error; // Re-throw so form knows it failed
    } finally {
      setLoading(false);
    }
  }, [userId, toast, loadPermissions]);

  // Handle permission revocation
  const handleRevokePermission = useCallback(async (permissionId: string, reason: string) => {
    try {
      setLoading(true);
      const result = await revokeTemporaryPermission(permissionId);
      
      if (result.success) {
        toast({
          title: 'Success',
          description: 'Permission revoked successfully',
        });
        loadPermissions(); // Refresh the list
      } else {
        throw new Error(result.error || 'Failed to revoke permission');
      }
    } catch (error) {
      console.error('Error revoking permission:', error);
      toast({
        title: 'Error',
        description: error instanceof Error ? error.message : 'Failed to revoke permission',
        variant: 'destructive',
      });
    } finally {
      setLoading(false);
    }
  }, [toast, loadPermissions]);

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
            <RefreshCw className="h-4 w-4" />
          </Button>
          
          {enableEmbeddedTimestamps && (
            <Button
              size="sm"
              onClick={() => setShowEmbeddedForm(true)}
            >
              <Plus className="h-4 w-4 mr-2" />
              Grant Permission
            </Button>
          )}
        </div>
      </div>

      {/* Stats Cards */}
      <PermissionStatsCards stats={permissionStats} />

      {/* Filters */}
      <PermissionFilters
        searchTerm={searchTerm}
        onSearchChange={setSearchTerm}
        filter={filter}
        onFilterChange={setFilter}
        totalCount={permissions.length}
        filteredCount={filteredPermissions.length}
      />

      {/* Permissions List */}
      <div className="space-y-4">
        {filteredPermissions.length === 0 ? (
          <Alert>
            <AlertDescription>
              {permissions.length === 0 
                ? 'No temporary permissions found for this user.' 
                : 'No permissions match your current filters.'
              }
            </AlertDescription>
          </Alert>
        ) : (
          filteredPermissions.map((permission) => {
            // Transform to expected format for PermissionListItem
            const transformedPermission = {
              id: permission.id,
              user_id: permission.userId,
              permission: permission.permission,
              resource: permission.permission.split(':')[1] || '*',
              action: permission.permission.split(':')[2] || '*',
              expires_at: permission.expiresAt,
              status: (!permission.isActive ? 'revoked' : new Date(permission.expiresAt) <= new Date() ? 'expired' : 'active') as 'active' | 'expired' | 'revoked',
              is_expired: new Date(permission.expiresAt) <= new Date(),
              reason: permission.reason,
              created_at: permission.grantedAt,
              revoked_at: undefined,
              revoked_by: undefined,
              revoked_reason: undefined
            };
            
            return (
              <PermissionListItem
                key={permission.id}
                permission={transformedPermission}
                onRevoke={handleRevokePermission}
                enableEmbeddedTimestamps={enableEmbeddedTimestamps}
              />
            );
          })
        )}
      </div>

      {/* Embedded Permission Form */}
      {showEmbeddedForm && (
        <EmbeddedPermissionForm
          userId={userId}
          isOpen={showEmbeddedForm}
          onClose={() => setShowEmbeddedForm(false)}
          onSubmit={handleCreateEmbeddedPermission}
          loading={loading}
        />
      )}
    </div>
  );
}

export default memo(TemporaryPermissionManagerRefactored);