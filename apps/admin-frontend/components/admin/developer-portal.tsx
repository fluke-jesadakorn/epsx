"use client";

import {
  Activity,
  BarChart3,
  BookOpen,
  Key,
  Shield
} from 'lucide-react';
import React, { useEffect, useState } from 'react';
import { toast } from 'sonner';

import { useSharedAuth } from '@/shared/components/auth';
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
export const DeveloperPortalPage: React.FC = () => {
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
        <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-[#1fc7d4]" />
        <span className="ml-2 text-muted-foreground text-sm">Initializing...</span>
      </div>
    );
  }

  if (accessDenied) {
    return (
      <div className="flex items-center justify-center min-h-[60vh]">
        <div className="text-center max-w-md">
          <Shield className="w-12 h-12 text-muted-foreground/30 mx-auto mb-4" />
          <h2 className="text-lg font-semibold text-foreground mb-2">Access Denied</h2>
          <p className="text-muted-foreground mb-4">{accessDenied.message}</p>
          {accessDenied.code ? (
            <p className="text-xs text-muted-foreground/60">Error code: {accessDenied.code}</p>
          ) : null}
        </div>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-[#1fc7d4]" />
        <span className="ml-2 text-muted-foreground text-sm">Loading developer portal...</span>
      </div>
    );
  }

  return (
    <div className="max-w-7xl mx-auto space-y-6">
      {/* Tab Navigation */}
      <div className="flex gap-1 p-1 bg-muted/50 border border-border/40 rounded-xl w-fit">
        <button
          onClick={() => setActiveTab('overview')}
          className={`px-4 py-2 rounded-lg text-sm font-semibold transition-colors flex items-center gap-2 ${activeTab === 'overview'
            ? 'bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white shadow-sm'
            : 'text-muted-foreground hover:text-foreground'
            }`}
        >
          <BarChart3 className="w-3.5 h-3.5" />
          Overview
        </button>
        <button
          onClick={() => setActiveTab('keys')}
          className={`px-4 py-2 rounded-lg text-sm font-semibold transition-colors flex items-center gap-2 ${activeTab === 'keys'
            ? 'bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white shadow-sm'
            : 'text-muted-foreground hover:text-foreground'
            }`}
        >
          <Key className="w-3.5 h-3.5" />
          API Keys
        </button>
        <button
          onClick={() => setActiveTab('docs')}
          className={`px-4 py-2 rounded-lg text-sm font-semibold transition-colors flex items-center gap-2 ${activeTab === 'docs'
            ? 'bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white shadow-sm'
            : 'text-muted-foreground hover:text-foreground'
            }`}
        >
          <BookOpen className="w-3.5 h-3.5" />
          Documentation
        </button>
        <button
          onClick={() => setActiveTab('usage')}
          className={`px-4 py-2 rounded-lg text-sm font-semibold transition-colors flex items-center gap-2 ${activeTab === 'usage'
            ? 'bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white shadow-sm'
            : 'text-muted-foreground hover:text-foreground'
            }`}
        >
          <Activity className="w-3.5 h-3.5" />
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
