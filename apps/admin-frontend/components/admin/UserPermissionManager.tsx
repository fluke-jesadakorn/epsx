'use client';

import { useToast } from '@/components/ui/toast';
import { adminLogger } from '@/lib/logger';
import React, { useEffect, useState } from 'react';
import type { UserWithPermissions } from '../../types/admin/iam';
import { PackageTier } from '../../types/admin/iam';
// import { PERMISSION_PROFILES } from '../../config/packagePermissions'; // Config removed
// import { iamService } from '../../services/iamService'; // Service removed

// Placeholder for removed dependencies
const PERMISSION_PROFILES: any[] = [];
const iamService = {
  getUser: async (...args: any[]) => null,
  updateUserPackageTier: async (...args: any[]) => {},
  applyPermissionProfileToUser: async (...args: any[]) => {},
  getUserWithPermissions: async (
    ...args: any[]
  ): Promise<UserWithPermissions | null> => null,
  previewPackageUpgrade: async (...args: any[]) => ({
    changes: [],
    newTier: 'FREE',
    addedPermissions: [],
    removedPermissions: [],
    currentPermissions: [],
    newPermissions: [],
  }),
  grantCustomPermission: async (...args: any[]) => {},
  revokeCustomPermission: async (...args: any[]) => {},
  bulkApplyProfile: async (...args: any[]) => {},
};

interface UserPermissionManagerProps {
  userId: string;
  onClose: () => void;
}

export const UserPermissionManager: React.FC<UserPermissionManagerProps> = ({
  userId,
  onClose,
}) => {
  const [user, setUser] = useState<UserWithPermissions | null>(null);
  const [loading, setLoading] = useState(true);
  const [selectedProfile, setSelectedProfile] = useState<string>('');
  const [customPermissionForm, setCustomPermissionForm] = useState({
    featureId: '',
    action: '',
    resource: '',
    reason: '',
    expiresAt: '',
  });
  const [isGranting, setIsGranting] = useState(false);
  const [isUpgrading, setIsUpgrading] = useState(false);
  const { addToast } = useToast();

  useEffect(() => {
    loadUserDetails();
  }, [userId]);

  // Handle escape key
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    };
    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  }, [onClose]);

  const loadUserDetails = async () => {
    try {
      setLoading(true);
      const userData = await iamService.getUserWithPermissions(userId);
      setUser(userData);
    } catch (error) {
      adminLogger.error('Failed to load user details', {
        userId,
        error: error instanceof Error ? error.message : String(error),
      });
      addToast({
        type: 'error',
        title: 'Failed to load user details',
        description: 'Please try again',
      });
    } finally {
      setLoading(false);
    }
  };

  const handlePackageUpgrade = async (newTier: PackageTier) => {
    if (!user) return;

    try {
      setIsUpgrading(true);
      // Preview the upgrade first
      const preview = await iamService.previewPackageUpgrade(userId, newTier);

      if (
        confirm(
          `This will add ${preview.addedPermissions.length} new permissions. Continue?`
        )
      ) {
        await iamService.updateUserPackageTier(
          userId,
          newTier,
          'current-admin-id'
        ); // Get from auth context
        await loadUserDetails();
        addToast({
          type: 'success',
          title: 'Package upgraded successfully!',
        });
      }
    } catch (error) {
      adminLogger.error('Failed to upgrade package', {
        userId,
        newTier,
        error: error instanceof Error ? error.message : String(error),
      });
      addToast({
        type: 'error',
        title: 'Failed to upgrade package permissions',
        description:
          error instanceof Error ? error.message : 'Please try again',
      });
    } finally {
      setIsUpgrading(false);
    }
  };

  const handleGrantCustomPermission = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!user) return;

    try {
      setIsGranting(true);
      await iamService.grantCustomPermission(
        userId,
        customPermissionForm.featureId,
        {
          action: customPermissionForm.action,
          resource: customPermissionForm.resource,
        },
        'current-admin-id', // Get from auth context
        {
          reason: customPermissionForm.reason,
          expiresAt: customPermissionForm.expiresAt
            ? new Date(customPermissionForm.expiresAt)
            : undefined,
        }
      );

      // Reset form
      setCustomPermissionForm({
        featureId: '',
        action: '',
        resource: '',
        reason: '',
        expiresAt: '',
      });

      await loadUserDetails();
      addToast({
        type: 'success',
        title: 'Custom permission granted successfully!',
      });
    } catch (error) {
      adminLogger.error('Failed to grant custom permission', {
        userId,
        featureId: customPermissionForm.featureId,
        error: error instanceof Error ? error.message : String(error),
      });
      addToast({
        type: 'error',
        title: 'Failed to grant custom permission',
        description:
          error instanceof Error ? error.message : 'Please try again',
      });
    } finally {
      setIsGranting(false);
    }
  };

  const handleRevokeCustomPermission = async (permissionId: string) => {
    const reason = prompt('Reason for revoking this permission:');
    if (!reason) return;

    try {
      await iamService.revokeCustomPermission(
        permissionId,
        'current-admin-id',
        reason
      );
      await loadUserDetails();
      addToast({
        type: 'success',
        title: 'Permission revoked successfully!',
      });
    } catch (error) {
      adminLogger.error('Failed to revoke permission', {
        permissionId,
        error: error instanceof Error ? error.message : String(error),
      });
      addToast({
        type: 'error',
        title: 'Failed to revoke permission',
        description:
          error instanceof Error ? error.message : 'Please try again',
      });
    }
  };

  const handleApplyProfile = async () => {
    if (!selectedProfile || !user) return;

    try {
      await iamService.bulkApplyProfile(
        [userId],
        selectedProfile,
        'current-admin-id'
      );
      await loadUserDetails();
      setSelectedProfile('');
      addToast({
        type: 'success',
        title: 'Permission profile applied successfully!',
      });
    } catch (error) {
      adminLogger.error('Failed to apply permission profile', {
        userId,
        profileId: selectedProfile,
        error: error instanceof Error ? error.message : String(error),
      });
      addToast({
        type: 'error',
        title: 'Failed to apply permission profile',
        description:
          error instanceof Error ? error.message : 'Please try again',
      });
    }
  };

  if (loading)
    return (
      <div
        className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-[9999] p-4"
        onClick={e => e.target === e.currentTarget && onClose()}
        role="dialog"
        aria-modal="true"
        aria-labelledby="loading-dialog"
      >
        <div className="bg-white dark:bg-gray-800 rounded-lg p-8 shadow-xl max-w-sm w-full">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto"></div>
          <p
            id="loading-dialog"
            className="mt-4 text-gray-600 dark:text-gray-300 text-center"
          >
            Loading user permissions...
          </p>
        </div>
      </div>
    );

  if (!user)
    return (
      <div
        className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-[9999] p-4"
        onClick={e => e.target === e.currentTarget && onClose()}
        role="dialog"
        aria-modal="true"
        aria-labelledby="error-dialog"
      >
        <div className="bg-white dark:bg-gray-800 rounded-lg p-8 shadow-xl max-w-sm w-full">
          <p
            id="error-dialog"
            className="text-red-600 dark:text-red-400 text-center"
          >
            User not found
          </p>
          <button
            onClick={onClose}
            className="mt-4 w-full min-h-[44px] px-4 py-3 bg-gray-500 hover:bg-gray-600 text-white rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2"
            autoFocus
          >
            Close
          </button>
        </div>
      </div>
    );

  return (
    <div
      className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-[9999] p-4"
      onClick={e => e.target === e.currentTarget && onClose()}
      role="dialog"
      aria-modal="true"
      aria-labelledby="permission-dialog-title"
    >
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-6xl w-full max-h-[90vh] flex flex-col">
        <div className="p-6 border-b border-gray-200 dark:border-gray-700 flex-shrink-0">
          <div className="flex justify-between items-center">
            <h2
              id="permission-dialog-title"
              className="text-2xl font-bold text-gray-900 dark:text-white"
            >
              Permission Management - {user.email}
            </h2>
            <button
              onClick={onClose}
              className="min-h-[44px] min-w-[44px] flex items-center justify-center text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2"
              aria-label="Close dialog"
            >
              <span className="text-2xl font-bold">×</span>
            </button>
          </div>
          <div className="mt-2 flex items-center space-x-4">
            <span
              className={`px-3 py-1 rounded-full text-sm font-medium ${
                user.subscriptionStatus === 'active'
                  ? 'bg-green-100 text-green-800'
                  : 'bg-red-100 text-red-800'
              }`}
            >
              {user.subscriptionStatus}
            </span>
            <span className="px-3 py-1 bg-blue-100 text-blue-800 rounded-full text-sm font-medium">
              {user.packageTier}
            </span>
          </div>
        </div>

        <div className="p-6 space-y-8 overflow-y-auto flex-1">
          {/* Package Upgrade Section */}
          <div className="bg-gray-50 rounded-lg p-4">
            <h3 className="text-lg font-semibold mb-4">Package Management</h3>
            <div className="flex flex-wrap gap-2">
              {Object.values(PackageTier).map(tier => (
                <button
                  key={tier}
                  onClick={() => handlePackageUpgrade(tier)}
                  disabled={user.packageTier === tier || isUpgrading}
                  className={`px-4 py-2 rounded-lg font-medium transition-colors ${
                    user.packageTier === tier
                      ? 'bg-blue-100 text-blue-800 cursor-not-allowed'
                      : isUpgrading
                        ? 'bg-gray-100 text-gray-400 cursor-not-allowed'
                        : 'bg-white border border-gray-300 hover:bg-gray-50'
                  }`}
                >
                  {tier.charAt(0).toUpperCase() + tier.slice(1)}
                  {user.packageTier === tier && ' (Current)'}
                </button>
              ))}
            </div>
          </div>

          {/* Profile Application Section */}
          <div className="bg-gray-50 rounded-lg p-4">
            <h3 className="text-lg font-semibold mb-4">
              Apply Permission Profile
            </h3>
            <div className="flex gap-4">
              <select
                value={selectedProfile}
                onChange={e => setSelectedProfile(e.target.value)}
                className="flex-1 border border-gray-300 rounded-lg px-3 py-2"
              >
                <option value="">Select a permission profile...</option>
                {PERMISSION_PROFILES.map(profile => (
                  <option key={profile.id} value={profile.id}>
                    {profile.name} - {profile.description}
                  </option>
                ))}
              </select>
              <button
                onClick={handleApplyProfile}
                disabled={!selectedProfile}
                className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Apply Profile
              </button>
            </div>
          </div>

          {/* Custom Permission Grant Section */}
          <div className="bg-gray-50 rounded-lg p-4">
            <h3 className="text-lg font-semibold mb-4">
              Grant Custom Permission
            </h3>
            <form onSubmit={handleGrantCustomPermission} className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Feature ID
                  </label>
                  <input
                    type="text"
                    value={customPermissionForm.featureId}
                    onChange={e =>
                      setCustomPermissionForm({
                        ...customPermissionForm,
                        featureId: e.target.value,
                      })
                    }
                    placeholder="e.g., api_boost"
                    className="w-full border border-gray-300 rounded-lg px-3 py-2"
                    required
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Action
                  </label>
                  <select
                    value={customPermissionForm.action}
                    onChange={e =>
                      setCustomPermissionForm({
                        ...customPermissionForm,
                        action: e.target.value,
                      })
                    }
                    className="w-full border border-gray-300 rounded-lg px-3 py-2"
                    required
                  >
                    <option value="">Select action...</option>
                    <option value="view">View</option>
                    <option value="create">Create</option>
                    <option value="edit">Edit</option>
                    <option value="delete">Delete</option>
                    <option value="execute">Execute</option>
                    <option value="*">All Actions</option>
                  </select>
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Resource
                  </label>
                  <input
                    type="text"
                    value={customPermissionForm.resource}
                    onChange={e =>
                      setCustomPermissionForm({
                        ...customPermissionForm,
                        resource: e.target.value,
                      })
                    }
                    placeholder="e.g., api:special_feature"
                    className="w-full border border-gray-300 rounded-lg px-3 py-2"
                    required
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Expires At (Optional)
                  </label>
                  <input
                    type="datetime-local"
                    value={customPermissionForm.expiresAt}
                    onChange={e =>
                      setCustomPermissionForm({
                        ...customPermissionForm,
                        expiresAt: e.target.value,
                      })
                    }
                    className="w-full border border-gray-300 rounded-lg px-3 py-2"
                  />
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Reason
                </label>
                <textarea
                  value={customPermissionForm.reason}
                  onChange={e =>
                    setCustomPermissionForm({
                      ...customPermissionForm,
                      reason: e.target.value,
                    })
                  }
                  placeholder="Reason for granting this permission..."
                  className="w-full border border-gray-300 rounded-lg px-3 py-2 rows-2"
                  required
                />
              </div>

              <button
                type="submit"
                disabled={isGranting}
                className="w-full bg-green-600 text-white rounded-lg py-2 hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isGranting ? 'Granting...' : 'Grant Custom Permission'}
              </button>
            </form>
          </div>

          {/* Current Permissions Display */}
          <div className="space-y-6">
            {/* Package Permissions */}
            <div>
              <h3 className="text-lg font-semibold mb-4">
                Package Permissions
              </h3>
              <div className="bg-green-50 rounded-lg p-4">
                {user.packagePermissions?.length > 0 ? (
                  <div className="space-y-2">
                    {user.packagePermissions.map((permission, index) => (
                      <div
                        key={index}
                        className="flex justify-between items-center py-2 px-3 bg-white rounded border"
                      >
                        <div>
                          <span className="font-medium">
                            {permission.featureId}
                          </span>
                          <span className="text-gray-500 ml-2">
                            {permission.permission.action} on{' '}
                            {permission.permission.resource}
                          </span>
                        </div>
                        <span className="text-sm text-green-600 bg-green-100 px-2 py-1 rounded">
                          Package
                        </span>
                      </div>
                    ))}
                  </div>
                ) : (
                  <p className="text-gray-500">No package permissions found</p>
                )}
              </div>
            </div>

            {/* Custom Permissions */}
            <div>
              <h3 className="text-lg font-semibold mb-4">Custom Permissions</h3>
              <div className="bg-blue-50 rounded-lg p-4">
                {user.customPermissions?.length > 0 ? (
                  <div className="space-y-2">
                    {user.customPermissions.map(permission => (
                      <div
                        key={permission.id}
                        className="flex justify-between items-center py-2 px-3 bg-white rounded border"
                      >
                        <div>
                          <span className="font-medium">
                            {permission.featureId}
                          </span>
                          <span className="text-gray-500 ml-2">
                            {permission.permission.action} on{' '}
                            {permission.permission.resource}
                          </span>
                          {permission.expiresAt && (
                            <span className="text-sm text-orange-600 ml-2">
                              (Expires:{' '}
                              {new Date(
                                permission.expiresAt
                              ).toLocaleDateString()}
                              )
                            </span>
                          )}
                          {permission.reason && (
                            <div className="text-sm text-gray-600 mt-1">
                              Reason: {permission.reason}
                            </div>
                          )}
                        </div>
                        <div className="flex items-center space-x-2">
                          <span className="text-sm text-blue-600 bg-blue-100 px-2 py-1 rounded">
                            Custom
                          </span>
                          <button
                            onClick={() =>
                              handleRevokeCustomPermission(permission.id)
                            }
                            className="text-red-600 hover:text-red-800 text-sm"
                          >
                            Revoke
                          </button>
                        </div>
                      </div>
                    ))}
                  </div>
                ) : (
                  <p className="text-gray-500">No custom permissions granted</p>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
