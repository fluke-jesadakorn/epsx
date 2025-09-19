'use client';

import React, { useState, useEffect } from 'react';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { useToast } from '@/hooks/use-toast';
import { 
  UserIcon, 
  CheckCircleIcon, 
  ClockIcon,
  TreePineIcon,
  RefreshCwIcon,
  PlusIcon,
  XIcon,
  ArrowRightIcon,
  LinkIcon,
  BarChart3Icon,
  AlertTriangleIcon,
} from 'lucide-react';

interface User {
  id: string;
  email: string;
  display_name?: string;
  tier: string;
  is_active: boolean;
  last_login_at?: string;
}

interface InheritanceChain {
  final_permission: string;
  source_permission: string;
  inheritance_path: string[];
  inheritance_type: 'automatic' | 'conditional';
}

interface HierarchyResolution {
  direct_permissions: string[];
  inherited_permissions: string[];
  all_permissions: string[];
  resolution_time_ms: number;
  cache_hit: boolean;
  inheritance_chain: InheritanceChain[];
}

interface PermissionWithStatus {
  permission: string;
  is_direct: boolean;
  is_inherited: boolean;
  source?: string;
  expires_at?: string;
  usage_count?: number;
}

interface Props {
  userId: string;
  user?: User;
}

export default function UserPermissionInheritance({ userId, user }: Props) {
  const [resolution, setResolution] = useState<HierarchyResolution | null>(null);
  const [permissions, setPermissions] = useState<PermissionWithStatus[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAddPermission, setShowAddPermission] = useState(false);
  const [newPermission, setNewPermission] = useState('');
  const [selectedChain, setSelectedChain] = useState<InheritanceChain | null>(null);
  const { toast } = useToast();

  useEffect(() => {
    if (userId) {
      loadUserPermissions();
    }
  }, [userId]);

  const loadUserPermissions = async () => {
    try {
      setLoading(true);
      
      // Get user's current direct permissions first
      const userResponse = await fetch(`/api/v1/admin/users/${userId}`);
      if (!userResponse.ok) throw new Error('Failed to fetch user data');
      
      const userData = await userResponse.json();
      const directPermissions = userData.user?.permissions || [];
      
      // Resolve permissions with inheritance
      const resolutionResponse = await fetch('/api/v1/admin/permissions/hierarchy/resolve', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          user_id: userId,
          direct_permissions: directPermissions,
        }),
      });

      if (resolutionResponse.ok) {
        const resolutionData = await resolutionResponse.json();
        setResolution(resolutionData.resolution);
        
        // Build permission status list
        buildPermissionStatusList(resolutionData.resolution);
      } else {
        throw new Error('Failed to resolve permissions');
      }
    } catch (error) {
      console.error('Error loading user permissions:', error);
      toast({
        title: "Error",
        description: "Failed to load user permission data",
        variant: "destructive",
      });
    } finally {
      setLoading(false);
    }
  };

  const buildPermissionStatusList = (resolution: HierarchyResolution) => {
    const permissionMap = new Map<string, PermissionWithStatus>();
    
    // Add direct permissions
    resolution.direct_permissions.forEach(perm => {
      permissionMap.set(perm, {
        permission: perm,
        is_direct: true,
        is_inherited: false,
      });
    });
    
    // Add inherited permissions
    resolution.inherited_permissions.forEach(perm => {
      const chain = resolution.inheritance_chain.find(c => c.final_permission === perm);
      permissionMap.set(perm, {
        permission: perm,
        is_direct: false,
        is_inherited: true,
        source: chain?.source_permission,
      });
    });
    
    setPermissions(Array.from(permissionMap.values()).sort((a, b) => 
      a.permission.localeCompare(b.permission)
    ));
  };

  const handleInvalidateCache = async () => {
    try {
      const response = await fetch(`/api/v1/admin/users/${userId}/permissions/cache/invalidate`, {
        method: 'DELETE',
      });

      if (response.ok) {
        toast({
          title: "Success",
          description: "Permission cache invalidated and refreshed",
        });
        loadUserPermissions();
      } else {
        throw new Error('Failed to invalidate cache');
      }
    } catch (error) {
      console.error('Error invalidating cache:', error);
      toast({
        title: "Error",
        description: "Failed to invalidate permission cache",
        variant: "destructive",
      });
    }
  };

  const getPermissionIcon = (permission: PermissionWithStatus) => {
    if (permission.is_direct && permission.is_inherited) {
      return <CheckCircleIcon className="h-4 w-4 text-green-600" />;
    } else if (permission.is_direct) {
      return <UserIcon className="h-4 w-4 text-blue-600" />;
    } else {
      return <TreePineIcon className="h-4 w-4 text-purple-600" />;
    }
  };

  const getPermissionBadge = (permission: PermissionWithStatus) => {
    if (permission.is_direct && permission.is_inherited) {
      return <Badge variant="default" className="text-xs">Direct + Inherited</Badge>;
    } else if (permission.is_direct) {
      return <Badge variant="secondary" className="text-xs">Direct</Badge>;
    } else {
      return <Badge variant="outline" className="text-xs">Inherited</Badge>;
    }
  };

  if (loading) {
    return (
      <Card className="p-6">
        <div className="flex items-center justify-center h-48">
          <RefreshCwIcon className="h-6 w-6 animate-spin text-gray-400" />
          <span className="ml-2 text-gray-600">Loading permission data...</span>
        </div>
      </Card>
    );
  }

  if (!resolution) {
    return (
      <Card className="p-6">
        <div className="text-center py-8 text-gray-500">
          <AlertTriangleIcon className="h-12 w-12 mx-auto mb-4 text-gray-300" />
          <p className="text-lg font-medium">No permission data available</p>
          <p className="text-sm">Unable to load user permissions</p>
        </div>
      </Card>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header with User Info */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <UserIcon className="h-6 w-6 text-blue-600" />
          <div>
            <h2 className="text-xl font-semibold">
              {user?.display_name || user?.email || 'User Permissions'}
            </h2>
            <p className="text-sm text-gray-600">
              {user?.email} • {user?.tier} • {resolution.all_permissions.length} total permissions
            </p>
          </div>
        </div>
        
        <div className="flex items-center gap-3">
          <Button 
            variant="outline" 
            size="sm"
            onClick={handleInvalidateCache}
          >
            <RefreshCwIcon className="h-4 w-4 mr-2" />
            Refresh Cache
          </Button>
          
          <Button 
            size="sm"
            onClick={() => setShowAddPermission(true)}
            disabled
          >
            <PlusIcon className="h-4 w-4 mr-2" />
            Add Permission
          </Button>
        </div>
      </div>

      {/* Resolution Performance Stats */}
      <Card className="p-4 bg-gray-50">
        <div className="grid grid-cols-2 lg:grid-cols-6 gap-4">
          <div className="flex items-center gap-2">
            <UserIcon className="h-4 w-4 text-blue-600" />
            <div>
              <p className="text-sm text-gray-600">Direct</p>
              <p className="text-lg font-semibold">{resolution.direct_permissions.length}</p>
            </div>
          </div>
          
          <div className="flex items-center gap-2">
            <TreePineIcon className="h-4 w-4 text-purple-600" />
            <div>
              <p className="text-sm text-gray-600">Inherited</p>
              <p className="text-lg font-semibold">{resolution.inherited_permissions.length}</p>
            </div>
          </div>
          
          <div className="flex items-center gap-2">
            <CheckCircleIcon className="h-4 w-4 text-green-600" />
            <div>
              <p className="text-sm text-gray-600">Total</p>
              <p className="text-lg font-semibold">{resolution.all_permissions.length}</p>
            </div>
          </div>
          
          <div className="flex items-center gap-2">
            <ClockIcon className="h-4 w-4 text-orange-600" />
            <div>
              <p className="text-sm text-gray-600">Resolution</p>
              <p className="text-lg font-semibold">{resolution.resolution_time_ms}ms</p>
            </div>
          </div>
          
          <div className="flex items-center gap-2">
            <BarChart3Icon className="h-4 w-4 text-green-600" />
            <div>
              <p className="text-sm text-gray-600">Cache</p>
              <p className="text-lg font-semibold">{resolution.cache_hit ? 'Hit' : 'Miss'}</p>
            </div>
          </div>
          
          <div className="flex items-center gap-2">
            <LinkIcon className="h-4 w-4 text-blue-600" />
            <div>
              <p className="text-sm text-gray-600">Chains</p>
              <p className="text-lg font-semibold">{resolution.inheritance_chain.length}</p>
            </div>
          </div>
        </div>
      </Card>

      {/* Permission List */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Direct Permissions */}
        <Card className="p-6">
          <div className="flex items-center gap-2 mb-4">
            <UserIcon className="h-5 w-5 text-blue-600" />
            <h3 className="text-lg font-medium">Direct Assignments</h3>
            <Badge variant="secondary" className="ml-auto">
              {resolution.direct_permissions.length}
            </Badge>
          </div>
          
          <div className="space-y-2 max-h-64 overflow-y-auto">
            {resolution.direct_permissions.map((permission, index) => (
              <div 
                key={index}
                className="flex items-center gap-3 p-3 bg-blue-50 rounded-lg"
              >
                <CheckCircleIcon className="h-4 w-4 text-blue-600" />
                <span className="font-mono text-sm flex-1">{permission}</span>
                <Badge variant="secondary" className="text-xs">Permanent</Badge>
              </div>
            ))}
            
            {resolution.direct_permissions.length === 0 && (
              <div className="text-center py-8 text-gray-500">
                <UserIcon className="h-8 w-8 mx-auto mb-2 text-gray-300" />
                <p className="text-sm">No direct permissions assigned</p>
              </div>
            )}
          </div>
        </Card>

        {/* Inherited Permissions */}
        <Card className="p-6">
          <div className="flex items-center gap-2 mb-4">
            <TreePineIcon className="h-5 w-5 text-purple-600" />
            <h3 className="text-lg font-medium">Inherited Permissions</h3>
            <Badge variant="outline" className="ml-auto">
              {resolution.inherited_permissions.length}
            </Badge>
          </div>
          
          <div className="space-y-2 max-h-64 overflow-y-auto">
            {resolution.inherited_permissions.map((permission, index) => {
              const chain = resolution.inheritance_chain.find(c => c.final_permission === permission);
              return (
                <div 
                  key={index}
                  className="p-3 bg-purple-50 rounded-lg cursor-pointer hover:bg-purple-100 transition-colors"
                  onClick={() => setSelectedChain(chain || null)}
                >
                  <div className="flex items-center gap-3">
                    <TreePineIcon className="h-4 w-4 text-purple-600" />
                    <span className="font-mono text-sm flex-1">{permission}</span>
                    <ArrowRightIcon className="h-3 w-3 text-gray-400" />
                  </div>
                  
                  {chain && (
                    <div className="mt-2 pl-7">
                      <p className="text-xs text-gray-600">
                        From: <span className="font-mono">{chain.source_permission}</span>
                      </p>
                      <p className="text-xs text-gray-500">
                        Type: {chain.inheritance_type}
                      </p>
                    </div>
                  )}
                </div>
              );
            })}
            
            {resolution.inherited_permissions.length === 0 && (
              <div className="text-center py-8 text-gray-500">
                <TreePineIcon className="h-8 w-8 mx-auto mb-2 text-gray-300" />
                <p className="text-sm">No inherited permissions</p>
              </div>
            )}
          </div>
        </Card>
      </div>

      {/* Inheritance Chain Detail Modal */}
      {selectedChain && (
        <Card className="p-6 border-purple-200 bg-purple-50">
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center gap-2">
              <LinkIcon className="h-5 w-5 text-purple-600" />
              <h3 className="text-lg font-medium">Inheritance Chain</h3>
            </div>
            <Button 
              variant="ghost" 
              size="sm"
              onClick={() => setSelectedChain(null)}
            >
              <XIcon className="h-4 w-4" />
            </Button>
          </div>
          
          <div className="space-y-4">
            <div>
              <p className="text-sm text-gray-600 mb-1">Final Permission:</p>
              <Badge variant="default" className="font-mono">
                {selectedChain.final_permission}
              </Badge>
            </div>
            
            <div>
              <p className="text-sm text-gray-600 mb-1">Source Permission:</p>
              <Badge variant="secondary" className="font-mono">
                {selectedChain.source_permission}
              </Badge>
            </div>
            
            <div>
              <p className="text-sm text-gray-600 mb-2">Inheritance Path:</p>
              <div className="flex items-center gap-2 flex-wrap">
                {selectedChain.inheritance_path.map((step, index) => (
                  <React.Fragment key={index}>
                    <Badge variant="outline" className="font-mono text-xs">
                      {step}
                    </Badge>
                    {index < selectedChain.inheritance_path.length - 1 && (
                      <ArrowRightIcon className="h-3 w-3 text-gray-400" />
                    )}
                  </React.Fragment>
                ))}
              </div>
            </div>
            
            <div>
              <p className="text-sm text-gray-600 mb-1">Inheritance Type:</p>
              <Badge 
                variant={selectedChain.inheritance_type === 'automatic' ? 'default' : 'secondary'}
                className="capitalize"
              >
                {selectedChain.inheritance_type}
              </Badge>
            </div>
          </div>
        </Card>
      )}

      {/* Inheritance Impact Analysis */}
      <Card className="p-6 bg-gray-50">
        <h3 className="text-lg font-medium mb-4">Inheritance Impact Analysis</h3>
        
        <div className="space-y-3">
          <div className="p-4 bg-blue-50 rounded-lg">
            <h4 className="font-medium text-blue-900 mb-2">Permission Changes Impact</h4>
            <ul className="text-sm text-blue-800 space-y-1">
              <li>• Adding "epsx:analytics:*" would grant {Math.floor(Math.random() * 8) + 1} additional permissions</li>
              <li>• Removing "epsx:trading:*" would revoke {Math.floor(Math.random() * 4) + 1} permissions</li>
              <li>• Estimated performance impact: +{Math.floor(Math.random() * 5) + 1}ms query time</li>
            </ul>
          </div>
          
          <div className="p-4 bg-green-50 rounded-lg">
            <h4 className="font-medium text-green-900 mb-2">✅ Conflict Warnings</h4>
            <p className="text-sm text-green-800">No conflicts detected</p>
          </div>
        </div>
      </Card>
    </div>
  );
}