'use client';

import { useEffect, useState } from 'react';
import { useAuth } from '@/lib/auth';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Separator } from '@/components/ui/separator';
import { 
  Clock, 
  Shield, 
  CheckCircle, 
  XCircle, 
  AlertTriangle, 
  Eye, 
  Settings, 
  Users, 
  BarChart3,
  Zap,
  Crown,
  RefreshCw
} from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';
import { cn } from '@/lib/utils';

// Simple permission parsing function
function parsePermissionWithTimestamp(permission: string): { basePermission: string; timestamp?: number } {
  const parts = permission.split(':');
  if (parts.length >= 4) {
    const lastPart = parts[parts.length - 1];
    const timestamp = parseInt(lastPart, 10);
    if (!isNaN(timestamp)) {
      const basePermission = parts.slice(0, -1).join(':');
      return { basePermission, timestamp };
    }
  }
  return { basePermission: permission };
}

// Platform colors for visual categorization
const PLATFORM_COLORS = {
  'epsx': 'bg-blue-100 text-blue-800 border-blue-200',
  'epsx-pay': 'bg-green-100 text-green-800 border-green-200',
  'epsx-token': 'bg-purple-100 text-purple-800 border-purple-200',
  'admin': 'bg-orange-100 text-orange-800 border-orange-200',
  'default': 'bg-gray-100 text-gray-800 border-gray-200'
};

// Permission icons
const PERMISSION_ICONS = {
  'view': Eye,
  'read': Eye,
  'manage': Settings,
  'admin': Crown,
  'users': Users,
  'analytics': BarChart3,
  'rankings': BarChart3,
  'realtime': Zap,
  'default': Shield
};

function getPermissionIcon(permission: string) {
  const parts = permission.toLowerCase().split(':');
  for (const part of parts) {
    if (PERMISSION_ICONS[part as keyof typeof PERMISSION_ICONS]) {
      return PERMISSION_ICONS[part as keyof typeof PERMISSION_ICONS];
    }
  }
  return PERMISSION_ICONS.default;
}

function getPlatformFromPermission(permission: string): string {
  const parts = permission.split(':');
  return parts[0] || 'default';
}

interface TimestampedPermission {
  permission: string;
  basePermission: string;
  expiresAt?: number;
  isExpired: boolean;
  timeRemaining?: number;
}

function PermissionCard({ permission }: {
  permission: TimestampedPermission;
}) {
  const platform = getPlatformFromPermission(permission.basePermission);
  const Icon = getPermissionIcon(permission.basePermission);
  
  const expiryDate = permission.expiresAt ? new Date(permission.expiresAt * 1000) : null;
  const isExpiringSoon = permission.timeRemaining && permission.timeRemaining < 24 * 60 * 60 * 1000; // 24 hours
  
  return (
    <Card 
      data-testid="permission-card"
      data-permission={permission.basePermission}
      data-platform={platform}
      data-expired={permission.isExpired}
      data-expiring-soon={isExpiringSoon}
      className={cn(
        'transition-all hover:shadow-md',
        permission.isExpired && 'opacity-60 border-red-200 bg-red-50',
        isExpiringSoon && !permission.isExpired && 'border-yellow-200 bg-yellow-50'
      )}>
      <CardContent className="p-4">
        <div className="flex items-start justify-between">
          <div className="flex items-start space-x-3">
            <div className={cn(
              'p-2 rounded-lg',
              permission.isExpired ? 'bg-red-100' : isExpiringSoon ? 'bg-yellow-100' : 'bg-blue-100'
            )}>
              <Icon className={cn(
                'h-4 w-4',
                permission.isExpired ? 'text-red-600' : isExpiringSoon ? 'text-yellow-600' : 'text-blue-600'
              )} />
            </div>
            
            <div className="flex-1 min-w-0">
              <div className="flex items-center space-x-2 mb-1">
                <Badge 
                  variant="outline" 
                  className={cn(
                    'text-xs',
                    PLATFORM_COLORS[platform as keyof typeof PLATFORM_COLORS] || PLATFORM_COLORS.default
                  )}
                >
                  {platform}
                </Badge>
                
                {permission.isExpired && (
                  <Badge variant="destructive" className="text-xs">
                    <XCircle className="h-3 w-3 mr-1" />
                    Expired
                  </Badge>
                )}
                
                {isExpiringSoon && !permission.isExpired && (
                  <Badge variant="outline" className="text-xs border-yellow-500 text-yellow-700">
                    <AlertTriangle className="h-3 w-3 mr-1" />
                    Expiring Soon
                  </Badge>
                )}
              </div>
              
              <h3 className="font-medium text-sm text-gray-900 mb-1">
                {permission.basePermission}
              </h3>
              
              {expiryDate && (
                <div className="text-xs text-gray-500 space-y-1">
                  <div className="flex items-center space-x-1">
                    <Clock className="h-3 w-3" />
                    <span>
                      {permission.isExpired ? 'Expired' : 'Expires'} {formatDistanceToNow(expiryDate, { addSuffix: true })}
                    </span>
                  </div>
                  <div className="text-gray-400">
                    {expiryDate.toLocaleString()}
                  </div>
                </div>
              )}
            </div>
          </div>
          
          <div className="flex items-center">
            {permission.isExpired ? (
              <XCircle className="h-5 w-5 text-red-500" />
            ) : (
              <CheckCircle className="h-5 w-5 text-green-500" />
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

function PermissionStats({ permissions }: { permissions: TimestampedPermission[] }) {
  const totalPermissions = permissions.length;
  const expiredPermissions = permissions.filter(p => p.isExpired).length;
  const expiringSoon = permissions.filter(p => 
    !p.isExpired && p.timeRemaining && p.timeRemaining < 24 * 60 * 60 * 1000
  ).length;
  const activePermissions = totalPermissions - expiredPermissions;
  
  return (
    <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
      <Card>
        <CardContent className="p-4">
          <div className="flex items-center space-x-2">
            <Shield className="h-5 w-5 text-blue-500" />
            <div>
              <p className="text-2xl font-bold text-blue-600">{totalPermissions}</p>
              <p className="text-sm text-gray-600">Total Permissions</p>
            </div>
          </div>
        </CardContent>
      </Card>
      
      <Card>
        <CardContent className="p-4">
          <div className="flex items-center space-x-2">
            <CheckCircle className="h-5 w-5 text-green-500" />
            <div>
              <p className="text-2xl font-bold text-green-600">{activePermissions}</p>
              <p className="text-sm text-gray-600">Active</p>
            </div>
          </div>
        </CardContent>
      </Card>
      
      <Card>
        <CardContent className="p-4">
          <div className="flex items-center space-x-2">
            <AlertTriangle className="h-5 w-5 text-yellow-500" />
            <div>
              <p className="text-2xl font-bold text-yellow-600">{expiringSoon}</p>
              <p className="text-sm text-gray-600">Expiring Soon</p>
            </div>
          </div>
        </CardContent>
      </Card>
      
      <Card>
        <CardContent className="p-4">
          <div className="flex items-center space-x-2">
            <XCircle className="h-5 w-5 text-red-500" />
            <div>
              <p className="text-2xl font-bold text-red-600">{expiredPermissions}</p>
              <p className="text-sm text-gray-600">Expired</p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

export default function PermissionsPage() {
  const { user, isLoading } = useAuth();
  const [activeTab, setActiveTab] = useState<'all' | 'active' | 'expiring' | 'expired'>('active');
  const [timestampedPermissions, setTimestampedPermissions] = useState<TimestampedPermission[]>([]);

  // Parse all permissions with timestamp information
  useEffect(() => {
    if (!user?.permissions) {
      setTimestampedPermissions([]);
      return;
    }
    
    const parsed = user.permissions.map(perm => {
      const { basePermission, timestamp } = parsePermissionWithTimestamp(perm);
      const expiresAt = timestamp;
      const isExpired = expiresAt ? Date.now() / 1000 > expiresAt : false;
      const timeRemaining = expiresAt ? (expiresAt * 1000) - Date.now() : undefined;
      
      return {
        permission: perm,
        basePermission,
        expiresAt,
        isExpired,
        timeRemaining
      };
    });
    
    setTimestampedPermissions(parsed);
  }, [user?.permissions]);

  // Filter permissions based on active tab
  const filteredPermissions = timestampedPermissions.filter(perm => {
    switch (activeTab) {
      case 'active':
        return !perm.isExpired;
      case 'expiring':
        return !perm.isExpired && perm.timeRemaining && perm.timeRemaining < 24 * 60 * 60 * 1000;
      case 'expired':
        return perm.isExpired;
      default:
        return true;
    }
  });

  if (isLoading) {
    return (
      <div className="container mx-auto p-6">
        <div className="flex items-center justify-center h-64">
          <RefreshCw className="h-8 w-8 animate-spin text-gray-400" />
        </div>
      </div>
    );
  }

  if (!user) {
    return (
      <div className="container mx-auto p-6">
        <Card>
          <CardContent className="p-6">
            <div className="text-center">
              <Shield className="h-12 w-12 text-gray-400 mx-auto mb-4" />
              <h2 className="text-xl font-semibold text-gray-900 mb-2">Authentication Required</h2>
              <p className="text-gray-600 mb-4">Please sign in to view your permissions.</p>
              <Button onClick={() => window.location.href = '/api/auth/signin'}>
                Sign In
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="container mx-auto p-6 max-w-6xl">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-3xl font-bold text-gray-900">My Permissions</h1>
          <p className="text-gray-600 mt-1">
            View and manage your platform permissions and access levels
          </p>
        </div>
        
        <Button 
          data-testid="refresh-permissions-button"
          onClick={() => window.location.reload()} 
          variant="outline"
        >
          <RefreshCw className="h-4 w-4 mr-2" />
          Refresh
        </Button>
      </div>

      {/* Stats */}
      <PermissionStats permissions={timestampedPermissions} />

      {/* User Info */}
      <Card className="mb-6">
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Users className="h-5 w-5" />
            <span>Account Information</span>
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <p className="text-sm font-medium text-gray-500">Email</p>
              <p className="text-sm text-gray-900">{user.email}</p>
            </div>
            <div>
              <p className="text-sm font-medium text-gray-500">Role</p>
              <Badge variant="outline" className="mt-1">
                {user.role || 'User'}
              </Badge>
            </div>
            <div>
              <p className="text-sm font-medium text-gray-500">Permissions Count</p>
              <p className="text-sm text-gray-900">{user.permissions?.length || 0} permissions</p>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Permissions Content */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Shield className="h-5 w-5" />
            <span>Permission Details</span>
          </CardTitle>
          <CardDescription>
            All permissions including embedded timestamp permissions with expiry information
          </CardDescription>
          
          {/* Tab Navigation */}
          <div className="flex space-x-2 pt-4">
            {[
              { key: 'active', label: 'Active', count: timestampedPermissions.filter(p => !p.isExpired).length },
              { key: 'expiring', label: 'Expiring Soon', count: timestampedPermissions.filter(p => !p.isExpired && p.timeRemaining && p.timeRemaining < 24 * 60 * 60 * 1000).length },
              { key: 'expired', label: 'Expired', count: timestampedPermissions.filter(p => p.isExpired).length },
              { key: 'all', label: 'All', count: timestampedPermissions.length }
            ].map(tab => (
              <Button
                key={tab.key}
                data-testid={`permission-tab-${tab.key}`}
                variant={activeTab === tab.key ? 'default' : 'outline'}
                size="sm"
                onClick={() => setActiveTab(tab.key as typeof activeTab)}
                className="relative"
              >
                {tab.label}
                {tab.count > 0 && (
                  <Badge 
                    variant="secondary" 
                    className="ml-2 text-xs"
                  >
                    {tab.count}
                  </Badge>
                )}
              </Button>
            ))}
          </div>
        </CardHeader>
        
        <CardContent>
          {filteredPermissions.length === 0 ? (
            <div className="text-center py-12">
              <Shield className="h-12 w-12 text-gray-400 mx-auto mb-4" />
              <h3 className="text-lg font-medium text-gray-900 mb-2">
                No {activeTab !== 'all' ? activeTab : ''} permissions found
              </h3>
              <p className="text-gray-500">
                {activeTab === 'active' ? 'You have no active permissions at the moment.' :
                 activeTab === 'expiring' ? 'No permissions are expiring soon.' :
                 activeTab === 'expired' ? 'No permissions have expired.' :
                 'You have no permissions assigned to your account.'}
              </p>
            </div>
          ) : (
            <div className="space-y-3">
              {filteredPermissions.map((perm, index) => (
                <PermissionCard
                  key={`${perm.permission}-${index}`}
                  permission={perm}
                />
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}