'use client';

import { useState, useCallback } from 'react';
import { 
  X, 
  Plus, 
  Calendar, 
  Clock, 
  Shield, 
  AlertCircle, 
  CheckCircle,
  Trash2,
  Users,
  Key
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
  grantPermission,
  revokePermission,
  grantTemporaryPermission,
  extendTemporaryPermission
} from '@/app/actions/permission-actions';
import type { 
  PermissionPlatform, 
  PermissionAction, 
  PermissionResource,
  PermissionExpiryInfo
} from '@/types/admin/embedded-permissions';

interface User {
  id: string;
  firebase_uid?: string;
  email: string;
  firstName?: string;
  lastName?: string;
}

interface GrantPermissionModalProps {
  isOpen: boolean;
  onClose: () => void;
  selectedUser?: User | null;
  existingPermissions?: PermissionExpiryInfo[];
  onPermissionGranted?: (userId: string) => void;
}

const PLATFORMS: { value: PermissionPlatform; label: string; description: string }[] = [
  { value: 'admin', label: 'Admin', description: 'Cross-platform administrative permissions' },
  { value: 'epsx', label: 'EPSX', description: 'Main EPSX platform permissions' },
  { value: 'epsx-pay', label: 'EPSX Pay', description: 'Payment platform permissions' },
  { value: 'epsx-token', label: 'EPSX Token', description: 'Token platform permissions' },
];

const RESOURCES: { value: PermissionResource; label: string; description: string }[] = [
  { value: 'users', label: 'Users', description: 'User management operations' },
  { value: 'analytics', label: 'Analytics', description: 'Analytics and reporting' },
  { value: 'payments', label: 'Payments', description: 'Payment processing' },
  { value: 'tokens', label: 'Tokens', description: 'Token operations' },
  { value: 'system', label: 'System', description: 'System configuration' },
  { value: '*', label: 'All Resources', description: 'Access to all resources' },
];

const ACTIONS: { value: PermissionAction; label: string; description: string; risk: 'low' | 'medium' | 'high' }[] = [
  { value: 'view', label: 'View', description: 'Read-only access', risk: 'low' },
  { value: 'create', label: 'Create', description: 'Create new items', risk: 'medium' },
  { value: 'edit', label: 'Edit', description: 'Modify existing items', risk: 'medium' },
  { value: 'delete', label: 'Delete', description: 'Delete items', risk: 'high' },
  { value: 'manage', label: 'Manage', description: 'Full management access', risk: 'high' },
  { value: '*', label: 'All Actions', description: 'All permissions', risk: 'high' },
];

const DURATION_PRESETS = [
  { hours: 1, label: '1 Hour' },
  { hours: 4, label: '4 Hours' },
  { hours: 8, label: '8 Hours' },
  { hours: 24, label: '1 Day' },
  { hours: 48, label: '2 Days' },
  { hours: 168, label: '1 Week' },
  { hours: 720, label: '30 Days' },
];

export function GrantPermissionModal({
  isOpen,
  onClose,
  selectedUser,
  existingPermissions = [],
  onPermissionGranted,
}: GrantPermissionModalProps) {
  const [platform, setPlatform] = useState<PermissionPlatform>('epsx');
  const [resource, setResource] = useState<PermissionResource>('analytics');
  const [action, setAction] = useState<PermissionAction>('view');
  const [durationType, setDurationType] = useState<'permanent' | 'temporary'>('temporary');
  const [duration, setDuration] = useState(24); // hours
  const [customDuration, setCustomDuration] = useState('');
  const [reason, setReason] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [mode, setMode] = useState<'grant' | 'extend' | 'revoke'>('grant');
  const [selectedPermission, setSelectedPermission] = useState<string>('');

  const resetForm = useCallback(() => {
    setPlatform('epsx');
    setResource('analytics');
    setAction('view');
    setDurationType('temporary');
    setDuration(24);
    setCustomDuration('');
    setReason('');
    setError(null);
    setSuccess(null);
    setMode('grant');
    setSelectedPermission('');
  }, []);

  const handleClose = useCallback(() => {
    resetForm();
    onClose();
  }, [resetForm, onClose]);

  const getPermissionPreview = () => {
    const basePermission = `${platform}:${resource}:${action}`;
    if (durationType === 'permanent') {
      return basePermission;
    }
    const actualDuration = customDuration ? parseInt(customDuration) : duration;
    const expiryTimestamp = Math.floor(Date.now() / 1000) + (actualDuration * 3600);
    return `${basePermission}:${expiryTimestamp}`;
  };

  const getExpiryDate = () => {
    if (durationType === 'permanent') return null;
    const actualDuration = customDuration ? parseInt(customDuration) : duration;
    return new Date(Date.now() + (actualDuration * 3600 * 1000));
  };

  const getRiskColor = (risk: string) => {
    switch (risk) {
      case 'low': return 'text-green-600 bg-green-100 dark:bg-green-900/20';
      case 'medium': return 'text-yellow-600 bg-yellow-100 dark:bg-yellow-900/20';
      case 'high': return 'text-red-600 bg-red-100 dark:bg-red-900/20';
      default: return 'text-gray-600 bg-gray-100 dark:bg-gray-900/20';
    }
  };

  const handleGrantPermission = async () => {
    if (!selectedUser) return;

    try {
      setIsSubmitting(true);
      setError(null);

      const userId = selectedUser.firebase_uid || selectedUser.id;
      const basePermission = `${platform}:${resource}:${action}`;

      let result;
      if (durationType === 'permanent') {
        // Grant permanent permission
        result = await grantPermission(userId, basePermission);
      } else {
        // Grant temporary permission with expiry
        const actualDuration = customDuration ? parseInt(customDuration) : duration;
        const expiryDate = new Date(Date.now() + (actualDuration * 3600 * 1000));
        result = await grantTemporaryPermission(userId, basePermission, expiryDate);
      }

      if (result.success) {
        setSuccess(`Permission granted successfully!`);
        onPermissionGranted?.(userId);
        setTimeout(() => {
          handleClose();
        }, 2000);
      } else {
        setError(result.error || 'Failed to grant permission');
      }
    } catch (err: any) {
      setError(err.message || 'An unexpected error occurred');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleExtendPermission = async () => {
    if (!selectedUser || !selectedPermission) return;

    try {
      setIsSubmitting(true);
      setError(null);

      const userId = selectedUser.firebase_uid || selectedUser.id;
      const actualDuration = customDuration ? parseInt(customDuration) : duration;
      const newExpiryDate = new Date(Date.now() + (actualDuration * 3600 * 1000));
      
      const result = await extendTemporaryPermission(userId, selectedPermission, newExpiryDate);

      if (result.success) {
        setSuccess(`Permission extended successfully!`);
        onPermissionGranted?.(userId);
        setTimeout(() => {
          handleClose();
        }, 2000);
      } else {
        setError(result.error || 'Failed to extend permission');
      }
    } catch (err: any) {
      setError(err.message || 'An unexpected error occurred');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleRevokePermission = async () => {
    if (!selectedUser || !selectedPermission) return;

    try {
      setIsSubmitting(true);
      setError(null);

      const userId = selectedUser.firebase_uid || selectedUser.id;
      
      const result = await revokePermission(userId, selectedPermission);

      if (result.success) {
        setSuccess(`Permission revoked successfully!`);
        onPermissionGranted?.(userId);
        setTimeout(() => {
          handleClose();
        }, 2000);
      } else {
        setError(result.error || 'Failed to revoke permission');
      }
    } catch (err: any) {
      setError(err.message || 'An unexpected error occurred');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleSubmit = () => {
    switch (mode) {
      case 'grant':
        handleGrantPermission();
        break;
      case 'extend':
        handleExtendPermission();
        break;
      case 'revoke':
        handleRevokePermission();
        break;
    }
  };

  if (!isOpen) return null;

  const selectedActionInfo = ACTIONS.find(a => a.value === action);

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="bg-white dark:bg-gray-800 rounded-lg max-w-2xl w-full max-h-[90vh] overflow-y-auto">
        <div className="p-6 border-b border-gray-200 dark:border-gray-700">
          <div className="flex justify-between items-start">
            <div>
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                Permission Management
              </h3>
              {selectedUser && (
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  {selectedUser.firstName && selectedUser.lastName 
                    ? `${selectedUser.firstName} ${selectedUser.lastName} (${selectedUser.email})`
                    : selectedUser.email
                  }
                </p>
              )}
            </div>
            <button
              onClick={handleClose}
              className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
            >
              <X className="w-5 h-5" />
            </button>
          </div>

          {/* Mode Selection */}
          <div className="mt-4 flex gap-2">
            {[
              { key: 'grant', label: 'Grant', icon: Plus, color: 'blue' },
              { key: 'extend', label: 'Extend', icon: Clock, color: 'green' },
              { key: 'revoke', label: 'Revoke', icon: Trash2, color: 'red' }
            ].map(({ key, label, icon: Icon, color }) => (
              <button
                key={key}
                onClick={() => setMode(key as any)}
                className={`flex items-center gap-2 px-3 py-2 text-sm rounded-md transition-colors ${
                  mode === key
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
          {/* Existing Permissions (for extend/revoke modes) */}
          {(mode === 'extend' || mode === 'revoke') && existingPermissions.length > 0 && (
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Select Permission
              </label>
              <select
                value={selectedPermission}
                onChange={(e) => setSelectedPermission(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              >
                <option value="">Select a permission...</option>
                {existingPermissions.map((perm, index) => (
                  <option key={index} value={perm.permission}>
                    {perm.base_permission} 
                    {perm.expires_at ? ` (expires ${new Date(perm.expires_at * 1000).toLocaleDateString()})` : ' (permanent)'}
                  </option>
                ))}
              </select>
            </div>
          )}

          {/* Grant Mode - Permission Builder */}
          {mode === 'grant' && (
            <div className="space-y-4">
              {/* Platform Selection */}
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Platform
                </label>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                  {PLATFORMS.map((p) => (
                    <label
                      key={p.value}
                      className={`relative flex items-start p-3 rounded-lg border cursor-pointer transition-colors ${
                        platform === p.value
                          ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                          : 'border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700'
                      }`}
                    >
                      <input
                        type="radio"
                        value={p.value}
                        checked={platform === p.value}
                        onChange={(e) => setPlatform(e.target.value as PermissionPlatform)}
                        className="sr-only"
                      />
                      <div className="flex-1">
                        <div className="text-sm font-medium text-gray-900 dark:text-white">
                          {p.label}
                        </div>
                        <div className="text-xs text-gray-500 dark:text-gray-400">
                          {p.description}
                        </div>
                      </div>
                    </label>
                  ))}
                </div>
              </div>

              {/* Resource Selection */}
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Resource
                </label>
                <select
                  value={resource}
                  onChange={(e) => setResource(e.target.value as PermissionResource)}
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                >
                  {RESOURCES.map((r) => (
                    <option key={r.value} value={r.value}>
                      {r.label} - {r.description}
                    </option>
                  ))}
                </select>
              </div>

              {/* Action Selection */}
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Action
                </label>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                  {ACTIONS.map((a) => (
                    <label
                      key={a.value}
                      className={`relative flex items-start p-3 rounded-lg border cursor-pointer transition-colors ${
                        action === a.value
                          ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                          : 'border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700'
                      }`}
                    >
                      <input
                        type="radio"
                        value={a.value}
                        checked={action === a.value}
                        onChange={(e) => setAction(e.target.value as PermissionAction)}
                        className="sr-only"
                      />
                      <div className="flex-1">
                        <div className="flex items-center justify-between">
                          <div className="text-sm font-medium text-gray-900 dark:text-white">
                            {a.label}
                          </div>
                          <span className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${getRiskColor(a.risk)}`}>
                            {a.risk}
                          </span>
                        </div>
                        <div className="text-xs text-gray-500 dark:text-gray-400">
                          {a.description}
                        </div>
                      </div>
                    </label>
                  ))}
                </div>
              </div>

              {/* Duration Type */}
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Duration Type
                </label>
                <div className="grid grid-cols-2 gap-3">
                  <label
                    className={`relative flex items-center p-3 rounded-lg border cursor-pointer transition-colors ${
                      durationType === 'temporary'
                        ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                        : 'border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700'
                    }`}
                  >
                    <input
                      type="radio"
                      value="temporary"
                      checked={durationType === 'temporary'}
                      onChange={(e) => setDurationType('temporary')}
                      className="sr-only"
                    />
                    <Clock className="w-5 h-5 mr-3 text-yellow-600" />
                    <div>
                      <div className="text-sm font-medium text-gray-900 dark:text-white">
                        Temporary
                      </div>
                      <div className="text-xs text-gray-500 dark:text-gray-400">
                        Expires after set time
                      </div>
                    </div>
                  </label>
                  
                  <label
                    className={`relative flex items-center p-3 rounded-lg border cursor-pointer transition-colors ${
                      durationType === 'permanent'
                        ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                        : 'border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700'
                    }`}
                  >
                    <input
                      type="radio"
                      value="permanent"
                      checked={durationType === 'permanent'}
                      onChange={(e) => setDurationType('permanent')}
                      className="sr-only"
                    />
                    <Shield className="w-5 h-5 mr-3 text-green-600" />
                    <div>
                      <div className="text-sm font-medium text-gray-900 dark:text-white">
                        Permanent
                      </div>
                      <div className="text-xs text-gray-500 dark:text-gray-400">
                        Never expires
                      </div>
                    </div>
                  </label>
                </div>
              </div>
            </div>
          )}

          {/* Duration Settings (for temporary permissions and extend/revoke modes) */}
          {(durationType === 'temporary' || mode === 'extend') && (
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                {mode === 'extend' ? 'Additional Duration' : 'Duration'}
              </label>
              <div className="space-y-3">
                <div className="grid grid-cols-3 gap-2">
                  {DURATION_PRESETS.map((preset) => (
                    <button
                      key={preset.hours}
                      type="button"
                      onClick={() => {
                        setDuration(preset.hours);
                        setCustomDuration('');
                      }}
                      className={`px-3 py-2 text-sm rounded-md transition-colors ${
                        duration === preset.hours && !customDuration
                          ? 'bg-blue-600 text-white'
                          : 'bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'
                      }`}
                    >
                      {preset.label}
                    </button>
                  ))}
                </div>
                <div className="flex items-center gap-2">
                  <input
                    type="number"
                    value={customDuration}
                    onChange={(e) => setCustomDuration(e.target.value)}
                    placeholder="Custom hours"
                    className="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                  />
                  <span className="text-sm text-gray-500 dark:text-gray-400">hours</span>
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
              placeholder="Explain why this permission is being granted/modified..."
            />
          </div>

          {/* Permission Preview */}
          {mode === 'grant' && (
            <div className="bg-gray-50 dark:bg-gray-700 p-4 rounded-lg">
              <div className="flex items-start gap-3">
                <Key className="w-5 h-5 text-blue-600 mt-0.5" />
                <div className="flex-1">
                  <h4 className="text-sm font-medium text-gray-900 dark:text-white mb-2">
                    Permission Preview
                  </h4>
                  <div className="bg-white dark:bg-gray-800 p-3 rounded border font-mono text-sm">
                    {getPermissionPreview()}
                  </div>
                  {durationType === 'temporary' && (
                    <p className="text-xs text-gray-500 dark:text-gray-400 mt-2">
                      Expires: {getExpiryDate()?.toLocaleString()}
                    </p>
                  )}
                  {selectedActionInfo && (
                    <div className="mt-2">
                      <span className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${getRiskColor(selectedActionInfo.risk)}`}>
                        <AlertCircle className="w-3 h-3 mr-1" />
                        {selectedActionInfo.risk.toUpperCase()} RISK
                      </span>
                    </div>
                  )}
                </div>
              </div>
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
            disabled={isSubmitting || (mode !== 'grant' && !selectedPermission)}
          >
            {isSubmitting ? (
              <div className="animate-spin rounded-full h-4 w-4 border-2 border-white border-t-transparent mr-2" />
            ) : (
              <>
                {mode === 'grant' && <Plus className="w-4 h-4 mr-2" />}
                {mode === 'extend' && <Clock className="w-4 h-4 mr-2" />}
                {mode === 'revoke' && <Trash2 className="w-4 h-4 mr-2" />}
              </>
            )}
            {mode === 'grant' ? 'Grant Permission' : mode === 'extend' ? 'Extend Permission' : 'Revoke Permission'}
          </Button>
        </div>
      </div>
    </div>
  );
}