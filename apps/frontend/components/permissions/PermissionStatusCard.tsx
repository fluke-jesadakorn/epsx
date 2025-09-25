'use client';

import React from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { 
  Shield, 
  ShieldCheck, 
  ShieldAlert, 
  Clock, 
  TrendingUp, 
  AlertTriangle,
  RefreshCw,
  Eye,
  EyeOff
} from 'lucide-react';
import { useGranularPermissions } from '@/hooks/useGranularPermissions';
import { useState } from 'react';
import { formatDistanceToNow } from 'date-fns';

interface PermissionStatusCardProps {
  userId?: string;
  showDetails?: boolean;
  showExpiry?: boolean;
  className?: string;
}

interface PermissionDisplayItem {
  permission: string;
  platform: string;
  resource: string;
  action: string;
  source: string;
  granted_at: number;
  expires_at?: number;
  is_expired: boolean;
  expires_in_human?: string;
  health_status: 'healthy' | 'expiring' | 'expired';
}

export function PermissionStatusCard({ 
  userId,
  showDetails = false,
  showExpiry = true,
  className = ''
}: PermissionStatusCardProps) {
  const { 
    refreshPermissions,
    loading, 
    error 
  } = useGranularPermissions();

  const [isRefreshing, setIsRefreshing] = useState(false);
  const [showAllPermissions, setShowAllPermissions] = useState(false);

  // Mock data - in real implementation, this would come from the hook
  const mockPermissionHealth = {
    total_permissions: 8,
    active_permissions: 6,
    expired_permissions: 1,
    expiring_soon_permissions: 1,
    health_score: 75,
    next_expiry: Math.floor(Date.now() / 1000) + (2 * 24 * 60 * 60), // 2 days from now
    time_until_next_expiry: 2 * 24 * 60 * 60 * 1000 // 2 days in milliseconds
  };

  const mockPermissions: PermissionDisplayItem[] = [
    {
      permission: 'epsx:analytics:view',
      platform: 'epsx',
      resource: 'analytics',
      action: 'view',
      source: 'Subscription',
      granted_at: Math.floor(Date.now() / 1000) - (30 * 24 * 60 * 60),
      is_expired: false,
      health_status: 'healthy'
    },
    {
      permission: 'epsx:analytics:export',
      platform: 'epsx',
      resource: 'analytics',
      action: 'export',
      source: 'Subscription',
      granted_at: Math.floor(Date.now() / 1000) - (30 * 24 * 60 * 60),
      expires_at: Math.floor(Date.now() / 1000) + (2 * 24 * 60 * 60),
      is_expired: false,
      expires_in_human: '2 days',
      health_status: 'expiring'
    },
    {
      permission: 'epsx:realtime:access',
      platform: 'epsx',
      resource: 'realtime',
      action: 'access',
      source: 'Trial',
      granted_at: Math.floor(Date.now() / 1000) - (15 * 24 * 60 * 60),
      expires_at: Math.floor(Date.now() / 1000) - (24 * 60 * 60),
      is_expired: true,
      expires_in_human: '1 day ago',
      health_status: 'expired'
    },
    {
      permission: 'epsx:profile:manage',
      platform: 'epsx',
      resource: 'profile',
      action: 'manage',
      source: 'System',
      granted_at: Math.floor(Date.now() / 1000) - (60 * 24 * 60 * 60),
      is_expired: false,
      health_status: 'healthy'
    }
  ];

  const handleRefresh = async () => {
    setIsRefreshing(true);
    try {
      await refreshPermissions();
    } catch (err) {
      console.error('Failed to refresh permissions:', err);
    } finally {
      setIsRefreshing(false);
    }
  };

  const getHealthIcon = (status: 'healthy' | 'expiring' | 'expired') => {
    switch (status) {
      case 'healthy': return <ShieldCheck className="h-4 w-4 text-green-500" />;
      case 'expiring': return <ShieldAlert className="h-4 w-4 text-yellow-500" />;
      case 'expired': return <Shield className="h-4 w-4 text-red-500" />;
    }
  };

  const getHealthBadge = (status: 'healthy' | 'expiring' | 'expired') => {
    switch (status) {
      case 'healthy': return <Badge variant="default" className="bg-green-100 text-green-800">Active</Badge>;
      case 'expiring': return <Badge variant="destructive" className="bg-yellow-100 text-yellow-800">Expiring Soon</Badge>;
      case 'expired': return <Badge variant="destructive">Expired</Badge>;
    }
  };

  const getSourceBadge = (source: string) => {
    const colors: Record<string, string> = {
      Subscription: 'bg-green-100 text-green-800',
      Trial: 'bg-purple-100 text-purple-800',
      Admin: 'bg-blue-100 text-blue-800',
      System: 'bg-indigo-100 text-indigo-800',
      Legacy: 'bg-gray-100 text-gray-800'
    };
    
    return (
      <Badge variant="outline" className={colors[source] || 'bg-gray-100 text-gray-800'}>
        {source}
      </Badge>
    );
  };

  const getHealthColor = (score: number): string => {
    if (score >= 80) return 'text-green-600';
    if (score >= 60) return 'text-yellow-600';
    if (score >= 40) return 'text-orange-600';
    return 'text-red-600';
  };

  const getHealthProgressColor = (score: number): string => {
    if (score >= 80) return 'bg-green-500';
    if (score >= 60) return 'bg-yellow-500';
    if (score >= 40) return 'bg-orange-500';
    return 'bg-red-500';
  };

  const activePermissions = mockPermissions.filter(p => !p.is_expired);
  const expiringPermissions = mockPermissions.filter(p => p.health_status === 'expiring');
  const expiredPermissions = mockPermissions.filter(p => p.is_expired);

  if (loading && !mockPermissionHealth) {
    return (
      <Card className={className}>
        <CardContent className="flex items-center justify-center p-6">
          <div className="text-center">
            <RefreshCw className="h-8 w-8 animate-spin mx-auto mb-2" />
            <p>Loading permission status...</p>
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
              Failed to load permission status: {error.message}
              <Button 
                variant="outline" 
                size="sm" 
                className="ml-2"
                onClick={handleRefresh}
                disabled={isRefreshing}
              >
                <RefreshCw className={`h-3 w-3 ${isRefreshing ? 'animate-spin' : ''}`} />
                Retry
              </Button>
            </AlertDescription>
          </Alert>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className={`space-y-4 ${className}`}>
      {/* Main Status Card */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <Shield className="h-5 w-5" />
                Permission Status
              </CardTitle>
              <CardDescription>
                Your current access permissions and health
              </CardDescription>
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={handleRefresh}
              disabled={isRefreshing}
            >
              <RefreshCw className={`h-4 w-4 ${isRefreshing ? 'animate-spin' : ''}`} />
              Refresh
            </Button>
          </div>
        </CardHeader>
        
        <CardContent>
          {/* Health Score */}
          <div className="mb-6">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium">Permission Health</span>
              <span className={`text-lg font-bold ${getHealthColor(mockPermissionHealth.health_score)}`}>
                {mockPermissionHealth.health_score}%
              </span>
            </div>
            <Progress 
              value={mockPermissionHealth.health_score} 
              className="h-2"
              style={{
                backgroundColor: '#f0f0f0'
              }}
            />
            <div className="flex justify-between text-xs text-muted-foreground mt-1">
              <span>Poor</span>
              <span>Excellent</span>
            </div>
          </div>

          {/* Metrics Grid */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
            <div className="text-center">
              <div className="text-2xl font-bold text-green-600">
                {mockPermissionHealth.active_permissions}
              </div>
              <div className="text-sm text-muted-foreground">Active</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-yellow-600">
                {mockPermissionHealth.expiring_soon_permissions}
              </div>
              <div className="text-sm text-muted-foreground">Expiring Soon</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-red-600">
                {mockPermissionHealth.expired_permissions}
              </div>
              <div className="text-sm text-muted-foreground">Expired</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-blue-600">
                {mockPermissionHealth.total_permissions}
              </div>
              <div className="text-sm text-muted-foreground">Total</div>
            </div>
          </div>

          {/* Next Expiry Alert */}
          {showExpiry && mockPermissionHealth.next_expiry && (
            <Alert className="mb-4">
              <Clock className="h-4 w-4" />
              <AlertDescription>
                Your next permission expires in{' '}
                <strong>
                  {formatDistanceToNow(new Date(mockPermissionHealth.next_expiry * 1000))}
                </strong>
              </AlertDescription>
            </Alert>
          )}

          {/* Expiring Permissions Warning */}
          {expiringPermissions.length > 0 && (
            <Alert className="mb-4">
              <ShieldAlert className="h-4 w-4 text-yellow-500" />
              <AlertDescription>
                <strong>{expiringPermissions.length} permission(s) expiring soon:</strong>
                <div className="mt-2 space-y-1">
                  {expiringPermissions.map(perm => (
                    <div key={perm.permission} className="text-sm">
                      <code className="bg-muted px-1 rounded">{perm.permission}</code>
                      <span className="ml-2 text-muted-foreground">
                        expires {perm.expires_in_human}
                      </span>
                    </div>
                  ))}
                </div>
              </AlertDescription>
            </Alert>
          )}

          {/* Expired Permissions Alert */}
          {expiredPermissions.length > 0 && (
            <Alert variant="destructive" className="mb-4">
              <Shield className="h-4 w-4" />
              <AlertDescription>
                <strong>{expiredPermissions.length} permission(s) have expired:</strong>
                <div className="mt-2 space-y-1">
                  {expiredPermissions.map(perm => (
                    <div key={perm.permission} className="text-sm">
                      <code className="bg-muted px-1 rounded">{perm.permission}</code>
                      <span className="ml-2 text-muted-foreground">
                        expired {perm.expires_in_human}
                      </span>
                    </div>
                  ))}
                </div>
              </AlertDescription>
            </Alert>
          )}
        </CardContent>
      </Card>

      {/* Detailed Permissions */}
      {showDetails && (
        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <CardTitle className="text-lg">Permission Details</CardTitle>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setShowAllPermissions(!showAllPermissions)}
              >
                {showAllPermissions ? (
                  <EyeOff className="h-4 w-4" />
                ) : (
                  <Eye className="h-4 w-4" />
                )}
                {showAllPermissions ? 'Hide' : 'Show'} All
              </Button>
            </div>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {(showAllPermissions ? mockPermissions : activePermissions).map((perm) => (
                <div key={perm.permission} className="flex items-center justify-between p-3 border rounded-lg">
                  <div className="flex-1">
                    <div className="flex items-center gap-2 mb-1">
                      {getHealthIcon(perm.health_status)}
                      <code className="text-sm font-mono">{perm.permission}</code>
                      {getHealthBadge(perm.health_status)}
                    </div>
                    <div className="flex items-center gap-2 text-xs text-muted-foreground">
                      {getSourceBadge(perm.source)}
                      <span>•</span>
                      <span>Granted {formatDistanceToNow(new Date(perm.granted_at * 1000), { addSuffix: true })}</span>
                      {perm.expires_at && (
                        <>
                          <span>•</span>
                          <span className={perm.is_expired ? 'text-red-600' : perm.health_status === 'expiring' ? 'text-yellow-600' : ''}>
                            {perm.is_expired ? `Expired ${perm.expires_in_human}` : `Expires ${perm.expires_in_human}`}
                          </span>
                        </>
                      )}
                      {!perm.expires_at && (
                        <>
                          <span>•</span>
                          <span className="text-green-600">Permanent</span>
                        </>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
            
            {!showAllPermissions && mockPermissions.length > activePermissions.length && (
              <div className="text-center mt-4">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setShowAllPermissions(true)}
                >
                  Show {mockPermissions.length - activePermissions.length} more permissions
                </Button>
              </div>
            )}
          </CardContent>
        </Card>
      )}
    </div>
  );
}

export default PermissionStatusCard;