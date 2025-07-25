'use client';

import { useToast } from '@/components/ui/toast';
import React, { useEffect, useState } from 'react';
import { adminLogger } from '../../lib/logger';
import { iamService } from '../../services/iamService';
import type { UserWithPermissions } from '../../types/admin/iam';
import { PackageTier, SubscriptionStatus } from '../../types/admin/iam';
import { UserPermissionManager } from './UserPermissionManager';

export const UserManagementList: React.FC = () => {
  const [users, setUsers] = useState<UserWithPermissions[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedUser, setSelectedUser] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'users' | 'debug'>('users');
  const [filters, setFilters] = useState({
    packageTier: '',
    subscriptionStatus: '',
    hasCustomPermissions: false,
  });
  const { addToast } = useToast();

  useEffect(() => {
    loadUsers();
  }, [filters]);

  const loadUsers = async () => {
    try {
      setLoading(true);

      // Use the API route that properly merges Firebase Auth + Firestore data
      const response = await fetch('/api/admin/user-management/users');
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
          roles: [],
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
        })
      );

      // Apply filters on the client side
      let filteredUsers = transformedUsers;

      if (filters.packageTier) {
        filteredUsers = filteredUsers.filter(
          user => user.packageTier === filters.packageTier
        );
      }

      if (filters.subscriptionStatus) {
        filteredUsers = filteredUsers.filter(
          user => user.subscriptionStatus === filters.subscriptionStatus
        );
      }

      if (filters.hasCustomPermissions) {
        filteredUsers = filteredUsers.filter(
          user => user.customPermissions.length > 0
        );
      }

      setUsers(filteredUsers);
    } catch (error) {
      adminLogger.error('Failed to load users', {
        error: error instanceof Error ? error.message : String(error),
      });
      addToast({
        type: 'error',
        title: 'Failed to load users',
        description: 'Please try again later',
      });
    } finally {
      setLoading(false);
    }
  };

  const handlePackageUpgrade = async (userId: string, newTier: PackageTier) => {
    try {
      await iamService.updateUserPackageTier(
        userId,
        newTier,
        'current-admin-id'
      );
      await loadUsers(); // Refresh the list
      addToast({
        type: 'success',
        title: 'Package upgraded successfully!',
      });
    } catch (error) {
      adminLogger.error('Failed to upgrade package', {
        userId,
        newTier,
        error: error instanceof Error ? error.message : String(error),
      });
      addToast({
        type: 'error',
        title: 'Failed to upgrade package',
        description:
          error instanceof Error ? error.message : 'Please try again',
      });
    }
  };

  const getTierColor = (tier: PackageTier) => {
    switch (tier) {
      case PackageTier.ENTERPRISE:
        return 'bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-300';
      case PackageTier.PLATINUM:
        return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300';
      case PackageTier.GOLD:
        return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300';
      case PackageTier.SILVER:
        return 'bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400';
      case PackageTier.BRONZE:
        return 'bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-300';
      default:
        return 'bg-gray-100 text-gray-500 dark:bg-gray-700 dark:text-gray-400';
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active':
        return 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300';
      case 'trial':
        return 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300';
      default:
        return 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300';
    }
  };

  return (
    <div className="space-y-6">
      {/* Tab Navigation */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow">
        <div className="border-b border-gray-200 dark:border-gray-700">
          <nav className="-mb-px flex">
            <button
              onClick={() => setActiveTab('users')}
              className={`py-4 px-6 border-b-2 font-medium text-sm ${
                activeTab === 'users'
                  ? 'border-blue-500 text-blue-600 dark:text-blue-400'
                  : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300'
              }`}
            >
              Users ({users.length})
            </button>
            <button
              onClick={() => setActiveTab('debug')}
              className={`py-4 px-6 border-b-2 font-medium text-sm ${
                activeTab === 'debug'
                  ? 'border-blue-500 text-blue-600 dark:text-blue-400'
                  : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300'
              }`}
            >
              Firebase Debug
            </button>
          </nav>
        </div>

        {/* Tab Content */}
        {activeTab === 'users' ? (
          <div className="space-y-0">
            {/* Filters */}
            <div className="bg-gray-50 dark:bg-gray-800 p-6 border-b border-gray-200 dark:border-gray-700">
              <h3 className="text-lg font-semibold mb-4 text-gray-900 dark:text-gray-100">
                Filters
              </h3>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    Package Tier
                  </label>
                  <select
                    value={filters.packageTier}
                    onChange={e =>
                      setFilters({ ...filters, packageTier: e.target.value })
                    }
                    className="w-full border border-gray-300 dark:border-gray-600 rounded-lg px-3 py-2 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                  >
                    <option value="">All Tiers</option>
                    {Object.values(PackageTier).map(tier => (
                      <option key={tier} value={tier}>
                        {tier.charAt(0).toUpperCase() + tier.slice(1)}
                      </option>
                    ))}
                  </select>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    Subscription Status
                  </label>
                  <select
                    value={filters.subscriptionStatus}
                    onChange={e =>
                      setFilters({
                        ...filters,
                        subscriptionStatus: e.target.value,
                      })
                    }
                    className="w-full border border-gray-300 dark:border-gray-600 rounded-lg px-3 py-2 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                  >
                    <option value="">All Statuses</option>
                    {Object.values(SubscriptionStatus).map(status => (
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
                      onChange={e =>
                        setFilters({
                          ...filters,
                          hasCustomPermissions: e.target.checked,
                        })
                      }
                      className="mr-2 rounded border-gray-300 dark:border-gray-600"
                    />
                    <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                      Has Custom Permissions
                    </span>
                  </label>
                </div>
              </div>
            </div>

            {/* User List */}
            <div>
              <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
                <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                  Users ({users.length})
                </h3>
              </div>

              {loading ? (
                <div className="p-6 text-center">
                  <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto"></div>
                  <p className="mt-2 text-gray-600 dark:text-gray-400">
                    Loading users...
                  </p>
                </div>
              ) : users.length === 0 ? (
                <div className="p-6 text-center text-gray-500 dark:text-gray-400">
                  No users found matching the current filters.
                </div>
              ) : (
                <>
                  {/* Desktop Table View */}
                  <div className="hidden lg:block">
                    <div className="overflow-x-auto shadow-sm">
                      <div className="relative">
                        {/* Scroll indicator */}
                        <div className="absolute top-0 right-0 w-8 h-full bg-gradient-to-l from-white dark:from-gray-900 to-transparent pointer-events-none z-10"></div>
                        <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                          <thead className="bg-gray-50 dark:bg-gray-800">
                            <tr>
                              <th className="px-4 py-4 text-left text-sm font-medium text-gray-700 dark:text-gray-300 uppercase tracking-wider min-w-[250px]">
                                Email
                              </th>
                              <th className="px-4 py-4 text-left text-sm font-medium text-gray-700 dark:text-gray-300 uppercase tracking-wider min-w-[180px]">
                                User
                              </th>
                              <th className="px-4 py-4 text-left text-sm font-medium text-gray-700 dark:text-gray-300 uppercase tracking-wider">
                                Package
                              </th>
                              <th className="px-4 py-4 text-left text-sm font-medium text-gray-700 dark:text-gray-300 uppercase tracking-wider">
                                Status
                              </th>
                              <th className="px-4 py-4 text-left text-sm font-medium text-gray-700 dark:text-gray-300 uppercase tracking-wider">
                                Permissions
                              </th>
                              <th className="px-4 py-4 text-left text-sm font-medium text-gray-700 dark:text-gray-300 uppercase tracking-wider">
                                Last Payment
                              </th>
                              <th className="px-4 py-4 text-left text-sm font-medium text-gray-700 dark:text-gray-300 uppercase tracking-wider min-w-[160px]">
                                Actions
                              </th>
                            </tr>
                          </thead>
                          <tbody className="bg-white dark:bg-gray-900 divide-y divide-gray-200 dark:divide-gray-700">
                            {users.map(user => (
                              <tr
                                key={user.id}
                                className="hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors"
                              >
                                <td className="px-4 py-4 whitespace-nowrap">
                                  <div className="flex items-center">
                                    <div className="text-sm font-medium text-gray-900 dark:text-gray-100">
                                      {user.email || 'No email'}
                                    </div>
                                    {user.email && (
                                      <span
                                        className={`ml-2 inline-flex items-center px-2 py-1 rounded text-xs font-medium ${
                                          user.emailVerified
                                            ? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300'
                                            : 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300'
                                        }`}
                                      >
                                        {user.emailVerified ? '✓' : '⚠'}
                                      </span>
                                    )}
                                  </div>
                                </td>

                                <td className="px-4 py-4 whitespace-nowrap">
                                  <div className="flex items-center">
                                    <div className="h-10 w-10 rounded-full bg-blue-100 dark:bg-blue-900 flex items-center justify-center flex-shrink-0">
                                      <span className="text-sm font-medium text-blue-600 dark:text-blue-300">
                                        {(
                                          user.displayName ||
                                          user.name ||
                                          user.email
                                        )
                                          ?.charAt(0)
                                          .toUpperCase()}
                                      </span>
                                    </div>
                                    <div className="ml-3 min-w-0 flex-1">
                                      <div className="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
                                        {user.displayName ||
                                          user.name ||
                                          user.email?.split('@')[0] ||
                                          'Unknown'}
                                      </div>
                                      <div className="text-sm text-gray-500 dark:text-gray-400 truncate">
                                        ID: {user.id.substring(0, 8)}...
                                      </div>
                                    </div>
                                  </div>
                                </td>

                                <td className="px-4 py-4 whitespace-nowrap">
                                  <span
                                    className={`px-3 py-1 text-sm font-medium rounded-full ${getTierColor(user.packageTier)}`}
                                  >
                                    {user.packageTier}
                                  </span>
                                </td>

                                <td className="px-4 py-4 whitespace-nowrap">
                                  <span
                                    className={`px-3 py-1 text-sm font-medium rounded-full ${getStatusColor(user.subscriptionStatus)}`}
                                  >
                                    {user.subscriptionStatus}
                                  </span>
                                </td>

                                <td className="px-4 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                                  <div className="flex flex-col space-y-1">
                                    <span className="bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300 px-2 py-1 text-xs rounded text-center">
                                      {user.packagePermissions?.length || 0}{' '}
                                      Package
                                    </span>
                                    {user.customPermissions &&
                                      user.customPermissions.length > 0 && (
                                        <span className="bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300 px-2 py-1 text-xs rounded text-center">
                                          {user.customPermissions.length} Custom
                                        </span>
                                      )}
                                  </div>
                                </td>

                                <td className="px-4 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                                  {user.lastPaymentDate
                                    ? new Date(
                                        user.lastPaymentDate
                                      ).toLocaleDateString()
                                    : 'Never'}
                                </td>

                                <td className="px-4 py-4 whitespace-nowrap text-sm font-medium">
                                  <div className="flex flex-col space-y-2">
                                    <button
                                      onClick={() => setSelectedUser(user.id)}
                                      className="min-h-[36px] px-3 py-2 text-blue-600 hover:text-blue-900 dark:text-blue-400 dark:hover:text-blue-300 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded-md transition-colors text-left"
                                    >
                                      Manage Permissions
                                    </button>
                                    <select
                                      onChange={e => {
                                        if (e.target.value) {
                                          if (
                                            confirm(
                                              `Upgrade ${user.email} to ${e.target.value}?`
                                            )
                                          ) {
                                            handlePackageUpgrade(
                                              user.id,
                                              e.target.value as PackageTier
                                            );
                                          }
                                          e.target.value = '';
                                        }
                                      }}
                                      className="min-h-[36px] text-sm border border-gray-300 dark:border-gray-600 rounded-md px-3 py-2 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                      defaultValue=""
                                    >
                                      <option value="" disabled>
                                        Upgrade to...
                                      </option>
                                      {Object.values(PackageTier)
                                        .filter(
                                          tier => tier !== user.packageTier
                                        )
                                        .map(tier => (
                                          <option key={tier} value={tier}>
                                            {tier}
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
                    </div>
                  </div>

                  {/* Mobile Card View */}
                  <div className="lg:hidden space-y-4">
                    {users.map(user => (
                      <div
                        key={user.id}
                        className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-4 shadow-sm"
                      >
                        {/* Header with avatar and name */}
                        <div className="flex items-center space-x-3 mb-3">
                          <div className="h-12 w-12 rounded-full bg-blue-100 dark:bg-blue-900 flex items-center justify-center flex-shrink-0">
                            <span className="text-base font-medium text-blue-600 dark:text-blue-300">
                              {(user.displayName || user.name || user.email)
                                ?.charAt(0)
                                .toUpperCase()}
                            </span>
                          </div>
                          <div className="min-w-0 flex-1">
                            <div className="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
                              {user.displayName ||
                                user.name ||
                                user.email?.split('@')[0] ||
                                'Unknown'}
                            </div>
                            <div className="text-sm text-gray-500 dark:text-gray-400 truncate">
                              {user.email || 'No email'}
                            </div>
                          </div>
                          {user.email && (
                            <span
                              className={`inline-flex items-center px-2 py-1 rounded text-xs font-medium ${
                                user.emailVerified
                                  ? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300'
                                  : 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300'
                              }`}
                            >
                              {user.emailVerified ? '✓' : '⚠'}
                            </span>
                          )}
                        </div>

                        {/* Status badges */}
                        <div className="flex flex-wrap gap-2 mb-3">
                          <span
                            className={`px-3 py-1 text-sm font-medium rounded-full ${getTierColor(user.packageTier)}`}
                          >
                            {user.packageTier}
                          </span>
                          <span
                            className={`px-3 py-1 text-sm font-medium rounded-full ${getStatusColor(user.subscriptionStatus)}`}
                          >
                            {user.subscriptionStatus}
                          </span>
                        </div>

                        {/* Permissions and payment info */}
                        <div className="grid grid-cols-2 gap-4 text-sm mb-4">
                          <div>
                            <div className="text-gray-500 dark:text-gray-400 mb-1">
                              Permissions
                            </div>
                            <div className="space-y-1">
                              <span className="bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300 px-2 py-1 text-xs rounded block text-center">
                                {user.packagePermissions?.length || 0} Package
                              </span>
                              {user.customPermissions &&
                                user.customPermissions.length > 0 && (
                                  <span className="bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300 px-2 py-1 text-xs rounded block text-center">
                                    {user.customPermissions.length} Custom
                                  </span>
                                )}
                            </div>
                          </div>
                          <div>
                            <div className="text-gray-500 dark:text-gray-400 mb-1">
                              Last Payment
                            </div>
                            <div className="text-gray-900 dark:text-gray-100">
                              {user.lastPaymentDate
                                ? new Date(
                                    user.lastPaymentDate
                                  ).toLocaleDateString()
                                : 'Never'}
                            </div>
                          </div>
                        </div>

                        {/* Actions */}
                        <div className="space-y-2">
                          <button
                            onClick={() => setSelectedUser(user.id)}
                            className="w-full min-h-[44px] px-4 py-3 text-blue-600 hover:text-blue-900 dark:text-blue-400 dark:hover:text-blue-300 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded-lg transition-colors border border-blue-200 dark:border-blue-800"
                          >
                            Manage Permissions
                          </button>
                          <select
                            onChange={e => {
                              if (e.target.value) {
                                if (
                                  confirm(
                                    `Upgrade ${user.email} to ${e.target.value}?`
                                  )
                                ) {
                                  handlePackageUpgrade(
                                    user.id,
                                    e.target.value as PackageTier
                                  );
                                }
                                e.target.value = '';
                              }
                            }}
                            className="w-full min-h-[44px] text-sm border border-gray-300 dark:border-gray-600 rounded-lg px-4 py-3 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                            defaultValue=""
                          >
                            <option value="" disabled>
                              Upgrade to...
                            </option>
                            {Object.values(PackageTier)
                              .filter(tier => tier !== user.packageTier)
                              .map(tier => (
                                <option key={tier} value={tier}>
                                  {tier}
                                </option>
                              ))}
                          </select>
                        </div>
                      </div>
                    ))}
                  </div>
                </>
              )}
            </div>
          </div>
        ) : (
          <div className="p-6">
            <div className="text-center text-gray-500 dark:text-gray-400">
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
                  <div className="mt-2 p-4 bg-gray-100 dark:bg-gray-800 rounded">
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

        {/* Permission Manager Modal */}
        {selectedUser && (
          <UserPermissionManager
            userId={selectedUser}
            onClose={() => setSelectedUser(null)}
          />
        )}
      </div>
    </div>
  );
};
