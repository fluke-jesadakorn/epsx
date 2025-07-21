'use client';

import React, { useState } from 'react';
import { X, User, Mail, Save, AlertTriangle } from 'lucide-react';
import { PackageTier, SubscriptionStatus } from '../../../types/admin/iam-enhanced';

interface AddUserModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
}

export const AddUserModal: React.FC<AddUserModalProps> = ({ isOpen, onClose, onSuccess }) => {
  const [formData, setFormData] = useState({
    email: '',
    displayName: '',
    packageTier: PackageTier.FREE,
    subscriptionStatus: SubscriptionStatus.TRIAL,
  });
  const [loading, setLoading] = useState(false);
  const [errors, setErrors] = useState<Record<string, string>>({});

  const validateForm = () => {
    const newErrors: Record<string, string> = {};
    
    if (!formData.email) {
      newErrors.email = 'Email is required';
    } else if (!/\S+@\S+\.\S+/.test(formData.email)) {
      newErrors.email = 'Email is invalid';
    }
    
    if (!formData.displayName) {
      newErrors.displayName = 'Display name is required';
    }
    
    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!validateForm()) return;
    
    try {
      setLoading(true);
      
      // This would be implemented in the IAM service
      console.log('Creating user:', formData);
      // await iamService.createUser(formData);
      
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      onSuccess();
      onClose();
      
      // Reset form
      setFormData({
        email: '',
        displayName: '',
        packageTier: PackageTier.FREE,
        subscriptionStatus: SubscriptionStatus.TRIAL,
      });
    } catch (error) {
      console.error('Failed to create user:', error);
      setErrors({ submit: 'Failed to create user. Please try again.' });
    } finally {
      setLoading(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 modal-overlay flex items-center justify-center z-50 p-4">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full modal-enter">
        {/* Header */}
        <div className="bg-gray-50 dark:bg-gray-700 px-6 py-4 border-b border-gray-200 dark:border-gray-600 rounded-t-lg">
          <div className="flex items-center justify-between">
            <div className="flex items-center">
              <User className="h-6 w-6 text-blue-600 mr-2" />
              <h2 className="text-lg font-semibold text-gray-900 dark:text-white">Add New User</h2>
            </div>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
            >
              <X className="h-6 w-6" />
            </button>
          </div>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="p-6">
          <div className="space-y-4">
            {/* Email */}
            <div>
              <label htmlFor="email" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                <Mail className="h-4 w-4 inline mr-1" />
                Email Address *
              </label>
              <input
                type="email"
                id="email"
                value={formData.email}
                onChange={(e) => setFormData({ ...formData, email: e.target.value })}
                className={`w-full border rounded-lg px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 ${
                  errors.email ? 'border-red-300' : 'border-gray-300'
                }`}
                placeholder="user@example.com"
              />
              {errors.email && (
                <p className="text-sm text-red-600 mt-1">{errors.email}</p>
              )}
            </div>

            {/* Display Name */}
            <div>
              <label htmlFor="displayName" className="block text-sm font-medium text-gray-700 mb-1">
                Display Name *
              </label>
              <input
                type="text"
                id="displayName"
                value={formData.displayName}
                onChange={(e) => setFormData({ ...formData, displayName: e.target.value })}
                className={`w-full border rounded-lg px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 ${
                  errors.displayName ? 'border-red-300' : 'border-gray-300'
                }`}
                placeholder="John Doe"
              />
              {errors.displayName && (
                <p className="text-sm text-red-600 mt-1">{errors.displayName}</p>
              )}
            </div>

            {/* Package Tier */}
            <div>
              <label htmlFor="packageTier" className="block text-sm font-medium text-gray-700 mb-1">
                Package Tier
              </label>
              <select
                id="packageTier"
                value={formData.packageTier}
                onChange={(e) => setFormData({ ...formData, packageTier: e.target.value as PackageTier })}
                className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              >
                {Object.values(PackageTier).map((tier) => (
                  <option key={tier} value={tier}>
                    {tier.charAt(0).toUpperCase() + tier.slice(1)}
                  </option>
                ))}
              </select>
            </div>

            {/* Subscription Status */}
            <div>
              <label htmlFor="subscriptionStatus" className="block text-sm font-medium text-gray-700 mb-1">
                Subscription Status
              </label>
              <select
                id="subscriptionStatus"
                value={formData.subscriptionStatus}
                onChange={(e) => setFormData({ ...formData, subscriptionStatus: e.target.value as SubscriptionStatus })}
                className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              >
                {Object.values(SubscriptionStatus).map((status) => (
                  <option key={status} value={status}>
                    {status.charAt(0).toUpperCase() + status.slice(1)}
                  </option>
                ))}
              </select>
            </div>

            {/* Submit Error */}
            {errors.submit && (
              <div className="bg-red-50 border border-red-200 rounded-lg p-3">
                <div className="flex">
                  <AlertTriangle className="h-5 w-5 text-red-400 mr-2 mt-0.5" />
                  <p className="text-sm text-red-700">{errors.submit}</p>
                </div>
              </div>
            )}
          </div>

          {/* Actions */}
          <div className="flex justify-end gap-3 mt-6 pt-4 border-t border-gray-200">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={loading}
              className="px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center"
            >
              <Save className="h-4 w-4 mr-2" />
              {loading ? 'Creating...' : 'Create User'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};
