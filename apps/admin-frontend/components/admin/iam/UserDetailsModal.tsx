'use client';

import React, { useState } from 'react';
import { X, User, Crown, Shield, Calendar, Mail, Save, AlertTriangle } from 'lucide-react';
import type { UserWithPermissions } from '../../../types/admin/iam-enhanced';
import { PackageTier, SubscriptionStatus } from '../../../types/admin/iam-enhanced';
import { iamService } from '../../../services/iamService';

interface UserDetailsModalProps {
  user: UserWithPermissions;
  onClose: () => void;
  onUpdate: () => void;
}

export const UserDetailsModal: React.FC<UserDetailsModalProps> = ({ user, onClose, onUpdate }) => {
  const [activeTab, setActiveTab] = useState<'details' | 'permissions' | 'activity'>('details');
  const [editMode, setEditMode] = useState(false);
  const [editedUser, setEditedUser] = useState({
    packageTier: user.packageTier,
    subscriptionStatus: user.subscriptionStatus,
    displayName: user.displayName || '',
  });
  const [loading, setLoading] = useState(false);

  const handleSave = async () => {
    try {
      setLoading(true);
      
      if (editedUser.packageTier !== user.packageTier) {
        await iamService.updateUserPackageTier(user.id, editedUser.packageTier, 'current-admin-id');
      }
      
      onUpdate();
      setEditMode(false);
    } catch (error) {
      console.error('Failed to update user:', error);
      alert('Failed to update user');
    } finally {
      setLoading(false);
    }
  };

  const getTierColor = (tier: PackageTier) => {
    switch (tier) {
      case PackageTier.ENTERPRISE:
        return 'bg-purple-100 text-purple-800 border-purple-200';
      case PackageTier.PLATINUM:
        return 'bg-gray-100 text-gray-800 border-gray-200';
      case PackageTier.GOLD:
        return 'bg-yellow-100 text-yellow-800 border-yellow-200';
      case PackageTier.SILVER:
        return 'bg-blue-100 text-blue-800 border-blue-200';
      case PackageTier.BRONZE:
        return 'bg-orange-100 text-orange-800 border-orange-200';
      default:
        return 'bg-gray-100 text-gray-500 border-gray-200';
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active':
        return 'bg-green-100 text-green-800';
      case 'trial':
        return 'bg-blue-100 text-blue-800';
      case 'expired':
        return 'bg-red-100 text-red-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-lg shadow-xl max-w-4xl w-full max-h-[90vh] overflow-hidden">
        {/* Header */}
        <div className="bg-gray-50 px-6 py-4 border-b border-gray-200">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-4">
              <div className="h-12 w-12 rounded-full bg-blue-100 flex items-center justify-center">
                <User className="h-6 w-6 text-blue-600" />
              </div>
              <div>
                <h2 className="text-xl font-semibold text-gray-900">
                  {user.displayName || user.name || user.email?.split('@')[0]}
                </h2>
                <p className="text-sm text-gray-500">{user.email}</p>
              </div>
            </div>
            <div className="flex items-center gap-2">
              {editMode ? (
                <>
                  <button
                    onClick={() => setEditMode(false)}
                    className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
                  >
                    Cancel
                  </button>
                  <button
                    onClick={handleSave}
                    disabled={loading}
                    className="px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 disabled:opacity-50 flex items-center"
                  >
                    <Save className="h-4 w-4 mr-2" />
                    {loading ? 'Saving...' : 'Save'}
                  </button>
                </>
              ) : (
                <button
                  onClick={() => setEditMode(true)}
                  className="px-4 py-2 text-sm font-medium text-blue-600 bg-blue-50 border border-blue-200 rounded-md hover:bg-blue-100"
                >
                  Edit
                </button>
              )}
              <button
                onClick={onClose}
                className="text-gray-400 hover:text-gray-600"
              >
                <X className="h-6 w-6" />
              </button>
            </div>
          </div>

          {/* Status Badges */}
          <div className="flex items-center gap-3 mt-4">
            <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border ${getTierColor(user.packageTier)}`}>
              <Crown className="h-3 w-3 mr-1" />
              {user.packageTier.toUpperCase()}
            </span>
            <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusColor(user.subscriptionStatus)}`}>
              {user.subscriptionStatus}
            </span>
            {user.customPermissions && user.customPermissions.length > 0 && (
              <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-purple-100 text-purple-800">
                <Shield className="h-3 w-3 mr-1" />
                {user.customPermissions.length} Custom Permissions
              </span>
            )}
          </div>
        </div>

        {/* Tabs */}
        <div className="border-b border-gray-200">
          <nav className="-mb-px flex">
            {[
              { id: 'details', label: 'Details', icon: User },
              { id: 'permissions', label: 'Permissions', icon: Shield },
              { id: 'activity', label: 'Activity', icon: Calendar },
            ].map((tab) => {
              const Icon = tab.icon;
              return (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id as any)}
                  className={`flex items-center px-6 py-4 text-sm font-medium border-b-2 ${
                    activeTab === tab.id
                      ? 'border-blue-500 text-blue-600'
                      : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                  }`}
                >
                  <Icon className="h-4 w-4 mr-2" />
                  {tab.label}
                </button>
              );
            })}
          </nav>
        </div>

        {/* Content */}
        <div className="p-6 max-h-96 overflow-y-auto">
          {activeTab === 'details' && (
            <div className="space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div>
                  <h3 className="text-lg font-medium text-gray-900 mb-4">User Information</h3>
                  <div className="space-y-4">
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">
                        Display Name
                      </label>
                      {editMode ? (
                        <input
                          type="text"
                          value={editedUser.displayName}
                          onChange={(e) => setEditedUser({ ...editedUser, displayName: e.target.value })}
                          className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:border-blue-500"
                        />
                      ) : (
                        <p className="text-sm text-gray-900">{user.displayName || 'Not set'}</p>
                      )}
                    </div>
                    
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">
                        <Mail className="h-4 w-4 inline mr-1" />
                        Email
                      </label>
                      <p className="text-sm text-gray-900">{user.email}</p>
                    </div>
                    
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">
                        User ID
                      </label>
                      <p className="text-sm text-gray-500 font-mono">{user.id}</p>
                    </div>
                  </div>
                </div>

                <div>
                  <h3 className="text-lg font-medium text-gray-900 mb-4">Subscription Details</h3>
                  <div className="space-y-4">
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">
                        Package Tier
                      </label>
                      {editMode ? (
                        <select
                          value={editedUser.packageTier}
                          onChange={(e) => setEditedUser({ ...editedUser, packageTier: e.target.value as PackageTier })}
                          className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:border-blue-500"
                        >
                          {Object.values(PackageTier).map((tier) => (
                            <option key={tier} value={tier}>
                              {tier.charAt(0).toUpperCase() + tier.slice(1)}
                            </option>
                          ))}
                        </select>
                      ) : (
                        <p className="text-sm text-gray-900">{user.packageTier}</p>
                      )}
                    </div>
                    
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">
                        Subscription Status
                      </label>
                      {editMode ? (
                        <select
                          value={editedUser.subscriptionStatus}
                          onChange={(e) => setEditedUser({ ...editedUser, subscriptionStatus: e.target.value as SubscriptionStatus })}
                          className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:border-blue-500"
                        >
                          {Object.values(SubscriptionStatus).map((status) => (
                            <option key={status} value={status}>
                              {status.charAt(0).toUpperCase() + status.slice(1)}
                            </option>
                          ))}
                        </select>
                      ) : (
                        <p className="text-sm text-gray-900">{user.subscriptionStatus}</p>
                      )}
                    </div>
                    
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">
                        <Calendar className="h-4 w-4 inline mr-1" />
                        Last Payment Date
                      </label>
                      <p className="text-sm text-gray-900">
                        {user.lastPaymentDate ? new Date(user.lastPaymentDate).toLocaleDateString() : 'Never'}
                      </p>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          )}

          {activeTab === 'permissions' && (
            <div className="space-y-6">
              <div>
                <h3 className="text-lg font-medium text-gray-900 mb-4">Package Permissions</h3>
                {user.packagePermissions && user.packagePermissions.length > 0 ? (
                  <div className="space-y-2">
                    {user.packagePermissions.map((permission, index) => (
                      <div key={index} className="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                        <div>
                          <p className="text-sm font-medium text-gray-900">{permission.featureId}</p>
                          <p className="text-xs text-gray-500">{permission.permission.action} on {permission.permission.resource}</p>
                        </div>
                        <span className="text-xs bg-blue-100 text-blue-800 px-2 py-1 rounded-full">Package</span>
                      </div>
                    ))}
                  </div>
                ) : (
                  <p className="text-sm text-gray-500">No package permissions found.</p>
                )}
              </div>

              <div>
                <h3 className="text-lg font-medium text-gray-900 mb-4">Custom Permissions</h3>
                {user.customPermissions && user.customPermissions.length > 0 ? (
                  <div className="space-y-2">
                    {user.customPermissions.map((permission) => (
                      <div key={permission.id} className="flex items-center justify-between p-3 bg-yellow-50 rounded-lg border border-yellow-200">
                        <div>
                          <p className="text-sm font-medium text-gray-900">{permission.featureId}</p>
                          <p className="text-xs text-gray-500">{permission.permission.action} on {permission.permission.resource}</p>
                          {permission.reason && (
                            <p className="text-xs text-gray-500 mt-1">Reason: {permission.reason}</p>
                          )}
                        </div>
                        <div className="text-right">
                          <span className="text-xs bg-yellow-100 text-yellow-800 px-2 py-1 rounded-full">Custom</span>
                          {permission.expiresAt && (
                            <p className="text-xs text-gray-500 mt-1">
                              Expires: {new Date(permission.expiresAt).toLocaleDateString()}
                            </p>
                          )}
                        </div>
                      </div>
                    ))}
                  </div>
                ) : (
                  <p className="text-sm text-gray-500">No custom permissions granted.</p>
                )}
              </div>
            </div>
          )}

          {activeTab === 'activity' && (
            <div className="space-y-6">
              <div>
                <h3 className="text-lg font-medium text-gray-900 mb-4">Recent Activity</h3>
                <div className="space-y-3">
                  <div className="flex items-center p-3 bg-gray-50 rounded-lg">
                    <AlertTriangle className="h-5 w-5 text-yellow-500 mr-3" />
                    <div>
                      <p className="text-sm text-gray-900">Activity logs not implemented yet</p>
                      <p className="text-xs text-gray-500">This feature will show user activity history</p>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
