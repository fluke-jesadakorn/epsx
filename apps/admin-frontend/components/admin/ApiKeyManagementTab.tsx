'use client';

import { Key, Plus, Copy, Eye, EyeOff, Trash2, Settings } from 'lucide-react';
import { memo, useCallback } from 'react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';

interface ApiKey {
  id: string;
  client_name: string;
  client_description?: string;
  client_contact_email: string;
  key_preview: string;
  status: 'active' | 'revoked' | 'expired';
  total_requests: number;
  rate_limit_per_minute: number;
  last_used_at?: string;
  created_at: string;
  expires_at?: string;
  allowed_modules: Array<{
    module_id: string;
    access_level: string;
  }>;
}

interface ApiKeyManagementTabProps {
  apiKeys: ApiKey[];
  showKeyValue: string | null;
  onToggleKeyVisibility: (keyId: string) => void;
  onRevokeApiKey: (keyId: string, keyName: string) => void;
  onCreateApiKey: () => void;
  onCopyToClipboard: (text: string, label: string) => void;
}

// Access levels configuration
const ACCESS_LEVELS = [
  {
    value: 'bronze',
    label: 'Bronze',
    color: 'text-amber-600',
    bgColor: 'bg-amber-100 text-amber-800',
  },
  {
    value: 'silver',
    label: 'Silver',
    color: 'text-gray-500',
    bgColor: 'bg-gray-100 text-gray-800',
  },
  {
    value: 'gold',
    label: 'Gold',
    color: 'text-yellow-500',
    bgColor: 'bg-yellow-100 text-yellow-800',
  },
  {
    value: 'platinum',
    label: 'Platinum',
    color: 'text-purple-600',
    bgColor: 'bg-purple-100 text-purple-800',
  },
  {
    value: 'enterprise',
    label: 'Enterprise',
    color: 'text-blue-600',
    bgColor: 'bg-blue-100 text-blue-800',
  },
];

function ApiKeyManagementTab({
  apiKeys,
  showKeyValue,
  onToggleKeyVisibility,
  onRevokeApiKey,
  onCreateApiKey,
  onCopyToClipboard,
}: ApiKeyManagementTabProps) {

  const getStatusColor = useCallback((status: string) => {
    switch (status) {
      case 'active':
        return 'bg-green-100 dark:bg-green-900/50 text-green-800 dark:text-green-200';
      case 'revoked':
        return 'bg-red-100 dark:bg-red-900/50 text-red-800 dark:text-red-200';
      case 'expired':
        return 'bg-yellow-100 dark:bg-yellow-900/50 text-yellow-800 dark:text-yellow-200';
      default:
        return 'bg-gray-100 dark:bg-gray-800 text-gray-800 dark:text-gray-300';
    }
  }, []);

  const getAccessLevelColor = useCallback((level: string) => {
    const accessLevel = ACCESS_LEVELS.find(al => al.value === level);
    return accessLevel?.bgColor || 'bg-gray-100 text-gray-800';
  }, []);

  const handleRevoke = useCallback((apiKey: ApiKey) => {
    onRevokeApiKey(apiKey.id, apiKey.client_name);
  }, [onRevokeApiKey]);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            API Key Management
          </h2>
          <p className="text-sm text-gray-600 dark:text-gray-300">
            Create and manage API keys for third-party integrations
          </p>
        </div>
        <Button onClick={onCreateApiKey}>
          <Plus className="w-4 h-4 mr-2" />
          Create API Key
        </Button>
      </div>

      <div className="grid grid-cols-1 gap-6">
        {apiKeys.map(apiKey => (
          <Card key={apiKey.id}>
            <CardContent className="p-6">
              <div className="flex items-start justify-between mb-4">
                <div className="flex items-center space-x-4">
                  <div className="p-3 bg-blue-100 dark:bg-blue-900/50 rounded-lg">
                    <Key className="w-6 h-6 text-blue-600 dark:text-blue-400" />
                  </div>
                  <div>
                    <h3 className="font-semibold text-gray-900 dark:text-gray-100">
                      {apiKey.client_name}
                    </h3>
                    <p className="text-sm text-gray-500 dark:text-gray-400">
                      {apiKey.client_description || 'No description'}
                    </p>
                    <div className="flex items-center space-x-4 mt-1">
                      <span className="text-xs text-gray-500 dark:text-gray-400">
                        Created: {new Date(apiKey.created_at).toLocaleDateString()}
                      </span>
                      {apiKey.last_used_at && (
                        <span className="text-xs text-gray-500 dark:text-gray-400">
                          Last used: {new Date(apiKey.last_used_at).toLocaleDateString()}
                        </span>
                      )}
                    </div>
                  </div>
                </div>
                <div className="flex items-center space-x-2">
                  <Badge className={getStatusColor(apiKey.status)}>
                    {apiKey.status}
                  </Badge>
                  {apiKey.status === 'active' && (
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => handleRevoke(apiKey)}
                    >
                      <Trash2 className="w-4 h-4 mr-1" />
                      Revoke
                    </Button>
                  )}
                </div>
              </div>

              <div className="space-y-4">
                {/* API Key Display */}
                <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                  <div className="flex items-center justify-between mb-2">
                    <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
                      API Key
                    </label>
                    <div className="flex space-x-2">
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => onToggleKeyVisibility(apiKey.id)}
                      >
                        {showKeyValue === apiKey.id ? (
                          <EyeOff className="w-4 h-4" />
                        ) : (
                          <Eye className="w-4 h-4" />
                        )}
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => onCopyToClipboard(
                          showKeyValue === apiKey.id ? apiKey.id : apiKey.key_preview,
                          'API Key'
                        )}
                      >
                        <Copy className="w-4 h-4" />
                      </Button>
                    </div>
                  </div>
                  <code className="text-sm bg-white dark:bg-gray-800 border rounded px-3 py-2 block font-mono">
                    {showKeyValue === apiKey.id ? apiKey.id : apiKey.key_preview}
                  </code>
                </div>

                {/* Usage Stats */}
                <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
                  <div className="text-center">
                    <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                      {apiKey.total_requests.toLocaleString()}
                    </p>
                    <p className="text-sm text-gray-600 dark:text-gray-300">
                      Total Requests
                    </p>
                  </div>
                  <div className="text-center">
                    <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                      {apiKey.rate_limit_per_minute}
                    </p>
                    <p className="text-sm text-gray-600 dark:text-gray-300">
                      Rate Limit/min
                    </p>
                  </div>
                  <div className="text-center">
                    <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                      {apiKey.allowed_modules.length}
                    </p>
                    <p className="text-sm text-gray-600 dark:text-gray-300">
                      Modules
                    </p>
                  </div>
                </div>

                {/* Module Access */}
                <div>
                  <h4 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    Module Access
                  </h4>
                  <div className="flex flex-wrap gap-2">
                    {apiKey.allowed_modules.map((module, index) => (
                      <Badge
                        key={index}
                        className={getAccessLevelColor(module.access_level)}
                      >
                        {module.module_id}: {module.access_level}
                      </Badge>
                    ))}
                  </div>
                </div>

                {/* Contact Info */}
                <div className="flex items-center justify-between text-sm">
                  <span className="text-gray-600 dark:text-gray-300">
                    Contact: {apiKey.client_contact_email}
                  </span>
                  {apiKey.expires_at && (
                    <span className="text-gray-600 dark:text-gray-300">
                      Expires: {new Date(apiKey.expires_at).toLocaleDateString()}
                    </span>
                  )}
                </div>
              </div>
            </CardContent>
          </Card>
        ))}

        {apiKeys.length === 0 && (
          <Card>
            <CardContent className="p-12 text-center">
              <Key className="w-12 h-12 text-gray-400 mx-auto mb-4" />
              <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
                No API Keys
              </h3>
              <p className="text-gray-600 dark:text-gray-300 mb-4">
                Create your first API key to start integrating with our services.
              </p>
              <Button onClick={onCreateApiKey}>
                <Plus className="w-4 h-4 mr-2" />
                Create API Key
              </Button>
            </CardContent>
          </Card>
        )}
      </div>
    </div>
  );
}

export default memo(ApiKeyManagementTab);