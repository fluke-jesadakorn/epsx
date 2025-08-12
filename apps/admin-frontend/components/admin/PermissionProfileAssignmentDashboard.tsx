'use client';

import { useState, useEffect, useCallback } from 'react';
import { AdminService as _AdminService } from '@/services/adminService';
import { useAdminAuth } from '@/context/simple-admin-auth';
import { 
  Shield, 
  Users, 
  User, 
  Plus as _Plus,
  Search,
  Filter as _Filter,
  CheckCircle,
  AlertTriangle,
  Zap,
  UserPlus,
  Tag,
  Clock as _Clock,
  Settings
} from 'lucide-react';

// Types for permission profile system
interface PermissionProfile {
  id: string;
  name: string;
  description: string;
  category: string;
  target_tier: string;
  is_active: boolean;
  permissions_count: number;
  tags: string[];
  created_at: string;
  version: string;
  metadata: {
    requires_approval: boolean;
    max_assignments?: number;
    use_cases: string[];
    warnings: string[];
  };
}

interface PermissionProfileAssignmentRequest {
  permission_profile_id: string;
  user_ids: string[];
  reason?: string;
  merge_permissions?: boolean;
  expires_at?: string;
  notify_users?: boolean;
}

interface AssignmentResult {
  permission_profile_id: string;
  successful_assignments: Array<{
    user_id: string;
    features_unlocked: string[];
    permissions_added: string[];
    assignment_type: string;
  }>;
  failed_assignments: Array<{
    user_id: string;
    error: string;
    error_code: string;
  }>;
  total_assigned: number;
  total_failed: number;
  applied_at: string;
}

export function PermissionProfileAssignmentDashboard() {
  const { user } = useAdminAuth();
  const [permissionProfiles, setPermissionProfiles] = useState<PermissionProfile[]>([]);
  const [selectedPermissionProfile, setSelectedPermissionProfile] = useState<PermissionProfile | null>(null);
  const [selectedUsers, setSelectedUsers] = useState<string[]>([]);
  const [userSearchTerm, setUserSearchTerm] = useState('');
  const [permissionProfileFilter, setPermissionProfileFilter] = useState({
    category: '',
    tier: '',
    active_only: true
  });
  const [assignmentReason, setAssignmentReason] = useState('');
  const [notifyUsers, setNotifyUsers] = useState(true);
  const [loading, setLoading] = useState(false);
  const [assignmentMode, setAssignmentMode] = useState<'single' | 'bulk'>('single');
  const [assignmentResult, setAssignmentResult] = useState<AssignmentResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const loadPermissionProfiles = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      // Use the server action to get admin permission profiles from backend
      const { serverGetAdminPermissionProfiles } = await import('@epsx/api-client');
      const searchParams = new URLSearchParams({
        category: permissionProfileFilter.category,
        package_tier: permissionProfileFilter.tier,
        active_only: permissionProfileFilter.active_only.toString(),
        limit: '50',
        offset: '0'
      });
      
      const response = await serverGetAdminPermissionProfiles(searchParams);
      
      if (response.error) {
        throw new Error(response.error);
      }

      const data = response.data;
      setPermissionProfiles(data.permission_profiles || []);
    } catch (err: any) {
      console.error('Failed to load permission profiles:', err);
      setError(err.message || 'Failed to load permission profiles');
    } finally {
      setLoading(false);
    }
  }, [permissionProfileFilter]);

  useEffect(() => {
    loadPermissionProfiles();
  }, [permissionProfileFilter, loadPermissionProfiles]);

  const handlePermissionProfileSelect = async (permissionProfile: PermissionProfile) => {
    setSelectedPermissionProfile(permissionProfile);
    setSelectedUsers([]);
    setAssignmentResult(null);
    setError(null);
  };

  const handleUserSelection = (userId: string) => {
    setSelectedUsers(prev => {
      if (prev.includes(userId)) {
        return prev.filter(id => id !== userId);
      } else {
        return [...prev, userId];
      }
    });
  };

  const handleAssignPermissionProfile = async () => {
    if (!selectedPermissionProfile || selectedUsers.length === 0) {
      setError('Please select a permission profile and at least one user');
      return;
    }

    try {
      setLoading(true);
      setError(null);

      // Use the server action to assign permission profiles
      const { serverAssignPermissionProfile } = await import('@epsx/api-client');
      
      const assignmentRequest: PermissionProfileAssignmentRequest = {
        permission_profile_id: selectedPermissionProfile.id,
        user_ids: selectedUsers,
        reason: assignmentReason || `Admin direct assignment by ${user?.email}`,
        merge_permissions: true,
        notify_users: notifyUsers,
      };

      const response = await serverAssignPermissionProfile(assignmentRequest);
      
      if (response.error) {
        throw new Error(response.error);
      }

      const result: AssignmentResult = response.data;
      setAssignmentResult(result);
      
      // Clear selections after successful assignment
      if (result.total_assigned > 0) {
        setSelectedUsers([]);
        setAssignmentReason('');
      }

    } catch (err: any) {
      console.error('Failed to assign permission profile:', err);
      setError(err.message || 'Failed to assign permission profile');
    } finally {
      setLoading(false);
    }
  };

  const getCategoryIcon = (category: string) => {
    switch (category.toLowerCase()) {
      case 'user': return <Users className="h-4 w-4" />;
      case 'admin': return <Shield className="h-4 w-4" />;
      case 'moderator': return <Settings className="h-4 w-4" />;
      default: return <User className="h-4 w-4" />;
    }
  };

  const getTierColor = (tier: string) => {
    switch (tier.toLowerCase()) {
      case 'bronze': return 'text-amber-600 bg-amber-100';
      case 'silver': return 'text-gray-600 bg-gray-100';
      case 'gold': return 'text-yellow-600 bg-yellow-100';
      case 'platinum': return 'text-purple-600 bg-purple-100';
      default: return 'text-blue-600 bg-blue-100';
    }
  };

  return (
    <div className="space-y-8 p-6">
      {/* Header */}
      <div className="flex items-center justify-between pancake-card pancake-card-hover p-6">
        <div className="flex items-center gap-4">
          <div className="p-3 rounded-full bg-gradient-to-r from-purple-500 to-blue-500">
            <User className="h-8 w-8 text-white" />
          </div>
          <div>
            <h1 className="text-3xl font-bold bg-gradient-to-r from-purple-500 to-blue-500 bg-clip-text text-transparent">
              Permission Profile Assignment Dashboard
            </h1>
            <p className="text-muted-foreground mt-1">
              Assign feature permission profiles directly to users without payment
            </p>
          </div>
        </div>
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2">
            <label className="text-sm font-medium">Mode:</label>
            <select
              value={assignmentMode}
              onChange={(e) => setAssignmentMode(e.target.value as 'single' | 'bulk')}
              className="px-3 py-1 border rounded-lg text-sm"
            >
              <option value="single">Single Assignment</option>
              <option value="bulk">Bulk Assignment</option>
            </select>
          </div>
        </div>
      </div>

      {/* Error Display */}
      {error && (
        <div className="pancake-card pancake-card-hover p-4 border-l-4 border-red-500">
          <div className="flex items-center gap-2 text-red-600">
            <AlertTriangle className="h-5 w-5" />
            <span className="font-semibold">Error: {error}</span>
          </div>
        </div>
      )}

      {/* Assignment Result */}
      {assignmentResult && (
        <div className="pancake-card pancake-card-hover p-6 border-l-4 border-green-500">
          <div className="flex items-center gap-3 mb-4">
            <CheckCircle className="h-6 w-6 text-green-600" />
            <h3 className="text-lg font-semibold text-green-600">Assignment Complete</h3>
          </div>
          <div className="grid grid-cols-2 gap-4 mb-4">
            <div className="text-center p-3 bg-green-50 rounded-lg">
              <div className="text-2xl font-bold text-green-600">{assignmentResult.total_assigned}</div>
              <div className="text-sm text-green-600">Successful</div>
            </div>
            <div className="text-center p-3 bg-red-50 rounded-lg">
              <div className="text-2xl font-bold text-red-600">{assignmentResult.total_failed}</div>
              <div className="text-sm text-red-600">Failed</div>
            </div>
          </div>
          {assignmentResult.failed_assignments.length > 0 && (
            <div className="mt-4">
              <h4 className="font-semibold text-red-600 mb-2">Failed Assignments:</h4>
              <ul className="space-y-1 text-sm">
                {assignmentResult.failed_assignments.map((failure, index) => (
                  <li key={index} className="text-red-600">
                    {failure.user_id}: {failure.error}
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}

      {/* Main Content Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        {/* Permission Profile Selection */}
        <div className="pancake-card">
          <div className="px-6 py-4 border-b border-border">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <User className="h-6 w-6 text-purple-500" />
                <h2 className="text-xl font-bold">Available Permission Profiles</h2>
              </div>
              <div className="flex items-center gap-2">
                <select
                  value={permissionProfileFilter.category}
                  onChange={(e) => setPermissionProfileFilter(prev => ({ ...prev, category: e.target.value }))}
                  className="px-3 py-1 border rounded text-sm"
                >
                  <option value="">All Categories</option>
                  <option value="user">User</option>
                  <option value="moderator">Moderator</option>
                  <option value="admin">Admin</option>
                </select>
                <select
                  value={permissionProfileFilter.tier}
                  onChange={(e) => setPermissionProfileFilter(prev => ({ ...prev, tier: e.target.value }))}
                  className="px-3 py-1 border rounded text-sm"
                >
                  <option value="">All Tiers</option>
                  <option value="Bronze">Bronze</option>
                  <option value="Silver">Silver</option>
                  <option value="Gold">Gold</option>
                  <option value="Platinum">Platinum</option>
                </select>
              </div>
            </div>
          </div>
          <div className="p-6">
            {loading ? (
              <div className="flex items-center justify-center h-32">
                <div className="animate-spin rounded-full h-8 w-8 border-4 border-purple-500/20 border-t-purple-500"></div>
              </div>
            ) : (
              <div className="space-y-3 max-h-96 overflow-y-auto">
                {permissionProfiles.map((permissionProfile) => (
                  <div
                    key={permissionProfile.id}
                    onClick={() => handlePermissionProfileSelect(permissionProfile)}
                    className={`p-4 border rounded-lg cursor-pointer transition-all hover:shadow-md permission-profile-card ${
                      selectedPermissionProfile?.id === permissionProfile.id
                        ? 'border-purple-500 bg-purple-50'
                        : 'border-border hover:border-purple-300'
                    }`}
                  >
                    <div className="flex items-start justify-between mb-2">
                      <div className="flex items-center gap-2">
                        {getCategoryIcon(permissionProfile.category)}
                        <h3 className="font-semibold">{permissionProfile.name}</h3>
                      </div>
                      <div className="flex items-center gap-2">
                        <span className={`px-2 py-1 rounded-full text-xs font-medium ${getTierColor(permissionProfile.target_tier)}`}>
                          {permissionProfile.target_tier}
                        </span>
                        {permissionProfile.is_active ? (
                          <CheckCircle className="h-4 w-4 text-green-500" />
                        ) : (
                          <AlertTriangle className="h-4 w-4 text-yellow-500" />
                        )}
                      </div>
                    </div>
                    <p className="text-sm text-muted-foreground mb-2">{permissionProfile.description}</p>
                    <div className="flex items-center justify-between text-xs text-muted-foreground">
                      <span>{permissionProfile.permissions_count} permissions</span>
                      <span>v{permissionProfile.version}</span>
                    </div>
                    {permissionProfile.tags.length > 0 && (
                      <div className="flex items-center gap-1 mt-2">
                        <Tag className="h-3 w-3" />
                        <div className="flex gap-1">
                          {permissionProfile.tags.slice(0, 3).map((tag) => (
                            <span key={tag} className="px-1 py-0.5 bg-gray-100 rounded text-xs">
                              {tag}
                            </span>
                          ))}
                          {permissionProfile.tags.length > 3 && (
                            <span className="px-1 py-0.5 bg-gray-100 rounded text-xs">
                              +{permissionProfile.tags.length - 3}
                            </span>
                          )}
                        </div>
                      </div>
                    )}
                  </div>
                ))}
                {permissionProfiles.length === 0 && (
                  <div className="text-center py-8 text-muted-foreground">
                    <User className="h-12 w-12 mx-auto mb-3 opacity-50" />
                    <p>No permission profiles found matching the filters</p>
                  </div>
                )}
              </div>
            )}
          </div>
        </div>

        {/* Assignment Configuration */}
        <div className="pancake-card">
          <div className="px-6 py-4 border-b border-border">
            <div className="flex items-center gap-3">
              <UserPlus className="h-6 w-6 text-blue-500" />
              <h2 className="text-xl font-bold">Assignment Configuration</h2>
            </div>
          </div>
          <div className="p-6">
            {selectedPermissionProfile ? (
              <div className="space-y-6">
                {/* Selected Permission Profile Info */}
                <div className="p-4 bg-purple-50 rounded-lg border border-purple-200">
                  <div className="flex items-center gap-2 mb-2">
                    <User className="h-5 w-5 text-purple-600" />
                    <h3 className="font-semibold text-purple-900">{selectedPermissionProfile.name}</h3>
                  </div>
                  <p className="text-sm text-purple-700 mb-3">{selectedPermissionProfile.description}</p>
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <span className="font-medium">Category:</span> {selectedPermissionProfile.category}
                    </div>
                    <div>
                      <span className="font-medium">Tier:</span> {selectedPermissionProfile.target_tier}
                    </div>
                    <div>
                      <span className="font-medium">Permissions:</span> {selectedPermissionProfile.permissions_count}
                    </div>
                    <div>
                      <span className="font-medium">Version:</span> {selectedPermissionProfile.version}
                    </div>
                  </div>
                  {selectedPermissionProfile.metadata.warnings.length > 0 && (
                    <div className="mt-3 p-2 bg-yellow-50 border border-yellow-200 rounded">
                      <div className="flex items-center gap-1 text-yellow-800 text-sm font-medium mb-1">
                        <AlertTriangle className="h-4 w-4" />
                        Warnings:
                      </div>
                      <ul className="text-xs text-yellow-700 list-disc list-inside">
                        {selectedPermissionProfile.metadata.warnings.map((warning, index) => (
                          <li key={index}>{warning}</li>
                        ))}
                      </ul>
                    </div>
                  )}
                </div>

                {/* User Selection */}
                <div>
                  <label className="block text-sm font-medium mb-2">Select Users</label>
                  <div className="relative">
                    <Search className="h-4 w-4 absolute left-3 top-3 text-muted-foreground" />
                    <input
                      type="text"
                      placeholder="Search users by email..."
                      value={userSearchTerm}
                      onChange={(e) => setUserSearchTerm(e.target.value)}
                      className="w-full pl-10 pr-4 py-2 border rounded-lg"
                    />
                  </div>
                  <div className="mt-2 p-3 border rounded-lg bg-gray-50 min-h-[100px]">
                    {selectedUsers.length > 0 ? (
                      <div className="flex flex-wrap gap-2">
                        {selectedUsers.map((userId) => (
                          <div
                            key={userId}
                            className="flex items-center gap-2 px-3 py-1 bg-blue-100 text-blue-800 rounded-full text-sm"
                          >
                            <span>{userId}</span>
                            <button
                              onClick={() => handleUserSelection(userId)}
                              className="hover:text-blue-600"
                            >
                              ×
                            </button>
                          </div>
                        ))}
                      </div>
                    ) : (
                      <div className="text-center text-muted-foreground text-sm">
                        No users selected. Search and click to add users.
                      </div>
                    )}
                  </div>
                </div>

                {/* Assignment Reason */}
                <div>
                  <label className="block text-sm font-medium mb-2">Assignment Reason</label>
                  <textarea
                    placeholder="Enter reason for this assignment (optional)"
                    value={assignmentReason}
                    onChange={(e) => setAssignmentReason(e.target.value)}
                    rows={3}
                    className="w-full p-3 border rounded-lg"
                  />
                </div>

                {/* Options */}
                <div className="space-y-3">
                  <div className="flex items-center gap-2">
                    <input
                      type="checkbox"
                      id="notify-users"
                      checked={notifyUsers}
                      onChange={(e) => setNotifyUsers(e.target.checked)}
                      className="rounded"
                    />
                    <label htmlFor="notify-users" className="text-sm">
                      Notify users about permission profile assignment
                    </label>
                  </div>
                </div>

                {/* Assignment Button */}
                <button
                  onClick={handleAssignPermissionProfile}
                  disabled={loading || selectedUsers.length === 0}
                  className="w-full bg-gradient-to-r from-purple-500 to-blue-500 text-white px-6 py-3 rounded-lg font-semibold hover:from-purple-600 hover:to-blue-600 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                >
                  {loading ? (
                    <>
                      <div className="animate-spin rounded-full h-4 w-4 border-2 border-white/20 border-t-white"></div>
                      Assigning...
                    </>
                  ) : (
                    <>
                      <Zap className="h-4 w-4" />
                      Assign Permission Profile to {selectedUsers.length} User{selectedUsers.length !== 1 ? 's' : ''}
                    </>
                  )}
                </button>
              </div>
            ) : (
              <div className="text-center py-12 text-muted-foreground">
                <User className="h-16 w-16 mx-auto mb-4 opacity-50" />
                <h3 className="text-lg font-semibold mb-2">Select a Permission Profile</h3>
                <p>Choose a permission profile from the left panel to configure assignment</p>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}