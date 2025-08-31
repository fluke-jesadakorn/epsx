'use client';

import { useState } from 'react';
import { Plus, X, Eye, Edit, Trash2, Shield, Users, BarChart3, Coins, Vote, Database } from 'lucide-react';

interface Platform {
  id: string;
  code: string;
  name: string;
  description: string;
}

interface User {
  id: string;
  email: string;
  name: string;
  role: string;
}

interface StructuredPermission {
  id: string;
  platform: string;
  resource: string;
  action: string;
  description?: string;
}

interface CrossPlatformPermissionManagerProps {
  platforms: Platform[];
  users: User[];
  selectedUser?: User;
  permissions: StructuredPermission[];
  onGrantPermission: (userId: string, permission: string) => Promise<void>;
  onRevokePermission: (userId: string, permission: string) => Promise<void>;
  onBulkPermissionUpdate: (permissions: { userId: string; permissions: string[] }[]) => Promise<void>;
}

// Resource configurations for each platform
const platformResources: { [key: string]: { [key: string]: { icon: typeof Shield; description: string; actions: string[] } } } = {
  'epsx': {
    'analytics': { icon: BarChart3, description: 'EPS analytics and reporting', actions: ['read', 'write', 'export'] },
    'users': { icon: Users, description: 'User management', actions: ['read', 'write', 'manage'] },
    'settings': { icon: Database, description: 'System settings', actions: ['read', 'write'] },
  },
  'epsx-pay': {
    'transactions': { icon: Coins, description: 'Crypto transactions', actions: ['read', 'create', 'approve'] },
    'wallets': { icon: Shield, description: 'Wallet management', actions: ['read', 'manage', 'create'] },
    'defi': { icon: BarChart3, description: 'DeFi protocols', actions: ['read', 'interact', 'manage'] },
  },
  'epsx-token': {
    'governance': { icon: Vote, description: 'Governance and voting', actions: ['read', 'vote', 'propose'] },
    'treasury': { icon: Database, description: 'Treasury management', actions: ['view', 'approve', 'execute'] },
    'contracts': { icon: Shield, description: 'Smart contracts', actions: ['read', 'deploy', 'upgrade'] },
  },
};

export function CrossPlatformPermissionManager({
  platforms,
  users,
  selectedUser,
  permissions,
  onGrantPermission,
  onRevokePermission,
  onBulkPermissionUpdate,
}: CrossPlatformPermissionManagerProps) {
  const [selectedPlatform, setSelectedPlatform] = useState<string>('epsx');
  const [selectedResource, setSelectedResource] = useState<string>('');
  const [selectedAction, setSelectedAction] = useState<string>('');
  const [isBuilding, setIsBuilding] = useState(false);
  const [newPermissionUser, setNewPermissionUser] = useState<string>('');

  const currentPlatform = platforms.find(p => p.code === selectedPlatform);
  const resources = platformResources[selectedPlatform] || {};
  
  const actions = selectedResource && resources[selectedResource] 
    ? resources[selectedResource].actions 
    : [];

  const buildPermission = () => {
    if (!selectedPlatform || !selectedResource || !selectedAction) return '';
    return `${selectedPlatform}:${selectedResource}:${selectedAction}`;
  };

  const handleGrantPermission = async () => {
    if (!newPermissionUser) return;
    
    const permission = buildPermission();
    if (!permission) return;

    try {
      await onGrantPermission(newPermissionUser, permission);
      // Reset form
      setNewPermissionUser('');
      setSelectedResource('');
      setSelectedAction('');
      setIsBuilding(false);
    } catch (error) {
      console.error('Failed to grant permission:', error);
    }
  };

  const handleRevokePermission = async (userId: string, permission: string) => {
    try {
      await onRevokePermission(userId, permission);
    } catch (error) {
      console.error('Failed to revoke permission:', error);
    }
  };

  const getUserPermissions = (userId: string, platform: string) => {
    return permissions.filter(p => 
      p.platform === platform && 
      // In real implementation, this would filter by userId
      true
    );
  };

  const parsePermission = (permission: string) => {
    const [platform, resource, action] = permission.split(':');
    return { platform, resource, action };
  };

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
            Cross-Platform Permissions
          </h2>
          <p className="text-gray-600 dark:text-gray-400">
            Manage structured permissions across EPSX ecosystem
          </p>
        </div>
        <button
          onClick={() => setIsBuilding(!isBuilding)}
          className="inline-flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
        >
          <Plus className="h-4 w-4" />
          Grant Permission
        </button>
      </div>

      {/* Platform Selector */}
      <div className="mb-6">
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Platform
        </label>
        <div className="flex gap-2">
          {platforms.map((platform) => (
            <button
              key={platform.id}
              onClick={() => setSelectedPlatform(platform.code)}
              className={`px-4 py-2 rounded-lg font-medium transition-colors ${
                selectedPlatform === platform.code
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'
              }`}
            >
              {platform.name}
            </button>
          ))}
        </div>
      </div>

      {/* Permission Builder */}
      {isBuilding && (
        <div className="bg-gray-50 dark:bg-gray-900 rounded-lg p-4 mb-6">
          <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4">
            Grant New Permission
          </h3>
          
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-4">
            {/* User Selector */}
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                User
              </label>
              <select
                value={newPermissionUser}
                onChange={(e) => setNewPermissionUser(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              >
                <option value="">Select user...</option>
                {users.map((user) => (
                  <option key={user.id} value={user.id}>
                    {user.name} ({user.email})
                  </option>
                ))}
              </select>
            </div>

            {/* Resource Selector */}
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Resource
              </label>
              <select
                value={selectedResource}
                onChange={(e) => setSelectedResource(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              >
                <option value="">Select resource...</option>
                {Object.entries(resources).map(([key, resource]) => (
                  <option key={key} value={key}>
                    {key} - {resource.description}
                  </option>
                ))}
              </select>
            </div>

            {/* Action Selector */}
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Action
              </label>
              <select
                value={selectedAction}
                onChange={(e) => setSelectedAction(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                disabled={!selectedResource}
              >
                <option value="">Select action...</option>
                {actions.map((action) => (
                  <option key={action} value={action}>
                    {action}
                  </option>
                ))}
              </select>
            </div>

            {/* Actions */}
            <div className="flex items-end gap-2">
              <button
                onClick={handleGrantPermission}
                disabled={!newPermissionUser || !selectedResource || !selectedAction}
                className="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Grant
              </button>
              <button
                onClick={() => setIsBuilding(false)}
                className="px-4 py-2 bg-gray-500 text-white rounded-lg hover:bg-gray-600 transition-colors"
              >
                Cancel
              </button>
            </div>
          </div>

          {/* Preview */}
          {buildPermission() && (
            <div className="mt-4 p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
              <p className="text-sm font-medium text-blue-900 dark:text-blue-100">
                Permission Preview: <code className="font-mono bg-blue-100 dark:bg-blue-800 px-2 py-1 rounded">{buildPermission()}</code>
              </p>
            </div>
          )}
        </div>
      )}

      {/* Current Platform Resources */}
      <div className="space-y-4">
        <h3 className="text-lg font-medium text-gray-900 dark:text-white">
          {currentPlatform?.name} Resources
        </h3>
        
        {Object.entries(resources).map(([resourceKey, resource]) => {
          const ResourceIcon = resource.icon;
          
          return (
            <div key={resourceKey} className="border border-gray-200 dark:border-gray-700 rounded-lg p-4">
              <div className="flex items-center gap-3 mb-3">
                <ResourceIcon className="h-5 w-5 text-gray-500" />
                <div>
                  <h4 className="font-medium text-gray-900 dark:text-white">
                    {resourceKey}
                  </h4>
                  <p className="text-sm text-gray-500 dark:text-gray-400">
                    {resource.description}
                  </p>
                </div>
              </div>
              
              <div className="flex flex-wrap gap-2">
                {resource.actions.map((action) => {
                  const permission = `${selectedPlatform}:${resourceKey}:${action}`;
                  const hasPermission = permissions.some(p => 
                    p.platform === selectedPlatform && 
                    p.resource === resourceKey && 
                    p.action === action
                  );
                  
                  return (
                    <span
                      key={action}
                      className={`px-2 py-1 rounded text-xs font-medium ${
                        hasPermission 
                          ? 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400'
                          : 'bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400'
                      }`}
                    >
                      {action}
                    </span>
                  );
                })}
              </div>
            </div>
          );
        })}
      </div>

      {/* User Permissions List */}
      {selectedUser && (
        <div className="mt-8">
          <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4">
            Permissions for {selectedUser.name}
          </h3>
          
          <div className="space-y-2">
            {getUserPermissions(selectedUser.id, selectedPlatform).map((permission) => (
              <div
                key={permission.id}
                className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-900 rounded-lg"
              >
                <div>
                  <code className="font-mono text-sm bg-gray-200 dark:bg-gray-700 px-2 py-1 rounded">
                    {`${permission.platform}:${permission.resource}:${permission.action}`}
                  </code>
                  {permission.description && (
                    <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                      {permission.description}
                    </p>
                  )}
                </div>
                <button
                  onClick={() => handleRevokePermission(selectedUser.id, `${permission.platform}:${permission.resource}:${permission.action}`)}
                  className="p-1 text-red-600 hover:bg-red-100 dark:hover:bg-red-900/20 rounded"
                >
                  <X className="h-4 w-4" />
                </button>
              </div>
            ))}
            
            {getUserPermissions(selectedUser.id, selectedPlatform).length === 0 && (
              <p className="text-gray-500 dark:text-gray-400 text-center py-4">
                No permissions granted for {currentPlatform?.name}
              </p>
            )}
          </div>
        </div>
      )}
    </div>
  );
}