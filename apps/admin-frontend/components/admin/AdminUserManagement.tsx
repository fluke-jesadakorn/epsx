'use client';

import { useState, useEffect } from 'react';
import { AdminService, type AdminUser } from '@/services/adminService';
import { UserLevel, USER_LEVEL_CONFIGS } from '@/types/admin/userLevels';
import { 
  Users, 
  Search, 
  CheckCircle, 
  AlertTriangle,
  UserX,
  RefreshCw,
  Crown,
  History,
  Star
} from 'lucide-react';

export function AdminUserManagement() {
  const [users, setUsers] = useState<AdminUser[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [filterRole, setFilterRole] = useState<string>('all');
  const [filterStatus, setFilterStatus] = useState<string>('all');
  const [actionLoading, setActionLoading] = useState<string | null>(null);
  
  // User level assignment state
  const [showLevelModal, setShowLevelModal] = useState(false);
  const [selectedUser, setSelectedUser] = useState<AdminUser | null>(null);
  const [selectedLevel, setSelectedLevel] = useState<UserLevel>(UserLevel.BRONZE);
  const [levelReason, setLevelReason] = useState('');
  const [showLevelHistory, setShowLevelHistory] = useState(false);
  const [levelHistory, setLevelHistory] = useState<any[]>([]);

  useEffect(() => {
    loadUsers();
  }, []);

  const loadUsers = async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await AdminService.listUsers({ maxResults: 1000 });
      setUsers(result.users);
    } catch (err: any) {
      console.error('Failed to load users:', err);
      setError(err.message || 'Failed to load users');
    } finally {
      setLoading(false);
    }
  };

  const handleRoleChange = async (uid: string, newRole: string) => {
    try {
      setActionLoading(uid);
      await AdminService.setUserRole(uid, newRole);
      await loadUsers(); // Refresh the list
    } catch (err: any) {
      console.error('Failed to change role:', err);
      alert('Failed to change user role: ' + err.message);
    } finally {
      setActionLoading(null);
    }
  };

  const handleStatusToggle = async (uid: string, disabled: boolean) => {
    try {
      setActionLoading(uid);
      await AdminService.updateUserStatus(uid, !disabled);
      await loadUsers(); // Refresh the list
    } catch (err: any) {
      console.error('Failed to toggle status:', err);
      alert('Failed to update user status: ' + err.message);
    } finally {
      setActionLoading(null);
    }
  };

  const handleDeleteUser = async (uid: string, email: string) => {
    if (!confirm(`Are you sure you want to delete user ${email}? This action cannot be undone.`)) {
      return;
    }

    try {
      setActionLoading(uid);
      await AdminService.deleteUser(uid);
      await loadUsers(); // Refresh the list
    } catch (err: any) {
      console.error('Failed to delete user:', err);
      alert('Failed to delete user: ' + err.message);
    } finally {
      setActionLoading(null);
    }
  };

  const handleSendPasswordReset = async (email: string) => {
    try {
      setActionLoading(email);
      await AdminService.sendPasswordResetEmail(email);
      alert('Password reset link generated. Check console for link.');
    } catch (err: any) {
      console.error('Failed to send password reset:', err);
      alert('Failed to send password reset: ' + err.message);
    } finally {
      setActionLoading(null);
    }
  };

  // User level assignment functions
  const handleLevelChange = async (uid: string, newLevel: UserLevel, reason?: string) => {
    try {
      setActionLoading(uid);
      await AdminService.setUserLevel(uid, newLevel, reason);
      await loadUsers(); // Refresh the list
      setShowLevelModal(false);
      setLevelReason('');
    } catch (err: any) {
      console.error('Failed to change user level:', err);
      alert('Failed to change user level: ' + err.message);
    } finally {
      setActionLoading(null);
    }
  };

  const openLevelModal = (user: AdminUser) => {
    setSelectedUser(user);
    setSelectedLevel(user.customClaims?.userLevel || UserLevel.BRONZE);
    setShowLevelModal(true);
  };

  const handleShowLevelHistory = async (uid: string) => {
    try {
      const history = await AdminService.getUserLevelHistory(uid);
      setLevelHistory(history);
      setShowLevelHistory(true);
    } catch (err: any) {
      console.error('Failed to fetch level history:', err);
      alert('Failed to fetch level history: ' + err.message);
    }
  };

  const getUserLevelBadge = (user: AdminUser) => {
    const userLevel = user.customClaims?.userLevel || UserLevel.BRONZE;
    const config = USER_LEVEL_CONFIGS[userLevel];
    
    return (
      <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${config.color}`}>
        <Crown className="h-3 w-3 mr-1" />
        {config.name}
      </span>
    );
  };

  // Filter users based on search and filters
  const filteredUsers = users.filter(user => {
    const matchesSearch = user.email.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         (user.displayName?.toLowerCase().includes(searchTerm.toLowerCase()));
    
    const matchesRole = filterRole === 'all' || 
                       (filterRole === 'admin' && user.customClaims?.role === 'ADMIN') ||
                       (filterRole === 'user' && (user.customClaims?.role === 'USER' || !user.customClaims?.role));
    
    const matchesStatus = filterStatus === 'all' ||
                         (filterStatus === 'verified' && user.emailVerified) ||
                         (filterStatus === 'unverified' && !user.emailVerified) ||
                         (filterStatus === 'disabled' && user.disabled) ||
                         (filterStatus === 'active' && !user.disabled);

    return matchesSearch && matchesRole && matchesStatus;
  });

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
        <div className="flex items-center gap-2 text-red-800">
          <AlertTriangle className="h-5 w-5" />
          <span>Error: {error}</span>
        </div>
        <button
          onClick={loadUsers}
          className="mt-2 px-3 py-1 text-sm bg-red-600 text-white rounded hover:bg-red-700"
        >
          Retry
        </button>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
            User Management
          </h1>
          <p className="text-gray-600 dark:text-gray-400">
            Manage user accounts, roles, and permissions
          </p>
        </div>
        <button
          onClick={loadUsers}
          disabled={loading}
          className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
        >
          <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
          Refresh
        </button>
      </div>

      {/* Filters */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-4">
        <div className="flex flex-col sm:flex-row gap-4">
          {/* Search */}
          <div className="flex-1">
            <div className="relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
              <input
                type="text"
                placeholder="Search users by email or name..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              />
            </div>
          </div>

          {/* Role Filter */}
          <div className="sm:w-48">
            <select
              value={filterRole}
              onChange={(e) => setFilterRole(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            >
              <option value="all">All Roles</option>
              <option value="admin">Admin</option>
              <option value="user">User</option>
            </select>
          </div>

          {/* Status Filter */}
          <div className="sm:w-48">
            <select
              value={filterStatus}
              onChange={(e) => setFilterStatus(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            >
              <option value="all">All Status</option>
              <option value="verified">Verified</option>
              <option value="unverified">Unverified</option>
              <option value="active">Active</option>
              <option value="disabled">Disabled</option>
            </select>
          </div>
        </div>
      </div>

      {/* Results Summary */}
      <div className="flex items-center justify-between text-sm text-gray-600 dark:text-gray-400">
        <span>Showing {filteredUsers.length} of {users.length} users</span>
      </div>

      {/* Users Table */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 overflow-hidden">
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
            <thead className="bg-gray-50 dark:bg-gray-700">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                  User
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                  Role
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                  User Level
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                  Status
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                  Last Sign In
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                  Actions
                </th>
              </tr>
            </thead>
            <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
              {filteredUsers.map((user) => (
                <tr key={user.uid} className="hover:bg-gray-50 dark:hover:bg-gray-700">
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="flex items-center">
                      <div className="h-10 w-10 rounded-full bg-gray-200 dark:bg-gray-600 flex items-center justify-center">
                        <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                          {user.email.charAt(0).toUpperCase()}
                        </span>
                      </div>
                      <div className="ml-4">
                        <div className="text-sm font-medium text-gray-900 dark:text-white">
                          {user.displayName || 'No name'}
                        </div>
                        <div className="text-sm text-gray-500 dark:text-gray-400">
                          {user.email}
                        </div>
                        <div className="text-xs text-gray-400 dark:text-gray-500">
                          ID: {user.uid.substring(0, 8)}...
                        </div>
                      </div>
                    </div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <select
                      value={user.customClaims?.role || 'USER'}
                      onChange={(e) => handleRoleChange(user.uid, e.target.value)}
                      disabled={actionLoading === user.uid}
                      className="text-xs px-2 py-1 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                    >
                      <option value="USER">User</option>
                      <option value="ADMIN">Admin</option>
                    </select>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="flex items-center gap-2">
                      {getUserLevelBadge(user)}
                      <button
                        onClick={() => openLevelModal(user)}
                        disabled={actionLoading === user.uid}
                        className="text-xs text-blue-600 hover:text-blue-800 disabled:opacity-50"
                        title="Assign Level"
                      >
                        <Star className="h-3 w-3" />
                      </button>
                      <button
                        onClick={() => handleShowLevelHistory(user.uid)}
                        disabled={actionLoading === user.uid}
                        className="text-xs text-gray-600 hover:text-gray-800 disabled:opacity-50"
                        title="View History"
                      >
                        <History className="h-3 w-3" />
                      </button>
                    </div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="flex flex-col gap-1">
                      {user.emailVerified ? (
                        <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200">
                          <CheckCircle className="h-3 w-3 mr-1" />
                          Verified
                        </span>
                      ) : (
                        <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200">
                          <AlertTriangle className="h-3 w-3 mr-1" />
                          Unverified
                        </span>
                      )}
                      {user.disabled && (
                        <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200">
                          <UserX className="h-3 w-3 mr-1" />
                          Disabled
                        </span>
                      )}
                    </div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                    {user.metadata.lastSignInTime 
                      ? new Date(user.metadata.lastSignInTime).toLocaleDateString()
                      : 'Never'
                    }
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm">
                    <div className="flex items-center gap-2">
                      <button
                        onClick={() => handleStatusToggle(user.uid, user.disabled)}
                        disabled={actionLoading === user.uid}
                        className={`px-2 py-1 text-xs rounded ${
                          user.disabled 
                            ? 'bg-green-100 text-green-800 hover:bg-green-200' 
                            : 'bg-yellow-100 text-yellow-800 hover:bg-yellow-200'
                        } disabled:opacity-50`}
                      >
                        {user.disabled ? 'Enable' : 'Disable'}
                      </button>
                      <button
                        onClick={() => handleSendPasswordReset(user.email)}
                        disabled={actionLoading === user.email}
                        className="px-2 py-1 text-xs bg-blue-100 text-blue-800 rounded hover:bg-blue-200 disabled:opacity-50"
                      >
                        Reset Password
                      </button>
                      <button
                        onClick={() => handleDeleteUser(user.uid, user.email)}
                        disabled={actionLoading === user.uid}
                        className="px-2 py-1 text-xs bg-red-100 text-red-800 rounded hover:bg-red-200 disabled:opacity-50"
                      >
                        Delete
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {filteredUsers.length === 0 && (
        <div className="text-center py-12">
          <Users className="mx-auto h-12 w-12 text-gray-400" />
          <h3 className="mt-2 text-sm font-medium text-gray-900 dark:text-white">
            No users found
          </h3>
          <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
            Try adjusting your search or filter criteria.
          </p>
        </div>
      )}

      {/* User Level Assignment Modal */}
      {showLevelModal && selectedUser && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md">
            <h3 className="text-lg font-semibold mb-4">Assign User Level</h3>
            
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium mb-1">User</label>
                <p className="text-sm text-gray-600">{selectedUser.email}</p>
              </div>

              <div>
                <label className="block text-sm font-medium mb-1">Select Level</label>
                <select
                  value={selectedLevel}
                  onChange={(e) => setSelectedLevel(e.target.value as UserLevel)}
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                >
                  {Object.values(UserLevel).map((level) => {
                    const config = USER_LEVEL_CONFIGS[level];
                    return (
                      <option key={level} value={level}>
                        {config.name} - {config.description}
                      </option>
                    );
                  })}
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium mb-1">Level Details</label>
                <div className="text-sm text-gray-600 space-y-1">
                  <p>Token Multiplier: {USER_LEVEL_CONFIGS[selectedLevel].tokenMultiplier}x</p>
                  <p>Max Tokens: {USER_LEVEL_CONFIGS[selectedLevel].maxTokens === -1 ? 'Unlimited' : USER_LEVEL_CONFIGS[selectedLevel].maxTokens}</p>
                  <p>Benefits: {USER_LEVEL_CONFIGS[selectedLevel].benefits.join(', ')}</p>
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium mb-1">Reason (Optional)</label>
                <textarea
                  value={levelReason}
                  onChange={(e) => setLevelReason(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                  rows={3}
                  placeholder="Enter reason for level change..."
                />
              </div>
            </div>

            <div className="flex justify-end gap-3 mt-6">
              <button
                onClick={() => setShowLevelModal(false)}
                className="px-4 py-2 text-gray-600 hover:text-gray-800"
              >
                Cancel
              </button>
              <button
                onClick={() => handleLevelChange(selectedUser.uid, selectedLevel, levelReason)}
                disabled={actionLoading === selectedUser.uid}
                className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
              >
                Assign Level
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Level History Modal */}
      {showLevelHistory && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-2xl">
            <h3 className="text-lg font-semibold mb-4">User Level History</h3>
            
            <div className="space-y-3 max-h-96 overflow-y-auto">
              {levelHistory.map((entry, index) => (
                <div key={index} className="border-l-4 border-blue-500 pl-4 py-2">
                  <div className="flex items-center gap-2">
                    <span className={`px-2 py-1 rounded text-xs font-medium ${USER_LEVEL_CONFIGS[entry.userLevel as UserLevel].color}`}>
                      {USER_LEVEL_CONFIGS[entry.userLevel as UserLevel].name}
                    </span>
                    <span className="text-sm text-gray-500">
                      {new Date(entry.assignedAt).toLocaleDateString()}
                    </span>
                  </div>
                  <p className="text-sm text-gray-600 mt-1">
                    Assigned by: {entry.assignedBy}
                  </p>
                  {entry.reason && (
                    <p className="text-sm text-gray-500 mt-1">
                      Reason: {entry.reason}
                    </p>
                  )}
                </div>
              ))}
            </div>

            <div className="flex justify-end mt-6">
              <button
                onClick={() => setShowLevelHistory(false)}
                className="px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
