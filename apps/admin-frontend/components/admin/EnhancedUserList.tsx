'use client';

import React, { useEffect, useState } from 'react';
import type { UserWithPermissions } from '../../types/admin/iam-enhanced';
import {
  PackageTier,
  SubscriptionStatus,
} from '../../types/admin/iam-enhanced';
import { UserPermissionManager } from './UserPermissionManager';
// import { FirebaseIAMDebugPanel } from './firebase-iam-debug-panel';
import { iamService } from '../../services/iamService';

export const EnhancedUserList: React.FC = () => {
  const [users, setUsers] = useState<UserWithPermissions[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedUser, setSelectedUser] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'users' | 'debug'>('users');
  const [filters, setFilters] = useState({
    packageTier: '',
    subscriptionStatus: '',
    hasCustomPermissions: false,
  });

  useEffect(() => {
    loadUsers();
  }, [filters]);

  const loadUsers = async () => {
    try {
      setLoading(true);

      // Use the API route that properly merges Firebase Auth + Firestore data
      const response = await fetch('/api/admin/users');
      if (!response.ok) {
        throw new Error('Failed to fetch users from API');
      }

      const data = await response.json();

      // Transform API response to match UserWithPermissions interface
      const transformedUsers: UserWithPermissions[] = data.users.map(
        (user: any) => ({
          id: user.uid,
          email: user.email || '',
          name: user.displayName || '',
          displayName: user.displayName,
          emailVerified: user.emailVerified,
          disabled: user.disabled,
          roles: [], // Will be populated by IAM service if needed
          groups: [],
          attachedPolicies: [],
          status: user.disabled ? 'disabled' : 'active',
          lastActivity: user.metadata?.lastSignInTime || '',
          createdAt: user.metadata?.creationTime || '',
          updatedAt: user.lastUpdated || user.metadata?.creationTime || '',
          packageTier:
            user.userLevel === 'GOLD'
              ? PackageTier.GOLD
              : user.userLevel === 'PLATINUM'
                ? PackageTier.PLATINUM
                : user.userLevel === 'BRONZE'
                  ? PackageTier.BRONZE
                  : user.userLevel === 'SILVER'
                    ? PackageTier.SILVER
                    : user.userLevel === 'ENTERPRISE'
                      ? PackageTier.ENTERPRISE
                      : PackageTier.FREE,
          subscriptionStatus: user.disabled ? 'CANCELLED' : 'ACTIVE',
          lastPaymentDate: undefined,
          customPermissions: [],
          effectivePermissions: [],
          packagePermissions: [],
        }),
      );

      // Apply filters on the client side
      let filteredUsers = transformedUsers;

      if (filters.packageTier) {
        filteredUsers = filteredUsers.filter(
          (user) => user.packageTier === filters.packageTier,
        );
      }

      if (filters.subscriptionStatus) {
        filteredUsers = filteredUsers.filter(
          (user) => user.subscriptionStatus === filters.subscriptionStatus,
        );
      }

      if (filters.hasCustomPermissions) {
        filteredUsers = filteredUsers.filter(
          (user) => user.customPermissions.length > 0,
        );
      }

      setUsers(filteredUsers);
    } catch (error) {
      console.error('Failed to load users:', error);
      alert('Failed to load users');
    } finally {
      setLoading(false);
    }
  };

  const handlePackageUpgrade = async (userId: string, newTier: PackageTier) => {
    try {
      await iamService.updateUserPackageTier(
        userId,
        newTier,
        'current-admin-id',
      );
      await loadUsers(); // Refresh the list
      alert('Package upgraded successfully!');
    } catch (error) {
      console.error('Failed to upgrade package:', error);
      alert('Failed to upgrade package');
    }
  };

  const getTierColor = (tier: PackageTier) => {
    switch (tier) {
      case PackageTier.ENTERPRISE:
        return 'bg-purple-100 text-purple-800';
      case PackageTier.PLATINUM:
        return 'bg-gray-100 text-gray-800';
      case PackageTier.GOLD:
        return 'bg-yellow-100 text-yellow-800';
      case PackageTier.SILVER:
        return 'bg-gray-100 text-gray-600';
      case PackageTier.BRONZE:
        return 'bg-orange-100 text-orange-800';
      default:
        return 'bg-gray-100 text-gray-500';
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active':
        return 'bg-green-100 text-green-800';
      case 'trial':
        return 'bg-blue-100 text-blue-800';
      default:
        return 'bg-red-100 text-red-800';
    }
  };

  const renderUsersTab = () => {
    return (
      <div className="space-y-0">
        {/* Filters */}
        <div className="bg-gray-50 p-6 border-b border-gray-200">
          <h3 className="text-lg font-semibold mb-4">Filters</h3>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Package Tier
              </label>
              <select
                value={filters.packageTier}
                onChange={(e) =>
                  setFilters({ ...filters, packageTier: e.target.value })
                }
                className="w-full border border-gray-300 rounded-lg px-3 py-2"
              >
                <option value="">All Tiers</option>
                {Object.values(PackageTier).map((tier) => (
                  <option key={tier} value={tier}>
                    {tier.charAt(0).toUpperCase() + tier.slice(1)}
                  </option>
                ))}
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Subscription Status
              </label>
              <select
                value={filters.subscriptionStatus}
                onChange={(e) =>
                  setFilters({ ...filters, subscriptionStatus: e.target.value })
                }
                className="w-full border border-gray-300 rounded-lg px-3 py-2"
              >
                <option value="">All Statuses</option>
                {Object.values(SubscriptionStatus).map((status) => (
                  <option key={status} value={status}>
                    {status.charAt(0).toUpperCase() + status.slice(1)}
                  </option>
                ))}
              </select>
            </div>

            <div className="flex items-end">
              <label className="flex items-center">
                <input
                  type="checkbox"
                  checked={filters.hasCustomPermissions}
                  onChange={(e) =>
                    setFilters({
                      ...filters,
                      hasCustomPermissions: e.target.checked,
                    })
                  }
                  className="mr-2"
                />
                <span className="text-sm font-medium text-gray-700">
                  Has Custom Permissions
                </span>
              </label>
            </div>
          </div>
        </div>

        {/* User List */}
        <div>
          <div className="px-6 py-4 border-b border-gray-200">
            <h3 className="text-lg font-semibold">Users ({users.length})</h3>
          </div>

          {loading ? (
            <div className="p-6 text-center">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto"></div>
              <p className="mt-2 text-gray-600">Loading users...</p>
            </div>
          ) : users.length === 0 ? (
            <div className="p-6 text-center text-gray-500">
              No users found matching the current filters.
            </div>
          ) : (
            <div className="overflow-x-auto">
              <table className="min-w-full divide-y divide-gray-200">
                <thead className="bg-gray-50">
                  <tr>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider min-w-[200px]">
                      Email
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      User
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Package
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Status
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Permissions
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Last Payment
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Actions
                    </th>
                  </tr>
                </thead>
                <tbody className="bg-white divide-y divide-gray-200">
                  {users.map((user) => (
                    <tr key={user.id} className="hover:bg-gray-50">
                      {/* Email Column - First and most prominent */}
                      <td className="px-6 py-4 whitespace-nowrap bg-blue-50">
                        <div className="flex items-center">
                          <div className="text-sm font-medium text-gray-900">
                            {user.email || 'No email'}
                          </div>
                          {user.email && (
                            <span
                              className={`ml-2 inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium ${
                                user.emailVerified
                                  ? 'bg-green-100 text-green-700'
                                  : 'bg-yellow-100 text-yellow-700'
                              }`}
                            >
                              {user.emailVerified ? '✓' : '⚠'}
                            </span>
                          )}
                        </div>
                      </td>

                      {/* User Column - Display name and avatar */}
                      <td className="px-6 py-4 whitespace-nowrap">
                        <div className="flex items-center">
                          <div className="h-10 w-10 rounded-full bg-blue-100 flex items-center justify-center">
                            <span className="text-sm font-medium text-blue-600">
                              {(user.displayName || user.name || user.email)
                                ?.charAt(0)
                                .toUpperCase()}
                            </span>
                          </div>
                          <div className="ml-4">
                            <div className="text-sm font-medium text-gray-900">
                              {user.displayName ||
                                user.name ||
                                user.email?.split('@')[0] ||
                                'Unknown'}
                            </div>
                            <div className="text-sm text-gray-500">
                              User ID: {user.id.substring(0, 8)}...
                            </div>
                          </div>
                        </div>
                      </td>

                      <td className="px-6 py-4 whitespace-nowrap">
                        <span
                          className={`px-2 py-1 text-xs font-medium rounded-full ${getTierColor(user.packageTier)}`}
                        >
                          {user.packageTier}
                        </span>
                      </td>

                      <td className="px-6 py-4 whitespace-nowrap">
                        <span
                          className={`px-2 py-1 text-xs font-medium rounded-full ${getStatusColor(user.subscriptionStatus)}`}
                        >
                          {user.subscriptionStatus}
                        </span>
                      </td>

                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                        <div className="flex space-x-2">
                          <span className="bg-green-100 text-green-800 px-2 py-1 text-xs rounded">
                            {user.packagePermissions?.length || 0} Package
                          </span>
                          {user.customPermissions &&
                            user.customPermissions.length > 0 && (
                              <span className="bg-blue-100 text-blue-800 px-2 py-1 text-xs rounded">
                                {user.customPermissions.length} Custom
                              </span>
                            )}
                        </div>
                      </td>

                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                        {user.lastPaymentDate
                          ? new Date(user.lastPaymentDate).toLocaleDateString()
                          : 'Never'}
                      </td>

                      <td className="px-6 py-4 whitespace-nowrap text-sm font-medium space-y-1">
                        <div>
                          <button
                            onClick={() => setSelectedUser(user.id)}
                            className="text-blue-600 hover:text-blue-900 block"
                          >
                            Manage Permissions
                          </button>
                        </div>

                        <div className="relative">
                          <select
                            onChange={(e) => {
                              if (e.target.value) {
                                if (
                                  confirm(
                                    `Upgrade ${user.email} to ${e.target.value}?`,
                                  )
                                ) {
                                  handlePackageUpgrade(
                                    user.id,
                                    e.target.value as PackageTier,
                                  );
                                }
                                e.target.value = ''; // Reset selection
                              }
                            }}
                            className="text-green-600 hover:text-green-900 bg-transparent border-none cursor-pointer text-sm"
                            defaultValue=""
                          >
                            <option value="">Upgrade Package</option>
                            {Object.values(PackageTier)
                              .filter((tier) => tier !== user.packageTier)
                              .map((tier) => (
                                <option key={tier} value={tier}>
                                  To{' '}
                                  {tier.charAt(0).toUpperCase() + tier.slice(1)}
                                </option>
                              ))}
                          </select>
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      </div>
    );
  };

  return (
    <div className="space-y-6">
      {/* Tab Navigation */}
      <div className="bg-white rounded-lg shadow">
        <div className="border-b border-gray-200">
          <nav className="-mb-px flex">
            <button
              onClick={() => setActiveTab('users')}
              className={`py-4 px-6 border-b-2 font-medium text-sm ${
                activeTab === 'users'
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              Users ({users.length})
            </button>
            <button
              onClick={() => setActiveTab('debug')}
              className={`py-4 px-6 border-b-2 font-medium text-sm ${
                activeTab === 'debug'
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              Firebase Debug
            </button>
          </nav>
        </div>

        {/* Tab Content */}
        {activeTab === 'users' ? (
          <div className="p-0">{renderUsersTab()}</div>
        ) : (
          <div className="p-6">
            <div className="text-center text-gray-500">
              <h3 className="text-lg font-semibold mb-4">
                Firebase IAM Debug Panel
              </h3>
              <p>Debug panel temporarily disabled for testing.</p>
              <div className="mt-4">
                <p className="text-sm">
                  The users you're seeing are:{' '}
                  <strong>
                    {users.length > 0 ? 'Firebase/Mock Data' : 'No users found'}
                  </strong>
                </p>
                {users.length > 0 && (
                  <div className="mt-2 p-4 bg-gray-100 rounded">
                    <p className="text-sm font-medium">Sample User Info:</p>
                    <p className="text-xs">Email: {users[0]?.email}</p>
                    <p className="text-xs">
                      Name: {users[0]?.name || 'Not set'}
                    </p>
                    <p className="text-xs">
                      Display Name: {users[0]?.displayName || 'Not set'}
                    </p>
                    <p className="text-xs">Package: {users[0]?.packageTier}</p>
                  </div>
                )}
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Permission Manager Modal */}
      {selectedUser && (
        <UserPermissionManager
          userId={selectedUser}
          onClose={() => setSelectedUser(null)}
        />
      )}
    </div>
  );
};
