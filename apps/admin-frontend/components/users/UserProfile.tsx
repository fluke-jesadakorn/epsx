/**
 * User Profile - Details and History
 * Consolidates: UserProfileHeader, UserCard, UserOverviewContent, UserActivityContent,
 * UserModulesContent, UserPackagesContent, ActivityTimelineCard, LoginHistoryCard,
 * UserPermissionHealthIndicator, UserStatusBadge, UserTabNavigation
 */

'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { toast } from '@/hooks/use-toast';
import { 
  User as UserIcon,
  Mail,
  Calendar,
  Shield,
  Activity,
  Clock,
  MapPin,
  Globe,
  Smartphone,
  AlertTriangle,
  CheckCircle,
  XCircle,
  TrendingUp,
  TrendingDown,
  Edit,
  MoreHorizontal,
  RefreshCw
} from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar';
import { Progress } from '@/components/ui/progress';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';

import type { User, ActivityLog, AuditLog, PermissionHealth } from '@/types/core';
import { adminClient } from '@/lib/api/unified-admin-client';
import { logger } from '@/lib/logger';

interface UserProfileProps {
  userId?: string;
  user?: any;
  currentUser?: any;
  onUserUpdate?: (user: User) => void;
  onPermissionChange?: (userId: string, permissions: string[]) => void;
  className?: string;
}

interface LoginSession {
  id: string;
  ipAddress: string;
  userAgent: string;
  location?: {
    country: string;
    city: string;
  };
  device: {
    type: 'desktop' | 'mobile' | 'tablet';
    browser: string;
    os: string;
  };
  loginTime: string;
  lastActivity: string;
  isActive: boolean;
}

interface UserMetrics {
  totalLogins: number;
  lastLogin: string;
  averageSessionDuration: number;
  apiCallsThisMonth: number;
  dataExported: number;
  securityScore: number;
}

export function UserProfile({
  userId,
  user: initialUser,
  currentUser,
  onUserUpdate,
  onPermissionChange,
  className = ''
}: UserProfileProps) {
  const router = useRouter();
  const [user, setUser] = useState(initialUser);
  
  // If no user data provided, create a placeholder
  if (!user) {
    return (
      <div className="text-center py-12">
        <UserIcon className="w-16 h-16 mx-auto text-gray-400 mb-4" />
        <h2 className="text-xl font-semibold text-gray-600 mb-2">User not found</h2>
        <p className="text-gray-500">The requested user could not be loaded.</p>
        <Button onClick={() => router.push('/users')} className="mt-4">
          Back to Users
        </Button>
      </div>
    );
  }
  // State management
  const [activeTab, setActiveTab] = useState('overview');
  const [isLoading, setIsLoading] = useState(false);
  const [activityLogs, setActivityLogs] = useState<ActivityLog[]>([]);
  const [loginHistory, setLoginHistory] = useState<LoginSession[]>([]);
  const [userMetrics, setUserMetrics] = useState<UserMetrics | null>(null);
  const [permissionHealth, setPermissionHealth] = useState<PermissionHealth | null>(null);

  // Calculate user stats
  const permissions = user.permissions || [];
  const userStats = {
    accountAge: Math.floor((Date.now() - new Date(user.createdAt || Date.now()).getTime()) / (1000 * 60 * 60 * 24)),
    totalPermissions: permissions.length,
    activePermissions: permissions.filter((p: string) => !p.includes(':')).length,
    temporaryPermissions: permissions.filter((p: string) => p.includes(':')).length,
    healthScore: permissionHealth?.healthScore || 95
  };

  // Load user activity data
  const loadActivityData = async () => {
    setIsLoading(true);
    try {
      // Mock data - replace with actual API calls
      const mockActivityLogs: ActivityLog[] = [
        {
          id: '1',
          userId: user.id,
          type: 'login',
          description: 'User logged in from new device',
          timestamp: new Date(Date.now() - 3600000).toISOString(),
          metadata: { device: 'Chrome on Windows', ip: '192.168.1.100' }
        },
        {
          id: '2',
          userId: user.id,
          type: 'permission_change',
          description: 'Permission granted: epsx:analytics:view',
          timestamp: new Date(Date.now() - 7200000).toISOString(),
          metadata: { permission: 'epsx:analytics:view', grantedBy: 'admin@epsx.io' }
        },
        {
          id: '3',
          userId: user.id,
          type: 'api_call',
          description: 'API access: GET /api/v1/analytics/eps-rankings',
          timestamp: new Date(Date.now() - 10800000).toISOString(),
          metadata: { endpoint: '/api/v1/analytics/eps-rankings', status: 200 }
        }
      ];

      const mockLoginHistory: LoginSession[] = [
        {
          id: '1',
          ipAddress: '192.168.1.100',
          userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
          location: { country: 'United States', city: 'San Francisco' },
          device: { type: 'desktop', browser: 'Chrome', os: 'Windows' },
          loginTime: new Date(Date.now() - 3600000).toISOString(),
          lastActivity: new Date(Date.now() - 1800000).toISOString(),
          isActive: true
        },
        {
          id: '2',
          ipAddress: '10.0.0.50',
          userAgent: 'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)',
          location: { country: 'United States', city: 'San Francisco' },
          device: { type: 'mobile', browser: 'Safari', os: 'iOS' },
          loginTime: new Date(Date.now() - 86400000).toISOString(),
          lastActivity: new Date(Date.now() - 82800000).toISOString(),
          isActive: false
        }
      ];

      const mockMetrics: UserMetrics = {
        totalLogins: 47,
        lastLogin: new Date(Date.now() - 3600000).toISOString(),
        averageSessionDuration: 2.5,
        apiCallsThisMonth: 234,
        dataExported: 12,
        securityScore: 85
      };

      const mockPermissionHealth: PermissionHealth = {
        userId: user.id,
        totalPermissions: permissions.length,
        expiringPermissions: permissions.filter((p: string) => p.includes(':')).slice(0, 2),
        expiredPermissions: [],
        healthScore: 92,
        lastCalculated: new Date().toISOString(),
        recommendations: [
          'Review temporary permissions',
          'Consider upgrading to permanent access for frequently used features'
        ]
      };

      setActivityLogs(mockActivityLogs);
      setLoginHistory(mockLoginHistory);
      setUserMetrics(mockMetrics);
      setPermissionHealth(mockPermissionHealth);

    } catch (error) {
      logger.error('Failed to load user activity', { userId: user?.id, error });
    } finally {
      setIsLoading(false);
    }
  };

  // Get activity type icon
  const getActivityIcon = (type: ActivityLog['type']) => {
    switch (type) {
      case 'login': return <UserIcon className="w-4 h-4 text-green-500" />;
      case 'logout': return <XCircle className="w-4 h-4 text-red-500" />;
      case 'permission_change': return <Shield className="w-4 h-4 text-blue-500" />;
      case 'role_change': return <TrendingUp className="w-4 h-4 text-purple-500" />;
      case 'profile_update': return <Edit className="w-4 h-4 text-yellow-500" />;
      case 'api_call': return <Activity className="w-4 h-4 text-gray-500" />;
      default: return <Clock className="w-4 h-4 text-gray-500" />;
    }
  };

  // Get device icon
  const getDeviceIcon = (deviceType: string) => {
    switch (deviceType) {
      case 'mobile': return <Smartphone className="w-4 h-4 text-blue-500" />;
      case 'tablet': return <Smartphone className="w-4 h-4 text-green-500" />;
      default: return <Globe className="w-4 h-4 text-purple-500" />;
    }
  };

  // Initial load
  useEffect(() => {
    loadActivityData();
  }, [user.id]);

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Page Header */}
      <div className="text-center mb-8">
        <div className="relative inline-block">
          <h1 className="text-4xl sm:text-5xl font-bold bg-gradient-to-r from-yellow-600 via-orange-600 via-pink-600 to-purple-600 bg-clip-text text-transparent mb-4">
            👤 User Profile
          </h1>
          <div className="absolute -top-2 -right-2 w-4 h-4 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-full animate-pulse"></div>
        </div>
        <p className="text-base sm:text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
          View and manage user information, activity, and permissions
        </p>
      </div>
      
      {/* User Header */}
      <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-blue-400/20 via-purple-400/20 to-pink-400/20 p-0.5">
        <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-3xl p-6">
          <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-blue-400 to-purple-500 rounded-full blur-sm animate-pulse opacity-60"></div>
          
          <div className="flex flex-col lg:flex-row items-start gap-6">
            <div className="flex items-center gap-4 flex-1">
              {/* User Avatar */}
              <div className="relative">
                <Avatar className="w-20 h-20 border-4 border-yellow-400/30">
                  <AvatarImage src={`https://api.dicebear.com/7.x/initials/svg?seed=${user.email || 'user'}`} />
                  <AvatarFallback className="text-xl font-bold bg-gradient-to-r from-yellow-400 to-orange-500 text-white">
                    {(user.email || 'U').slice(0, 2).toUpperCase()}
                  </AvatarFallback>
                </Avatar>
                <div className={`absolute -bottom-1 -right-1 w-6 h-6 rounded-full border-2 border-white dark:border-gray-900 ${user.isActive ? 'bg-green-500' : 'bg-gray-400'}`}></div>
              </div>

              {/* User Info */}
              <div className="flex-1">
                <div className="flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-3 mb-3">
                  <h2 className="text-2xl lg:text-3xl font-bold text-gray-800 dark:text-gray-200">
                    {user.displayName || user.name || (user.email?.split('@')[0]) || 'Unknown User'}
                  </h2>
                  <div className="flex gap-2">
                    <Badge 
                      variant={user.isActive ? 'default' : 'secondary'}
                      className={`${user.isActive 
                        ? 'bg-gradient-to-r from-green-500 to-emerald-500 text-white'
                        : 'bg-gray-200 text-gray-700'
                      } px-3 py-1 rounded-full`}
                    >
                      {user.isActive ? '✅ Active' : '❌ Inactive'}
                    </Badge>
                    <Badge 
                      variant={user.role === 'admin' ? 'destructive' : 'outline'}
                      className={`${
                        user.role === 'admin' 
                          ? 'bg-gradient-to-r from-red-500 to-pink-500 text-white'
                          : user.role === 'premium' 
                          ? 'bg-gradient-to-r from-purple-500 to-indigo-500 text-white'
                          : 'border-2 border-blue-300 text-blue-600'
                      } px-3 py-1 rounded-full`}
                    >
                      {user.role === 'admin' && '👑 '}
                      {user.role === 'premium' && '⭐ '}
                      {user.role === 'user' && '👤 '}
                      {user.role}
                    </Badge>
                  </div>
                </div>

                <div className="grid grid-cols-1 sm:grid-cols-3 gap-3 text-sm text-gray-600 dark:text-gray-400">
                  <div className="flex items-center gap-2 p-2 bg-blue-50 dark:bg-blue-900/20 rounded-2xl">
                    <Mail className="w-4 h-4 text-blue-500" />
                    <span className="truncate">{user.email || 'No email'}</span>
                  </div>
                  <div className="flex items-center gap-2 p-2 bg-purple-50 dark:bg-purple-900/20 rounded-2xl">
                    <Calendar className="w-4 h-4 text-purple-500" />
                    <span>Joined {new Date(user.createdAt || Date.now()).toLocaleDateString()}</span>
                  </div>
                  <div className="flex items-center gap-2 p-2 bg-green-50 dark:bg-green-900/20 rounded-2xl">
                    <Shield className="w-4 h-4 text-green-500" />
                    <span>{user.permissions?.length || 0} permissions</span>
                  </div>
                </div>

                {user.lastLoginAt && (
                  <p className="text-xs text-gray-500 dark:text-gray-400 mt-2 flex items-center gap-1">
                    <Clock className="w-3 h-3" />
                    Last seen: {new Date(user.lastLoginAt).toLocaleString()}
                  </p>
                )}
              </div>
            </div>

            {/* Action Menu */}
            <div className="flex gap-2">
              <Button
                onClick={() => router.push(`/users/${user.id}/edit`)}
                className="bg-gradient-to-r from-blue-500 to-purple-500 hover:from-blue-600 hover:to-purple-600 rounded-2xl"
              >
                <Edit className="w-4 h-4 mr-2" />
                Edit Profile
              </Button>
              
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button variant="outline" size="sm" className="rounded-2xl border-2">
                    <MoreHorizontal className="w-4 h-4" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end" className="bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl border border-gray-200/50 dark:border-gray-700/50 rounded-2xl shadow-xl">
                  <DropdownMenuItem className="rounded-xl">
                    <Shield className="w-4 h-4 mr-2" />
                    Manage Permissions
                  </DropdownMenuItem>
                  <DropdownMenuItem className="rounded-xl">
                    <RefreshCw className="w-4 h-4 mr-2" />
                    Reset Password
                  </DropdownMenuItem>
                  <DropdownMenuItem className="rounded-xl text-red-600">
                    <XCircle className="w-4 h-4 mr-2" />
                    Deactivate User
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
          </div>
        </div>
      </div>

      {/* Quick Stats */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-blue-400/20 via-cyan-400/20 to-blue-500/20 p-0.5">
          <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-3xl p-6 text-center">
            <div className="absolute top-2 right-2 w-3 h-3 bg-gradient-to-r from-blue-400 to-cyan-500 rounded-full blur-sm animate-pulse opacity-60"></div>
            <Calendar className="w-8 h-8 text-blue-500 mx-auto mb-3" />
            <p className="text-sm text-gray-500 dark:text-gray-400 mb-1">Account Age</p>
            <p className="text-2xl font-bold text-blue-600">{userStats.accountAge}d</p>
          </div>
        </div>

        <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-green-400/20 via-emerald-400/20 to-green-500/20 p-0.5">
          <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-3xl p-6 text-center">
            <div className="absolute top-2 right-2 w-3 h-3 bg-gradient-to-r from-green-400 to-emerald-500 rounded-full blur-sm animate-pulse opacity-60"></div>
            <Shield className="w-8 h-8 text-green-500 mx-auto mb-3" />
            <p className="text-sm text-gray-500 dark:text-gray-400 mb-1">Permissions</p>
            <p className="text-2xl font-bold text-green-600">{userStats.totalPermissions}</p>
          </div>
        </div>

        <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-purple-400/20 via-pink-400/20 to-purple-500/20 p-0.5">
          <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-3xl p-6 text-center">
            <div className="absolute top-2 right-2 w-3 h-3 bg-gradient-to-r from-purple-400 to-pink-500 rounded-full blur-sm animate-pulse opacity-60"></div>
            <CheckCircle className="w-8 h-8 text-purple-500 mx-auto mb-3" />
            <p className="text-sm text-gray-500 dark:text-gray-400 mb-1">Health Score</p>
            <p className="text-2xl font-bold text-purple-600">{userStats.healthScore}%</p>
          </div>
        </div>

        <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-orange-400/20 via-yellow-400/20 to-orange-500/20 p-0.5">
          <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-3xl p-6 text-center">
            <div className="absolute top-2 right-2 w-3 h-3 bg-gradient-to-r from-orange-400 to-yellow-500 rounded-full blur-sm animate-pulse opacity-60"></div>
            <TrendingUp className="w-8 h-8 text-orange-500 mx-auto mb-3" />
            <p className="text-sm text-gray-500 dark:text-gray-400 mb-1">Package Tier</p>
            <p className="text-lg font-bold text-orange-600 capitalize">{user.packageTier || 'basic'}</p>
          </div>
        </div>
      </div>

      {/* Main Content Tabs */}
      <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-gray-400/20 via-slate-400/20 to-gray-500/20 p-0.5">
        <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-3xl p-2">
          <Tabs value={activeTab} onValueChange={setActiveTab}>
            <TabsList className="grid w-full grid-cols-4 bg-transparent p-1 h-12">
              <TabsTrigger value="overview" className="rounded-2xl data-[state=active]:bg-gradient-to-r data-[state=active]:from-blue-500 data-[state=active]:to-purple-500 data-[state=active]:text-white font-medium">
                📋 Overview
              </TabsTrigger>
              <TabsTrigger value="activity" className="rounded-2xl data-[state=active]:bg-gradient-to-r data-[state=active]:from-green-500 data-[state=active]:to-emerald-500 data-[state=active]:text-white font-medium">
                📈 Activity
              </TabsTrigger>
              <TabsTrigger value="sessions" className="rounded-2xl data-[state=active]:bg-gradient-to-r data-[state=active]:from-orange-500 data-[state=active]:to-yellow-500 data-[state=active]:text-white font-medium">
                💻 Sessions
              </TabsTrigger>
              <TabsTrigger value="security" className="rounded-2xl data-[state=active]:bg-gradient-to-r data-[state=active]:from-red-500 data-[state=active]:to-pink-500 data-[state=active]:text-white font-medium">
                🔒 Security
              </TabsTrigger>
            </TabsList>

            {/* Overview Tab */}
            <TabsContent value="overview" className="space-y-6 mt-6">
              <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                {/* User Details */}
                <div className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-blue-400/20 via-cyan-400/20 to-blue-500/20 p-0.5">
                  <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-3xl p-6">
                    <div className="absolute top-4 right-4 w-3 h-3 bg-gradient-to-r from-blue-400 to-cyan-500 rounded-full blur-sm animate-pulse opacity-60"></div>
                    <h3 className="text-lg font-semibold text-gray-800 dark:text-gray-200 mb-4 flex items-center gap-2">
                      <UserIcon className="w-5 h-5 text-blue-500" />
                      User Details
                    </h3>
              <div className="space-y-3">
                <div className="flex justify-between">
                  <span className="text-gray-400">User ID</span>
                  <span className="text-white font-mono text-sm">{user.id}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Email</span>
                  <span className="text-white">{user.email || 'No email'}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Display Name</span>
                  <span className="text-white">{user.displayName || 'Not set'}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Role</span>
                  <Badge variant={user.role === 'admin' ? 'destructive' : 'secondary'}>
                    {user.role}
                  </Badge>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Package Tier</span>
                  <Badge variant="outline">{user.packageTier}</Badge>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Status</span>
                  <Badge variant={user.isActive ? 'default' : 'secondary'}>
                    {user.isActive ? 'Active' : 'Inactive'}
                  </Badge>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Created</span>
                  <span className="text-white">{new Date(user.createdAt || Date.now()).toLocaleDateString()}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Last Updated</span>
                  <span className="text-white">{new Date(user.updatedAt || Date.now()).toLocaleDateString()}</span>
                </div>
              </div>
                  </div>
                </div>

            {/* Permission Health */}
            {permissionHealth && (
              <Card className="p-6">
                <h3 className="text-lg font-medium text-white mb-4">Permission Health</h3>
                <div className="space-y-4">
                  <div>
                    <div className="flex justify-between items-center mb-2">
                      <span className="text-sm text-gray-400">Overall Health</span>
                      <span className="text-sm font-medium text-green-400">
                        {permissionHealth.healthScore}%
                      </span>
                    </div>
                    <Progress value={permissionHealth.healthScore} className="h-2" />
                  </div>

                  <div className="grid grid-cols-2 gap-4">
                    <div className="text-center">
                      <div className="text-2xl font-bold text-white">
                        {permissionHealth.totalPermissions}
                      </div>
                      <div className="text-xs text-gray-400">Total</div>
                    </div>
                    <div className="text-center">
                      <div className="text-2xl font-bold text-yellow-400">
                        {permissionHealth.expiringPermissions.length}
                      </div>
                      <div className="text-xs text-gray-400">Expiring</div>
                    </div>
                  </div>

                  {permissionHealth.recommendations && permissionHealth.recommendations.length > 0 && (
                    <div className="bg-yellow-500/10 border border-yellow-500/20 rounded-lg p-3">
                      <h4 className="text-sm font-medium text-yellow-400 mb-2">Recommendations</h4>
                      <ul className="space-y-1">
                        {permissionHealth.recommendations.map((rec, index) => (
                          <li key={index} className="text-xs text-gray-300 flex items-start gap-2">
                            <AlertTriangle className="w-3 h-3 text-yellow-500 mt-0.5 flex-shrink-0" />
                            {rec}
                          </li>
                        ))}
                      </ul>
                    </div>
                  )}
                </div>
              </Card>
            )}
          </div>

          {/* Usage Metrics */}
          {userMetrics && (
            <Card className="p-6">
              <h3 className="text-lg font-medium text-white mb-4">Usage Metrics</h3>
              <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                <div className="text-center">
                  <div className="text-2xl font-bold text-blue-400">{userMetrics.totalLogins}</div>
                  <div className="text-sm text-gray-400">Total Logins</div>
                </div>
                <div className="text-center">
                  <div className="text-2xl font-bold text-green-400">{userMetrics.averageSessionDuration}h</div>
                  <div className="text-sm text-gray-400">Avg Session</div>
                </div>
                <div className="text-center">
                  <div className="text-2xl font-bold text-purple-400">{userMetrics.apiCallsThisMonth}</div>
                  <div className="text-sm text-gray-400">API Calls</div>
                </div>
                <div className="text-center">
                  <div className="text-2xl font-bold text-orange-400">{userMetrics.dataExported}</div>
                  <div className="text-sm text-gray-400">Data Exports</div>
                </div>
              </div>
            </Card>
          )}
        </TabsContent>

        {/* Activity Tab */}
        <TabsContent value="activity" className="space-y-4">
          <Card className="p-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-medium text-white">Recent Activity</h3>
              <Button variant="outline" size="sm" onClick={loadActivityData}>
                <RefreshCw className="w-4 h-4 mr-1" />
                Refresh
              </Button>
            </div>

            {isLoading ? (
              <div className="text-center py-8">
                <RefreshCw className="w-8 h-8 animate-spin mx-auto text-blue-500" />
                <p className="text-gray-400 mt-2">Loading activity...</p>
              </div>
            ) : (
              <div className="space-y-4">
                {activityLogs.map(log => (
                  <div key={log.id} className="flex gap-4 p-3 bg-gray-800/30 rounded-lg">
                    <div className="flex-shrink-0 mt-1">
                      {getActivityIcon(log.type)}
                    </div>
                    <div className="flex-1">
                      <div className="flex items-start justify-between">
                        <div>
                          <p className="text-sm font-medium text-white">{log.description}</p>
                          <div className="flex items-center gap-2 mt-1">
                            <Badge variant="secondary">
                              {log.type.replace('_', ' ')}
                            </Badge>
                            {log.metadata && Object.keys(log.metadata).length > 0 && (
                              <span className="text-xs text-gray-500">
                                {Object.entries(log.metadata).map(([key, value]) => 
                                  `${key}: ${value}`
                                ).join(', ')}
                              </span>
                            )}
                          </div>
                        </div>
                        <span className="text-xs text-gray-500">
                          {new Date(log.timestamp).toLocaleString()}
                        </span>
                      </div>
                    </div>
                  </div>
                ))}

                {activityLogs.length === 0 && (
                  <div className="text-center py-8 text-gray-400">
                    <Activity className="w-12 h-12 mx-auto mb-4 opacity-50" />
                    <p>No recent activity found</p>
                  </div>
                )}
              </div>
            )}
          </Card>
        </TabsContent>

        {/* Sessions Tab */}
        <TabsContent value="sessions" className="space-y-4">
          <Card className="p-6">
            <h3 className="text-lg font-medium text-white mb-4">Login Sessions</h3>
            
            <div className="space-y-4">
              {loginHistory.map(session => (
                <div key={session.id} className="flex items-center gap-4 p-4 bg-gray-800/30 rounded-lg">
                  <div className="flex-shrink-0">
                    {getDeviceIcon(session.device.type)}
                  </div>
                  <div className="flex-1">
                    <div className="flex items-center justify-between mb-1">
                      <div className="flex items-center gap-2">
                        <span className="font-medium text-white">
                          {session.device.browser} on {session.device.os}
                        </span>
                        {session.isActive && (
                          <Badge variant="default">Active</Badge>
                        )}
                      </div>
                      <span className="text-xs text-gray-500">
                        {new Date(session.loginTime).toLocaleString()}
                      </span>
                    </div>
                    
                    <div className="flex items-center gap-4 text-xs text-gray-400">
                      <div className="flex items-center gap-1">
                        <Globe className="w-3 h-3" />
                        {session.ipAddress}
                      </div>
                      {session.location && (
                        <div className="flex items-center gap-1">
                          <MapPin className="w-3 h-3" />
                          {session.location.city}, {session.location.country}
                        </div>
                      )}
                      <div className="flex items-center gap-1">
                        <Clock className="w-3 h-3" />
                        Last active: {new Date(session.lastActivity).toLocaleString()}
                      </div>
                    </div>
                  </div>
                  
                  {!session.isActive && (
                    <Button variant="outline" size="sm">
                      Revoke
                    </Button>
                  )}
                </div>
              ))}

              {loginHistory.length === 0 && (
                <div className="text-center py-8 text-gray-400">
                  <UserIcon className="w-12 h-12 mx-auto mb-4 opacity-50" />
                  <p>No login sessions found</p>
                </div>
              )}
            </div>
          </Card>
        </TabsContent>

        {/* Security Tab */}
        <TabsContent value="security" className="space-y-4">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Security Score */}
            <Card className="p-6">
              <h3 className="text-lg font-medium text-white mb-4">Security Score</h3>
              <div className="space-y-4">
                {userMetrics && (
                  <div>
                    <div className="flex justify-between items-center mb-2">
                      <span className="text-sm text-gray-400">Overall Security</span>
                      <span className="text-sm font-medium text-green-400">
                        {userMetrics.securityScore}%
                      </span>
                    </div>
                    <Progress value={userMetrics.securityScore} className="h-2" />
                  </div>
                )}

                <div className="space-y-2">
                  <div className="flex items-center justify-between p-2 bg-green-500/10 rounded">
                    <div className="flex items-center gap-2">
                      <CheckCircle className="w-4 h-4 text-green-500" />
                      <span className="text-sm">Strong Password</span>
                    </div>
                    <Badge variant="default">Good</Badge>
                  </div>

                  <div className="flex items-center justify-between p-2 bg-yellow-500/10 rounded">
                    <div className="flex items-center gap-2">
                      <AlertTriangle className="w-4 h-4 text-yellow-500" />
                      <span className="text-sm">Two-Factor Authentication</span>
                    </div>
                    <Badge variant="secondary">Not Enabled</Badge>
                  </div>

                  <div className="flex items-center justify-between p-2 bg-green-500/10 rounded">
                    <div className="flex items-center gap-2">
                      <CheckCircle className="w-4 h-4 text-green-500" />
                      <span className="text-sm">Email Verified</span>
                    </div>
                    <Badge variant="default">Verified</Badge>
                  </div>
                </div>
              </div>
            </Card>

            {/* Security Actions */}
            <Card className="p-6">
              <h3 className="text-lg font-medium text-white mb-4">Security Actions</h3>
              <div className="space-y-3">
                <Button variant="outline" className="w-full justify-start">
                  <RefreshCw className="w-4 h-4 mr-2" />
                  Force Password Reset
                </Button>
                <Button variant="outline" className="w-full justify-start">
                  <XCircle className="w-4 h-4 mr-2" />
                  Revoke All Sessions
                </Button>
                <Button variant="outline" className="w-full justify-start">
                  <Shield className="w-4 h-4 mr-2" />
                  Enable 2FA
                </Button>
                <Button variant="outline" className="w-full justify-start">
                  <AlertTriangle className="w-4 h-4 mr-2" />
                  Lock Account
                </Button>
              </div>
            </Card>
          </div>
        </TabsContent>
          </Tabs>
        </div>
      </div>
    </div>
  );
}

export default UserProfile;