'use client';

import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { 
  Table, 
  TableBody, 
  TableCell, 
  TableHead, 
  TableHeader, 
  TableRow 
} from '@/components/ui/table';
import { 
  Dialog, 
  DialogContent, 
  DialogDescription, 
  DialogHeader, 
  DialogTitle, 
  DialogTrigger 
} from '@/components/ui/dialog';
import { 
  Users, 
  Shield, 
  AlertTriangle, 
  TrendingUp, 
  Clock, 
  Activity,
  Search,
  RotateCcw,
  UserCheck,
  UserX,
  Filter,
  Eye,
  Settings
} from 'lucide-react';
import { useAdminPermissionDashboard } from '@/hooks/useGranularPermissions';
import { 
  AdminPermissionDashboard as DashboardData,
  UserPermissionOverview,
  PermissionSearchFilters,
  PermissionSource,
  AdminPermissionRowData
} from '@/types/granular-permissions';
import GranularPermissionManager from './GranularPermissionManager';
import { formatDistanceToNow } from 'date-fns';

interface AdminPermissionDashboardProps {
  refreshInterval?: number; // milliseconds
  showRecentActivity?: boolean;
  className?: string;
}

export function AdminPermissionDashboard({ 
  refreshInterval = 30000, // 30 seconds
  showRecentActivity = true,
  className = ''
}: AdminPermissionDashboardProps) {
  const { 
    dashboard, 
    refreshDashboard, 
    getAllUsersWithPermissions, 
    loading, 
    error 
  } = useAdminPermissionDashboard();

  const [searchQuery, setSearchQuery] = useState('');
  const [users, setUsers] = useState<UserPermissionOverview[]>([]);
  const [filteredUsers, setFilteredUsers] = useState<UserPermissionOverview[]>([]);
  const [selectedUser, setSelectedUser] = useState<string | null>(null);
  const [filters, setFilters] = useState<PermissionSearchFilters>({});
  const [showFilters, setShowFilters] = useState(false);

  // Auto-refresh dashboard
  useEffect(() => {
    if (refreshInterval > 0) {
      const interval = setInterval(refreshDashboard, refreshInterval);
      return () => clearInterval(interval);
    }
  }, [refreshInterval, refreshDashboard]);

  // Load users with permissions
  useEffect(() => {
    const loadUsers = async () => {
      try {
        const userData = await getAllUsersWithPermissions(filters);
        setUsers(userData);
      } catch (err) {
        console.error('Failed to load users:', err);
      }
    };

    loadUsers();
  }, [filters, getAllUsersWithPermissions]);

  // Filter users based on search query
  useEffect(() => {
    if (!searchQuery.trim()) {
      setFilteredUsers(users);
    } else {
      const query = searchQuery.toLowerCase();
      const filtered = users.filter(user => 
        user.email.toLowerCase().includes(query) ||
        user.display_name?.toLowerCase().includes(query) ||
        user.user_id.toLowerCase().includes(query) ||
        Object.keys(user.permissions).some(perm => perm.toLowerCase().includes(query))
      );
      setFilteredUsers(filtered);
    }
  }, [searchQuery, users]);

  const getHealthStatusColor = (healthScore: number): string => {
    if (healthScore >= 80) return 'text-green-600';
    if (healthScore >= 60) return 'text-yellow-600';
    if (healthScore >= 40) return 'text-orange-600';
    return 'text-red-600';
  };

  const getHealthStatusBadge = (healthScore: number) => {
    if (healthScore >= 80) return <Badge variant="default" className="bg-green-100 text-green-800">Healthy</Badge>;
    if (healthScore >= 60) return <Badge variant="default" className="bg-yellow-100 text-yellow-800">Warning</Badge>;
    if (healthScore >= 40) return <Badge variant="destructive" className="bg-orange-100 text-orange-800">Issues</Badge>;
    return <Badge variant="destructive">Critical</Badge>;
  };

  if (error) {
    return (
      <Card className={className}>
        <CardContent className="p-6">
          <Alert variant="destructive">
            <AlertTriangle className="h-4 w-4" />
            <AlertDescription>
              Failed to load permission dashboard: {error.message}
            </AlertDescription>
          </Alert>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Permission Dashboard</h1>
          <p className="text-muted-foreground">Monitor and manage user permissions across the platform</p>
        </div>
        <Button
          variant="outline"
          onClick={refreshDashboard}
          disabled={loading}
        >
          <RotateCcw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
          Refresh
        </Button>
      </div>

      {/* Metrics Cards */}
      {dashboard && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Total Users</CardTitle>
              <Users className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{dashboard.total_users_with_permissions.toLocaleString()}</div>
              <p className="text-xs text-muted-foreground">with permissions</p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Total Permissions</CardTitle>
              <Shield className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{dashboard.total_permissions_granted.toLocaleString()}</div>
              <p className="text-xs text-muted-foreground">granted permissions</p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Expiring Soon</CardTitle>
              <Clock className="h-4 w-4 text-yellow-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-yellow-600">
                {dashboard.expiring_permissions_24h.toLocaleString()}
              </div>
              <p className="text-xs text-muted-foreground">within 24 hours</p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">System Health</CardTitle>
              <Activity className={`h-4 w-4 ${getHealthStatusColor(dashboard.system_health_score)}`} />
            </CardHeader>
            <CardContent>
              <div className={`text-2xl font-bold ${getHealthStatusColor(dashboard.system_health_score)}`}>
                {dashboard.system_health_score}%
              </div>
              <p className="text-xs text-muted-foreground">overall health</p>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Recent Activity */}
      {showRecentActivity && dashboard && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <Card>
            <CardHeader>
              <CardTitle className="text-lg">Recent Grants</CardTitle>
              <CardDescription>Latest permissions granted</CardDescription>
            </CardHeader>
            <CardContent>
              {dashboard.recent_grants.length === 0 ? (
                <p className="text-muted-foreground text-center py-4">No recent activity</p>
              ) : (
                <div className="space-y-2">
                  {dashboard.recent_grants.slice(0, 5).map((audit) => (
                    <div key={audit.id} className="flex items-center justify-between p-2 border rounded">
                      <div className="flex-1">
                        <div className="text-sm font-medium">{audit.permission}</div>
                        <div className="text-xs text-muted-foreground">
                          {audit.user_id} • {formatDistanceToNow(new Date(audit.performed_at * 1000), { addSuffix: true })}
                        </div>
                      </div>
                      <UserCheck className="h-4 w-4 text-green-500" />
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-lg">Recent Revocations</CardTitle>
              <CardDescription>Latest permissions revoked</CardDescription>
            </CardHeader>
            <CardContent>
              {dashboard.recent_revocations.length === 0 ? (
                <p className="text-muted-foreground text-center py-4">No recent activity</p>
              ) : (
                <div className="space-y-2">
                  {dashboard.recent_revocations.slice(0, 5).map((audit) => (
                    <div key={audit.id} className="flex items-center justify-between p-2 border rounded">
                      <div className="flex-1">
                        <div className="text-sm font-medium">{audit.permission}</div>
                        <div className="text-xs text-muted-foreground">
                          {audit.user_id} • {formatDistanceToNow(new Date(audit.performed_at * 1000), { addSuffix: true })}
                        </div>
                      </div>
                      <UserX className="h-4 w-4 text-red-500" />
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      )}

      {/* Users Table */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>Users & Permissions ({filteredUsers.length})</CardTitle>
              <CardDescription>Manage permissions for all users</CardDescription>
            </div>
            <div className="flex gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => setShowFilters(!showFilters)}
              >
                <Filter className="h-4 w-4" />
                Filters
              </Button>
            </div>
          </div>
          
          {/* Search and Filters */}
          <div className="space-y-4">
            <div className="relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder="Search users, emails, or permissions..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pl-10"
              />
            </div>

            {showFilters && (
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4 p-4 border rounded">
                <div>
                  <label className="text-sm font-medium">Source</label>
                  <select 
                    className="w-full mt-1 p-2 border rounded"
                    value={filters.source || ''}
                    onChange={(e) => setFilters(prev => ({ ...prev, source: e.target.value as PermissionSource || undefined }))}
                  >
                    <option value="">All Sources</option>
                    <option value="Admin">Admin</option>
                    <option value="Subscription">Subscription</option>
                    <option value="Trial">Trial</option>
                    <option value="System">System</option>
                    <option value="Legacy">Legacy</option>
                  </select>
                </div>
                <div>
                  <label className="text-sm font-medium">Health Status</label>
                  <select 
                    className="w-full mt-1 p-2 border rounded"
                    value={filters.is_expired ? 'expired' : filters.has_expiring_soon ? 'expiring' : ''}
                    onChange={(e) => {
                      if (e.target.value === 'expired') {
                        setFilters(prev => ({ ...prev, is_expired: true, has_expiring_soon: undefined }));
                      } else if (e.target.value === 'expiring') {
                        setFilters(prev => ({ ...prev, has_expiring_soon: true, is_expired: undefined }));
                      } else {
                        setFilters(prev => ({ ...prev, is_expired: undefined, has_expiring_soon: undefined }));
                      }
                    }}
                  >
                    <option value="">All Users</option>
                    <option value="expiring">Expiring Soon</option>
                    <option value="expired">Has Expired</option>
                  </select>
                </div>
                <div>
                  <label className="text-sm font-medium">Permission Pattern</label>
                  <Input
                    placeholder="e.g., admin:*, epsx:analytics:*"
                    value={filters.permission_pattern || ''}
                    onChange={(e) => setFilters(prev => ({ ...prev, permission_pattern: e.target.value || undefined }))}
                    className="mt-1"
                  />
                </div>
              </div>
            )}
          </div>
        </CardHeader>
        <CardContent>
          <div className="rounded-md border">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>User</TableHead>
                  <TableHead>Permissions</TableHead>
                  <TableHead>Health Score</TableHead>
                  <TableHead>Last Activity</TableHead>
                  <TableHead>Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {filteredUsers.map((user) => (
                  <TableRow key={user.user_id}>
                    <TableCell>
                      <div>
                        <div className="font-medium">{user.display_name || user.email}</div>
                        <div className="text-xs text-muted-foreground">{user.user_id}</div>
                      </div>
                    </TableCell>
                    <TableCell>
                      <div className="space-y-1">
                        <div className="text-sm">
                          Total: {user.health.total_permissions} 
                          <span className="text-muted-foreground mx-1">•</span>
                          Active: {user.health.active_permissions}
                        </div>
                        {user.health.expired_permissions > 0 && (
                          <Badge variant="destructive" className="text-xs">
                            {user.health.expired_permissions} expired
                          </Badge>
                        )}
                        {user.health.expiring_soon_permissions > 0 && (
                          <Badge variant="destructive" className="text-xs bg-yellow-100 text-yellow-800">
                            {user.health.expiring_soon_permissions} expiring
                          </Badge>
                        )}
                      </div>
                    </TableCell>
                    <TableCell>
                      <div className="flex items-center gap-2">
                        <span className={`text-sm font-medium ${getHealthStatusColor(user.health.health_score)}`}>
                          {user.health.health_score}%
                        </span>
                        {getHealthStatusBadge(user.health.health_score)}
                      </div>
                    </TableCell>
                    <TableCell>
                      <div className="text-sm text-muted-foreground">
                        {user.last_activity ? 
                          formatDistanceToNow(new Date(user.last_activity * 1000), { addSuffix: true }) :
                          'Unknown'
                        }
                      </div>
                    </TableCell>
                    <TableCell>
                      <Dialog>
                        <DialogTrigger asChild>
                          <Button 
                            size="sm" 
                            variant="outline"
                            onClick={() => setSelectedUser(user.user_id)}
                          >
                            <Settings className="h-3 w-3" />
                            Manage
                          </Button>
                        </DialogTrigger>
                        <DialogContent className="max-w-6xl max-h-[90vh] overflow-y-auto">
                          <DialogHeader>
                            <DialogTitle>Manage Permissions</DialogTitle>
                            <DialogDescription>
                              Manage permissions for {user.display_name || user.email}
                            </DialogDescription>
                          </DialogHeader>
                          {selectedUser && (
                            <GranularPermissionManager
                              userId={selectedUser}
                              userEmail={user.email}
                              userName={user.display_name}
                              showAuditLog={true}
                            />
                          )}
                        </DialogContent>
                      </Dialog>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
            
            {filteredUsers.length === 0 && (
              <div className="text-center py-8 text-muted-foreground">
                No users found matching your criteria.
              </div>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

export default AdminPermissionDashboard;