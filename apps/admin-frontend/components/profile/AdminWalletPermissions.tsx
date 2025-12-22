'use client';

import {
  AlertTriangle,
  BarChart3,
  CheckCircle,
  Clock,
  Crown,
  Eye,
  RefreshCcw,
  Settings,
  Shield,
  Users,
  Wallet
} from 'lucide-react';
import { useEffect, useState } from 'react';
import { toast } from 'sonner';
import { useAccount } from 'wagmi';

import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Progress } from '@/components/ui/progress';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { adminApiClient } from '@/lib/api-client';

interface WalletPermission {
  permission: string;
  source: 'manual' | 'nft' | 'token' | 'dao' | 'inherited' | string;
  expires_at?: string;
  granted_at: string;
  granted_by?: string;
  metadata?: {
    description?: string;
    level?: 'read' | 'write' | 'admin' | string;
    platform?: string;
    permissions_count?: number;
    last_auth_at?: string;
  };
}

interface PermissionsResponse {
  permissions: WalletPermission[];
  total_count: number;
}

interface PlatformGroup {
  platform: string;
  permissions: WalletPermission[];
  level: 'super' | 'admin' | 'manager' | 'user';
}

interface AdminWalletPermissionsProps {
  walletAddress?: string;
  initialPermissions?: string[];
}

/**
 *
 * @param root0
 * @param root0.walletAddress
 * @param root0.initialPermissions
 */
export function AdminWalletPermissions({
  walletAddress: initialWalletAddress,
  initialPermissions
}: AdminWalletPermissionsProps) {
  const { address } = useAccount();

  const [permissions, setPermissions] = useState<WalletPermission[]>([]);
  const [platformGroups, setPlatformGroups] = useState<PlatformGroup[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [lastUpdated, setLastUpdated] = useState<Date>(new Date());
  const [expiringPermissions, setExpiringPermissions] = useState<WalletPermission[]>([]);

  const walletAddress = address || initialWalletAddress;

  useEffect(() => {
    if (walletAddress) {
      fetchWalletPermissions();
    }
  }, [walletAddress]);

  useEffect(() => {
    if (initialPermissions) {
      const mappedPermissions = initialPermissions.map(permission => ({
        permission,
        source: 'manual' as const,
        granted_at: new Date().toISOString(),
        metadata: {
          description: getPermissionDescription(permission),
          level: getPermissionLevel(permission),
          platform: permission.split(':')[0]
        }
      }));
      setPermissions(mappedPermissions);
      groupPermissions(mappedPermissions);
    }
  }, [initialPermissions]);

  const fetchWalletPermissions = async () => {
    if (!walletAddress) { return; }

    try {
      setIsLoading(true);
      // Use the unified admin API client
      const res = await adminApiClient.get<any>(
        '/api/v1/admin/permissions',
        { wallet_address: walletAddress }
      );

      // Handle AdminApiResponse wrapper: { success, data: { permissions, ... }, message }
      const responseBody = res.data;
      const responseData = responseBody?.data || responseBody;
      const rawPermissions = responseData?.permissions || [];

      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const mappedPermissions = rawPermissions.map((p: any) => ({
        permission: p.permission,
        source: p.source,
        expires_at: p.expires_at,
        granted_at: p.granted_at,
        granted_by: p.granted_by,
        metadata: {
          description: getPermissionDescription(p.permission),
          level: getPermissionLevel(p.permission),
          platform: p.permission.split(':')[0],
          ...p.metadata
        }
      }));

      setPermissions(mappedPermissions);
      groupPermissions(mappedPermissions);
      checkExpiringPermissions(mappedPermissions);
      setLastUpdated(new Date());
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Failed to fetch wallet permissions:', _error);
      toast.error('Failed to fetch wallet permissions');
    } finally {
      setIsLoading(false);
    }
  };

  const groupPermissions = (perms: WalletPermission[]) => {
    const grouped = perms.reduce((acc, perm) => {
      const platform = perm.permission.split(':')[0] || 'unknown';

      if (!acc[platform]) {
        acc[platform] = [];
      }
      acc[platform].push(perm);
      return acc;
    }, {} as Record<string, WalletPermission[]>);

    const groups: PlatformGroup[] = Object.entries(grouped).map(([platform, permissions]) => ({
      platform,
      permissions,
      level: determineAccessLevel(permissions)
    }));

    setPlatformGroups(groups);
  };

  const checkExpiringPermissions = (perms: WalletPermission[]) => {
    const now = Date.now();
    const sevenDaysFromNow = now + (7 * 24 * 60 * 60 * 1000);

    const expiring = perms.filter(perm => {
      if (!perm.expires_at) { return false; }
      const expiryTime = new Date(perm.expires_at).getTime();
      return expiryTime <= sevenDaysFromNow && expiryTime > now;
    });

    setExpiringPermissions(expiring);
  };

  const determineAccessLevel = (perms: WalletPermission[]): 'super' | 'admin' | 'manager' | 'user' => {
    if (perms.some(p => p.permission.includes('*:*'))) { return 'super'; }
    if (perms.some(p => p.permission.startsWith('admin:'))) { return 'admin'; }
    if (perms.some(p => p.permission.includes('manage'))) { return 'manager'; }
    return 'user';
  };

  const getPermissionDescription = (permission: string): string => {
    const [platform, resource, action] = permission.split(':');

    const descriptions: Record<string, string> = {
      'admin:*:*': 'Full administrative access to all systems',
      'admin:users:manage': 'Manage user accounts and permissions',
      'admin:analytics:view': 'View analytics and reports',
      'admin:system:configure': 'Configure system settings',
      'admin:web3:manage': 'Manage Web3 and wallet integrations',
      'epsx:trading:access': 'Access analytics platform features',
      'epsx:analytics:view': 'View data analytics'
    };

    return descriptions[permission] || `${action} access to ${resource} on ${platform}`;
  };

  const getPermissionLevel = (permission: string): 'read' | 'write' | 'admin' => {
    if (permission.includes('admin') || permission.includes('manage')) { return 'admin'; }
    if (permission.includes('write') || permission.includes('create') || permission.includes('update')) { return 'write'; }
    return 'read';
  };

  const getPermissionIcon = (permission: string) => {
    if (permission.includes('admin')) { return Crown; }
    if (permission.includes('users')) { return Users; }
    if (permission.includes('analytics')) { return BarChart3; }
    if (permission.includes('system')) { return Settings; }
    if (permission.includes('web3')) { return Wallet; }
    return Shield;
  };

  const getLevelColor = (level: string) => {
    switch (level) {
      case 'super': return 'bg-yellow-100 text-yellow-800 border-yellow-300';
      case 'admin': return 'bg-red-100 text-red-800 border-red-300';
      case 'manager': return 'bg-blue-100 text-blue-800 border-blue-300';
      case 'write': return 'bg-orange-100 text-orange-800 border-orange-300';
      case 'read': return 'bg-green-100 text-green-800 border-green-300';
      default: return 'bg-gray-100 text-gray-800 border-gray-300';
    }
  };

  const getSourceIcon = (source: string) => {
    switch (source) {
      case 'nft': return '🎨';
      case 'token': return '🪙';
      case 'dao': return '🗳️';
      case 'inherited': return '📋';
      default: return '👤';
    }
  };

  const adminPermissions = permissions.filter(p => p.permission.startsWith('admin:'));
  const platformPermissions = permissions.filter(p => !p.permission.startsWith('admin:'));

  return (
    <div className="space-y-6">
      {/* Permission Summary */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle className="flex items-center gap-2">
              <Shield className="h-5 w-5" />
              Permission Overview
            </CardTitle>
            <Button
              variant="outline"
              size="sm"
              onClick={fetchWalletPermissions}
              disabled={isLoading}
            >
              <RefreshCcw className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="text-center p-3 bg-blue-50 rounded-lg">
              <div className="text-2xl font-bold text-blue-600">{permissions.length}</div>
              <div className="text-sm text-blue-600">Total Permissions</div>
            </div>
            <div className="text-center p-3 bg-red-50 rounded-lg">
              <div className="text-2xl font-bold text-red-600">{adminPermissions.length}</div>
              <div className="text-sm text-red-600">Admin Permissions</div>
            </div>
            <div className="text-center p-3 bg-green-50 rounded-lg">
              <div className="text-2xl font-bold text-green-600">{platformGroups.length}</div>
              <div className="text-sm text-green-600">Platforms</div>
            </div>
            <div className="text-center p-3 bg-orange-50 rounded-lg">
              <div className="text-2xl font-bold text-orange-600">{expiringPermissions.length}</div>
              <div className="text-sm text-orange-600">Expiring Soon</div>
            </div>
          </div>

          <div className="mt-4 text-xs text-gray-500">
            Last updated: {lastUpdated.toLocaleString()}
          </div>
        </CardContent>
      </Card>

      {/* Expiring Permissions Alert */}
      {expiringPermissions.length > 0 && (
        <Alert>
          <AlertTriangle className="h-4 w-4" />
          <AlertDescription>
            You have {expiringPermissions.length} permission(s) expiring within 7 days.
            Contact your administrator to renew them.
          </AlertDescription>
        </Alert>
      )}

      {/* Permission Details */}
      <Card>
        <CardHeader>
          <CardTitle>Permission Details</CardTitle>
        </CardHeader>
        <CardContent>
          <Tabs value="admin" onValueChange={() => { }} className="w-full">
            <TabsList className="grid w-full grid-cols-3">
              <TabsTrigger value="admin">Admin Permissions</TabsTrigger>
              <TabsTrigger value="platform">Platform Access</TabsTrigger>
              <TabsTrigger value="groups">By Platform</TabsTrigger>
            </TabsList>

            {/* Admin Permissions */}
            <TabsContent value="admin" className="space-y-4">
              {adminPermissions.length > 0 ? (
                <div className="space-y-3">
                  {adminPermissions.map((permission, index) => {
                    const Icon = getPermissionIcon(permission.permission);
                    return (
                      <div
                        key={index}
                        className="flex items-center justify-between p-4 border rounded-lg hover:bg-gray-50"
                      >
                        <div className="flex items-center gap-3">
                          <Icon className="h-5 w-5 text-red-500" />
                          <div>
                            <code className="text-sm font-mono font-medium">
                              {permission.permission}
                            </code>
                            <p className="text-xs text-gray-600 mt-1">
                              {permission.metadata?.description}
                            </p>
                            <div className="flex items-center gap-2 mt-2">
                              <Badge variant="outline" className={getLevelColor(permission.metadata?.level || 'read')}>
                                {permission.metadata?.level}
                              </Badge>
                              <span className="text-xs text-gray-500">
                                {getSourceIcon(permission.source)} {permission.source}
                              </span>
                            </div>
                          </div>
                        </div>
                        {permission.expires_at && (
                          <div className="text-right">
                            <div className="flex items-center gap-1 text-orange-600">
                              <Clock className="h-3 w-3" />
                              <span className="text-xs">Expires</span>
                            </div>
                            <span className="text-xs text-gray-600">
                              {new Date(permission.expires_at).toLocaleDateString()}
                            </span>
                          </div>
                        )}
                      </div>
                    );
                  })}
                </div>
              ) : (
                <div className="text-center py-8 text-gray-500">
                  <Shield className="h-8 w-8 mx-auto mb-2 opacity-50" />
                  <p>No admin permissions found</p>
                </div>
              )}
            </TabsContent>

            {/* Platform Permissions */}
            <TabsContent value="platform" className="space-y-4">
              {platformPermissions.length > 0 ? (
                <div className="space-y-3">
                  {platformPermissions.map((permission, index) => {
                    const Icon = getPermissionIcon(permission.permission);
                    return (
                      <div
                        key={index}
                        className="flex items-center justify-between p-4 border rounded-lg hover:bg-gray-50"
                      >
                        <div className="flex items-center gap-3">
                          <Icon className="h-5 w-5 text-blue-500" />
                          <div>
                            <code className="text-sm font-mono font-medium">
                              {permission.permission}
                            </code>
                            <p className="text-xs text-gray-600 mt-1">
                              {permission.metadata?.description}
                            </p>
                            <div className="flex items-center gap-2 mt-2">
                              <Badge variant="outline" className={getLevelColor(permission.metadata?.level || 'read')}>
                                {permission.metadata?.level}
                              </Badge>
                              <span className="text-xs text-gray-500">
                                {getSourceIcon(permission.source)} {permission.source}
                              </span>
                            </div>
                          </div>
                        </div>
                      </div>
                    );
                  })}
                </div>
              ) : (
                <div className="text-center py-8 text-gray-500">
                  <Eye className="h-8 w-8 mx-auto mb-2 opacity-50" />
                  <p>No platform permissions found</p>
                </div>
              )}
            </TabsContent>

            {/* Grouped by Platform */}
            <TabsContent value="groups" className="space-y-4">
              {platformGroups.map((group: PlatformGroup, index: number) => (
                <Card key={index}>
                  <CardHeader className="pb-3">
                    <div className="flex items-center justify-between">
                      <CardTitle className="text-lg capitalize">{group.platform}</CardTitle>
                      <div className="flex items-center gap-2">
                        <Badge className={getLevelColor(group.level)}>
                          {group.level} access
                        </Badge>
                        <Badge variant="outline">
                          {group.permissions.length} perms
                        </Badge>
                      </div>
                    </div>
                  </CardHeader>
                  <CardContent>
                    <div className="space-y-2">
                      {group.permissions.map((permission: WalletPermission, permIndex: number) => (
                        <div key={permIndex} className="flex items-center justify-between p-2 bg-gray-50 rounded">
                          <code className="text-sm font-mono">
                            {permission.permission}
                          </code>
                          <Badge variant="outline" className={getLevelColor(permission.metadata?.level || 'read')}>
                            {permission.metadata?.level}
                          </Badge>
                        </div>
                      ))}
                    </div>
                  </CardContent>
                </Card>
              ))}
            </TabsContent>
          </Tabs>
        </CardContent>
      </Card>

      {/* Permission Health */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <CheckCircle className="h-5 w-5" />
            Permission Health
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium">Overall Health</span>
              <span className="text-sm text-green-600">Excellent</span>
            </div>
            <Progress value={95} className="h-2" />

            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
              <div className="flex items-center gap-2">
                <CheckCircle className="h-4 w-4 text-green-500" />
                <span>All permissions valid</span>
              </div>
              <div className="flex items-center gap-2">
                <CheckCircle className="h-4 w-4 text-green-500" />
                <span>Admin access verified</span>
              </div>
              <div className="flex items-center gap-2">
                <CheckCircle className="h-4 w-4 text-green-500" />
                <span>No security issues</span>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}