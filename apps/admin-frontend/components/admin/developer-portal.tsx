
import {
  Activity,
  AlertTriangle,
  BarChart3,
  BookOpen,
  Copy,
  Key,
  Shield
} from 'lucide-react';
import React, { useEffect, useState } from 'react';
import { toast } from 'sonner';

import { Button } from '@/components/ui/button';
import { logger } from '@/lib/logger';
import { createPlansClient, type ApiKeyResponse, type Module, type Plan } from '@/shared/api/plans';
import { createUsersClient } from '@/shared/api/users';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import { copyToClipboard as copyToClipboardUtil, createAdminApiClient } from '@/shared/utils';

import { ApiKeyManager } from './developer-portal/api-key-manager';
import { DocumentationViewer } from './developer-portal/documentation-viewer';
import { PortalOverview } from './developer-portal/portal-overview';
import { UsageAnalytics } from './developer-portal/usage-analytics';

/**
 * Main Developer Portal component
 */
export const DeveloperPortal: React.FC = () => {
  const { isLoading: authLoading } = useSharedAuth();

  const [activeTab, setActiveTab] = useState<
    'overview' | 'keys' | 'docs' | 'usage'
  >('overview');
  const [apiKeys, setApiKeys] = useState<ApiKeyResponse[]>([]);
  const [modules, setModules] = useState<Module[]>([]);
  const [availablePlans, setAvailablePlans] = useState<Plan[]>([]);
  const [loading, setLoading] = useState(true);
  const [accessDenied, setAccessDenied] = useState<{ message: string; code?: string } | null>(null);
  const [newApiKey, setNewApiKey] = useState<string | null>(null);

  // Load initial data
  useEffect(() => {
    void loadData();

    // Check for URL parameters (success message, new API key)
    const urlParams = new URLSearchParams(window.location.search);
    if (urlParams.get('success') === 'true') {
      const clientName = urlParams.get('client_name');
      const newKey = urlParams.get('new_key');

      if (clientName) {
        toast.success(`API key for "${clientName}" created successfully!`);
      }

      if (newKey && newKey !== 'key-created') {
        setNewApiKey(newKey);
      }

      // Clean up URL
      window.history.replaceState({}, '', window.location.pathname);
    }
  }, []);

  const loadData = async () => {
    try {
      setLoading(true);
      setAccessDenied(null);
      const apiClient = createAdminApiClient();
      const usersClient = createUsersClient(apiClient);
      const plansClient = createPlansClient(apiClient);

      // Load API Keys using PlansApi (Admin)
      try {
        const keysRes = await plansClient.listApiKeys();
        if (keysRes.success && keysRes.data) {
          setApiKeys(keysRes.data.api_keys ?? []);
        }
      } catch (error) {
        const err = error as { status?: number; code?: string; message?: string };
        // Handle Access Denied from backend
        if (err.status === 403 || err.code === 'PERMISSION_DENIED') {
          setAccessDenied({
            message: err.message ?? 'You don\'t have permission to access the developer portal.',
            code: err.code
          });
          return;
        }
        if (err.status !== 404) {
          logger.warn('Failed to load API keys', { error });
        }
        setApiKeys([]);
      }

      // Load Modules
      try {
        const modulesRes = await plansClient.getModules();
        if (modulesRes.success && modulesRes.data) {
          setModules(modulesRes.data.modules ?? []);
        }
      } catch (error) {
        logger.warn('Failed to load modules', { error });
        setModules([]);
      }

      // Load Available Plans
      try {
        const plansRes = await plansClient.listPlans({ is_active: true });
        if (plansRes.success && plansRes.data) {
          setAvailablePlans(plansRes.data.data ?? []); // PaginatedResponse has data property which is array
        }
      } catch (error) {
        logger.warn('Failed to load available plans', { error });
      }
    } catch (error) {
      const err = error as { status?: number; code?: string; message?: string };
      if (err.status === 403 || err.code === 'PERMISSION_DENIED') {
        setAccessDenied({
          message: err.message ?? 'You don\'t have permission to access the developer portal.',
          code: err.code
        });
        return;
      }
      logger.error('Failed to load developer portal data', { error });
    } finally {
      setLoading(false);
    }
  };

  // Handle API key revocation
  const handleRevokeApiKey = async (keyId: string, keyName: string) => {
    const reason = prompt(
      `Are you sure you want to revoke the API key for "${keyName}"? Please provide a reason:`
    );
    if (!reason) { return; }

    try {
      const apiClient = createAdminApiClient();
      const plansClient = createPlansClient(apiClient);
      const response = await plansClient.revokeApiKey(keyId);
      if (response.success) {
        toast.success('API key revoked successfully');
        void loadData();
      } else {
        toast.error('Failed to revoke API key');
      }
    } catch (_error) {
      logger.error('Failed to revoke API key', { keyId, _error });
      toast.error('Failed to revoke API key');
    }
  };

  // Copy to clipboard
  const copyToClipboard = async (text: string, label: string) => {
    const success = await copyToClipboardUtil(text);
    if (success) {
      toast.success(`${label} copied to clipboard`);
    } else {
      toast.error('Failed to copy to clipboard');
    }
  };

  // Show loading state while auth is initializing
  if (authLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="rounded-full h-8 w-8 border-b-2 border-blue-600 opacity-75" />
        <span className="ml-2 text-gray-600 dark:text-gray-300">Initializing...</span>
      </div>
    );
  }

  if (accessDenied) {
    return (
      <div className="flex items-center justify-center min-h-[60vh]">
        <div className="text-center max-w-md">
          <Shield className="w-16 h-16 text-gray-400 mx-auto mb-4" />
          <h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-2">
            Access Denied
          </h2>
          <p className="text-gray-600 dark:text-gray-300 mb-4">
            {accessDenied.message}
          </p>
          {accessDenied.code && (
            <p className="text-sm text-gray-500 dark:text-gray-400">
              Error code: {accessDenied.code}
            </p>
          )}
        </div>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="rounded-full h-8 w-8 border-b-2 border-blue-600 opacity-75" />
        <span className="ml-2 text-gray-600 dark:text-gray-300">Loading developer portal...</span>
      </div>
    );
  }

  return (
    <div className="p-6 max-w-7xl mx-auto">
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-2">
          Developer Portal
        </h1>
        <p className="text-gray-600 dark:text-gray-300">
          Manage API keys and third-party integrations
        </p>
      </div>

      {/* Tab Navigation */}
      <div className="flex space-x-1 mb-6 bg-gray-100 dark:bg-gray-800 p-1 rounded-lg w-fit">
        <button
          onClick={() => setActiveTab('overview')}
          className={`px-4 py-2 rounded-md font-medium transition-colors ${activeTab === 'overview'
            ? 'bg-white dark:bg-gray-700 text-blue-600 dark:text-blue-400 shadow-sm'
            : 'text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-gray-100'
            }`}
        >
          <BarChart3 className="w-4 h-4 inline mr-2" />
          Overview
        </button>
        <button
          onClick={() => setActiveTab('keys')}
          className={`px-4 py-2 rounded-md font-medium transition-colors ${activeTab === 'keys'
            ? 'bg-white dark:bg-gray-700 text-blue-600 dark:text-blue-400 shadow-sm'
            : 'text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-gray-100'
            }`}
        >
          <Key className="w-4 h-4 inline mr-2" />
          API Keys
        </button>
        <button
          onClick={() => setActiveTab('docs')}
          className={`px-4 py-2 rounded-md font-medium transition-colors ${activeTab === 'docs'
            ? 'bg-white dark:bg-gray-700 text-blue-600 dark:text-blue-400 shadow-sm'
            : 'text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-gray-100'
            }`}
        >
          <BookOpen className="w-4 h-4 inline mr-2" />
          Documentation
        </button>
        <button
          onClick={() => setActiveTab('usage')}
          className={`px-4 py-2 rounded-md font-medium transition-colors ${activeTab === 'usage'
            ? 'bg-white dark:bg-gray-700 text-blue-600 dark:text-blue-400 shadow-sm'
            : 'text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-gray-100'
            }`}
        >
          <Activity className="w-4 h-4 inline mr-2" />
          Usage Analytics
        </button>
      </div>

      {activeTab === 'overview' && <PortalOverview apiKeys={apiKeys} modules={modules} />}
      {activeTab === 'keys' && (
        <ApiKeyManager
          apiKeys={apiKeys}
          showKeyValue={null}
          onRevoke={handleRevokeApiKey}
          onCopy={copyToClipboard}
        />
      )}
      {activeTab === 'docs' && <DocumentationViewer modules={modules} availablePlans={availablePlans} />}
      {activeTab === 'usage' && <UsageAnalytics apiKeys={apiKeys} />}

      {/* New API Key Display */}
      {newApiKey && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-60">
          <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-md">
            <div className="p-6 border-b border-gray-200 dark:border-gray-600">
              <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                API Key Created
              </h2>
            </div>

            <div className="p-6">
              <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4 mb-4">
                <div className="flex items-start">
                  <AlertTriangle className="w-5 h-5 text-yellow-600 mt-0.5 mr-2" />
                  <div>
                    <h3 className="font-medium text-yellow-800">Important</h3>
                    <p className="text-sm text-yellow-700 mt-1">
                      This is the only time you&apos;ll see your API key. Please
                      copy it and store it securely.
                    </p>
                  </div>
                </div>
              </div>

              <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                <div className="flex items-center justify-between mb-2">
                  <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
                    Your API Key
                  </label>
                  <button
                    onClick={() => copyToClipboard(newApiKey, 'API Key')}
                    className="p-1 text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300"
                  >
                    <Copy className="w-4 h-4" />
                  </button>
                </div>
                <code className="block text-sm font-mono text-gray-900 dark:text-gray-100 bg-white dark:bg-gray-800 p-3 rounded border border-gray-200 dark:border-gray-600 break-all">
                  {newApiKey}
                </code>
              </div>
            </div>

            <div className="p-6 border-t border-gray-200 dark:border-gray-600 bg-gray-50 dark:bg-gray-700 flex justify-end">
              <Button onClick={() => setNewApiKey(null)}>
                I&apos;ve Saved the Key
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
