/**
 * User Permissions - Permission Management Integration
 * Consolidates: UserPermissionsClient, UserPermissionsServer, UserPermissionsContent,
 * PermissionAssignmentCard, PermissionHistoryCard, PermissionStatsCards,
 * InteractivePermissionTreeView, PermissionConflictResolver, PermissionValidator,
 * PermissionRecommendations, PermissionImpactAnalysis
 */

'use client';

import { useState, useEffect, useMemo } from 'react';
import { 
  Shield,
  Plus,
  Minus,
  Clock,
  Calendar,
  AlertTriangle,
  CheckCircle,
  XCircle,
  TrendingUp,
  Eye,
  Edit,
  Trash2,
  RefreshCw,
  Filter,
  Search,
  Download,
  Settings
} from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Checkbox } from '@/components/ui/checkbox';
import { Switch } from '@/components/ui/switch';
import { Progress } from '@/components/ui/progress';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Label } from '@/components/ui/label';

import type { User, Permission, PermissionHealth, Platform } from '@/types/core';
import { adminClient } from '@/lib/api/unified-admin-client';

interface UserPermissionsProps {
  user: User;
  availablePermissions?: Array<{
    id: string;
    name: string;
    description: string;
    platform: Platform;
    category: string;
    risk: 'low' | 'medium' | 'high';
  }>;
  onPermissionChange?: (userId: string, permissions: string[]) => void;
  onPermissionGranted?: (permission: string) => void;
  onPermissionRevoked?: (permission: string) => void;
  className?: string;
}

interface PermissionItem {
  id: string;
  permission: string;
  platform: Platform;
  resource: string;
  action: string;
  granted: boolean;
  expiresAt?: string;
  grantedAt: string;
  grantedBy: string;
  isExpired: boolean;
  isExpiring: boolean;
  healthStatus: 'healthy' | 'warning' | 'critical';
}

interface PermissionRecommendation {
  type: 'grant' | 'revoke' | 'extend' | 'review';
  permission: string;
  reason: string;
  confidence: number;
  impact: 'low' | 'medium' | 'high';
}

export function UserPermissions({
  user,
  availablePermissions = [],
  onPermissionChange,
  onPermissionGranted,
  onPermissionRevoked,
  className = ''
}: UserPermissionsProps) {
  // State management
  const [activeTab, setActiveTab] = useState('current');
  const [isLoading, setIsLoading] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [filterPlatform, setFilterPlatform] = useState<string>('all');
  const [filterStatus, setFilterStatus] = useState<string>('all');
  const [showGrantDialog, setShowGrantDialog] = useState(false);
  const [selectedPermissions, setSelectedPermissions] = useState<string[]>([]);
  
  // Permission data state
  const [permissionItems, setPermissionItems] = useState<PermissionItem[]>([]);
  const [permissionHealth, setPermissionHealth] = useState<PermissionHealth | null>(null);
  const [recommendations, setRecommendations] = useState<PermissionRecommendation[]>([]);

  // Process user permissions into structured format
  const processPermissions = (): PermissionItem[] => {
    const now = Date.now() / 1000;
    
    return (user.permissions || []).map((permission, index) => {
      const parts = permission.split(':');
      const hasTimestamp = parts.length === 4 && !isNaN(parseInt(parts[3]));
      const expiresAt = hasTimestamp ? new Date(parseInt(parts[3]) * 1000).toISOString() : undefined;
      const isExpired = hasTimestamp && parseInt(parts[3]) <= now;
      const isExpiring = hasTimestamp && parseInt(parts[3]) <= now + (7 * 24 * 60 * 60);
      
      return {
        id: `${user.id}-${index}`,
        permission,
        platform: (parts[0] as Platform) || 'epsx',
        resource: parts[1] || 'unknown',
        action: parts[2] || 'unknown',
        granted: true,
        expiresAt,
        grantedAt: user.createdAt, // Fallback - would be actual grant time in real implementation
        grantedBy: 'admin', // Would be actual granter in real implementation
        isExpired,
        isExpiring: isExpiring && !isExpired,
        healthStatus: isExpired ? 'critical' : isExpiring ? 'warning' : 'healthy'
      };
    });
  };

  // Filtered permissions based on search and filters
  const filteredPermissions = useMemo(() => {
    let filtered = permissionItems;

    // Search filter
    if (searchQuery) {
      filtered = filtered.filter(item => 
        item.permission.toLowerCase().includes(searchQuery.toLowerCase()) ||
        item.resource.toLowerCase().includes(searchQuery.toLowerCase()) ||
        item.action.toLowerCase().includes(searchQuery.toLowerCase())
      );
    }

    // Platform filter
    if (filterPlatform !== 'all') {
      filtered = filtered.filter(item => item.platform === filterPlatform);
    }

    // Status filter
    if (filterStatus !== 'all') {
      filtered = filtered.filter(item => {
        switch (filterStatus) {
          case 'active': return !item.isExpired;
          case 'expiring': return item.isExpiring;
          case 'expired': return item.isExpired;
          default: return true;
        }
      });
    }

    return filtered;
  }, [permissionItems, searchQuery, filterPlatform, filterStatus]);

  // Available permissions that user doesn't have
  const availableToGrant = useMemo(() => {
    const userPermissionIds = new Set(user.permissions);
    return availablePermissions.filter(perm => !userPermissionIds.has(perm.id));
  }, [availablePermissions, user.permissions]);

  // Permission statistics
  const permissionStats = useMemo(() => {
    const total = permissionItems.length;
    const active = permissionItems.filter(p => !p.isExpired).length;
    const expiring = permissionItems.filter(p => p.isExpiring).length;
    const expired = permissionItems.filter(p => p.isExpired).length;
    
    return { total, active, expiring, expired };
  }, [permissionItems]);

  // Load permission data
  const loadPermissionData = async () => {
    setIsLoading(true);
    try {
      const items = processPermissions();
      setPermissionItems(items);

      // Load permission health
      const healthResponse = await adminClient.getPermissionExpiryStatus(user.id);
      if (healthResponse.success) {
        setPermissionHealth(healthResponse.data as PermissionHealth);
      }

      // Generate mock recommendations
      const mockRecommendations: PermissionRecommendation[] = [
        {
          type: 'extend',
          permission: 'epsx:analytics:view',
          reason: 'User frequently accesses analytics. Consider permanent access.',
          confidence: 85,
          impact: 'medium'
        },
        {
          type: 'revoke',
          permission: 'epsx:export:csv',
          reason: 'Permission has been unused for 30+ days.',
          confidence: 70,
          impact: 'low'
        }
      ];
      setRecommendations(mockRecommendations);

    } catch (error) {
      console.error('Failed to load permission data:', error);
    } finally {
      setIsLoading(false);
    }
  };

  // Grant permission handler
  const handleGrantPermission = async (permission: string, expiresAt?: string) => {
    try {
      const response = await adminClient.grantPermission(user.id, permission, expiresAt);
      if (response.success) {
        const updatedPermissions = [...(user.permissions || []), permission];
        onPermissionChange?.(user.id, updatedPermissions);
        onPermissionGranted?.(permission);
        loadPermissionData(); // Refresh data
      }
    } catch (error) {
      console.error('Failed to grant permission:', error);
    }
  };

  // Revoke permission handler
  const handleRevokePermission = async (permission: string) => {
    try {
      const response = await adminClient.revokePermission(user.id, permission);
      if (response.success) {
        const updatedPermissions = (user.permissions || []).filter(p => p !== permission);
        onPermissionChange?.(user.id, updatedPermissions);
        onPermissionRevoked?.(permission);
        loadPermissionData(); // Refresh data
      }
    } catch (error) {
      console.error('Failed to revoke permission:', error);
    }
  };

  // Bulk grant permissions
  const handleBulkGrant = async () => {
    if (selectedPermissions.length === 0) return;

    try {
      for (const permission of selectedPermissions) {
        await handleGrantPermission(permission);
      }
      setSelectedPermissions([]);
      setShowGrantDialog(false);
    } catch (error) {
      console.error('Bulk grant failed:', error);
    }
  };

  // Get status color
  const getStatusColor = (status: PermissionItem['healthStatus']) => {
    switch (status) {
      case 'healthy': return 'text-green-400';
      case 'warning': return 'text-yellow-400';
      case 'critical': return 'text-red-400';
      default: return 'text-gray-400';
    }
  };

  // Get status icon
  const getStatusIcon = (status: PermissionItem['healthStatus']) => {
    switch (status) {
      case 'healthy': return <CheckCircle className="w-4 h-4 text-green-500" />;
      case 'warning': return <AlertTriangle className="w-4 h-4 text-yellow-500" />;
      case 'critical': return <XCircle className="w-4 h-4 text-red-500" />;
      default: return <Shield className="w-4 h-4 text-gray-500" />;
    }
  };

  // Initial load
  useEffect(() => {
    loadPermissionData();
  }, [user.permissions]);

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        <div>
          <h2 className="text-2xl font-bold text-white">User Permissions</h2>
          <p className="text-gray-400">Manage permissions for {user.email}</p>
        </div>
        <div className="flex gap-2">
          <Button
            variant="outline"
                        onClick={loadPermissionData}
            disabled={isLoading}
          >
            <RefreshCw className={`w-4 h-4 mr-1 ${isLoading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
          <Button
            variant="outline"
                      >
            <Download className="w-4 h-4 mr-1" />
            Export
          </Button>
          <Button
                        onClick={() => setShowGrantDialog(true)}
          >
            <Plus className="w-4 h-4 mr-1" />
            Grant Permission
          </Button>
        </div>
      </div>

      {/* Permission Stats */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card className="p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-400">Total Permissions</p>
              <p className="text-2xl font-bold text-white">{permissionStats.total}</p>
            </div>
            <Shield className="w-8 h-8 text-blue-500" />
          </div>
        </Card>
        
        <Card className="p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-400">Active</p>
              <p className="text-2xl font-bold text-green-400">{permissionStats.active}</p>
            </div>
            <CheckCircle className="w-8 h-8 text-green-500" />
          </div>
        </Card>
        
        <Card className="p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-400">Expiring Soon</p>
              <p className="text-2xl font-bold text-yellow-400">{permissionStats.expiring}</p>
            </div>
            <Clock className="w-8 h-8 text-yellow-500" />
          </div>
        </Card>
        
        <Card className="p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-400">Expired</p>
              <p className="text-2xl font-bold text-red-400">{permissionStats.expired}</p>
            </div>
            <XCircle className="w-8 h-8 text-red-500" />
          </div>
        </Card>
      </div>

      {/* Filters */}
      <Card className="p-4">
        <div className="flex flex-col sm:flex-row gap-4">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-4 h-4" />
            <Input
              placeholder="Search permissions..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-10"
            />
          </div>
          
          <Select value={filterPlatform} onValueChange={setFilterPlatform}>
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
          
          <Select value={filterStatus} onValueChange={setFilterStatus}>
            <SelectTrigger className="w-full sm:w-48">
              <SelectValue placeholder="All Status" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Status</SelectItem>
              <SelectItem value="active">Active</SelectItem>
              <SelectItem value="expiring">Expiring Soon</SelectItem>
              <SelectItem value="expired">Expired</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </Card>

      {/* Main Content Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="current">Current Permissions</TabsTrigger>
          <TabsTrigger value="available">Available</TabsTrigger>
          <TabsTrigger value="recommendations">Recommendations</TabsTrigger>
        </TabsList>

        {/* Current Permissions Tab */}
        <TabsContent value="current" className="space-y-4">
          <Card className="p-6">
            <div className="space-y-4">
              {filteredPermissions.length === 0 ? (
                <div className="text-center py-8 text-gray-400">
                  <Shield className="w-12 h-12 mx-auto mb-4 opacity-50" />
                  <p>No permissions found matching your criteria.</p>
                </div>
              ) : (
                <div className="space-y-3">
                  {filteredPermissions.map(permission => (
                    <div
                      key={permission.id}
                      className="flex items-center justify-between p-4 bg-gray-800/30 rounded-lg"
                    >
                      <div className="flex items-center gap-4 flex-1">
                        <div className="flex-shrink-0">
                          {getStatusIcon(permission.healthStatus)}
                        </div>
                        
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2 mb-1">
                            <Badge variant="secondary">
                              {permission.platform}
                            </Badge>
                            <code className="text-sm bg-gray-800 px-2 py-1 rounded">
                              {permission.resource}:{permission.action}
                            </code>
                          </div>
                          
                          <div className="flex items-center gap-4 text-xs text-gray-400">
                            <span>Granted: {new Date(permission.grantedAt).toLocaleDateString()}</span>
                            {permission.expiresAt && (
                              <span className={permission.isExpiring ? 'text-yellow-400' : ''}>
                                Expires: {new Date(permission.expiresAt).toLocaleDateString()}
                              </span>
                            )}
                            <span>By: {permission.grantedBy}</span>
                          </div>
                        </div>

                        <div className={`text-sm font-medium ${getStatusColor(permission.healthStatus)}`}>
                          {permission.isExpired ? 'Expired' : 
                           permission.isExpiring ? 'Expiring Soon' : 'Active'}
                        </div>
                      </div>

                      <div className="flex items-center gap-2">
                        {permission.isExpiring && (
                          <Button
                            variant="outline"
                                                        onClick={() => {
                              // Extend permission logic
                              console.log('Extend permission:', permission.permission);
                            }}
                          >
                            <Clock className="w-4 h-4 mr-1" />
                            Extend
                          </Button>
                        )}
                        
                        <Button
                          variant="destructive"
                                                    onClick={() => handleRevokePermission(permission.permission)}
                        >
                          <Minus className="w-4 h-4 mr-1" />
                          Revoke
                        </Button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </Card>
        </TabsContent>

        {/* Available Permissions Tab */}
        <TabsContent value="available" className="space-y-4">
          <Card className="p-6">
            <div className="space-y-4">
              <div className="flex justify-between items-center">
                <h3 className="text-lg font-medium text-white">Available Permissions</h3>
                {selectedPermissions.length > 0 && (
                  <Button onClick={handleBulkGrant}>
                    Grant {selectedPermissions.length} Permission(s)
                  </Button>
                )}
              </div>

              {availableToGrant.length === 0 ? (
                <div className="text-center py-8 text-gray-400">
                  <CheckCircle className="w-12 h-12 mx-auto mb-4 opacity-50" />
                  <p>User has all available permissions.</p>
                </div>
              ) : (
                <div className="space-y-2">
                  {availableToGrant.map(permission => (
                    <label
                      key={permission.id}
                      className="flex items-start gap-3 p-4 hover:bg-gray-800/50 rounded-lg cursor-pointer"
                    >
                      <Checkbox
                        checked={selectedPermissions.includes(permission.id)}
                        onCheckedChange={(checked) => {
                          if (checked) {
                            setSelectedPermissions([...selectedPermissions, permission.id]);
                          } else {
                            setSelectedPermissions(selectedPermissions.filter(p => p !== permission.id));
                          }
                        }}
                      />
                      <div className="flex-1">
                        <div className="flex items-center gap-2 mb-2">
                          <Badge variant="secondary">
                            {permission.platform}
                          </Badge>
                          <Badge 
                            variant={permission.risk === 'high' ? 'destructive' : 'outline'}
                          >
                            {permission.risk} risk
                          </Badge>
                          <span className="font-medium text-white">{permission.name}</span>
                        </div>
                        <p className="text-sm text-gray-400">{permission.description}</p>
                        <div className="text-xs text-gray-500 mt-1">
                          Category: {permission.category}
                        </div>
                      </div>
                    </label>
                  ))}
                </div>
              )}
            </div>
          </Card>
        </TabsContent>

        {/* Recommendations Tab */}
        <TabsContent value="recommendations" className="space-y-4">
          <Card className="p-6">
            <h3 className="text-lg font-medium text-white mb-4">AI Recommendations</h3>
            
            {recommendations.length === 0 ? (
              <div className="text-center py-8 text-gray-400">
                <TrendingUp className="w-12 h-12 mx-auto mb-4 opacity-50" />
                <p>No recommendations available at this time.</p>
              </div>
            ) : (
              <div className="space-y-4">
                {recommendations.map((rec, index) => (
                  <div
                    key={index}
                    className="flex items-start gap-4 p-4 bg-gray-800/30 rounded-lg"
                  >
                    <div className="flex-shrink-0 mt-1">
                      {rec.type === 'grant' && <Plus className="w-4 h-4 text-green-500" />}
                      {rec.type === 'revoke' && <Minus className="w-4 h-4 text-red-500" />}
                      {rec.type === 'extend' && <Clock className="w-4 h-4 text-blue-500" />}
                      {rec.type === 'review' && <Eye className="w-4 h-4 text-yellow-500" />}
                    </div>
                    
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-2">
                        <Badge variant="secondary">
                          {rec.type}
                        </Badge>
                        <Badge 
                          variant={rec.impact === 'high' ? 'destructive' : 'outline'} 
                                                  >
                          {rec.impact} impact
                        </Badge>
                        <code className="text-sm bg-gray-800 px-2 py-1 rounded">
                          {rec.permission}
                        </code>
                      </div>
                      
                      <p className="text-sm text-gray-300 mb-2">{rec.reason}</p>
                      
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-2 text-xs text-gray-500">
                          <span>Confidence: {rec.confidence}%</span>
                          <Progress value={rec.confidence} className="w-20 h-1" />
                        </div>
                        
                        <div className="flex gap-2">
                          <Button variant="outline" size="sm">
                            Dismiss
                          </Button>
                          <Button size="sm">
                            Apply
                          </Button>
                        </div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </Card>
        </TabsContent>
      </Tabs>

      {/* Grant Permission Dialog */}
      <Dialog open={showGrantDialog} onOpenChange={setShowGrantDialog}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>Grant Permissions to {user.email}</DialogTitle>
          </DialogHeader>
          
          <div className="space-y-4 max-h-96 overflow-y-auto">
            {availableToGrant.map(permission => (
              <label
                key={permission.id}
                className="flex items-start gap-3 p-3 hover:bg-gray-800/50 rounded cursor-pointer"
              >
                <Checkbox
                  checked={selectedPermissions.includes(permission.id)}
                  onCheckedChange={(checked) => {
                    if (checked) {
                      setSelectedPermissions([...selectedPermissions, permission.id]);
                    } else {
                      setSelectedPermissions(selectedPermissions.filter(p => p !== permission.id));
                    }
                  }}
                />
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-1">
                    <Badge variant="secondary">
                      {permission.platform}
                    </Badge>
                    <span className="font-medium">{permission.name}</span>
                  </div>
                  <p className="text-sm text-gray-400">{permission.description}</p>
                </div>
              </label>
            ))}
          </div>

          <div className="flex justify-end gap-3 pt-4">
            <Button
              variant="outline"
              onClick={() => {
                setShowGrantDialog(false);
                setSelectedPermissions([]);
              }}
            >
              Cancel
            </Button>
            <Button
              onClick={handleBulkGrant}
              disabled={selectedPermissions.length === 0}
            >
              Grant {selectedPermissions.length} Permission(s)
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}

export default UserPermissions;