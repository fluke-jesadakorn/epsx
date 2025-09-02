'use client';

import React, { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { 
  Shield, 
  ShieldCheck, 
  ShieldAlert, 
  Eye, 
  Lock,
  CheckCircle,
  XCircle,
  Clock,
  RefreshCw
} from 'lucide-react';
import { useGranularPermissions } from '@/hooks/useGranularPermissions';
import GranularPermissionGuard, { 
  RequireGranularPermission,
  RequireAnyGranularPermission,
  RequireValidForDuration 
} from '@/components/guards/GranularPermissionGuard';
import PermissionStatusCard from '@/components/permissions/PermissionStatusCard';
import PermissionExpiryIndicator from '@/components/permissions/PermissionExpiryIndicator';

export function GranularPermissionExample() {
  const { 
    hasPermission, 
    hasAnyPermission,
    getPermissionExpiry,
    isPermissionExpiring,
    refreshPermissions,
    loading,
    error
  } = useGranularPermissions();

  const [selectedFeature, setSelectedFeature] = useState<string>('');

  // Example permissions to test
  const testPermissions = [
    'epsx:analytics:view',
    'epsx:analytics:export', 
    'epsx:realtime:access',
    'epsx:profile:manage',
    'epsx:billing:manage',
    'admin:users:read',
    'admin:users:manage',
    'epsx:rankings:view:1'
  ];

  const features = [
    {
      name: 'Analytics Dashboard',
      permission: 'epsx:analytics:view',
      description: 'View analytics data and charts'
    },
    {
      name: 'Data Export',
      permission: 'epsx:analytics:export',
      description: 'Export analytics data to CSV/Excel'
    },
    {
      name: 'Real-time Updates',
      permission: 'epsx:realtime:access',
      description: 'Receive live data updates'
    },
    {
      name: 'Profile Management',
      permission: 'epsx:profile:manage',
      description: 'Edit profile settings and preferences'
    },
    {
      name: 'Billing Access',
      permission: 'epsx:billing:manage',
      description: 'View and manage billing information'
    }
  ];

  const getPermissionStatus = (permission: string) => {
    if (loading) return 'loading';
    if (!hasPermission(permission)) return 'denied';
    
    const expiry = getPermissionExpiry(permission);
    if (!expiry) return 'unknown';
    if (expiry.is_expired) return 'expired';
    if (isPermissionExpiring(permission, 24)) return 'expiring';
    return 'active';
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'loading': return <RefreshCw className="h-4 w-4 animate-spin text-blue-500" />;
      case 'active': return <ShieldCheck className="h-4 w-4 text-green-500" />;
      case 'expiring': return <ShieldAlert className="h-4 w-4 text-yellow-500" />;
      case 'expired': return <Shield className="h-4 w-4 text-red-500" />;
      case 'denied': return <XCircle className="h-4 w-4 text-gray-400" />;
      default: return <Shield className="h-4 w-4 text-gray-400" />;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active': return 'text-green-600';
      case 'expiring': return 'text-yellow-600';
      case 'expired': return 'text-red-600';
      case 'denied': return 'text-gray-500';
      default: return 'text-gray-500';
    }
  };

  return (
    <div className="space-y-6 p-6">
      {/* Header */}
      <div>
        <h2 className="text-2xl font-bold flex items-center gap-2">
          <Shield className="h-6 w-6" />
          Granular Permissions Demo
        </h2>
        <p className="text-muted-foreground mt-2">
          Interactive demonstration of the granular permission system integration
        </p>
      </div>

      {/* Error Display */}
      {error && (
        <Alert variant="destructive">
          <XCircle className="h-4 w-4" />
          <AlertDescription>
            Permission Error: {error.message}
          </AlertDescription>
        </Alert>
      )}

      {/* Permission Status Card */}
      <PermissionStatusCard 
        showDetails={true}
        showExpiry={true}
        className="max-w-4xl"
      />

      {/* Feature Access Testing */}
      <Card>
        <CardHeader>
          <CardTitle>Feature Access Testing</CardTitle>
          <CardDescription>
            Test access to different features based on your current permissions
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {features.map((feature) => {
              const status = getPermissionStatus(feature.permission);
              return (
                <div key={feature.name} className="border rounded-lg p-4">
                  <div className="flex items-center justify-between mb-2">
                    <h4 className="font-medium">{feature.name}</h4>
                    <div className="flex items-center gap-1">
                      {getStatusIcon(status)}
                      <PermissionExpiryIndicator 
                        permission={feature.permission}
                        variant="badge"
                        className="ml-2"
                      />
                    </div>
                  </div>
                  <p className="text-sm text-muted-foreground mb-3">
                    {feature.description}
                  </p>
                  <div className="space-y-2">
                    <div className="text-xs">
                      <span className="font-medium">Permission: </span>
                      <code className="bg-muted px-1 rounded">{feature.permission}</code>
                    </div>
                    <div className="text-xs">
                      <span className="font-medium">Status: </span>
                      <span className={getStatusColor(status)}>{status}</span>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </CardContent>
      </Card>

      {/* Permission Guards Demonstration */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Basic Permission Guard */}
        <Card>
          <CardHeader>
            <CardTitle>Basic Permission Guard</CardTitle>
            <CardDescription>
              Content protected by a single permission
            </CardDescription>
          </CardHeader>
          <CardContent>
            <RequireGranularPermission 
              permission="epsx:analytics:view"
              fallback={
                <div className="p-4 bg-gray-50 rounded border text-center">
                  <Lock className="h-8 w-8 text-gray-400 mx-auto mb-2" />
                  <p className="text-gray-600">Analytics access required</p>
                </div>
              }
            >
              <div className="p-4 bg-green-50 rounded border">
                <CheckCircle className="h-5 w-5 text-green-600 inline mr-2" />
                <strong className="text-green-800">Analytics Access Granted!</strong>
                <p className="text-green-700 text-sm mt-1">
                  You can view analytics data and dashboards.
                </p>
              </div>
            </RequireGranularPermission>
          </CardContent>
        </Card>

        {/* Multi-Permission Guard */}
        <Card>
          <CardHeader>
            <CardTitle>Multi-Permission Guard</CardTitle>
            <CardDescription>
              Content requiring any of multiple permissions
            </CardDescription>
          </CardHeader>
          <CardContent>
            <RequireAnyGranularPermission 
              permissions={['epsx:analytics:export', 'admin:users:manage']}
              fallback={
                <div className="p-4 bg-gray-50 rounded border text-center">
                  <Lock className="h-8 w-8 text-gray-400 mx-auto mb-2" />
                  <p className="text-gray-600">Export or admin access required</p>
                </div>
              }
            >
              <div className="p-4 bg-blue-50 rounded border">
                <CheckCircle className="h-5 w-5 text-blue-600 inline mr-2" />
                <strong className="text-blue-800">Advanced Access Granted!</strong>
                <p className="text-blue-700 text-sm mt-1">
                  You have either export or admin permissions.
                </p>
              </div>
            </RequireAnyGranularPermission>
          </CardContent>
        </Card>

        {/* Time-Limited Permission Guard */}
        <Card>
          <CardHeader>
            <CardTitle>Time-Limited Guard</CardTitle>
            <CardDescription>
              Content requiring valid permission for 1 hour
            </CardDescription>
          </CardHeader>
          <CardContent>
            <RequireValidForDuration 
              permission="epsx:realtime:access"
              hours={1}
              fallback={
                <div className="p-4 bg-gray-50 rounded border text-center">
                  <Clock className="h-8 w-8 text-gray-400 mx-auto mb-2" />
                  <p className="text-gray-600">Valid real-time access required for 1+ hours</p>
                </div>
              }
            >
              <div className="p-4 bg-purple-50 rounded border">
                <CheckCircle className="h-5 w-5 text-purple-600 inline mr-2" />
                <strong className="text-purple-800">Real-time Access Valid!</strong>
                <p className="text-purple-700 text-sm mt-1">
                  Your real-time permission is valid for at least 1 hour.
                </p>
              </div>
            </RequireValidForDuration>
          </CardContent>
        </Card>

        {/* Complex Conditional Rendering */}
        <Card>
          <CardHeader>
            <CardTitle>Complex Conditional</CardTitle>
            <CardDescription>
              Multiple checks with different content
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {hasPermission('admin:users:manage') && (
                <Alert>
                  <Shield className="h-4 w-4" />
                  <AlertDescription>
                    <strong>Admin Access:</strong> You can manage all users
                  </AlertDescription>
                </Alert>
              )}
              
              {hasPermission('epsx:billing:manage') && (
                <Alert>
                  <CheckCircle className="h-4 w-4" />
                  <AlertDescription>
                    <strong>Billing Access:</strong> You can manage billing settings
                  </AlertDescription>
                </Alert>
              )}
              
              {hasAnyPermission(['epsx:analytics:export', 'epsx:realtime:access']) && (
                <Alert>
                  <Eye className="h-4 w-4" />
                  <AlertDescription>
                    <strong>Premium Features:</strong> You have access to advanced features
                  </AlertDescription>
                </Alert>
              )}
              
              {!hasAnyPermission(['admin:users:manage', 'epsx:billing:manage', 'epsx:analytics:export']) && (
                <Alert variant="destructive">
                  <Lock className="h-4 w-4" />
                  <AlertDescription>
                    <strong>Limited Access:</strong> Consider upgrading for more features
                  </AlertDescription>
                </Alert>
              )}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Permission Testing Section */}
      <Card>
        <CardHeader>
          <CardTitle>Permission Testing</CardTitle>
          <CardDescription>
            Test individual permissions and see detailed status
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {testPermissions.map((permission) => {
              const hasAccess = hasPermission(permission);
              const expiry = getPermissionExpiry(permission);
              
              return (
                <div key={permission} className="flex items-center justify-between p-3 border rounded">
                  <div className="flex-1">
                    <div className="flex items-center gap-2">
                      {hasAccess ? (
                        <ShieldCheck className="h-4 w-4 text-green-500" />
                      ) : (
                        <XCircle className="h-4 w-4 text-gray-400" />
                      )}
                      <code className="text-sm font-mono">{permission}</code>
                    </div>
                    <div className="text-xs text-muted-foreground mt-1">
                      Status: {hasAccess ? 'Granted' : 'Denied'}
                      {expiry && expiry.expires_at && (
                        <span className="ml-2">
                          • Expires: {expiry.expires_in_human || 'Unknown'}
                        </span>
                      )}
                      {expiry && !expiry.expires_at && (
                        <span className="ml-2">• Permanent</span>
                      )}
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    {hasAccess && (
                      <PermissionExpiryIndicator 
                        permission={permission}
                        variant="badge"
                        showCountdown={true}
                      />
                    )}
                    <Badge variant={hasAccess ? 'default' : 'outline'}>
                      {hasAccess ? 'Active' : 'Inactive'}
                    </Badge>
                  </div>
                </div>
              );
            })}
          </div>
        </CardContent>
      </Card>

      {/* Actions */}
      <div className="flex justify-center">
        <Button 
          onClick={refreshPermissions}
          disabled={loading}
          className="flex items-center gap-2"
        >
          <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
          Refresh Permissions
        </Button>
      </div>
    </div>
  );
}