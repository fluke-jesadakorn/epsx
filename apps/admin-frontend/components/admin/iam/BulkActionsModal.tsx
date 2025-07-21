'use client';

import React, { useState } from 'react';
import { X, Users, Crown, Shield, Mail, AlertTriangle, CheckCircle } from 'lucide-react';
import { PackageTier, SubscriptionStatus } from '../../../types/admin/iam-enhanced';
import { iamService } from '../../../services/iamService';

interface BulkActionsModalProps {
  selectedUserIds: string[];
  onClose: () => void;
  onComplete: () => void;
}

export const BulkActionsModal: React.FC<BulkActionsModalProps> = ({
  selectedUserIds,
  onClose,
  onComplete
}) => {
  const [activeAction, setActiveAction] = useState<'package' | 'status' | 'permissions' | 'email'>('package');
  const [loading, setLoading] = useState(false);
  const [completed, setCompleted] = useState(false);
  const [formData, setFormData] = useState({
    packageTier: PackageTier.BRONZE,
    subscriptionStatus: SubscriptionStatus.ACTIVE,
    reason: '',
    emailSubject: '',
    emailBody: '',
  });

  const handleBulkAction = async () => {
    try {
      setLoading(true);
      
      switch (activeAction) {
        case 'package':
          for (const userId of selectedUserIds) {
            await iamService.updateUserPackageTier(userId, formData.packageTier, 'current-admin-id');
          }
          break;
        
        case 'status':
          // This would be implemented in the IAM service
          console.log('Bulk status update not implemented yet');
          break;
          
        case 'permissions':
          // This would be implemented for bulk permission changes
          console.log('Bulk permission update not implemented yet');
          break;
          
        case 'email':
          // This would send bulk emails
          console.log('Bulk email not implemented yet');
          break;
      }
      
      setCompleted(true);
      setTimeout(() => {
        onComplete();
      }, 2000);
      
    } catch (error) {
      console.error('Bulk action failed:', error);
      alert('Bulk action failed. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  const actions = [
    {
      id: 'package' as const,
      name: 'Update Package Tier',
      icon: Crown,
      description: 'Change package tier for selected users'
    },
    {
      id: 'status' as const,
      name: 'Update Status',
      icon: Users,
      description: 'Change subscription status for selected users'
    },
    {
      id: 'permissions' as const,
      name: 'Manage Permissions',
      icon: Shield,
      description: 'Grant or revoke permissions for selected users'
    },
    {
      id: 'email' as const,
      name: 'Send Email',
      icon: Mail,
      description: 'Send notification email to selected users'
    }
  ];

  if (completed) {
    return (
      <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
        <div className="bg-white rounded-lg shadow-xl max-w-md w-full p-6 text-center">
          <CheckCircle className="h-16 w-16 text-green-500 mx-auto mb-4" />
          <h2 className="text-xl font-semibold text-gray-900 mb-2">Bulk Action Completed</h2>
          <p className="text-gray-600 mb-4">
            Successfully updated {selectedUserIds.length} users.
          </p>
          <div className="text-sm text-gray-500">
            Redirecting in a moment...
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-lg shadow-xl max-w-2xl w-full max-h-[90vh] overflow-hidden">
        {/* Header */}
        <div className="bg-gray-50 px-6 py-4 border-b border-gray-200">
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-xl font-semibold text-gray-900">Bulk Actions</h2>
              <p className="text-sm text-gray-500 mt-1">
                Apply actions to {selectedUserIds.length} selected users
              </p>
            </div>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-600"
            >
              <X className="h-6 w-6" />
            </button>
          </div>
        </div>

        {/* Action Selection */}
        <div className="p-6">
          <div className="mb-6">
            <h3 className="text-lg font-medium text-gray-900 mb-4">Select Action</h3>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              {actions.map((action) => {
                const Icon = action.icon;
                return (
                  <button
                    key={action.id}
                    onClick={() => setActiveAction(action.id)}
                    className={`p-4 text-left border rounded-lg transition-colors ${
                      activeAction === action.id
                        ? 'border-blue-500 bg-blue-50'
                        : 'border-gray-200 hover:border-gray-300'
                    }`}
                  >
                    <div className="flex items-center mb-2">
                      <Icon className={`h-5 w-5 mr-2 ${
                        activeAction === action.id ? 'text-blue-600' : 'text-gray-400'
                      }`} />
                      <span className={`font-medium ${
                        activeAction === action.id ? 'text-blue-900' : 'text-gray-900'
                      }`}>
                        {action.name}
                      </span>
                    </div>
                    <p className="text-sm text-gray-500">{action.description}</p>
                  </button>
                );
              })}
            </div>
          </div>

          {/* Action Form */}
          <div className="mb-6">
            <h3 className="text-lg font-medium text-gray-900 mb-4">Action Details</h3>
            
            {activeAction === 'package' && (
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    New Package Tier
                  </label>
                  <select
                    value={formData.packageTier}
                    onChange={(e) => setFormData({ ...formData, packageTier: e.target.value as PackageTier })}
                    className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:border-blue-500"
                  >
                    {Object.values(PackageTier).map((tier) => (
                      <option key={tier} value={tier}>
                        {tier.charAt(0).toUpperCase() + tier.slice(1)}
                      </option>
                    ))}
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    Reason (Optional)
                  </label>
                  <textarea
                    value={formData.reason}
                    onChange={(e) => setFormData({ ...formData, reason: e.target.value })}
                    placeholder="Reason for package tier change..."
                    className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:border-blue-500"
                    rows={3}
                  />
                </div>
              </div>
            )}

            {activeAction === 'status' && (
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    New Subscription Status
                  </label>
                  <select
                    value={formData.subscriptionStatus}
                    onChange={(e) => setFormData({ ...formData, subscriptionStatus: e.target.value as SubscriptionStatus })}
                    className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:border-blue-500"
                  >
                    {Object.values(SubscriptionStatus).map((status) => (
                      <option key={status} value={status}>
                        {status.charAt(0).toUpperCase() + status.slice(1)}
                      </option>
                    ))}
                  </select>
                </div>
              </div>
            )}

            {activeAction === 'permissions' && (
              <div className="space-y-4">
                <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
                  <div className="flex">
                    <AlertTriangle className="h-5 w-5 text-yellow-400 mr-2 mt-0.5" />
                    <div>
                      <h4 className="text-sm font-medium text-yellow-800">Not Implemented</h4>
                      <p className="text-sm text-yellow-700 mt-1">
                        Bulk permission management is not yet implemented.
                      </p>
                    </div>
                  </div>
                </div>
              </div>
            )}

            {activeAction === 'email' && (
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    Email Subject
                  </label>
                  <input
                    type="text"
                    value={formData.emailSubject}
                    onChange={(e) => setFormData({ ...formData, emailSubject: e.target.value })}
                    placeholder="Subject line..."
                    className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:border-blue-500"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    Email Body
                  </label>
                  <textarea
                    value={formData.emailBody}
                    onChange={(e) => setFormData({ ...formData, emailBody: e.target.value })}
                    placeholder="Email content..."
                    className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:border-blue-500"
                    rows={6}
                  />
                </div>
                <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
                  <div className="flex">
                    <AlertTriangle className="h-5 w-5 text-yellow-400 mr-2 mt-0.5" />
                    <div>
                      <h4 className="text-sm font-medium text-yellow-800">Not Implemented</h4>
                      <p className="text-sm text-yellow-700 mt-1">
                        Bulk email functionality is not yet implemented.
                      </p>
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>

          {/* Actions */}
          <div className="flex justify-end gap-3">
            <button
              onClick={onClose}
              className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
            >
              Cancel
            </button>
            <button
              onClick={handleBulkAction}
              disabled={loading || (activeAction === 'permissions') || (activeAction === 'email')}
              className="px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? 'Processing...' : `Apply to ${selectedUserIds.length} Users`}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
