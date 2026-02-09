import {
  Activity,
  BarChart3,
  BookOpen,
  Key,
  Shield
} from 'lucide-react';
import React, { useEffect, useState } from 'react';
import { toast } from 'sonner';

import { useSharedAuth } from '@/shared/components/auth/Provider';
import { copyToClipboard as copyToClipboardUtil } from '@/shared/utils';

import { ApiKeyManager } from './developer-portal/api-key-manager';
import { DocumentationViewer } from './developer-portal/documentation-viewer';
import { useDeveloperPortalData, useDeveloperPortalParams } from './developer-portal/hooks';
import { NewApiKeyModal } from './developer-portal/new-api-key-modal';
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
  const [newApiKey, setNewApiKey] = useState<string | null>(null);

  const {
    apiKeys,
    modules,
    availablePlans,
    loading,
    accessDenied,
    loadData,
    handleRevokeApiKey,
  } = useDeveloperPortalData();

  // Load initial data
  useEffect(() => {
    void loadData();
  }, [loadData]);

  // Handle URL parameters
  useDeveloperPortalParams(setNewApiKey);

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
          {accessDenied.code ? (
            <p className="text-sm text-gray-500 dark:text-gray-400">
              Error code: {accessDenied.code}
            </p>
          ) : null}
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
        <NewApiKeyModal
          apiKey={newApiKey}
          onClose={() => setNewApiKey(null)}
          onCopy={copyToClipboard}
        />
      )}
    </div>
  );
};
