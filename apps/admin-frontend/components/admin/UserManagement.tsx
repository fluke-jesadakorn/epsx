'use client';

import { AdminService } from '@/services/adminService';
import type { AdminUser } from '@/services/adminService';
import { USER_LEVEL_CONFIGS, UserLevel } from '@/types/admin/userLevels';
import { assignPermissionProfileAction, softDeleteUserAction } from '@/lib/actions/admin';
import {
  AlertTriangle,
  CheckCircle,
  Crown,
  History,
  RefreshCw,
  Search,
  Star,
  Users,
  UserX,
  Loader2,
  _Trash2,
} from 'lucide-react';
import { useEffect, useState, useTransition } from 'react';

import { _ConfirmDialog } from '@/components/ui/ConfirmDialog';
import { useToast } from '@/components/ui/toast';
import { _Card, _CardContent, _CardDescription, _CardHeader, _CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Textarea } from '@/components/ui/textarea';

interface UserManagementProps {
  initialData?: {
    users: AdminUser[];
    total: number;
  };
}

export function UserManagement({ initialData }: UserManagementProps) {
  const [users, setUsers] = useState<AdminUser[]>(initialData?.users || []);
  const [loading, setLoading] = useState(!initialData);
  const [error, setError] = useState<string | null>(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [filterRole, setFilterRole] = useState<string>('all');
  const [filterStatus, setFilterStatus] = useState<string>('all');
  const [actionLoading, setActionLoading] = useState<string | null>(null);
  const [isPending, startTransition] = useTransition();
  const { addToast } = useToast();

  // Confirm dialog state
  const [_confirmDialog, _setConfirmDialog] = useState<{
    open: boolean;
    uid?: string;
    email?: string;
  }>({ open: false });

  // User level assignment state
  const [showLevelModal, setShowLevelModal] = useState(false);
  const [selectedUser, setSelectedUser] = useState<AdminUser | null>(null);
  const [selectedLevel, setSelectedLevel] = useState<UserLevel>(UserLevel.BRONZE);
  const [levelReason, setLevelReason] = useState('');
  const [showLevelHistory, setShowLevelHistory] = useState(false);
  const [levelHistory, setLevelHistory] = useState<any[]>([]);

  // Permission profile assignment state
  const [showProfileModal, setShowProfileModal] = useState(false);
  const [profileUserId, setProfileUserId] = useState<string>('');
  const [profileId, setProfileId] = useState<string>('');
  const [expiresAt, setExpiresAt] = useState<string>('');

  // Soft delete state
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [userToDelete, setUserToDelete] = useState<string>('');
  const [deleteReason, setDeleteReason] = useState<string>('');

  useEffect(() => {
    if (!initialData) {
      loadUsers();
    }
  }, [initialData]);

  // Handle escape key for modals
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        if (showLevelModal) {
          setShowLevelModal(false);
        } else if (showLevelHistory) {
          setShowLevelHistory(false);
        } else if (showProfileModal) {
          setShowProfileModal(false);
        } else if (deleteDialogOpen) {
          setDeleteDialogOpen(false);
        }
      }
    };
    
    if (showLevelModal || showLevelHistory || showProfileModal || deleteDialogOpen) {
      document.addEventListener('keydown', handleEscape);
      return () => document.removeEventListener('keydown', handleEscape);
    }
    
    return undefined;
  }, [showLevelModal, showLevelHistory, showProfileModal, deleteDialogOpen]);

  const loadUsers = async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await AdminService.listUsers({ maxResults: 1000 });
      setUsers(result.users);
    } catch (err: any) {
      console.error('Failed to load users', { error: err instanceof Error ? err.message : err }, 'UserManagement.loadUsers');
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
      console.error('Failed to change role', { error: err instanceof Error ? err.message : err, uid, newRole }, 'UserManagement.handleRoleChange');
      addToast({
        type: 'error',
        title: 'Failed to change user role',
        description: err.message,
      });
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
      console.error('Failed to toggle status', { error: err instanceof Error ? err.message : err, uid, disabled }, 'UserManagement.handleStatusToggle');
      addToast({
        type: 'error',
        title: 'Failed to update user status',
        description: err.message,
      });
    } finally {
      setActionLoading(null);
    }
  };

  const handleDeleteUser = (uid: string, _email: string) => {
    setUserToDelete(uid);
    setDeleteDialogOpen(true);
  };

  const handleConfirmDelete = async () => {
    if (!userToDelete) return;
    
    startTransition(async () => {
      const formData = new FormData();
      formData.append('userId', userToDelete);
      formData.append('reason', deleteReason || 'Deleted via admin interface');

      const result = await softDeleteUserAction(formData);
      
      if (result.success) {
        addToast({
          type: 'success',
          title: 'User deleted successfully',
          description: 'User has been soft deleted',
        });
        setDeleteDialogOpen(false);
        setUserToDelete('');
        setDeleteReason('');
        await loadUsers(); // Refresh the list
      } else {
        addToast({
          type: 'error',
          title: 'Failed to delete user',
          description: result.error || 'Unknown error occurred',
        });
      }
    });
  };

  const handleSendPasswordReset = async (email: string) => {
    try {
      setActionLoading(email);
      await AdminService.sendResetEmail(email);
      addToast({
        type: 'success',
        title: 'Password reset link generated',
        description: 'Check console for link',
      });
    } catch (err: any) {
      console.error('Failed to send password reset', { error: err instanceof Error ? err.message : err, email }, 'UserManagement.handleSendPasswordReset');
      addToast({
        type: 'error',
        title: 'Failed to send password reset',
        description: err.message,
      });
    } finally {
      setActionLoading(null);
    }
  };

  // User level assignment functions
  const handleLevelChange = async (uid: string, newLevel: UserLevel, reason?: string) => {
    try {
      setActionLoading(uid);
      await AdminService.setLevel(uid, newLevel, reason);
      await loadUsers(); // Refresh the list
      setShowLevelModal(false);
      setLevelReason('');
    } catch (err: any) {
      console.error('Failed to change user level', { error: err instanceof Error ? err.message : err, uid, newLevel, reason }, 'UserManagement.handleLevelChange');
      addToast({
        type: 'error',
        title: 'Failed to change user level',
        description: err.message,
      });
    } finally {
      setActionLoading(null);
    }
  };

  const openLevelModal = (user: AdminUser) => {
    setSelectedUser(user);
    setSelectedLevel(user.userLevel || UserLevel.BRONZE);
    setShowLevelModal(true);
  };

  const handleShowLevelHistory = async (uid: string) => {
    try {
      const history = await AdminService.getLevelHistory(uid);
      setLevelHistory(history);
      setShowLevelHistory(true);
    } catch (err: any) {
      console.error('Failed to fetch level history', { error: err instanceof Error ? err.message : err, uid }, 'UserManagement.handleShowLevelHistory');
      addToast({
        type: 'error',
        title: 'Failed to fetch level history',
        description: err.message,
      });
    }
  };

  // Permission profile assignment functions
  const handleAssignProfile = () => {
    if (!profileUserId || !profileId) {
      addToast({
        type: 'error',
        title: 'Missing Information',
        description: 'Please select both a user and permission profile',
      });
      return;
    }

    startTransition(async () => {
      const formData = new FormData();
      formData.append('userId', profileUserId);
      formData.append('profileId', profileId);
      if (expiresAt) {
        formData.append('expiresAt', expiresAt);
      }

      const result = await assignPermissionProfileAction(formData);
      
      if (result.success) {
        addToast({
          type: 'success',
          title: 'Permission profile assigned successfully',
          description: `Profile ${profileId} assigned to user`,
        });
        setShowProfileModal(false);
        setProfileUserId('');
        setProfileId('');
        setExpiresAt('');
        await loadUsers(); // Refresh the list
      } else {
        addToast({
          type: 'error',
          title: 'Failed to assign permission profile',
          description: result.error || 'Unknown error occurred',
        });
      }
    });
  };

  const openProfileModal = (uid: string) => {
    setProfileUserId(uid);
    setShowProfileModal(true);
  };

  const getUserLevelBadge = (user: AdminUser) => {
    const userLevel = user.userLevel || UserLevel.BRONZE;
    const config = USER_LEVEL_CONFIGS[userLevel];

    return (
      <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${config.color}`}>
        <Crown className="h-3 w-3 mr-1" />
        {config.name}
      </span>
    );
  };

  // Filter users based on search and filters
  const filteredUsers = users.filter((user) => {
    const matchesSearch =
      user.email.toLowerCase().includes(searchTerm.toLowerCase()) ||
      user.displayName?.toLowerCase().includes(searchTerm.toLowerCase());

    const matchesRole =
      filterRole === 'all' ||
      (filterRole === 'admin' && user.customClaims?.role === 'ADMIN') ||
      (filterRole === 'user' &&
        (user.customClaims?.role === 'USER' || !user.customClaims?.role));

    const matchesStatus =
      filterStatus === 'all' ||
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
            Manage user accounts, roles, permissions, and access levels
          </p>
        </div>
        <div className="flex gap-3">
          <button
            onClick={() => setShowProfileModal(true)}
            className="flex items-center gap-2 px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700"
          >
            <Users className="h-4 w-4" />
            Assign Profile
          </button>
          <button
            onClick={loadUsers}
            disabled={loading}
            className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
          >
            <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
            Refresh
          </button>
        </div>
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
        <span>
          Showing {filteredUsers.length} of {users.length} users
        </span>
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
                      : 'Never'}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm">
                    <div className="flex items-center gap-2">
                      <button
                        onClick={() => openProfileModal(user.uid)}
                        disabled={actionLoading === user.uid}
                        className="px-2 py-1 text-xs bg-purple-100 text-purple-800 rounded hover:bg-purple-200 disabled:opacity-50"
                      >
                        Profile
                      </button>
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
        <div 
          className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-60 p-4"
          onClick={(e) => e.target === e.currentTarget && setShowLevelModal(false)}
          role="dialog"
          aria-modal="true"
          aria-labelledby="level-modal-title"
        >
          <div className="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md max-h-[90vh] overflow-y-auto">
            <div className="flex items-center justify-between mb-4">
              <h3 id="level-modal-title" className="text-lg font-semibold">Assign User Level</h3>
              <button
                onClick={() => setShowLevelModal(false)}
                className="min-h-[44px] min-w-[44px] flex items-center justify-center text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2"
                aria-label="Close dialog"
              >
                <span className="text-xl font-bold">×</span>
              </button>
            </div>

            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium mb-1">User</label>
                <p className="text-sm text-gray-600">{selectedUser.email}</p>
              </div>

              <div>
                <label className="block text-sm font-medium mb-1">
                  Select Level
                </label>
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
                <label className="block text-sm font-medium mb-1">
                  Level Details
                </label>
                <div className="text-sm text-gray-600 space-y-1">
                  <p>
                    Token Multiplier:{' '}
                    {USER_LEVEL_CONFIGS[selectedLevel].tokenMultiplier}x
                  </p>
                  <p>
                    Max Tokens:{' '}
                    {USER_LEVEL_CONFIGS[selectedLevel].maxTokens === -1
                      ? 'Unlimited'
                      : USER_LEVEL_CONFIGS[selectedLevel].maxTokens}
                  </p>
                  <p>
                    Benefits:{' '}
                    {USER_LEVEL_CONFIGS[selectedLevel].benefits.join(', ')}
                  </p>
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium mb-1">
                  Reason (Optional)
                </label>
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
                className="min-h-[44px] px-4 py-2 text-gray-600 hover:text-gray-800 dark:text-gray-300 dark:hover:text-gray-100 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2"
              >
                Cancel
              </button>
              <button
                onClick={() => handleLevelChange(selectedUser.uid, selectedLevel, levelReason)}
                disabled={actionLoading === selectedUser.uid}
                className="min-h-[44px] px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
              >
                {actionLoading === selectedUser.uid ? 'Assigning...' : 'Assign Level'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Permission Profile Assignment Modal */}
      {showProfileModal && (
        <div 
          className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-60 p-4"
          onClick={(e) => e.target === e.currentTarget && setShowProfileModal(false)}
          role="dialog"
          aria-modal="true"
          aria-labelledby="profile-modal-title"
        >
          <div className="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md max-h-[90vh] overflow-y-auto">
            <div className="flex items-center justify-between mb-4">
              <h3 id="profile-modal-title" className="text-lg font-semibold">Assign Permission Profile</h3>
              <button
                onClick={() => setShowProfileModal(false)}
                className="min-h-[44px] min-w-[44px] flex items-center justify-center text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2"
                aria-label="Close dialog"
              >
                <span className="text-xl font-bold">×</span>
              </button>
            </div>

            <div className="space-y-4">
              <div>
                <Label htmlFor="user-select">Select User</Label>
                <Select value={profileUserId} onValueChange={setProfileUserId}>
                  <SelectTrigger>
                    <SelectValue placeholder="Select a user..." />
                  </SelectTrigger>
                  <SelectContent>
                    {users.map((user) => (
                      <SelectItem key={user.uid} value={user.uid}>
                        {user.displayName || user.email}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <div>
                <Label htmlFor="profile-select">Permission Profile</Label>
                <Select value={profileId} onValueChange={setProfileId}>
                  <SelectTrigger>
                    <SelectValue placeholder="Select a profile..." />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="premium">Premium Access</SelectItem>
                    <SelectItem value="basic">Basic Access</SelectItem>
                    <SelectItem value="admin">Admin Access</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div>
                <Label htmlFor="expires-at">Expires At (Optional)</Label>
                <Input
                  id="expires-at"
                  type="datetime-local"
                  value={expiresAt}
                  onChange={(e) => setExpiresAt(e.target.value)}
                />
              </div>
            </div>

            <div className="flex justify-end gap-3 mt-6">
              <button
                onClick={() => setShowProfileModal(false)}
                className="min-h-[44px] px-4 py-2 text-gray-600 hover:text-gray-800 dark:text-gray-300 dark:hover:text-gray-100 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2"
              >
                Cancel
              </button>
              <button
                onClick={handleAssignProfile}
                disabled={isPending || !profileUserId || !profileId}
                className="min-h-[44px] px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 disabled:opacity-50 transition-colors focus:outline-none focus:ring-2 focus:ring-green-500 focus:ring-offset-2"
              >
                {isPending ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    Assigning...
                  </>
                ) : (
                  'Assign Profile'
                )}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Level History Modal */}
      {showLevelHistory && (
        <div 
          className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-60 p-4"
          onClick={(e) => e.target === e.currentTarget && setShowLevelHistory(false)}
          role="dialog"
          aria-modal="true"
          aria-labelledby="history-modal-title"
        >
          <div className="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-2xl max-h-[90vh] flex flex-col">
            <div className="flex items-center justify-between mb-4 flex-shrink-0">
              <h3 id="history-modal-title" className="text-lg font-semibold">User Level History</h3>
              <button
                onClick={() => setShowLevelHistory(false)}
                className="min-h-[44px] min-w-[44px] flex items-center justify-center text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2"
                aria-label="Close dialog"
              >
                <span className="text-xl font-bold">×</span>
              </button>
            </div>

            <div className="space-y-3 overflow-y-auto flex-1 min-h-0">
              {levelHistory.map((entry, index) => (
                <div key={index} className="border-l-4 border-blue-500 pl-4 py-2">
                  <div className="flex items-center gap-2">
                    <span
                      className={`px-2 py-1 rounded text-xs font-medium ${USER_LEVEL_CONFIGS[entry.userLevel as UserLevel].color}`}
                    >
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

            <div className="flex justify-end mt-6 flex-shrink-0">
              <button
                onClick={() => setShowLevelHistory(false)}
                className="min-h-[44px] px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition-colors focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Delete User Dialog */}
      <Dialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete User</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete this user? This action will soft delete the user account.
            </DialogDescription>
          </DialogHeader>
          
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="delete-reason">Reason for deletion (optional)</Label>
              <Textarea
                id="delete-reason"
                placeholder="Enter reason for deleting this user..."
                value={deleteReason}
                onChange={(e) => setDeleteReason(e.target.value)}
              />
            </div>
          </div>

          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => {
                setDeleteDialogOpen(false);
                setUserToDelete('');
                setDeleteReason('');
              }}
              disabled={isPending}
            >
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={handleConfirmDelete}
              disabled={isPending}
            >
              {isPending ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Deleting...
                </>
              ) : (
                'Delete User'
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}