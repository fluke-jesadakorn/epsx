'use client';

import { adminLogger } from '@/lib/logger';
import type { CustomPermission } from '@/types/admin/iam';
import { Edit, Eye, Filter, Key, Plus, Shield, Trash2 } from 'lucide-react';
import { useEffect, useState } from 'react';

interface PermissionStats {
  totalPermissions: number;
  activePermissions: number;
  categories: number;
  customPermissions: number;
}

interface PermissionWithDetails extends CustomPermission {
  category?: string;
  scope?: string;
}

const PERMISSION_CATEGORIES = [
  'All',
  'Dashboard',
  'API',
  'Data',
  'Admin',
  'Analytics',
  'Custom',
];

export function PermissionManagementDashboard() {
  const [permissions, setPermissions] = useState<PermissionWithDetails[]>([]);
  const [stats, setStats] = useState<PermissionStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedPermission, setSelectedPermission] =
    useState<PermissionWithDetails | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [activeCategory, setActiveCategory] = useState('All');
  const [searchTerm, setSearchTerm] = useState('');

  useEffect(() => {
    loadPermissions();
  }, []);

  const loadPermissions = async () => {
    try {
      setLoading(true);
      setError(null);

      const response = await fetch('/api/v1/admin/iam/custom-permissions');
      if (!response.ok) {
        throw new Error('Failed to fetch permissions');
      }

      const data = await response.json();

      // Transform permissions and add mock categories/scopes
      const permissionsWithDetails: PermissionWithDetails[] = (data || []).map(
        (perm: CustomPermission) => ({
          ...perm,
          category: getCategoryFromPermission(perm.permission.resource),
          scope: getScopeFromPermission(perm.permission.action),
        })
      );

      setPermissions(permissionsWithDetails);

      // Calculate stats
      setStats({
        totalPermissions: permissionsWithDetails.length,
        activePermissions: permissionsWithDetails.filter(p => p.isActive)
          .length,
        categories: new Set(permissionsWithDetails.map(p => p.category)).size,
        customPermissions: permissionsWithDetails.length,
      });
    } catch (err: any) {
      adminLogger.error('Failed to load permissions', { error: err.message });
      setError(err.message || 'Failed to load permissions');
    } finally {
      setLoading(false);
    }
  };

  const getCategoryFromPermission = (resource: string): string => {
    if (resource.includes('features:beta')) return 'Features';
    if (resource.includes('admin')) return 'Admin';
    if (resource.includes('data')) return 'Data';
    if (resource.includes('api')) return 'API';
    if (resource.includes('dashboard')) return 'Dashboard';
    return 'Custom';
  };

  const getScopeFromPermission = (action: string): string => {
    if (action.includes('view') || action.includes('read')) return 'READ';
    if (action.includes('write') || action.includes('create')) return 'WRITE';
    if (action.includes('delete')) return 'DELETE';
    if (action.includes('manage')) return 'MANAGE';
    return 'CUSTOM';
  };

  const filteredPermissions = permissions.filter(permission => {
    const matchesCategory =
      activeCategory === 'All' || permission.category === activeCategory;
    const matchesSearch =
      permission.permission.resource
        .toLowerCase()
        .includes(searchTerm.toLowerCase()) ||
      permission.permission.action
        .toLowerCase()
        .includes(searchTerm.toLowerCase());
    return matchesCategory && matchesSearch;
  });

  const handleDeletePermission = async (permissionId: string) => {
    if (!confirm('Are you sure you want to delete this permission?')) {
      return;
    }

    try {
      // TODO: Implement delete API call
      // Permission deletion would be handled by backend API
      // Remove from local state for now
      setPermissions(permissions.filter(perm => perm.id !== permissionId));
    } catch (err) {
      adminLogger.error('Failed to delete permission', {
        error:
          typeof err === 'object' && err !== null && 'message' in err
            ? (err as { message?: string }).message
            : String(err),
        permissionId,
      });
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-64">
        <div className="animate-spin rounded-full h-10 w-10 border-4 border-primary/20 border-t-primary"></div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
        <p className="text-red-800 dark:text-red-200">{error}</p>
        <button
          onClick={loadPermissions}
          className="mt-2 px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-md text-sm transition-colors"
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
            Permission Management
          </h1>
          <p className="text-gray-600 dark:text-gray-400">
            Manage custom permissions and access controls
          </p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="flex items-center space-x-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
        >
          <Plus className="w-4 h-4" />
          <span>Create Permission</span>
        </button>
      </div>

      {/* Stats Cards */}
      {stats && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
            <div className="flex items-center space-x-3">
              <Key className="w-8 h-8 text-blue-600" />
              <div>
                <p className="text-sm font-medium text-gray-600 dark:text-gray-400">
                  Total Permissions
                </p>
                <p className="text-2xl font-bold text-gray-900 dark:text-white">
                  {stats.totalPermissions}
                </p>
              </div>
            </div>
          </div>

          <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
            <div className="flex items-center space-x-3">
              <Shield className="w-8 h-8 text-green-600" />
              <div>
                <p className="text-sm font-medium text-gray-600 dark:text-gray-400">
                  Active
                </p>
                <p className="text-2xl font-bold text-gray-900 dark:text-white">
                  {stats.activePermissions}
                </p>
              </div>
            </div>
          </div>

          <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
            <div className="flex items-center space-x-3">
              <Filter className="w-8 h-8 text-purple-600" />
              <div>
                <p className="text-sm font-medium text-gray-600 dark:text-gray-400">
                  Categories
                </p>
                <p className="text-2xl font-bold text-gray-900 dark:text-white">
                  {stats.categories}
                </p>
              </div>
            </div>
          </div>

          <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
            <div className="flex items-center space-x-3">
              <Plus className="w-8 h-8 text-orange-600" />
              <div>
                <p className="text-sm font-medium text-gray-600 dark:text-gray-400">
                  Custom
                </p>
                <p className="text-2xl font-bold text-gray-900 dark:text-white">
                  {stats.customPermissions}
                </p>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Filters and Search */}
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
        <div className="flex flex-col sm:flex-row gap-4">
          {/* Category Tabs */}
          <div className="flex flex-wrap gap-2">
            {PERMISSION_CATEGORIES.map(category => (
              <button
                key={category}
                onClick={() => setActiveCategory(category)}
                className={`px-3 py-2 text-sm rounded-md transition-colors ${
                  activeCategory === category
                    ? 'bg-blue-600 text-white'
                    : 'bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'
                }`}
              >
                {category}
              </button>
            ))}
          </div>

          {/* Search */}
          <div className="flex-1">
            <input
              type="text"
              placeholder="Search permissions..."
              value={searchTerm}
              onChange={e => setSearchTerm(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400"
            />
          </div>
        </div>
      </div>

      {/* Permissions Table */}
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
        <div className="p-6 border-b border-gray-200 dark:border-gray-700">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
            Permissions ({filteredPermissions.length})
          </h2>
        </div>

        <div className="overflow-x-auto">
          <table className="w-full">
            <thead className="bg-gray-50 dark:bg-gray-700">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                  Permission
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                  Category
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                  Scope
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                  Status
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                  Granted At
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                  Actions
                </th>
              </tr>
            </thead>
            <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
              {filteredPermissions.map(permission => (
                <tr
                  key={permission.id}
                  className="hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
                >
                  <td className="px-6 py-4">
                    <div className="flex items-center">
                      <Key className="w-5 h-5 text-blue-600 mr-3" />
                      <div>
                        <div className="text-sm font-medium text-gray-900 dark:text-white">
                          {permission.permission.action}
                        </div>
                        <div className="text-sm text-gray-500 dark:text-gray-400">
                          {permission.permission.resource}
                        </div>
                      </div>
                    </div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200">
                      {permission.category}
                    </span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200">
                      {permission.scope}
                    </span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span
                      className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                        permission.isActive
                          ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
                          : 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200'
                      }`}
                    >
                      {permission.isActive ? 'Active' : 'Inactive'}
                    </span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                    {new Date(permission.grantedAt).toLocaleDateString()}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm font-medium">
                    <div className="flex items-center space-x-2">
                      <button
                        onClick={() => setSelectedPermission(permission)}
                        className="text-blue-600 hover:text-blue-900 dark:text-blue-400 dark:hover:text-blue-300"
                        title="View Details"
                      >
                        <Eye className="w-4 h-4" />
                      </button>
                      <button
                        onClick={() => {
                          /* TODO: Implement edit permission */
                        }}
                        className="text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-300"
                        title="Edit Permission"
                      >
                        <Edit className="w-4 h-4" />
                      </button>
                      <button
                        onClick={() => handleDeletePermission(permission.id)}
                        className="text-red-600 hover:text-red-900 dark:text-red-400 dark:hover:text-red-300"
                        title="Delete Permission"
                      >
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        {filteredPermissions.length === 0 && !loading && (
          <div className="text-center py-12">
            <Key className="mx-auto h-12 w-12 text-gray-400" />
            <h3 className="mt-2 text-sm font-medium text-gray-900 dark:text-white">
              {permissions.length === 0
                ? 'No permissions found'
                : 'No matching permissions'}
            </h3>
            <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
              {permissions.length === 0
                ? 'Get started by creating a new permission.'
                : 'Try adjusting your search or filter criteria.'}
            </p>
            {permissions.length === 0 && (
              <div className="mt-6">
                <button
                  onClick={() => setShowCreateModal(true)}
                  className="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700"
                >
                  <Plus className="w-4 h-4 mr-2" />
                  Create Permission
                </button>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Permission Details Modal */}
      {selectedPermission && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white dark:bg-gray-800 rounded-lg p-6 max-w-2xl w-full mx-4">
            <div className="flex justify-between items-center mb-4">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                Permission Details
              </h3>
              <button
                onClick={() => setSelectedPermission(null)}
                className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
              >
                ✕
              </button>
            </div>
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Action
                  </label>
                  <p className="mt-1 text-sm text-gray-900 dark:text-white">
                    {selectedPermission.permission.action}
                  </p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Resource
                  </label>
                  <p className="mt-1 text-sm text-gray-900 dark:text-white">
                    {selectedPermission.permission.resource}
                  </p>
                </div>
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Category
                  </label>
                  <p className="mt-1 text-sm text-gray-900 dark:text-white">
                    {selectedPermission.category}
                  </p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Scope
                  </label>
                  <p className="mt-1 text-sm text-gray-900 dark:text-white">
                    {selectedPermission.scope}
                  </p>
                </div>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                  Granted By
                </label>
                <p className="mt-1 text-sm text-gray-900 dark:text-white">
                  {selectedPermission.grantedBy}
                </p>
              </div>
              {selectedPermission.reason && (
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Reason
                  </label>
                  <p className="mt-1 text-sm text-gray-900 dark:text-white">
                    {selectedPermission.reason}
                  </p>
                </div>
              )}
            </div>
            <div className="mt-6 flex justify-end">
              <button
                onClick={() => setSelectedPermission(null)}
                className="px-4 py-2 bg-gray-300 hover:bg-gray-400 text-gray-700 rounded-md text-sm transition-colors"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Create Permission Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white dark:bg-gray-800 rounded-lg p-6 max-w-md w-full mx-4">
            <div className="flex justify-between items-center mb-4">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                Create New Permission
              </h3>
              <button
                onClick={() => setShowCreateModal(false)}
                className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
              >
                ✕
              </button>
            </div>
            <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
              Permission creation functionality will be implemented in the next
              phase.
            </p>
            <div className="flex justify-end">
              <button
                onClick={() => setShowCreateModal(false)}
                className="px-4 py-2 bg-gray-300 hover:bg-gray-400 text-gray-700 rounded-md text-sm transition-colors"
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
