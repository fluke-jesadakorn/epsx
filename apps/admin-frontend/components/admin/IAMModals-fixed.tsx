'use client';

import { useState, useEffect } from 'react';
import { X, Save, Plus, Trash2, Search, Check, AlertCircle } from 'lucide-react';
import type { User, Role, Policy, Group } from '@/types/admin/iam';

interface UserModalProps {
  user: User | null;
  onClose: () => void;
  onSave: () => void;
}

interface RoleModalProps {
  role: Role | null;
  onClose: () => void;
  onSave: () => void;
}

/**
 * User Modal Component
 * Handles creating and editing users with role and group assignments
 */
export function UserModal({ user, onClose, onSave }: UserModalProps) {
  const [formData, setFormData] = useState({
    email: '',
    name: '',
    roles: [] as string[],
    groups: [] as string[],
    status: 'active' as 'active' | 'inactive' | 'disabled',
  });

  const [errors, setErrors] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (user) {
      setFormData({
        email: user.email,
        name: user.name || '',
        roles: user.roles,
        groups: user.groups,
        status: user.status,
      });
    } else {
      setFormData({
        email: '',
        name: '',
        roles: [],
        groups: [],
        status: 'active',
      });
    }
  }, [user]);

  const availableRoles = [
    { id: '1', name: 'Bronze', description: 'Basic tier access', attachedPolicies: ['BronzePolicy'], createdAt: '2024-01-01', updatedAt: '2024-01-01' },
    { id: '2', name: 'Silver', description: 'Premium tier access', attachedPolicies: ['SilverPolicy'], createdAt: '2024-01-01', updatedAt: '2024-01-01' },
    { id: '3', name: 'Gold', description: 'Advanced tier access', attachedPolicies: ['GoldPolicy'], createdAt: '2024-01-01', updatedAt: '2024-01-01' },
    { id: '4', name: 'Platinum', description: 'Full access tier', attachedPolicies: ['PlatinumPolicy'], createdAt: '2024-01-01', updatedAt: '2024-01-01' },
    { id: '5', name: 'Admin', description: 'Administrative access', attachedPolicies: ['AdminPolicy'], createdAt: '2024-01-01', updatedAt: '2024-01-01' }
  ];

  const availableGroups = [
    { id: '1', name: 'Administrators', description: 'System administrators', memberCount: 2, attachedPolicies: ['AdminPolicy'], createdAt: '2024-01-01', updatedAt: '2024-01-01' },
    { id: '2', name: 'Users', description: 'Regular users', memberCount: 15, attachedPolicies: ['UserPolicy'], createdAt: '2024-01-01', updatedAt: '2024-01-01' },
    { id: '3', name: 'Premium', description: 'Premium users', memberCount: 8, attachedPolicies: ['PremiumPolicy'], createdAt: '2024-01-01', updatedAt: '2024-01-01' }
  ];

  const handleRoleToggle = (roleId: string) => {
    setFormData(prev => ({
      ...prev,
      roles: prev.roles.includes(roleId)
        ? prev.roles.filter(r => r !== roleId)
        : [...prev.roles, roleId]
    }));
  };

  const handleGroupToggle = (groupId: string) => {
    setFormData(prev => ({
      ...prev,
      groups: prev.groups.includes(groupId)
        ? prev.groups.filter(g => g !== groupId)
        : [...prev.groups, groupId]
    }));
  };

  const validateForm = () => {
    const newErrors: Record<string, string> = {};

    if (!formData.email.trim()) {
      newErrors.email = 'Email is required';
    } else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(formData.email)) {
      newErrors.email = 'Please enter a valid email address';
    }

    if (!formData.name.trim()) {
      newErrors.name = 'Name is required';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async () => {
    if (!validateForm()) return;

    setLoading(true);
    try {
      // TODO: Implement actual API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      onSave();
    } catch (error) {
      console.error('Failed to save user:', error);
    } finally {
      setLoading(false);
    }
  };

  if (!user && !onClose) return null;

  return (
    <div className="fixed inset-0 bg-gray-600 bg-opacity-50 dark:bg-gray-900 dark:bg-opacity-50 overflow-y-auto h-full w-full z-50">
      <div className="relative top-20 mx-auto p-5 border w-11/12 max-w-4xl shadow-lg rounded-md bg-white dark:bg-gray-800 border-gray-200 dark:border-gray-700">
        <div className="flex justify-between items-center mb-6">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
            {user ? 'Edit User' : 'Create New User'}
          </h3>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
          >
            <X className="h-5 w-5" />
          </button>
        </div>

        <div className="space-y-6">
          {/* Basic Information */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Email *
              </label>
              <input
                type="email"
                value={formData.email}
                onChange={(e) => setFormData({ ...formData, email: e.target.value })}
                className={`w-full px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white ${
                  errors.email ? 'border-red-500' : 'border-gray-300'
                }`}
                placeholder="user@example.com"
              />
              {errors.email && (
                <p className="mt-1 text-sm text-red-600 dark:text-red-400">{errors.email}</p>
              )}
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Name *
              </label>
              <input
                type="text"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                className={`w-full px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white ${
                  errors.name ? 'border-red-500' : 'border-gray-300'
                }`}
                placeholder="John Doe"
              />
              {errors.name && (
                <p className="mt-1 text-sm text-red-600 dark:text-red-400">{errors.name}</p>
              )}
            </div>
          </div>

          {/* Status */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Status
            </label>
            <select
              value={formData.status}
              onChange={(e) => setFormData({ ...formData, status: e.target.value as 'active' | 'inactive' | 'disabled' })}
              className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
            >
              <option value="active">Active</option>
              <option value="inactive">Inactive</option>
              <option value="disabled">Disabled</option>
            </select>
          </div>

          {/* Roles */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Roles
            </label>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              {availableRoles.map(role => (
                <label key={role.id} className="flex items-center space-x-3 p-3 border rounded-md hover:bg-gray-50 dark:hover:bg-gray-700 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={formData.roles.includes(role.name)}
                    onChange={() => handleRoleToggle(role.name)}
                    className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                  />
                  <div className="flex-1">
                    <div className="text-sm font-medium text-gray-900 dark:text-white">{role.name}</div>
                    <div className="text-xs text-gray-500 dark:text-gray-400">{role.description}</div>
                  </div>
                </label>
              ))}
            </div>
          </div>

          {/* Groups */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Groups
            </label>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              {availableGroups.map(group => (
                <label key={group.id} className="flex items-center space-x-3 p-3 border rounded-md hover:bg-gray-50 dark:hover:bg-gray-700 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={formData.groups.includes(group.name)}
                    onChange={() => handleGroupToggle(group.name)}
                    className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                  />
                  <div className="flex-1">
                    <div className="text-sm font-medium text-gray-900 dark:text-white">{group.name}</div>
                    <div className="text-xs text-gray-500 dark:text-gray-400">{group.description}</div>
                  </div>
                </label>
              ))}
            </div>
          </div>
        </div>

        {/* Actions */}
        <div className="flex justify-end space-x-3 mt-6 pt-6 border-t border-gray-200 dark:border-gray-600">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-gray-300 dark:border-gray-600 dark:hover:bg-gray-600"
          >
            Cancel
          </button>
          <button
            onClick={handleSubmit}
            disabled={loading}
            className="px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {loading ? (
              <>
                <div className="animate-spin -ml-1 mr-2 h-4 w-4 border-2 border-white border-t-transparent rounded-full inline-block"></div>
                Saving...
              </>
            ) : (
              <>
                <Save className="h-4 w-4 mr-2 inline-block" />
                {user ? 'Update User' : 'Create User'}
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
}

/**
 * Role Modal Component
 * Handles creating and editing roles with policy attachments
 */
export function RoleModal({ role, onClose, onSave }: RoleModalProps) {
  const [formData, setFormData] = useState({
    name: '',
    description: '',
    attachedPolicies: [] as string[],
  });

  const [errors, setErrors] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (role) {
      setFormData({
        name: role.name,
        description: role.description,
        attachedPolicies: role.attachedPolicies
      });
    } else {
      setFormData({
        name: '',
        description: '',
        attachedPolicies: [],
      });
    }
  }, [role]);

  const availablePolicies = [
    { id: '1', name: 'BronzePolicy', description: 'Basic access policy', policyDocument: {}, createdAt: '2024-01-01', updatedAt: '2024-01-01' },
    { id: '2', name: 'SilverPolicy', description: 'Premium access policy', policyDocument: {}, createdAt: '2024-01-01', updatedAt: '2024-01-01' },
    { id: '3', name: 'GoldPolicy', description: 'Advanced access policy', policyDocument: {}, createdAt: '2024-01-01', updatedAt: '2024-01-01' },
    { id: '4', name: 'PlatinumPolicy', description: 'Full access policy', policyDocument: {}, createdAt: '2024-01-01', updatedAt: '2024-01-01' },
    { id: '5', name: 'AdminPolicy', description: 'Administrative access policy', policyDocument: {}, createdAt: '2024-01-01', updatedAt: '2024-01-01' }
  ];

  const handlePolicyToggle = (policyName: string) => {
    setFormData(prev => ({
      ...prev,
      attachedPolicies: prev.attachedPolicies.includes(policyName)
        ? prev.attachedPolicies.filter(p => p !== policyName)
        : [...prev.attachedPolicies, policyName]
    }));
  };

  const validateForm = () => {
    const newErrors: Record<string, string> = {};

    if (!formData.name.trim()) {
      newErrors.name = 'Role name is required';
    }

    if (!formData.description.trim()) {
      newErrors.description = 'Description is required';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async () => {
    if (!validateForm()) return;

    setLoading(true);
    try {
      // TODO: Implement actual API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      onSave();
    } catch (error) {
      console.error('Failed to save role:', error);
    } finally {
      setLoading(false);
    }
  };

  if (!role && !onClose) return null;

  return (
    <div className="fixed inset-0 bg-gray-600 bg-opacity-50 dark:bg-gray-900 dark:bg-opacity-50 overflow-y-auto h-full w-full z-50">
      <div className="relative top-20 mx-auto p-5 border w-11/12 max-w-3xl shadow-lg rounded-md bg-white dark:bg-gray-800 border-gray-200 dark:border-gray-700">
        <div className="flex justify-between items-center mb-6">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
            {role ? 'Edit Role' : 'Create New Role'}
          </h3>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
          >
            <X className="h-5 w-5" />
          </button>
        </div>

        <div className="space-y-6">
          {/* Basic Information */}
          <div className="grid grid-cols-1 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Role Name *
              </label>
              <input
                type="text"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                className={`w-full px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white ${
                  errors.name ? 'border-red-500' : 'border-gray-300'
                }`}
                placeholder="e.g., Developer, Manager, Admin"
              />
              {errors.name && (
                <p className="mt-1 text-sm text-red-600 dark:text-red-400">{errors.name}</p>
              )}
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Description *
              </label>
              <textarea
                value={formData.description}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                rows={3}
                className={`w-full px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white ${
                  errors.description ? 'border-red-500' : 'border-gray-300'
                }`}
                placeholder="Describe the role and its responsibilities"
              />
              {errors.description && (
                <p className="mt-1 text-sm text-red-600 dark:text-red-400">{errors.description}</p>
              )}
            </div>
          </div>

          {/* Attached Policies */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Attached Policies
            </label>
            <div className="space-y-3">
              {availablePolicies.map(policy => (
                <label key={policy.id} className="flex items-center space-x-3 p-3 border rounded-md hover:bg-gray-50 dark:hover:bg-gray-700 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={formData.attachedPolicies.includes(policy.name)}
                    onChange={() => handlePolicyToggle(policy.name)}
                    className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                  />
                  <div className="flex-1">
                    <div className="text-sm font-medium text-gray-900 dark:text-white">{policy.name}</div>
                    <div className="text-xs text-gray-500 dark:text-gray-400">{policy.description}</div>
                  </div>
                </label>
              ))}
            </div>
          </div>
        </div>

        {/* Actions */}
        <div className="flex justify-end space-x-3 mt-6 pt-6 border-t border-gray-200 dark:border-gray-600">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-gray-300 dark:border-gray-600 dark:hover:bg-gray-600"
          >
            Cancel
          </button>
          <button
            onClick={handleSubmit}
            disabled={loading}
            className="px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {loading ? (
              <>
                <div className="animate-spin -ml-1 mr-2 h-4 w-4 border-2 border-white border-t-transparent rounded-full inline-block"></div>
                Saving...
              </>
            ) : (
              <>
                <Save className="h-4 w-4 mr-2 inline-block" />
                {role ? 'Update Role' : 'Create Role'}
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
}
