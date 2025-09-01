'use client';

import { useState, useCallback } from 'react';
import { 
  X, 
  Users, 
  Plus, 
  Clock, 
  Shield, 
  AlertCircle, 
  CheckCircle,
  Upload,
  Download
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
  grantBulkEmbeddedPermissionsAction,
  cleanupExpiredPermissionsAction
} from '@/lib/actions/embedded-permissions';
import type { 
  PermissionPlatform, 
  PermissionAction, 
  PermissionResource
} from '@/types/admin/embedded-permissions';

interface User {
  id: string;
  firebase_uid?: string;
  email: string;
  firstName?: string;
  lastName?: string;
}

interface BulkPermissionsModalProps {
  isOpen: boolean;
  onClose: () => void;
  users: User[];
  onOperationComplete?: () => void;
}

const BULK_OPERATIONS = [
  { key: 'grant', label: 'Grant Permissions', icon: Plus, color: 'blue' },
  { key: 'cleanup', label: 'Cleanup Expired', icon: Shield, color: 'red' },
];

export function BulkPermissionsModal({
  isOpen,
  onClose,
  users,
  onOperationComplete,
}: BulkPermissionsModalProps) {
  const [operation, setOperation] = useState<'grant' | 'cleanup'>('grant');
  const [selectedUserIds, setSelectedUserIds] = useState<string[]>([]);
  const [platform, setPlatform] = useState<PermissionPlatform>('epsx');
  const [resource, setResource] = useState<PermissionResource>('analytics');
  const [action, setAction] = useState<PermissionAction>('view');
  const [duration, setDuration] = useState(24);
  const [reason, setReason] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [results, setResults] = useState<any>(null);

  const resetForm = useCallback(() => {
    setOperation('grant');
    setSelectedUserIds([]);
    setPlatform('epsx');
    setResource('analytics');
    setAction('view');
    setDuration(24);
    setReason('');
    setError(null);
    setSuccess(null);
    setResults(null);
  }, []);

  const handleClose = useCallback(() => {
    resetForm();
    onClose();
  }, [resetForm, onClose]);

  const handleUserSelection = (userId: string, selected: boolean) => {
    if (selected) {
      setSelectedUserIds(prev => [...prev, userId]);
    } else {
      setSelectedUserIds(prev => prev.filter(id => id !== userId));
    }
  };

  const handleSelectAll = () => {
    setSelectedUserIds(users.map(u => u.firebase_uid || u.id));
  };

  const handleClearSelection = () => {
    setSelectedUserIds([]);
  };

  const handleGrantBulkPermissions = async () => {
    if (selectedUserIds.length === 0) {
      setError('Please select at least one user');
      return;
    }

    try {
      setIsSubmitting(true);
      setError(null);

      const expiryTimestamp = Math.floor(Date.now() / 1000) + (duration * 3600);
      
      const result = await grantBulkEmbeddedPermissionsAction({
        user_ids: selectedUserIds,
        permissions: [{
          base_permission: `${platform}:${resource}:${action}`,
          platform,
          resource,
          action,
          expiry_timestamp: expiryTimestamp,
        }],
        reason: reason || undefined,
      });

      if (result.success) {
        setSuccess(`Bulk operation completed successfully!`);
        setResults(result.data);
        onOperationComplete?.();
        setTimeout(() => {
          handleClose();
        }, 3000);
      } else {
        setError(result.error || 'Failed to grant bulk permissions');
      }
    } catch (err: any) {
      setError(err.message || 'An unexpected error occurred');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleCleanupExpired = async () => {
    try {
      setIsSubmitting(true);
      setError(null);

      const result = await cleanupExpiredPermissionsAction({
        dry_run: false,
        batch_size: 100,
      });

      if (result.success) {
        setSuccess(`Cleanup completed: ${result.data.cleaned} permissions cleaned`);
        setResults(result.data);
        onOperationComplete?.();
        setTimeout(() => {
          handleClose();
        }, 3000);
      } else {
        setError(result.error || 'Failed to cleanup expired permissions');
      }
    } catch (err: any) {
      setError(err.message || 'An unexpected error occurred');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleSubmit = () => {
    switch (operation) {
      case 'grant':
        handleGrantBulkPermissions();
        break;
      case 'cleanup':
        handleCleanupExpired();
        break;
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="bg-white dark:bg-gray-800 rounded-lg max-w-3xl w-full max-h-[90vh] overflow-y-auto">
        <div className="p-6 border-b border-gray-200 dark:border-gray-700">
          <div className="flex justify-between items-start">
            <div>
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                Bulk Permission Operations
              </h3>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                Perform bulk operations on user permissions
              </p>
            </div>
            <button
              onClick={handleClose}
              className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
            >
              <X className="w-5 h-5" />
            </button>
          </div>

          {/* Operation Selection */}
          <div className="mt-4 flex gap-2">
            {BULK_OPERATIONS.map(({ key, label, icon: Icon, color }) => (
              <button
                key={key}
                onClick={() => setOperation(key as any)}
                className={`flex items-center gap-2 px-3 py-2 text-sm rounded-md transition-colors ${
                  operation === key
                    ? `bg-${color}-600 text-white`
                    : 'bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'
                }`}
              >
                <Icon className="w-4 h-4" />
                {label}
              </button>
            ))}
          </div>
        </div>
        
        <div className="p-6 space-y-6">
          {/* Grant Permissions Mode */}
          {operation === 'grant' && (
            <>
              {/* User Selection */}
              <div>
                <div className="flex justify-between items-center mb-2">
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Select Users ({selectedUserIds.length} of {users.length} selected)
                  </label>
                  <div className="flex gap-2">
                    <button
                      onClick={handleSelectAll}
                      className="text-xs text-blue-600 hover:text-blue-800 dark:text-blue-400"
                    >
                      Select All
                    </button>
                    <button
                      onClick={handleClearSelection}
                      className="text-xs text-gray-600 hover:text-gray-800 dark:text-gray-400"
                    >
                      Clear
                    </button>
                  </div>
                </div>
                <div className="max-h-48 overflow-y-auto border border-gray-300 dark:border-gray-600 rounded-md p-3 space-y-2">
                  {users.map((user) => {
                    const userId = user.firebase_uid || user.id;
                    const isSelected = selectedUserIds.includes(userId);
                    
                    return (
                      <label
                        key={userId}
                        className="flex items-center space-x-3 cursor-pointer"
                      >
                        <input
                          type="checkbox"
                          checked={isSelected}
                          onChange={(e) => handleUserSelection(userId, e.target.checked)}
                          className="rounded border-gray-300 dark:border-gray-600"
                        />
                        <div className="flex-1">
                          <div className="text-sm font-medium text-gray-900 dark:text-white">
                            {user.firstName && user.lastName 
                              ? `${user.firstName} ${user.lastName}`
                              : user.email.split('@')[0]
                            }
                          </div>
                          <div className="text-xs text-gray-500 dark:text-gray-400">
                            {user.email}
                          </div>
                        </div>
                      </label>
                    );
                  })}
                </div>
              </div>

              {/* Permission Configuration */}
              <div className="grid grid-cols-3 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    Platform
                  </label>
                  <select
                    value={platform}
                    onChange={(e) => setPlatform(e.target.value as PermissionPlatform)}
                    className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                  >
                    <option value="epsx">EPSX</option>
                    <option value="epsx-pay">EPSX Pay</option>
                    <option value="epsx-token">EPSX Token</option>
                    <option value="admin">Admin</option>
                  </select>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    Resource
                  </label>
                  <select
                    value={resource}
                    onChange={(e) => setResource(e.target.value as PermissionResource)}
                    className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                  >
                    <option value="analytics">Analytics</option>
                    <option value="users">Users</option>
                    <option value="payments">Payments</option>
                    <option value="tokens">Tokens</option>
                    <option value="system">System</option>
                    <option value="*">All Resources</option>
                  </select>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    Action
                  </label>
                  <select
                    value={action}
                    onChange={(e) => setAction(e.target.value as PermissionAction)}
                    className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                  >
                    <option value="view">View</option>
                    <option value="create">Create</option>
                    <option value="edit">Edit</option>
                    <option value="delete">Delete</option>
                    <option value="manage">Manage</option>
                    <option value="*">All Actions</option>
                  </select>
                </div>
              </div>

              {/* Duration */}
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Duration (hours)
                </label>
                <input
                  type="number"
                  value={duration}
                  onChange={(e) => setDuration(parseInt(e.target.value) || 24)}
                  min={1}
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                />
              </div>
            </>
          )}

          {/* Cleanup Mode */}
          {operation === 'cleanup' && (
            <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
              <div className="flex items-start gap-3">
                <AlertCircle className="w-5 h-5 text-yellow-600 mt-0.5" />
                <div>
                  <h4 className="text-sm font-medium text-yellow-800 dark:text-yellow-200">
                    Cleanup Expired Permissions
                  </h4>
                  <p className="text-sm text-yellow-700 dark:text-yellow-300 mt-1">
                    This operation will permanently remove all expired permissions from the system.
                    This action cannot be undone.
                  </p>
                </div>
              </div>
            </div>
          )}

          {/* Reason */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Reason (optional)
            </label>
            <textarea
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              rows={3}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              placeholder="Explain the reason for this bulk operation..."
            />
          </div>

          {/* Results Display */}
          {results && (
            <div className="bg-gray-50 dark:bg-gray-700 p-4 rounded-lg">
              <h4 className="text-sm font-medium text-gray-900 dark:text-white mb-2">
                Operation Results
              </h4>
              {operation === 'grant' && results.summary && (
                <div className="space-y-2">
                  <p className="text-sm text-gray-600 dark:text-gray-400">
                    Total: {results.summary.total}, Successful: {results.summary.successful}, Failed: {results.summary.failed}
                  </p>
                  {results.failed && results.failed.length > 0 && (
                    <div className="max-h-32 overflow-y-auto">
                      <p className="text-sm font-medium text-red-600 dark:text-red-400">Failed Operations:</p>
                      {results.failed.map((fail: any, index: number) => (
                        <p key={index} className="text-xs text-red-600 dark:text-red-400">
                          {fail.user_id}: {fail.error}
                        </p>
                      ))}
                    </div>
                  )}
                </div>
              )}
              {operation === 'cleanup' && (
                <div className="space-y-2">
                  <p className="text-sm text-gray-600 dark:text-gray-400">
                    Cleaned: {results.cleaned}, Failed: {results.failed}
                  </p>
                  {results.details && results.details.length > 0 && (
                    <div className="max-h-32 overflow-y-auto">
                      <p className="text-sm font-medium text-gray-600 dark:text-gray-400">Details:</p>
                      {results.details.map((detail: any, index: number) => (
                        <p key={index} className="text-xs text-gray-600 dark:text-gray-400">
                          {detail.user_id}: {detail.status} ({detail.permission})
                        </p>
                      ))}
                    </div>
                  )}
                </div>
              )}
            </div>
          )}

          {/* Error/Success Messages */}
          {error && (
            <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
              <div className="flex items-start gap-3">
                <AlertCircle className="w-5 h-5 text-red-600 mt-0.5" />
                <div>
                  <h4 className="text-sm font-medium text-red-800 dark:text-red-200">
                    Error
                  </h4>
                  <p className="text-sm text-red-700 dark:text-red-300 mt-1">
                    {error}
                  </p>
                </div>
              </div>
            </div>
          )}

          {success && (
            <div className="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg p-4">
              <div className="flex items-start gap-3">
                <CheckCircle className="w-5 h-5 text-green-600 mt-0.5" />
                <div>
                  <h4 className="text-sm font-medium text-green-800 dark:text-green-200">
                    Success
                  </h4>
                  <p className="text-sm text-green-700 dark:text-green-300 mt-1">
                    {success}
                  </p>
                </div>
              </div>
            </div>
          )}
        </div>
        
        <div className="p-6 border-t border-gray-200 dark:border-gray-700 flex justify-end gap-3">
          <Button
            variant="outline"
            onClick={handleClose}
            disabled={isSubmitting}
          >
            Cancel
          </Button>
          <Button
            onClick={handleSubmit}
            disabled={isSubmitting || (operation === 'grant' && selectedUserIds.length === 0)}
          >
            {isSubmitting ? (
              <div className="animate-spin rounded-full h-4 w-4 border-2 border-white border-t-transparent mr-2" />
            ) : (
              <>
                {operation === 'grant' && <Plus className="w-4 h-4 mr-2" />}
                {operation === 'cleanup' && <Shield className="w-4 h-4 mr-2" />}
              </>
            )}
            {operation === 'grant' ? 'Grant Permissions' : 'Cleanup Expired'}
          </Button>
        </div>
      </div>
    </div>
  );
}