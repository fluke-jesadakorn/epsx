'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Textarea } from '@/components/ui/textarea';
import { 
  Key, 
  Plus, 
  Copy, 
  Eye, 
  EyeOff, 
  Trash2, 
  Shield, 
  Calendar,
  AlertTriangle,
  CheckCircle,
  ExternalLink
} from 'lucide-react';
import { useWeb3AuthContext } from '@/providers/Web3AuthProvider';
import { toast } from 'sonner';
import { formatDistanceToNow } from 'date-fns';

interface ApiKey {
  id: string;
  name: string;
  key: string;
  created_at: string;
  last_used_at?: string;
  usage_count: number;
  is_active: boolean;
  scopes: string[];
}

interface ApiKeyManagerProps {
  className?: string;
}

export function ApiKeyManager({ className = '' }: ApiKeyManagerProps) {
  const { isAuthenticated, hasApiAccess, userTier, generateApiKey } = useWeb3AuthContext();
  const [apiKeys, setApiKeys] = useState<ApiKey[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isGenerating, setIsGenerating] = useState(false);
  const [newKeyName, setNewKeyName] = useState('');
  const [showNewKey, setShowNewKey] = useState<string | null>(null);
  const [visibleKeys, setVisibleKeys] = useState<Set<string>>(new Set());

  // Fetch API keys on mount
  useEffect(() => {
    if (isAuthenticated && hasApiAccess) {
      fetchApiKeys();
    }
  }, [isAuthenticated, hasApiAccess]);

  const fetchApiKeys = async () => {
    try {
      setIsLoading(true);
      const response = await fetch('/api/auth/web3/api-keys', {
        credentials: 'include',
      });

      if (response.ok) {
        const { api_keys } = await response.json();
        setApiKeys(api_keys || []);
      }
    } catch (error) {
      console.error('Failed to fetch API keys:', error);
      toast.error('Failed to load API keys');
    } finally {
      setIsLoading(false);
    }
  };

  const handleGenerateKey = async () => {
    if (!newKeyName.trim()) {
      toast.error('Please enter a name for the API key');
      return;
    }

    try {
      setIsGenerating(true);
      const newKey = await generateApiKey(newKeyName.trim());
      
      // Refresh the list
      await fetchApiKeys();
      
      // Show the new key
      setShowNewKey(newKey);
      setNewKeyName('');
      
      toast.success('API key generated successfully');
    } catch (error: any) {
      console.error('Failed to generate API key:', error);
      toast.error(error.message || 'Failed to generate API key');
    } finally {
      setIsGenerating(false);
    }
  };

  const handleDeleteKey = async (keyId: string, keyName: string) => {
    if (!confirm(`Are you sure you want to delete the API key "${keyName}"? This action cannot be undone.`)) {
      return;
    }

    try {
      const response = await fetch(`/api/auth/web3/api-keys/${keyId}`, {
        method: 'DELETE',
        credentials: 'include',
      });

      if (response.ok) {
        await fetchApiKeys();
        toast.success('API key deleted successfully');
      } else {
        throw new Error('Failed to delete API key');
      }
    } catch (error) {
      console.error('Failed to delete API key:', error);
      toast.error('Failed to delete API key');
    }
  };

  const copyToClipboard = (text: string, description: string) => {
    navigator.clipboard.writeText(text).then(() => {
      toast.success(`${description} copied to clipboard`);
    }).catch(() => {
      toast.error('Failed to copy to clipboard');
    });
  };

  const toggleKeyVisibility = (keyId: string) => {
    setVisibleKeys(prev => {
      const newSet = new Set(prev);
      if (newSet.has(keyId)) {
        newSet.delete(keyId);
      } else {
        newSet.add(keyId);
      }
      return newSet;
    });
  };

  const formatApiKey = (key: string, isVisible: boolean) => {
    if (isVisible) return key;
    return `${key.slice(0, 8)}${'•'.repeat(32)}${key.slice(-8)}`;
  };

  // Access control checks
  if (!isAuthenticated) {
    return (
      <Card className={className}>
        <CardContent className="p-6">
          <div className="flex flex-col items-center gap-3 text-center">
            <Shield className="h-12 w-12 text-slate-400" />
            <h3 className="font-semibold text-slate-900 dark:text-slate-100">
              Authentication Required
            </h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">
              Please connect your Web3 wallet to access API key management.
            </p>
          </div>
        </CardContent>
      </Card>
    );
  }

  if (!hasApiAccess) {
    return (
      <Card className={className}>
        <CardContent className="p-6">
          <div className="flex flex-col items-center gap-4 text-center">
            <div className="p-3 bg-amber-100 dark:bg-amber-900/20 rounded-full">
              <Key className="h-8 w-8 text-amber-600 dark:text-amber-400" />
            </div>
            <div>
              <h3 className="font-semibold text-slate-900 dark:text-slate-100 mb-2">
                API Access Not Available
              </h3>
              <p className="text-sm text-slate-600 dark:text-slate-400 mb-4">
                API key management requires Enterprise tier access or special permissions.
              </p>
              <div className="flex items-center justify-center gap-2">
                <Badge variant="outline" className="text-xs">
                  Current: {userTier}
                </Badge>
                <Badge className="bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-300 text-xs">
                  Required: Enterprise
                </Badge>
              </div>
            </div>
            <Button
              variant="outline"
              onClick={() => window.open('/plans', '_blank')}
              className="flex items-center gap-2"
            >
              <ExternalLink className="h-4 w-4" />
              Upgrade to Enterprise
            </Button>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Key className="h-5 w-5 text-orange-500" />
            API Key Management
          </CardTitle>
          <p className="text-sm text-slate-600 dark:text-slate-400">
            Generate and manage API keys for programmatic access to EPSX services.
          </p>
        </CardHeader>
        <CardContent>
          {/* New Key Generation */}
          <div className="space-y-4">
            <div className="flex gap-3">
              <Input
                placeholder="Enter API key name (e.g., Production App, Analytics Dashboard)"
                value={newKeyName}
                onChange={(e) => setNewKeyName(e.target.value)}
                className="flex-1"
              />
              <Button
                onClick={handleGenerateKey}
                disabled={isGenerating || !newKeyName.trim()}
                className="flex items-center gap-2"
              >
                <Plus className="h-4 w-4" />
                {isGenerating ? 'Generating...' : 'Generate Key'}
              </Button>
            </div>

            {/* Security Notice */}
            <div className="p-3 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg">
              <div className="flex items-start gap-2">
                <AlertTriangle className="h-4 w-4 text-amber-600 dark:text-amber-400 mt-0.5" />
                <div className="text-xs text-amber-700 dark:text-amber-400">
                  <p className="font-medium mb-1">Important Security Notice:</p>
                  <ul className="space-y-1">
                    <li>• API keys provide full access to your account - treat them like passwords</li>
                    <li>• Store keys securely and never commit them to version control</li>
                    <li>• Regenerate keys immediately if compromised</li>
                    <li>• Use environment variables in production applications</li>
                  </ul>
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* New Key Display */}
      {showNewKey && (
        <Card className="border-green-200 dark:border-green-800">
          <CardHeader className="bg-green-50 dark:bg-green-900/20">
            <CardTitle className="flex items-center gap-2 text-green-800 dark:text-green-300">
              <CheckCircle className="h-5 w-5" />
              New API Key Generated
            </CardTitle>
          </CardHeader>
          <CardContent className="p-6">
            <div className="space-y-3">
              <p className="text-sm text-green-700 dark:text-green-400">
                Your new API key has been generated. Copy it now - you won't be able to see it again!
              </p>
              <div className="flex items-center gap-2 p-3 bg-slate-100 dark:bg-slate-800 rounded-lg border">
                <code className="flex-1 text-sm font-mono break-all">
                  {showNewKey}
                </code>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => copyToClipboard(showNewKey, 'API key')}
                >
                  <Copy className="h-4 w-4" />
                </Button>
              </div>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setShowNewKey(null)}
                className="w-full"
              >
                I've saved the key securely
              </Button>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Existing Keys */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center justify-between">
            <span>Your API Keys ({apiKeys.length})</span>
            {apiKeys.length > 0 && (
              <Button
                variant="outline"
                size="sm"
                onClick={fetchApiKeys}
                disabled={isLoading}
              >
                Refresh
              </Button>
            )}
          </CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <div className="flex items-center gap-2">
                <div className="h-4 w-4 bg-orange-500 rounded animate-pulse" />
                <span className="text-sm text-slate-600 dark:text-slate-400">
                  Loading API keys...
                </span>
              </div>
            </div>
          ) : apiKeys.length === 0 ? (
            <div className="text-center py-8">
              <Key className="h-8 w-8 text-slate-400 mx-auto mb-2" />
              <p className="text-sm text-slate-600 dark:text-slate-400">
                No API keys found. Generate your first key above.
              </p>
            </div>
          ) : (
            <div className="space-y-4">
              {apiKeys.map((key) => (
                <div
                  key={key.id}
                  className="p-4 border border-slate-200 dark:border-slate-700 rounded-lg space-y-3"
                >
                  {/* Key Header */}
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <div className="p-2 bg-slate-100 dark:bg-slate-800 rounded-lg">
                        <Key className="h-4 w-4 text-slate-600 dark:text-slate-400" />
                      </div>
                      <div>
                        <h4 className="font-medium text-slate-900 dark:text-slate-100">
                          {key.name}
                        </h4>
                        <div className="flex items-center gap-2 text-xs text-slate-600 dark:text-slate-400">
                          <Calendar className="h-3 w-3" />
                          <span>
                            Created {formatDistanceToNow(new Date(key.created_at), { addSuffix: true })}
                          </span>
                        </div>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <Badge variant={key.is_active ? 'default' : 'secondary'}>
                        {key.is_active ? 'Active' : 'Inactive'}
                      </Badge>
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => handleDeleteKey(key.id, key.name)}
                        className="text-red-600 hover:text-red-700 hover:bg-red-50"
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </div>
                  </div>

                  {/* API Key Display */}
                  <div className="space-y-2">
                    <div className="flex items-center gap-2">
                      <span className="text-xs font-medium text-slate-700 dark:text-slate-300">
                        API Key:
                      </span>
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => toggleKeyVisibility(key.id)}
                        className="h-6 px-2"
                      >
                        {visibleKeys.has(key.id) ? (
                          <EyeOff className="h-3 w-3" />
                        ) : (
                          <Eye className="h-3 w-3" />
                        )}
                      </Button>
                    </div>
                    <div className="flex items-center gap-2 p-2 bg-slate-50 dark:bg-slate-800 rounded border">
                      <code className="flex-1 text-xs font-mono break-all">
                        {formatApiKey(key.key, visibleKeys.has(key.id))}
                      </code>
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => copyToClipboard(key.key, 'API key')}
                        className="h-6 px-2"
                      >
                        <Copy className="h-3 w-3" />
                      </Button>
                    </div>
                  </div>

                  {/* Usage Stats */}
                  <div className="flex items-center justify-between text-xs text-slate-600 dark:text-slate-400">
                    <span>Usage: {key.usage_count} requests</span>
                    {key.last_used_at && (
                      <span>
                        Last used {formatDistanceToNow(new Date(key.last_used_at), { addSuffix: true })}
                      </span>
                    )}
                  </div>

                  {/* Scopes */}
                  {key.scopes && key.scopes.length > 0 && (
                    <div className="flex items-center gap-2">
                      <span className="text-xs font-medium text-slate-700 dark:text-slate-300">
                        Scopes:
                      </span>
                      <div className="flex gap-1">
                        {key.scopes.map((scope, index) => (
                          <Badge key={index} variant="outline" className="text-xs">
                            {scope}
                          </Badge>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* API Documentation */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <ExternalLink className="h-5 w-5 text-blue-500" />
            API Documentation
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <p className="text-sm text-slate-600 dark:text-slate-400">
              Learn how to use your API keys to access EPSX services programmatically.
            </p>
            
            <div className="space-y-3">
              <div className="p-3 bg-slate-50 dark:bg-slate-800 rounded-lg">
                <h4 className="text-sm font-medium text-slate-900 dark:text-slate-100 mb-1">
                  Authentication Header
                </h4>
                <code className="text-xs text-slate-600 dark:text-slate-400">
                  Authorization: Bearer YOUR_API_KEY
                </code>
              </div>
              
              <div className="p-3 bg-slate-50 dark:bg-slate-800 rounded-lg">
                <h4 className="text-sm font-medium text-slate-900 dark:text-slate-100 mb-1">
                  Base URL
                </h4>
                <code className="text-xs text-slate-600 dark:text-slate-400">
                  https://api.epsx.io/v1/
                </code>
              </div>
            </div>

            <div className="flex gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => window.open('/docs/api', '_blank')}
              >
                <ExternalLink className="h-4 w-4 mr-1" />
                API Documentation
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => window.open('/docs/api/examples', '_blank')}
              >
                Code Examples
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}