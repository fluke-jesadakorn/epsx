'use client';

import { useState, useEffect, useCallback } from 'react';
import {
  Users,
  Search,
  Shield,
  Clock,
  AlertTriangle,
  CheckCircle,
  Plus,
  Settings,
  Filter,
  RefreshCw,
  Calendar,
  Key,
  Eye,
  Edit3,
  Trash2,
  MoreHorizontal
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { AdminApiService } from '@/services/adminApiService';
import { EmbeddedPermissionsApi, EmbeddedPermissionHelpers } from '@/lib/api/embedded-permissions';
import { GrantPermissionModal } from './GrantPermissionModal';
import { BulkPermissionsModal } from './BulkPermissionsModal';
// Windows Phone + Pancakeswap colors
const theme = {
  pancake: 'linear-gradient(135deg, #FFD800 0%, #FFA726 100%)',
  blue: '#0078D4',
  green: '#107C10', 
  orange: '#FF8C00',
  red: '#D13438',
  dark: '#000000',
  surface: '#1A1A1A',
  border: '#404040'
};
import type {
  ExpiryStatusResponse,
  PermissionExpiryInfo,
  PermissionWithHealth,
  UserPermissionSummary
} from '@/types/admin/embedded-permissions';

interface User {
  id: string;
  firebase_uid?: string;
  email: string;
  firstName?: string;
  lastName?: string;
  role: string;
  isActive: boolean;
}

interface UserPermissionData extends User {
  permissionSummary?: UserPermissionSummary;
  expiryStatus?: ExpiryStatusResponse;
}

export function UserPermissionsHub() {
  const [users, setUsers] = useState<UserPermissionData[]>([]);
  const [selectedUser, setSelectedUser] = useState<UserPermissionData | null>(null);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [filterStatus, setFilterStatus] = useState<'all' | 'healthy' | 'expiring' | 'expired'>('all');
  const [showGrantModal, setShowGrantModal] = useState(false);
  const [showBulkModal, setShowBulkModal] = useState(false);

  // Load users and their permission data
  const loadUsers = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      // Get all users from admin API
      const usersData = await AdminApiService.getUsers();
      
      // Load permission data for each user
      const usersWithPermissions = await Promise.all(
        usersData.map(async (user: User) => {
          try {
            const expiryStatus = await EmbeddedPermissionsApi.getExpiryStatus(user.firebase_uid || user.id);
            const permissionSummary: UserPermissionSummary = {
              user_id: user.id,
              total_permissions: expiryStatus.permissions.length,
              active_permissions: expiryStatus.permissions.filter(p => !p.is_expired).length,
              expired_permissions: expiryStatus.permissions.filter(p => p.is_expired).length,
              expiring_soon_permissions: expiryStatus.permissions.filter(p => 
                p.time_remaining && p.time_remaining <= 24 * 60 * 60 * 1000 // 24 hours
              ).length,
              health_score: EmbeddedPermissionHelpers.calculateHealthScore(
                expiryStatus.permissions.map(p => p.permission)
              )
            };

            return {
              ...user,
              expiryStatus,
              permissionSummary
            };
          } catch (err) {
            console.warn(`Failed to load permission data for user ${user.email}:`, err);
            return {
              ...user,
              permissionSummary: {
                user_id: user.id,
                total_permissions: 0,
                active_permissions: 0,
                expired_permissions: 0,
                expiring_soon_permissions: 0,
                health_score: 100
              }
            };
          }
        })
      );

      setUsers(usersWithPermissions);
    } catch (err: any) {
      console.error('Failed to load users:', err);
      setError(err.message || 'Failed to load users');
    } finally {
      setLoading(false);
    }
  }, []);

  // Refresh user permission data
  const refreshUserData = useCallback(async (userId?: string) => {
    try {
      setRefreshing(true);
      
      if (userId) {
        // Refresh specific user
        const userIndex = users.findIndex(u => u.id === userId || u.firebase_uid === userId);
        if (userIndex >= 0) {
          const user = users[userIndex];
          const expiryStatus = await EmbeddedPermissionsApi.getExpiryStatus(user.firebase_uid || user.id);
          const permissionSummary: UserPermissionSummary = {
            user_id: user.id,
            total_permissions: expiryStatus.permissions.length,
            active_permissions: expiryStatus.permissions.filter(p => !p.is_expired).length,
            expired_permissions: expiryStatus.permissions.filter(p => p.is_expired).length,
            expiring_soon_permissions: expiryStatus.permissions.filter(p => 
              p.time_remaining && p.time_remaining <= 24 * 60 * 60 * 1000
            ).length,
            health_score: EmbeddedPermissionHelpers.calculateHealthScore(
              expiryStatus.permissions.map(p => p.permission)
            )
          };

          const updatedUsers = [...users];
          updatedUsers[userIndex] = {
            ...user,
            expiryStatus,
            permissionSummary
          };
          setUsers(updatedUsers);
          
          // Update selected user if it's the same one
          if (selectedUser && (selectedUser.id === userId || selectedUser.firebase_uid === userId)) {
            setSelectedUser(updatedUsers[userIndex]);
          }
        }
      } else {
        // Refresh all users
        await loadUsers();
      }
    } catch (err: any) {
      console.error('Failed to refresh user data:', err);
      setError(err.message || 'Failed to refresh user data');
    } finally {
      setRefreshing(false);
    }
  }, [users, selectedUser, loadUsers]);

  useEffect(() => {
    loadUsers();
  }, [loadUsers]);

  // Filter users based on search and status
  const filteredUsers = users.filter(user => {
    const matchesSearch = !searchTerm || 
      user.email.toLowerCase().includes(searchTerm.toLowerCase()) ||
      `${user.firstName || ''} ${user.lastName || ''}`.toLowerCase().includes(searchTerm.toLowerCase());
    
    if (!matchesSearch) return false;

    if (filterStatus === 'all') return true;
    
    const summary = user.permissionSummary;
    if (!summary) return filterStatus === 'healthy';
    
    switch (filterStatus) {
      case 'healthy':
        return summary.health_score >= 80;
      case 'expiring':
        return summary.expiring_soon_permissions > 0;
      case 'expired':
        return summary.expired_permissions > 0;
      default:
        return true;
    }
  });

  const getHealthStatusColor = (score: number) => {
    if (score >= 80) return 'text-green-600 bg-green-100 dark:bg-green-900/20';
    if (score >= 50) return 'text-yellow-600 bg-yellow-100 dark:bg-yellow-900/20';
    return 'text-red-600 bg-red-100 dark:bg-red-900/20';
  };

  const getHealthStatusIcon = (score: number) => {
    if (score >= 80) return <CheckCircle className="w-4 h-4" />;
    if (score >= 50) return <AlertTriangle className="w-4 h-4" />;
    return <AlertTriangle className="w-4 h-4" />;
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-64">
        <div className="animate-spin rounded-full h-10 w-10 border-4 border-primary/20 border-t-primary"></div>
      </div>
    );
  }

  return (
    <div 
      className="min-h-screen p-4 md:p-6" 
      style={{ background: theme.dark }}
    >
      {/* Header Tile */}
      <div 
        className="mb-6 p-6 transition-all duration-300 hover:scale-[1.02] cursor-pointer"
        style={{ background: theme.pancake }}
      >
        <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
          <div className="text-black">
            <h1 className="text-2xl md:text-4xl font-light mb-2">
              permissions
            </h1>
            <div className="flex items-center gap-4 text-sm">
              <span>{users.length} users</span>
              <span>•</span>
              <span>{users.filter(u => u.permissionSummary?.health_score >= 80).length} healthy</span>
            </div>
          </div>
          <div className="flex gap-2">
            <button
              onClick={() => refreshUserData()}
              disabled={refreshing}
              className="w-12 h-12 bg-black/20 hover:bg-black/30 transition-colors flex items-center justify-center"
            >
              <RefreshCw className={`w-5 h-5 text-black ${refreshing ? 'animate-spin' : ''}`} />
            </button>
            <button
              onClick={() => setShowBulkModal(true)}
              className="w-12 h-12 bg-black/20 hover:bg-black/30 transition-colors flex items-center justify-center"
            >
              <Settings className="w-5 h-5 text-black" />
            </button>
            <button
              onClick={() => setShowGrantModal(true)}
              className="w-12 h-12 bg-black/20 hover:bg-black/30 transition-colors flex items-center justify-center"
            >
              <Plus className="w-5 h-5 text-black" />
            </button>
          </div>
        </div>
      </div>

      {/* Stats Tiles Grid */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
        <div 
          className="w-40 h-40 p-4 transition-all duration-300 hover:scale-105 cursor-pointer relative overflow-hidden text-white"
          style={{ background: theme.blue }}
        >
          <div className="absolute inset-0 bg-gradient-to-br from-transparent to-black/20"></div>
          <div className="relative z-10 h-full flex flex-col justify-between">
            <Users className="w-8 h-8" />
            <div>
              <div className="text-3xl font-light">{users.length}</div>
              <div className="text-sm opacity-80">total</div>
            </div>
          </div>
        </div>

        <div 
          className="w-40 h-40 p-4 transition-all duration-300 hover:scale-105 cursor-pointer relative overflow-hidden text-white"
          style={{ background: theme.green }}
        >
          <div className="absolute inset-0 bg-gradient-to-br from-transparent to-black/20"></div>
          <div className="relative z-10 h-full flex flex-col justify-between">
            <CheckCircle className="w-8 h-8" />
            <div>
              <div className="text-3xl font-light">
                {users.filter(u => u.permissionSummary?.health_score >= 80).length}
              </div>
              <div className="text-sm opacity-80">healthy</div>
            </div>
          </div>
        </div>

        <div 
          className="w-40 h-40 p-4 transition-all duration-300 hover:scale-105 cursor-pointer relative overflow-hidden text-white"
          style={{ background: theme.orange }}
        >
          <div className="absolute inset-0 bg-gradient-to-br from-transparent to-black/20"></div>
          <div className="relative z-10 h-full flex flex-col justify-between">
            <Clock className="w-8 h-8" />
            <div>
              <div className="text-3xl font-light">
                {users.filter(u => u.permissionSummary?.expiring_soon_permissions > 0).length}
              </div>
              <div className="text-sm opacity-80">expiring</div>
            </div>
          </div>
        </div>

        <div 
          className="w-40 h-40 p-4 transition-all duration-300 hover:scale-105 cursor-pointer relative overflow-hidden text-white"
          style={{ background: theme.red }}
        >
          <div className="absolute inset-0 bg-gradient-to-br from-transparent to-black/20"></div>
          <div className="relative z-10 h-full flex flex-col justify-between">
            <AlertTriangle className="w-8 h-8" />
            <div>
              <div className="text-3xl font-light">
                {users.filter(u => u.permissionSummary?.expired_permissions > 0).length}
              </div>
              <div className="text-sm opacity-80">expired</div>
            </div>
          </div>
        </div>
      </div>

      {/* Filter Tiles */}
      <div className="flex flex-wrap gap-2 mb-6">
        {[
          { key: 'all', label: 'all', icon: Users, color: theme.surface },
          { key: 'healthy', label: 'healthy', icon: CheckCircle, color: theme.green },
          { key: 'expiring', label: 'expiring', icon: Clock, color: theme.orange },
          { key: 'expired', label: 'expired', icon: AlertTriangle, color: theme.red }
        ].map(({ key, label, icon: Icon, color }) => (
          <button
            key={key}
            onClick={() => setFilterStatus(key as any)}
            className={`px-4 py-2 text-sm font-light text-white transition-all duration-300 hover:scale-105 ${
              filterStatus === key
                ? 'ring-2 ring-white/50'
                : 'opacity-70 hover:opacity-100'
            }`}
            style={{ background: color }}
          >
            <Icon className="w-4 h-4 inline mr-2" />
            {label}
          </button>
        ))}
      </div>

      {/* Search Tile */}
      <div 
        className="mb-6 p-4"
        style={{ 
          background: theme.surface,
          border: `1px solid ${theme.border}`
        }}
      >
        <div className="relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-4 h-4" />
          <input
            type="text"
            placeholder="search users..."
            value={searchTerm}
            onChange={e => setSearchTerm(e.target.value)}
            className="w-full pl-10 pr-4 py-3 bg-transparent text-white placeholder-gray-400 border-none outline-none font-light"
          />
        </div>
      </div>

      {/* Users Tile Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
        {filteredUsers.map(user => {
          const summary = user.permissionSummary;
          const healthScore = summary?.health_score || 100;
          const getTileColor = () => {
            if (healthScore >= 80) return theme.blue;
            if (healthScore >= 50) return theme.orange;
            return theme.red;
          };
          
          return (
            <div
              key={user.id}
              className="w-80 h-40 p-4 transition-all duration-300 hover:scale-105 cursor-pointer relative overflow-hidden text-white"
              style={{ background: getTileColor() }}
              onClick={() => setSelectedUser(user)}
            >
              <div className="absolute inset-0 bg-gradient-to-br from-transparent to-black/20"></div>
              <div className="relative z-10 h-full flex flex-col justify-between">
                <div className="flex items-start justify-between">
                  <div className="w-12 h-12 bg-white/20 flex items-center justify-center">
                    <span className="text-lg font-light">
                      {(user.firstName?.[0] || user.email[0]).toUpperCase()}
                    </span>
                  </div>
                  <div className="text-right">
                    <div className="text-2xl font-light">{healthScore}%</div>
                    <div className="text-xs opacity-80">health</div>
                  </div>
                </div>
                <div>
                  <div className="font-light text-lg mb-1">
                    {user.firstName && user.lastName 
                      ? `${user.firstName} ${user.lastName}`
                      : user.email.split('@')[0]
                    }
                  </div>
                  <div className="flex items-center gap-4 text-sm opacity-80">
                    <span>{summary?.total_permissions || 0} total</span>
                    <span>{summary?.active_permissions || 0} active</span>
                    {summary?.expired_permissions > 0 && (
                      <span>{summary.expired_permissions} expired</span>
                    )}
                  </div>
                </div>
                <div className="absolute top-2 right-2 flex gap-1">
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      setSelectedUser(user);
                      setShowGrantModal(true);
                    }}
                    className="w-8 h-8 bg-white/20 hover:bg-white/30 transition-colors flex items-center justify-center"
                  >
                    <Plus className="w-4 h-4" />
                  </button>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      refreshUserData(user.firebase_uid || user.id);
                    }}
                    className="w-8 h-8 bg-white/20 hover:bg-white/30 transition-colors flex items-center justify-center"
                  >
                    <RefreshCw className="w-4 h-4" />
                  </button>
                </div>
              </div>
            </div>
          );
        })}
      </div>

      {filteredUsers.length === 0 && (
        <div className="text-center py-16 text-gray-400">
          <Users className="mx-auto h-16 w-16 mb-4 opacity-50" />
          <h3 className="text-xl font-light mb-2">no users found</h3>
          <p className="text-sm opacity-70">try adjusting your search or filter</p>
        </div>
      )}

      {/* Error Display */}
      {error && (
        <div 
          className="mt-6 p-4 transition-all duration-300 text-white"
          style={{ background: theme.red }}
        >
          <p className="font-light mb-3">{error}</p>
          <button
            onClick={loadUsers}
            className="px-6 py-2 bg-black/20 hover:bg-black/30 transition-colors font-light"
          >
            retry
          </button>
        </div>
      )}

      {/* User Detail Modal */}
      {selectedUser && (
        <div className="fixed inset-0 bg-black/80 flex items-center justify-center z-50 p-4">
          <div 
            className="max-w-4xl w-full max-h-[90vh] overflow-y-auto"
            style={{ background: theme.surface }}
          >
            {/* Modal Header */}
            <div 
              className="p-6"
              style={{ background: theme.pancake }}
            >
              <div className="flex justify-between items-center">
                <div className="text-black">
                  <h3 className="text-2xl font-light">
                    {selectedUser.firstName && selectedUser.lastName 
                      ? `${selectedUser.firstName} ${selectedUser.lastName}`
                      : selectedUser.email.split('@')[0]
                    }
                  </h3>
                  <p className="opacity-80">{selectedUser.email}</p>
                </div>
                <button
                  onClick={() => setSelectedUser(null)}
                  className="w-12 h-12 bg-black/20 hover:bg-black/30 transition-colors flex items-center justify-center text-black"
                >
                  <span className="text-xl">×</span>
                </button>
              </div>
            </div>
            
            <div className="p-6 space-y-6">
              {selectedUser.expiryStatus && (
                <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
                  <div 
                    className="w-32 h-32 p-4 text-white"
                    style={{ background: theme.blue }}
                  >
                    <div className="h-full flex flex-col justify-between">
                      <Key className="w-6 h-6" />
                      <div>
                        <div className="text-2xl font-light">
                          {selectedUser.expiryStatus.permissions.length}
                        </div>
                        <div className="text-sm opacity-80">total</div>
                      </div>
                    </div>
                  </div>
                  
                  <div 
                    className="w-32 h-32 p-4 text-white"
                    style={{ background: theme.green }}
                  >
                    <div className="h-full flex flex-col justify-between">
                      <CheckCircle className="w-6 h-6" />
                      <div>
                        <div className="text-2xl font-light">
                          {selectedUser.expiryStatus.permissions.filter(p => !p.is_expired).length}
                        </div>
                        <div className="text-sm opacity-80">active</div>
                      </div>
                    </div>
                  </div>
                  
                  <div 
                    className="w-32 h-32 p-4 text-white"
                    style={{ background: theme.orange }}
                  >
                    <div className="h-full flex flex-col justify-between">
                      <Clock className="w-6 h-6" />
                      <div>
                        <div className="text-2xl font-light">
                          {selectedUser.permissionSummary?.expiring_soon_permissions || 0}
                        </div>
                        <div className="text-sm opacity-80">expiring</div>
                      </div>
                    </div>
                  </div>
                  
                  <div 
                    className="w-32 h-32 p-4 text-white"
                    style={{ background: theme.red }}
                  >
                    <div className="h-full flex flex-col justify-between">
                      <AlertTriangle className="w-6 h-6" />
                      <div>
                        <div className="text-2xl font-light">
                          {selectedUser.expiryStatus.permissions.filter(p => p.is_expired).length}
                        </div>
                        <div className="text-sm opacity-80">expired</div>
                      </div>
                    </div>
                  </div>
                </div>
              )}

              {/* Permissions List */}
              {selectedUser.expiryStatus && selectedUser.expiryStatus.permissions.length > 0 && (
                <div>
                  <h4 className="text-lg font-light text-white mb-4">permissions</h4>
                  <div className="space-y-3 max-h-96 overflow-y-auto">
                    {selectedUser.expiryStatus.permissions.map((perm, index) => (
                      <div
                        key={index}
                        className={`p-4 border-l-4`}
                        style={{
                          background: perm.is_expired 
                            ? `${theme.red}20`
                            : perm.time_remaining && perm.time_remaining <= 24 * 60 * 60 * 1000
                            ? `${theme.orange}20`
                            : `${theme.green}20`,
                          borderLeftColor: perm.is_expired 
                            ? theme.red
                            : perm.time_remaining && perm.time_remaining <= 24 * 60 * 60 * 1000
                            ? theme.orange
                            : theme.green
                        }}
                      >
                        <div className="text-white font-light">
                          {perm.base_permission}
                        </div>
                        <div className="text-sm mt-1 text-gray-400">
                          {perm.expires_at ? `Expires: ${new Date(perm.expires_at * 1000).toLocaleDateString()}` : 'Never expires'}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {(!selectedUser.expiryStatus || selectedUser.expiryStatus.permissions.length === 0) && (
                <div className="text-center py-16 text-gray-400">
                  <Shield className="mx-auto h-16 w-16 mb-4 opacity-50" />
                  <h3 className="text-xl font-light mb-2">no permissions</h3>
                  <p className="text-sm opacity-70">this user has no permissions assigned</p>
                </div>
              )}
            </div>
            
            <div 
              className="p-6 flex justify-end gap-3"
              style={{ borderTop: `1px solid ${theme.border}` }}
            >
              <button
                onClick={() => setSelectedUser(null)}
                className="px-6 py-3 bg-gray-600 hover:bg-gray-700 text-white font-light transition-colors"
              >
                close
              </button>
              <button
                onClick={() => setShowGrantModal(true)}
                className="px-6 py-3 font-light text-black transition-colors flex items-center gap-2"
                style={{ background: theme.pancake }}
              >
                <Plus className="w-4 h-4" />
                grant
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Grant Permission Modal */}
      <GrantPermissionModal
        isOpen={showGrantModal}
        onClose={() => {
          setShowGrantModal(false);
          setSelectedUser(null);
        }}
        selectedUser={selectedUser}
        existingPermissions={selectedUser?.expiryStatus?.permissions}
        onPermissionGranted={(userId) => {
          refreshUserData(userId);
        }}
      />

      {/* Bulk Operations Modal */}
      <BulkPermissionsModal
        isOpen={showBulkModal}
        onClose={() => setShowBulkModal(false)}
        users={users}
        onOperationComplete={() => {
          loadUsers();
        }}
      />
    </div>
  );
}