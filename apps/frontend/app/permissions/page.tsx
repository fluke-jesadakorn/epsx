'use client';

import { GlobalAuthGuard } from '@/components/auth/global-auth-guard';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { useAuth } from '@/lib/auth';
import { cn } from '@/lib/utils';
import type {
  PermissionDefinition
} from '@/shared/api/permission-definitions';
import {
  getPermissionNote,
  getPermissionTitle,
  loadPermissionDefinitions
} from '@/shared/api/permission-definitions';
import { useApiClient } from '@/shared/hooks/use-api-client';
import { formatDistanceToNow } from 'date-fns';
import {
  AlertTriangle,
  BarChart3,
  CheckCircle,
  Clock,
  Crown,
  Download,
  Eye,
  History,
  PieChart,
  RefreshCw,
  Settings,
  Shield,
  TrendingUp,
  Users,
  XCircle,
  Zap
} from 'lucide-react';
import { useEffect, useState } from 'react';
// import {
//   getPermissionAnalytics,
//   getPermissionHistory,
//   exportPermissionsData
// } from '@/lib/actions/permissions';

// TODO: Implement these functions when backend is ready
const getPermissionAnalytics = async () => null;
const getPermissionHistory = async (_limit: number) => [];
const exportPermissionsData = async (format: string) => ({
  data: format === 'json' ? '[]' : '',
  filename: `permissions.${format}`
});

// Simple permission parsing function
function parsePermissionWithTimestamp(permission: string): {
  basePermission: string;
  timestamp?: number;
} {
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
  epsx: 'bg-blue-100 text-blue-800 border-blue-200',
  'epsx-pay': 'bg-green-100 text-green-800 border-green-200',
  'epsx-token': 'bg-purple-100 text-purple-800 border-purple-200',
  admin: 'bg-orange-100 text-orange-800 border-orange-200',
  default: 'bg-gray-100 text-gray-800 border-gray-200',
};

// Permission icons
const PERMISSION_ICONS = {
  view: Eye,
  read: Eye,
  manage: Settings,
  admin: Crown,
  users: Users,
  analytics: BarChart3,
  rankings: BarChart3,
  realtime: Zap,
  default: Shield,
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

interface PermissionHistoryItem {
  permission: string;
  action: 'granted' | 'revoked' | 'expired' | 'default';
  timestamp: string;
  reason?: string;
  grantedBy?: string;
}

interface AnalyticsData {
  platformDistribution: Record<string, number>;
  usageStats?: {
    mostUsedPermissions?: { permission: string; usage: number }[];
    recentlyGranted?: { permission: string; grantedAt: string }[];
  };
}

function PermissionCard({ permission, definitions }: { permission: TimestampedPermission; definitions: Map<string, PermissionDefinition> }) {
  const platform = getPlatformFromPermission(permission.basePermission);
  const Icon = getPermissionIcon(permission.basePermission);
  const title = getPermissionTitle(permission.basePermission, definitions);
  const note = getPermissionNote(permission.basePermission, definitions);

  const expiryDate = permission.expiresAt
    ? new Date(permission.expiresAt * 1000)
    : null;
  const isExpiringSoon =
    permission.timeRemaining && permission.timeRemaining < 24 * 60 * 60 * 1000; // 24 hours

  const cardContent = (
    <Card
      data-testid="permission-card"
      data-permission={permission.basePermission}
      data-platform={platform}
      data-expired={permission.isExpired}
      data-expiring-soon={isExpiringSoon}
      className={cn(
        'transition-all hover:shadow-md',
        permission.isExpired && 'border-red-200 bg-red-50 opacity-60',
        isExpiringSoon &&
        !permission.isExpired &&
        'border-yellow-200 bg-yellow-50'
      )}
    >
      <CardContent className="p-4">
        <div className="flex items-start justify-between">
          <div className="flex items-start space-x-3">
            <div
              className={cn(
                'rounded-lg p-2',
                permission.isExpired
                  ? 'bg-red-100'
                  : isExpiringSoon
                    ? 'bg-yellow-100'
                    : 'bg-blue-100'
              )}
            >
              <Icon
                className={cn(
                  'h-4 w-4',
                  permission.isExpired
                    ? 'text-red-600'
                    : isExpiringSoon
                      ? 'text-yellow-600'
                      : 'text-blue-600'
                )}
              />
            </div>

            <div className="min-w-0 flex-1">
              <div className="mb-1 flex items-center space-x-2">
                <Badge
                  variant="outline"
                  className={cn(
                    'text-xs',
                    PLATFORM_COLORS[platform as keyof typeof PLATFORM_COLORS] ||
                    PLATFORM_COLORS.default
                  )}
                >
                  {platform}
                </Badge>

                {permission.isExpired && (
                  <Badge variant="destructive" className="text-xs">
                    <XCircle className="mr-1 h-3 w-3" />
                    Expired
                  </Badge>
                )}

                {isExpiringSoon && !permission.isExpired && (
                  <Badge
                    variant="outline"
                    className="border-yellow-500 text-xs text-yellow-700"
                  >
                    <AlertTriangle className="mr-1 h-3 w-3" />
                    Expiring Soon
                  </Badge>
                )}
              </div>

              <h3 className="mb-1 text-sm font-medium text-gray-900 dark:text-gray-100">
                {title}
              </h3>
              {note && (
                <p className="text-xs text-gray-500 dark:text-gray-400 line-clamp-2">
                  {note}
                </p>
              )}
              <code className="text-[10px] text-gray-400 dark:text-gray-500 font-mono">
                {permission.basePermission}
              </code>

              {expiryDate && (
                <div className="space-y-1 text-xs text-gray-500 mt-2">
                  <div className="flex items-center space-x-1">
                    <Clock className="h-3 w-3" />
                    <span>
                      {permission.isExpired ? 'Expired' : 'Expires'}{' '}
                      {formatDistanceToNow(expiryDate, { addSuffix: true })}
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

  // Wrap in tooltip if we have a note
  if (note) {
    return (
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            {cardContent}
          </TooltipTrigger>
          <TooltipContent className="max-w-sm">
            <p className="font-medium">{title}</p>
            <p className="text-sm text-muted-foreground">{note}</p>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    );
  }

  return cardContent;
}

function PermissionStats({
  permissions,
}: {
  permissions: TimestampedPermission[];
}) {
  const totalPermissions = permissions.length;
  const expiredPermissions = permissions.filter(p => p.isExpired).length;
  const expiringSoon = permissions.filter(
    p =>
      !p.isExpired && p.timeRemaining && p.timeRemaining < 24 * 60 * 60 * 1000
  ).length;
  const activePermissions = totalPermissions - expiredPermissions;

  return (
    <div className="mb-6 grid grid-cols-1 gap-4 md:grid-cols-4">
      <Card>
        <CardContent className="p-4">
          <div className="flex items-center space-x-2">
            <Shield className="h-5 w-5 text-blue-500" />
            <div>
              <p className="text-2xl font-bold text-blue-600">
                {totalPermissions}
              </p>
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
              <p className="text-2xl font-bold text-green-600">
                {activePermissions}
              </p>
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
              <p className="text-2xl font-bold text-yellow-600">
                {expiringSoon}
              </p>
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
              <p className="text-2xl font-bold text-red-600">
                {expiredPermissions}
              </p>
              <p className="text-sm text-gray-600">Expired</p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// Analytics components
function PlatformDistributionChart({ distribution }: { distribution: Record<string, number> }) {
  const total = Object.values(distribution).reduce((sum, count) => sum + count, 0);

  return (
    <div className="space-y-3">
      {Object.entries(distribution).map(([platform, count]) => {
        const percentage = total > 0 ? (count / total) * 100 : 0;
        const platformColors = {
          epsx: 'bg-blue-500',
          admin: 'bg-orange-500',
          'epsx-pay': 'bg-green-500',
          'epsx-token': 'bg-purple-500',
        };

        return (
          <div key={platform} className="flex items-center space-x-3">
            <div className="flex-1">
              <div className="flex items-center justify-between mb-1">
                <span className="text-sm font-medium capitalize">{platform}</span>
                <span className="text-sm text-gray-500">{count} permissions</span>
              </div>
              <div className="w-full bg-gray-200 rounded-full h-2">
                <div
                  className={cn(
                    'h-2 rounded-full',
                    platformColors[platform as keyof typeof platformColors] || 'bg-gray-500'
                  )}
                  style={{ width: `${percentage}%` }}
                />
              </div>
            </div>
            <span className="text-sm font-medium text-gray-600 w-12 text-right">
              {percentage.toFixed(0)}%
            </span>
          </div>
        );
      })}
    </div>
  );
}

function PermissionHistory({ history }: { history: PermissionHistoryItem[] }) {
  if (history.length === 0) {
    return (
      <div className="text-center py-8 text-gray-500">
        <History className="h-12 w-12 mx-auto mb-4 text-gray-300" />
        <p className="text-lg font-medium">No permission history</p>
        <p className="text-sm">Your permission activity will appear here</p>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      {history.map((item, index) => {
        const getActionIcon = () => {
          switch (item.action) {
            case 'granted':
              return <CheckCircle className="h-4 w-4 text-green-500" />;
            case 'revoked':
              return <XCircle className="h-4 w-4 text-red-500" />;
            case 'expired':
              return <Clock className="h-4 w-4 text-yellow-500" />;
            default:
              return <Shield className="h-4 w-4 text-gray-500" />;
          }
        };

        const getActionColor = () => {
          switch (item.action) {
            case 'granted':
              return 'bg-green-50 border-green-200 text-green-800';
            case 'revoked':
              return 'bg-red-50 border-red-200 text-red-800';
            case 'expired':
              return 'bg-yellow-50 border-yellow-200 text-yellow-800';
            default:
              return 'bg-gray-50 border-gray-200 text-gray-800';
          }
        };

        return (
          <div key={index} className="flex items-start space-x-3 p-3 border rounded-lg">
            <div className="mt-0.5">{getActionIcon()}</div>
            <div className="flex-1 min-w-0">
              <div className="flex items-center justify-between mb-1">
                <span className="text-sm font-medium text-gray-900">
                  {item.permission}
                </span>
                <Badge variant="outline" className={cn('text-xs', getActionColor())}>
                  {item.action}
                </Badge>
              </div>
              <div className="text-xs text-gray-500 space-y-1">
                <div>{formatDistanceToNow(new Date(item.timestamp), { addSuffix: true })}</div>
                {item.reason && <div>Reason: {item.reason}</div>}
                {item.grantedBy && <div>By: {item.grantedBy}</div>}
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}

export default function PermissionsPage() {
  const { user, isLoading: _isLoading } = useAuth();
  const { base } = useApiClient({ platform: 'frontend' });
  const [activeTab, setActiveTab] = useState<
    'all' | 'active' | 'expiring' | 'expired' | 'analytics' | 'history'
  >('active');
  const [timestampedPermissions, setTimestampedPermissions] = useState<
    TimestampedPermission[]
  >([]);
  const [analytics, setAnalytics] = useState<AnalyticsData | null>(null);
  const [history, setHistory] = useState<PermissionHistoryItem[]>([]);
  const [isExporting, setIsExporting] = useState(false);
  const [permissionDefinitions, setPermissionDefinitions] = useState<Map<string, PermissionDefinition>>(new Map());

  // Load permission definitions
  useEffect(() => {
    loadPermissionDefinitions(base).then(setPermissionDefinitions);
  }, [base]);

  // Parse all permissions with timestamp information
  useEffect(() => {
    if (!user?.permissions) {
      setTimestampedPermissions([]);
      return;
    }

    // Convert granular permissions to string array format
    const permissionStrings =
      typeof user.permissions === 'object' && user.permissions !== null && !Array.isArray(user.permissions)
        ? Object.keys(user.permissions as Record<string, unknown>)
        : Array.isArray(user.permissions)
          ? (user.permissions)
          : [];

    const parsed = permissionStrings.map(perm => {
      const { basePermission, timestamp } = parsePermissionWithTimestamp(perm);
      const expiresAt = timestamp;
      const isExpired = expiresAt ? Date.now() / 1000 > expiresAt : false;
      const timeRemaining = expiresAt
        ? expiresAt * 1000 - Date.now()
        : undefined;

      return {
        permission: perm,
        basePermission,
        expiresAt,
        isExpired,
        timeRemaining,
      };
    });

    setTimestampedPermissions(parsed);
  }, [user?.permissions]);

  // Load analytics data
  useEffect(() => {
    if (activeTab === 'analytics' && user) {
      getPermissionAnalytics()
        .then(setAnalytics)
        .catch(error => {
        });
    }
  }, [activeTab, user]);

  // Load history data
  useEffect(() => {
    if (activeTab === 'history' && user) {
      getPermissionHistory(20)
        .then(setHistory)
        .catch(error => {
        });
    }
  }, [activeTab, user]);

  // Export permissions
  const handleExport = async (format: 'json' | 'csv') => {
    setIsExporting(true);
    try {
      const { data, filename } = await exportPermissionsData(format);
      const blob = new Blob([data], { type: format === 'json' ? 'application/json' : 'text/csv' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = filename;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } catch (error) {
    } finally {
      setIsExporting(false);
    }
  };

  // Filter permissions based on active tab
  const filteredPermissions = timestampedPermissions.filter(perm => {
    switch (activeTab) {
      case 'active':
        return !perm.isExpired;
      case 'expiring':
        return (
          !perm.isExpired &&
          perm.timeRemaining &&
          perm.timeRemaining < 24 * 60 * 60 * 1000
        );
      case 'expired':
        return perm.isExpired;
      default:
        return true;
    }
  });

  return (
    <div className="container mx-auto p-6">
      <GlobalAuthGuard title="My Permissions">
        <div className="max-w-6xl mx-auto">
          {/* Header */}
          <div className="mb-6 flex items-center justify-between">
            <div>
              <h1 className="text-3xl font-bold text-gray-900">My Permissions</h1>
              <p className="mt-1 text-gray-600">
                View and manage your platform permissions and access levels
              </p>
            </div>

            <div className="flex items-center space-x-2">
              <div className="relative">
                <Button
                  data-testid="export-button"
                  onClick={() => handleExport('json')}
                  disabled={isExporting}
                  variant="outline"
                  size="sm"
                >
                  <Download className="mr-2 h-4 w-4" />
                  {isExporting ? 'Exporting...' : 'Export'}
                </Button>
              </div>
              <Button
                data-testid="refresh-permissions-button"
                onClick={() => window.location.reload()}
                variant="outline"
                size="sm"
              >
                <RefreshCw className="mr-2 h-4 w-4" />
                Refresh
              </Button>
            </div>
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
              <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
                <div>
                  <p className="text-sm font-medium text-gray-500">
                    Wallet Address
                  </p>
                  <p className="text-sm text-gray-900">
                    {user?.wallet_address ?? 'Not connected'}
                  </p>
                </div>
                <div>
                  <p className="text-sm font-medium text-gray-500">
                    Permissions Count
                  </p>
                  <p className="text-sm text-gray-900">
                    {typeof user?.permissions === 'object' &&
                      user?.permissions !== null
                      ? Object.keys(user.permissions).length
                      : Array.isArray(user?.permissions)
                        ? (user.permissions as string[]).length
                        : 0}{' '}
                    permissions
                  </p>
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
                All permissions including embedded timestamp permissions with expiry
                information
              </CardDescription>

              {/* Tab Navigation */}
              <div className="flex flex-wrap space-x-2 pt-4">
                {[
                  {
                    key: 'active',
                    label: 'Active',
                    count: timestampedPermissions.filter(p => !p.isExpired).length,
                  },
                  {
                    key: 'expiring',
                    label: 'Expiring Soon',
                    count: timestampedPermissions.filter(
                      p =>
                        !p.isExpired &&
                        p.timeRemaining &&
                        p.timeRemaining < 24 * 60 * 60 * 1000
                    ).length,
                  },
                  {
                    key: 'expired',
                    label: 'Expired',
                    count: timestampedPermissions.filter(p => p.isExpired).length,
                  },
                  {
                    key: 'all',
                    label: 'All',
                    count: timestampedPermissions.length,
                  },
                  {
                    key: 'analytics',
                    label: 'Analytics',
                    count: null,
                  },
                  {
                    key: 'history',
                    label: 'History',
                    count: null,
                  },
                ].map(tab => (
                  <Button
                    key={tab.key}
                    data-testid={`permission-tab-${tab.key}`}
                    variant={activeTab === tab.key ? 'default' : 'outline'}
                    size="sm"
                    onClick={() => setActiveTab(tab.key as typeof activeTab)}
                    className="relative mb-2"
                  >
                    {tab.key === 'analytics' && <BarChart3 className="mr-2 h-4 w-4" />}
                    {tab.key === 'history' && <History className="mr-2 h-4 w-4" />}
                    {tab.label}
                    {tab.count !== null && tab.count > 0 && (
                      <Badge variant="secondary" className="ml-2 text-xs">
                        {tab.count}
                      </Badge>
                    )}
                  </Button>
                ))}
              </div>
            </CardHeader>

            <CardContent>
              {/* Analytics Tab Content */}
              {activeTab === 'analytics' && (
                <div className="space-y-6">
                  {analytics ? (
                    <>
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                        <Card>
                          <CardHeader>
                            <CardTitle className="flex items-center space-x-2">
                              <PieChart className="h-5 w-5" />
                              <span>Platform Distribution</span>
                            </CardTitle>
                            <CardDescription>
                              Breakdown of your permissions by platform
                            </CardDescription>
                          </CardHeader>
                          <CardContent>
                            <PlatformDistributionChart distribution={analytics.platformDistribution} />
                          </CardContent>
                        </Card>

                        <Card>
                          <CardHeader>
                            <CardTitle className="flex items-center space-x-2">
                              <TrendingUp className="h-5 w-5" />
                              <span>Usage Statistics</span>
                            </CardTitle>
                            <CardDescription>
                              Your most frequently used permissions
                            </CardDescription>
                          </CardHeader>
                          <CardContent>
                            <div className="space-y-3">
                              {analytics.usageStats?.mostUsedPermissions?.map((item, index: number) => (
                                <div key={index} className="flex items-center justify-between">
                                  <span className="text-sm font-medium">{item.permission}</span>
                                  <Badge variant="secondary">{item.usage} uses</Badge>
                                </div>
                              ))}
                            </div>
                          </CardContent>
                        </Card>
                      </div>

                      <Card>
                        <CardHeader>
                          <CardTitle className="flex items-center space-x-2">
                            <BarChart3 className="h-5 w-5" />
                            <span>Recently Granted Permissions</span>
                          </CardTitle>
                        </CardHeader>
                        <CardContent>
                          <div className="space-y-3">
                            {analytics.usageStats?.recentlyGranted?.map((item, index: number) => (
                              <div key={index} className="flex items-center justify-between">
                                <span className="text-sm font-medium">{item.permission}</span>
                                <span className="text-xs text-gray-500">
                                  {formatDistanceToNow(new Date(item.grantedAt), { addSuffix: true })}
                                </span>
                              </div>
                            ))}
                          </div>
                        </CardContent>
                      </Card>
                    </>
                  ) : (
                    <div className="text-center py-8">
                      <BarChart3 className="h-12 w-12 mx-auto mb-4 text-gray-300" />
                      <p className="text-lg font-medium">Loading analytics...</p>
                    </div>
                  )}
                </div>
              )}

              {/* History Tab Content */}
              {activeTab === 'history' && (
                <div className="space-y-4">
                  <div className="flex items-center justify-between">
                    <h3 className="text-lg font-medium">Permission Activity History</h3>
                    <Badge variant="outline">{history.length} activities</Badge>
                  </div>
                  <PermissionHistory history={history} />
                </div>
              )}

              {/* Permission Lists for Other Tabs */}
              {activeTab !== 'analytics' && activeTab !== 'history' && (
                <>
                  {filteredPermissions.length === 0 ? (
                    <div className="py-12 text-center">
                      <Shield className="mx-auto mb-4 h-12 w-12 text-gray-400" />
                      <h3 className="mb-2 text-lg font-medium text-gray-900">
                        No {activeTab !== 'all' ? activeTab : ''} permissions found
                      </h3>
                      <p className="text-gray-500">
                        {activeTab === 'active'
                          ? 'You have no active permissions at the moment.'
                          : activeTab === 'expiring'
                            ? 'No permissions are expiring soon.'
                            : activeTab === 'expired'
                              ? 'No permissions have expired.'
                              : 'You have no permissions assigned to your account.'}
                      </p>
                    </div>
                  ) : (
                    <div className="space-y-3">
                      {filteredPermissions.map((perm, index) => (
                        <PermissionCard
                          key={`${perm.permission}-${index}`}
                          permission={perm}
                          definitions={permissionDefinitions}
                        />
                      ))}
                    </div>
                  )}
                </>
              )}
            </CardContent>
          </Card>
        </div>
      </GlobalAuthGuard>
    </div>
  );
}
