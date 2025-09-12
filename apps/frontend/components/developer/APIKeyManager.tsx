'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle, Button } from '@/components/ui';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import type { User } from '@/lib/server-actions';

interface APIKey {
  id: string;
  name: string;
  key: string;
  lastUsed?: string;
  created: string;
  permissions: string[];
  isActive: boolean;
}

interface APIKeyManagerProps {
  currentUser: User;
}

export function APIKeyManager({ currentUser }: APIKeyManagerProps) {
  const [apiKeys, setApiKeys] = useState<APIKey[]>([]);
  const [newKeyName, setNewKeyName] = useState('');
  const [showNewKey, setShowNewKey] = useState(false);
  const [generatedKey, setGeneratedKey] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  // Mock data for demonstration
  useEffect(() => {
    setApiKeys([
      {
        id: '1',
        name: 'Production API',
        key: 'epsx_pk_live_1234567890abcdef',
        lastUsed: '2024-01-15T10:30:00Z',
        created: '2024-01-01T00:00:00Z',
        permissions: ['analytics:read', 'rankings:read'],
        isActive: true
      }
    ]);
  }, []);

  const generateAPIKey = async () => {
    if (!newKeyName.trim()) return;
    
    setIsLoading(true);
    
    try {
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      const newKey = {
        id: Date.now().toString(),
        name: newKeyName,
        key: `epsx_pk_${currentUser.role === 'premium' || currentUser.role === 'admin' ? 'live' : 'test'}_${Math.random().toString(36).substr(2, 24)}`,
        created: new Date().toISOString(),
        permissions: currentUser.role === 'admin' ? ['*'] : ['analytics:read', 'rankings:read'],
        isActive: true
      };

      setApiKeys(prev => [...prev, newKey]);
      setGeneratedKey(newKey.key);
      setShowNewKey(true);
      setNewKeyName('');
    } catch (error) {
      console.error('Failed to generate API key:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const revokeAPIKey = async (keyId: string) => {
    setIsLoading(true);
    try {
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 500));
      setApiKeys(prev => prev.map(key => 
        key.id === keyId ? { ...key, isActive: false } : key
      ));
    } catch (error) {
      console.error('Failed to revoke API key:', error);
    } finally {
      setIsLoading(false);
    }
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
                  placeholder="Enter key name (e.g., 'Production API')"
                  value={newKeyName}
                  onChange={(e) => setNewKeyName(e.target.value)}
                  className="flex-1"
                />
                <Button 
                  onClick={generateAPIKey}
                  disabled={!newKeyName.trim() || isLoading}
                >
                  {isLoading ? 'Generating...' : 'Generate Key'}
                </Button>
              </div>
              
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
          <CardTitle>Your API Keys</CardTitle>
        </CardHeader>
        <CardContent>
          {apiKeys.length === 0 ? (
            <div className="text-center py-8 text-gray-500 dark:text-gray-400">
              No API keys created yet
            </div>
          ) : (
            <div className="space-y-4">
              {apiKeys.map((apiKey) => (
                <div key={apiKey.id} className="border border-gray-200 dark:border-gray-700 rounded-lg p-4">
                  <div className="flex items-center justify-between mb-3">
                    <h3 className="font-semibold">{apiKey.name}</h3>
                    <div className="flex items-center gap-2">
                      <Badge 
                        variant={apiKey.isActive ? "default" : "secondary"}
                        className={apiKey.isActive ? "bg-green-500 text-white" : "bg-gray-500 text-white"}
                      >
                        {apiKey.isActive ? 'Active' : 'Revoked'}
                      </Badge>
                    </div>
                  </div>
                  
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                    <div>
                      <div className="text-gray-500 dark:text-gray-400">Key Preview</div>
                      <div className="font-mono bg-gray-100 dark:bg-gray-700 p-2 rounded text-xs break-all">
                        {apiKey.key.substring(0, 20)}...{apiKey.key.substring(-8)}
                      </div>
                    </div>
                    
                    <div>
                      <div className="text-gray-500 dark:text-gray-400">Permissions</div>
                      <div className="flex flex-wrap gap-1 mt-1">
                        {apiKey.permissions.map((permission, idx) => (
                          <Badge key={idx} variant="outline" className="text-xs">
                            {permission}
                          </Badge>
                        ))}
                      </div>
                    </div>
                    
                    <div>
                      <div className="text-gray-500 dark:text-gray-400">Created</div>
                      <div>{new Date(apiKey.created).toLocaleDateString()}</div>
                    </div>
                    
                    <div>
                      <div className="text-gray-500 dark:text-gray-400">Last Used</div>
                      <div>
                        {apiKey.lastUsed 
                          ? new Date(apiKey.lastUsed).toLocaleDateString()
                          : 'Never'
                        }
                      </div>
                    </div>
                  </div>
                  
                  {apiKey.isActive && (
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