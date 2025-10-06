'use client';

import { Shield, BarChart3, Key, BookOpen, Activity } from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useState, useEffect, useCallback, memo } from 'react';
import { toast } from 'react-hot-toast';

// Extracted tab components
import ApiKeyManagementTab from './ApiKeyManagementTab';
import DeveloperOverviewTab from './DeveloperOverviewTab';
import DocumentationTab from './DocumentationTab';
import UsageAnalyticsTab from './UsageAnalyticsTab';

// API Client
import { createPlansClient, type ApiKeyResponse as ApiKey, type Module } from '@/shared/api/plans';
import { createAdminApiClient } from '@/shared/utils/api-client';

interface DeveloperPortalProps {
  // Optional props for customization
  defaultTab?: 'overview' | 'keys' | 'docs' | 'usage';
  hideDownloads?: boolean;
}

function DeveloperPortalRefactored({ 
  defaultTab = 'overview', 
  hideDownloads = false 
}: DeveloperPortalProps) {
  const router = useRouter();
  
  // Core state
  const [activeTab, setActiveTab] = useState<'overview' | 'keys' | 'docs' | 'usage'>(defaultTab);
  const [apiKeys, setApiKeys] = useState<ApiKey[]>([]);
  const [modules, setModules] = useState<Module[]>([]);
  const [loading, setLoading] = useState(true);
  const [showKeyValue, setShowKeyValue] = useState<string | null>(null);

  // SECURITY: Proper permission checks - NEVER hardcode to true
  const hasModuleAccess = useCallback((module?: string) => {
    // TODO: Integrate with actual OIDC/permission system
    // Check user permissions against module access requirements
    return false; // Default to deny access for security
  }, []);
  
  const canPerformAction = useCallback((action?: string) => {
    // TODO: Integrate with actual OIDC/permission system
    // Check user permissions against specific action requirements
    return false; // Default to deny access for security
  }, []);
  const canManageApiKeys = hasModuleAccess() && canPerformAction();

  // Load data
  const loadData = useCallback(async () => {
    try {
      setLoading(true);
      const [keysRes, modulesRes] = await Promise.all([
        Promise.resolve({ success: true, data: { api_keys: [] } }),
        Promise.resolve({ success: true, data: { modules: [] } }),
      ]);

      if (keysRes.success && keysRes.data) {
        setApiKeys((keysRes.data as any)?.api_keys || []);
      }

      if (modulesRes.success && modulesRes.data) {
        setModules((modulesRes.data as any)?.modules || []);
      }
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Error loading developer portal data:', _error);
      toast.error('Failed to load developer portal data');
    } finally {
      setLoading(false);
    }
  }, []);

  // Load data on mount
  useEffect(() => {
    loadData();
  }, [loadData]);

  // Handle API key revocation
  const handleRevokeApiKey = useCallback(async (keyId: string, keyName: string) => {
    const reason = prompt(`Please provide a reason for revoking the API key "${keyName}":`);
    if (!reason) {return;}

    try {
      const apiClient = createAdminApiClient();
      const plansClient = createPlansClient(apiClient);
      const result = await plansClient.revokeApiKey(keyId, reason);
      if (result.success) {
        toast.success('API key revoked successfully');
        loadData(); // Refresh the data
      } else {
        throw new Error((result.error as any)?.message || 'Failed to revoke API key');
      }
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Error revoking API key:', _error);
      toast.error(_error instanceof Error ? _error.message : 'Failed to revoke API key');
    }
  }, [loadData]);

  // Handle API key creation navigation
  const handleCreateApiKey = useCallback(() => {
    router.push('/developer-portal/api-keys/create');
  }, [router]);

  // Handle key visibility toggle
  const handleToggleKeyVisibility = useCallback((keyId: string) => {
    setShowKeyValue(prev => prev === keyId ? null : keyId);
  }, []);

  // Copy to clipboard utility
  const copyToClipboard = useCallback(async (text: string, label: string) => {
    try {
      await navigator.clipboard.writeText(text);
      toast.success(`${label} copied to clipboard`);
    } catch {
      toast.error('Failed to copy to clipboard');
    }
  }, []);

  // Permission check
  if (!canManageApiKeys) {
    return (
      <div className="flex items-center justify-center min-h-[60vh]">
        <div className="text-center max-w-md">
          <Shield className="w-16 h-16 text-gray-400 mx-auto mb-4" />
          <h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-2">
            Access Denied
          </h2>
          <p className="text-gray-600 dark:text-gray-300 mb-4">
            You don't have permission to access the developer portal. This
            feature requires admin-level access.
          </p>
        </div>
      </div>
    );
  }

  // Loading state
  if (loading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="rounded-full h-8 w-8 border-b-2 border-blue-600 opacity-75"></div>
        <span className="ml-2 text-gray-600 dark:text-gray-300">Loading developer portal...</span>
      </div>
    );
  }

  return (
    <div className="p-6 max-w-7xl mx-auto">
      {/* Header */}
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
          className={`px-4 py-2 rounded-md font-medium ${
            activeTab === 'overview'
              ? 'bg-white dark:bg-gray-700 text-blue-600 dark:text-blue-400 shadow-sm'
              : 'text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-gray-100'
          }`}
        >
          <BarChart3 className="w-4 h-4 inline mr-2" />
          Overview
        </button>
        <button
          onClick={() => setActiveTab('keys')}
          className={`px-4 py-2 rounded-md font-medium ${
            activeTab === 'keys'
              ? 'bg-white dark:bg-gray-700 text-blue-600 dark:text-blue-400 shadow-sm'
              : 'text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-gray-100'
          }`}
        >
          <Key className="w-4 h-4 inline mr-2" />
          API Keys
        </button>
        <button
          onClick={() => setActiveTab('docs')}
          className={`px-4 py-2 rounded-md font-medium ${
            activeTab === 'docs'
              ? 'bg-white dark:bg-gray-700 text-blue-600 dark:text-blue-400 shadow-sm'
              : 'text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-gray-100'
          }`}
        >
          <BookOpen className="w-4 h-4 inline mr-2" />
          Documentation
        </button>
        <button
          onClick={() => setActiveTab('usage')}
          className={`px-4 py-2 rounded-md font-medium ${
            activeTab === 'usage'
              ? 'bg-white dark:bg-gray-700 text-blue-600 dark:text-blue-400 shadow-sm'
              : 'text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-gray-100'
          }`}
        >
          <Activity className="w-4 h-4 inline mr-2" />
          Usage
        </button>
      </div>

      {/* Tab Content */}
      {activeTab === 'overview' && (
        <DeveloperOverviewTab 
          apiKeys={apiKeys as any} 
          modules={modules as any} 
        />
      )}

      {activeTab === 'keys' && (
        <ApiKeyManagementTab
          apiKeys={apiKeys as any}
          showKeyValue={showKeyValue}
          onToggleKeyVisibility={handleToggleKeyVisibility}
          onRevokeApiKey={handleRevokeApiKey}
          onCreateApiKey={handleCreateApiKey}
          onCopyToClipboard={copyToClipboard}
        />
      )}

      {activeTab === 'docs' && (
        <DocumentationTab
          modules={modules as any}
          onCopyToClipboard={copyToClipboard}
        />
      )}

      {activeTab === 'usage' && (
        <UsageAnalyticsTab
          apiKeys={apiKeys as any}
        />
      )}
    </div>
  );
}

export default memo(DeveloperPortalRefactored);