'use client';

import { Button, Card, CardContent, CardHeader, CardTitle } from '@/components/ui';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import type { AuthUser } from '@/lib/server-actions';
import { createPlansClient, type ApiKeyResponse, type Module } from '@/shared/api/plans';
import { UnifiedApiClient } from '@/shared/utils/api-client';
import { useEffect, useState } from 'react';
import { toast } from 'sonner';

interface APIKeyManagerProps {
  currentUser: AuthUser;
}

export function APIKeyManager({ currentUser }: APIKeyManagerProps) {
  const [apiKeys, setApiKeys] = useState<ApiKeyResponse[]>([]);
  const [modules, setModules] = useState<Module[]>([]);
  const [newKeyName, setNewKeyName] = useState('');
  const [showNewKey, setShowNewKey] = useState(false);
  const [generatedKey, setGeneratedKey] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [selectedModules, setSelectedModules] = useState<string[]>([]);
  const [expiresAt, setExpiresAt] = useState<string>(''); // ISO date string or empty

  // Get API client
  const getApiClient = () => {
    const client = new UnifiedApiClient({
      baseURL: process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080',
      platform: 'frontend',
    });
    return createPlansClient(client);
  };

  // Load API keys and modules on mount
  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    setIsLoading(true);
    try {
      const client = getApiClient();

      // Load user's own API keys using user-facing endpoint
      const keysResponse = await client.listMyApiKeys({ limit: 100 });
      if (keysResponse.success && keysResponse.data) {
        setApiKeys((keysResponse.data as any).api_keys || []);
      }

      // Load available groups for creating keys
      const groupsResponse = await client.getAvailableGroups();
      if (groupsResponse.success && groupsResponse.data) {
        // Convert groups to Module format for compatibility
        const groups = (groupsResponse.data as any).groups || [];
        setModules(groups.map((g: any) => ({
          id: g.id,
          name: g.slug,
          display_name: g.name,
          description: g.description,
          category: g.group_type,
          status: g.is_active ? 'active' : 'inactive',
          access_levels: {},
          default_quotas: {},
          endpoints: [],
        })));
      }
    } catch (error) {
      console.error('Failed to load data:', error);
      // Don't show error toast - just use empty state
    } finally {
      setIsLoading(false);
    }
  };

  const generateAPIKey = async () => {
    if (!newKeyName.trim()) {
      toast.error('Please enter a key name');
      return;
    }

    setIsLoading(true);

    try {
      const client = getApiClient();

      // Create API key using user-facing endpoint
      const response = await client.createMyApiKey({
        client_name: newKeyName,
        client_description: `API key for ${currentUser.walletAddress}`,
        group_ids: selectedModules, // Selected groups
        ip_restrictions: [],
        expires_at: expiresAt || undefined, // Use selected expiration or undefined for no expiration
      });

      if (response.success && response.data) {
        const newKey = response.data as any;
        setGeneratedKey(newKey.full_key || 'Key created - check your keys');
        setShowNewKey(true);
        setNewKeyName('');
        setSelectedModules([]);
        setExpiresAt(''); // Reset expiration
        toast.success('API key created successfully!');

        // Reload keys list
        await loadData();
      } else {
        toast.error('Failed to create API key');
      }
    } catch (error) {
      console.error('Failed to generate API key:', error);
      toast.error('Failed to generate API key');
    } finally {
      setIsLoading(false);
    }
  };

  const revokeAPIKey = async (keyId: string) => {
    const confirmed = window.confirm('Are you sure you want to revoke this API key? This action cannot be undone.');
    if (!confirmed) return;

    setIsLoading(true);
    try {
      const client = getApiClient();
      // Use user-facing revoke endpoint
      const response = await client.revokeMyApiKey(keyId, 'Revoked by user');

      if (response.success) {
        toast.success('API key revoked successfully');
        await loadData();
      } else {
        toast.error('Failed to revoke API key');
      }
    } catch (error) {
      console.error('Failed to revoke API key:', error);
      toast.error('Failed to revoke API key');
    } finally {
      setIsLoading(false);
    }
  };

  const toggleModuleSelection = (moduleId: string) => {
    setSelectedModules(prev =>
      prev.includes(moduleId)
        ? prev.filter(id => id !== moduleId)
        : [...prev, moduleId]
    );
  };

  const canCreateKeys = currentUser.role === 'premium' || currentUser.role === 'admin';

  return (
    <div className="space-y-6">
      {/* Create New Key */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center justify-between">
            Create New API Key
            {!canCreateKeys && (
              <Badge variant="secondary" className="bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300">
                Premium Required
              </Badge>
            )}
          </CardTitle>
        </CardHeader>
        <CardContent>
          {canCreateKeys ? (
            <div className="space-y-4">
              <div className="flex gap-4">
                <Input
                  placeholder="Enter key name (e.g., 'My Trading Bot')"
                  value={newKeyName}
                  onChange={(e) => setNewKeyName(e.target.value)}
                  className="flex-1"
                />
              </div>

              {/* Module Selection */}
              {modules.length > 0 && (
                <div>
                  <div className="text-sm text-gray-600 dark:text-gray-400 mb-2">
                    Select API groups to access:
                  </div>
                  <div className="flex flex-wrap gap-2">
                    {modules.map((module) => (
                      <Badge
                        key={module.id}
                        variant={selectedModules.includes(module.id) ? "default" : "outline"}
                        className="cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700"
                        onClick={() => toggleModuleSelection(module.id)}
                      >
                        {module.display_name}
                      </Badge>
                    ))}
                  </div>
                </div>
              )}

              {/* Expiration Date Picker */}
              <div>
                <div className="text-sm text-gray-600 dark:text-gray-400 mb-2">
                  Key Expiration (optional):
                </div>
                <div className="flex items-center gap-4">
                  <Input
                    type="date"
                    value={expiresAt ? new Date(expiresAt).toISOString().split('T')[0] : ''}
                    onChange={(e) => {
                      if (e.target.value) {
                        // Set to end of day in UTC
                        const date = new Date(e.target.value);
                        date.setUTCHours(23, 59, 59, 999);
                        setExpiresAt(date.toISOString());
                      } else {
                        setExpiresAt('');
                      }
                    }}
                    min={new Date().toISOString().split('T')[0]}
                    className="max-w-[200px]"
                  />
                  {expiresAt && (
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => setExpiresAt('')}
                      className="text-gray-500"
                    >
                      Clear
                    </Button>
                  )}
                  <span className="text-xs text-gray-500 dark:text-gray-400">
                    {expiresAt ? `Expires: ${new Date(expiresAt).toLocaleDateString()}` : 'Never expires'}
                  </span>
                </div>
              </div>

              <Button
                onClick={generateAPIKey}
                disabled={!newKeyName.trim() || isLoading}
              >
                {isLoading ? 'Generating...' : 'Generate Key'}
              </Button>

              {showNewKey && generatedKey && (
                <div className="p-4 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg">
                  <h4 className="font-semibold text-green-800 dark:text-green-300 mb-2">
                    🎉 New API Key Generated
                  </h4>
                  <p className="text-sm text-green-700 dark:text-green-400 mb-3">
                    Make sure to copy your API key now. You won't be able to see it again!
                  </p>
                  <div className="bg-white dark:bg-gray-800 p-3 rounded border font-mono text-sm break-all">
                    {generatedKey}
                  </div>
                  <Button
                    size="sm"
                    className="mt-3"
                    onClick={() => {
                      navigator.clipboard.writeText(generatedKey);
                      toast.success('API key copied to clipboard');
                      setShowNewKey(false);
                    }}
                  >
                    Copy & Close
                  </Button>
                </div>
              )}
            </div>
          ) : (
            <div className="text-center py-8">
              <div className="text-gray-500 dark:text-gray-400 mb-4">
                API key generation requires a Premium subscription
              </div>
              <Button variant="outline">
                Upgrade to Premium
              </Button>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Existing Keys */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center justify-between">
            Your API Keys
            <Button variant="outline" size="sm" onClick={loadData} disabled={isLoading}>
              {isLoading ? 'Loading...' : 'Refresh'}
            </Button>
          </CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading && apiKeys.length === 0 ? (
            <div className="text-center py-8 text-gray-500 dark:text-gray-400">
              Loading...
            </div>
          ) : apiKeys.length === 0 ? (
            <div className="text-center py-8 text-gray-500 dark:text-gray-400">
              No API keys created yet
            </div>
          ) : (
            <div className="space-y-4">
              {apiKeys.map((apiKey) => (
                <div key={apiKey.id} className="border border-gray-200 dark:border-gray-700 rounded-lg p-4">
                  <div className="flex items-center justify-between mb-3">
                    <h3 className="font-semibold">{apiKey.client_name}</h3>
                    <div className="flex items-center gap-2">
                      <Badge
                        variant={apiKey.status === 'active' ? "default" : "secondary"}
                        className={apiKey.status === 'active' ? "bg-green-500 text-white" : "bg-gray-500 text-white"}
                      >
                        {apiKey.status === 'active' ? 'Active' : 'Revoked'}
                      </Badge>
                    </div>
                  </div>

                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                    <div>
                      <div className="text-gray-500 dark:text-gray-400">Key Preview</div>
                      <div className="font-mono bg-gray-100 dark:bg-gray-700 p-2 rounded text-xs break-all">
                        {apiKey.key_prefix}...
                      </div>
                    </div>

                    <div>
                      <div className="text-gray-500 dark:text-gray-400">Permission Groups</div>
                      <div className="flex flex-wrap gap-1 mt-1">
                        {((apiKey as any).groups?.length > 0) ? (
                          (apiKey as any).groups.map((group: { id: string; name: string; slug: string }, idx: number) => (
                            <Badge key={idx} variant="outline" className="text-xs">
                              {group.name}
                            </Badge>
                          ))
                        ) : (
                          <span className="text-gray-400">No groups assigned</span>
                        )}
                      </div>
                    </div>

                    <div>
                      <div className="text-gray-500 dark:text-gray-400">Created</div>
                      <div>{new Date(apiKey.created_at).toLocaleDateString()}</div>
                    </div>

                    <div>
                      <div className="text-gray-500 dark:text-gray-400">Total Requests</div>
                      <div>{apiKey.total_requests?.toLocaleString() || 0}</div>
                    </div>

                    <div>
                      <div className="text-gray-500 dark:text-gray-400">Expires</div>
                      <div>
                        {apiKey.expires_at ? (
                          (() => {
                            const expiresAt = new Date(apiKey.expires_at);
                            const now = new Date();
                            const daysUntilExpiry = Math.ceil((expiresAt.getTime() - now.getTime()) / (1000 * 60 * 60 * 24));
                            const isExpired = daysUntilExpiry < 0;
                            const isExpiringSoon = daysUntilExpiry >= 0 && daysUntilExpiry <= 7;
                            const isExpiringMedium = daysUntilExpiry > 7 && daysUntilExpiry <= 30;

                            return (
                              <span className={`inline-flex items-center gap-1 ${isExpired ? 'text-gray-500' :
                                  isExpiringSoon ? 'text-red-600 dark:text-red-400 font-medium' :
                                    isExpiringMedium ? 'text-yellow-600 dark:text-yellow-400' :
                                      'text-gray-900 dark:text-gray-100'
                                }`}>
                                {isExpired && '⚫ Expired '}
                                {isExpiringSoon && !isExpired && '🔴 '}
                                {isExpiringMedium && '🟡 '}
                                {expiresAt.toLocaleDateString()}
                                {isExpiringSoon && !isExpired && ` (${daysUntilExpiry}d)`}
                              </span>
                            );
                          })()
                        ) : (
                          <span className="text-green-600 dark:text-green-400">Never</span>
                        )}
                      </div>
                    </div>
                  </div>

                  {apiKey.status === 'active' && (
                    <div className="mt-4 pt-3 border-t border-gray-200 dark:border-gray-700">
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => revokeAPIKey(apiKey.id)}
                        disabled={isLoading}
                        className="text-red-600 hover:text-red-700 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-900/20"
                      >
                        Revoke Key
                      </Button>
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Security Notice */}
      <Card className="bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800">
        <CardContent className="p-6">
          <h3 className="font-semibold text-blue-800 dark:text-blue-300 mb-2">
            🛡️ Security Best Practices
          </h3>
          <ul className="text-sm text-blue-700 dark:text-blue-400 space-y-1 list-disc list-inside">
            <li>Never commit API keys to version control</li>
            <li>Use environment variables to store keys securely</li>
            <li>Rotate keys regularly for production applications</li>
            <li>Monitor key usage and revoke unused keys</li>
            <li>Use different keys for different environments</li>
          </ul>
        </CardContent>
      </Card>
    </div>
  );
}