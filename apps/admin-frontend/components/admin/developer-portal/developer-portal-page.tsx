'use client';

import { Shield } from 'lucide-react';
import { useRouter } from 'next/navigation';
import React, { useCallback, useEffect, useState } from 'react';
import { toast } from 'sonner';

import { PageHeader, PageTabs } from '@/components/shared';

import { EditExpirationModal } from './modals/edit-expiration-modal';
import { RevokeKeyModal } from './modals/revoke-key-modal';
import { ApiKeysTab } from './tabs/api-keys-tab';
import { DocumentationTab } from './tabs/documentation-tab';
import { OverviewTab } from './tabs/overview-tab';
import { UsageAnalyticsTab } from './tabs/usage-analytics-tab';

import { logger } from '@/lib/logger';
import { createPlansClient, type ApiKeyResponse, type Module } from '@/shared/api/plans';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import { copyToClipboard, createAdminApiClient } from '@/shared/utils';

type TabType = 'overview' | 'keys' | 'docs' | 'usage';

/**
 * Main Developer Portal page component for managing user API keys
 */
export const DeveloperPortalPage: React.FC = () => {
    const router = useRouter();
    const { isLoading: authLoading } = useSharedAuth();

    // Data state
    const [apiKeys, setApiKeys] = useState<ApiKeyResponse[]>([]);
    const [modules, setModules] = useState<Module[]>([]);
    const [loading, setLoading] = useState(true);
    const [accessDenied, setAccessDenied] = useState<{ message: string; code?: string } | null>(null);

    // UI state
    const [activeTab, setActiveTab] = useState<TabType>('overview');
    const [newApiKey, setNewApiKey] = useState<string | null>(null);

    // Modal state
    const [revokeModal, setRevokeModal] = useState<{
        isOpen: boolean;
        apiKey: ApiKeyResponse | null;
        isLoading: boolean;
    }>({ isOpen: false, apiKey: null, isLoading: false });

    const [expirationModal, setExpirationModal] = useState<{
        isOpen: boolean;
        apiKey: ApiKeyResponse | null;
        isLoading: boolean;
    }>({ isOpen: false, apiKey: null, isLoading: false });

    // Load data
    const loadData = useCallback(async () => {
        try {
            setLoading(true);
            setAccessDenied(null);

            const apiClient = createAdminApiClient();
            const plansClient = createPlansClient(apiClient);

            // Load API keys
            try {
                const keysRes = await plansClient.listApiKeys();
                if (keysRes.success) {
                    setApiKeys((keysRes.data as any)?.api_keys ?? []);
                }
            } catch (error: any) {
                if (error?.status === 403 ?? error?.code === 'PERMISSION_DENIED') {
                    setAccessDenied({
                        message: error?.message ?? "You don't have permission to access the developer portal.",
                        code: error?.code,
                    });
                    return;
                }
                if (error?.status !== 404) {
                    logger.warn('Failed to load API keys', { error });
                }
                setApiKeys([]);
            }

            // Load modules
            try {
                const modulesRes = await plansClient.getModules({ status: 'active' });
                if (modulesRes.success) {
                    setModules((modulesRes.data as any)?.modules ?? []);
                }
            } catch (error: any) {
                if (error?.status === 403 ?? error?.code === 'PERMISSION_DENIED') {
                    setAccessDenied({
                        message: error?.message ?? "You don't have permission to access the developer portal.",
                        code: error?.code,
                    });
                    return;
                }
                if (error?.status !== 404) {
                    logger.warn('Failed to load modules', { error });
                }
                setModules([]);
            }
        } catch (error: any) {
            if (error?.status === 403 ?? error?.code === 'PERMISSION_DENIED') {
                setAccessDenied({
                    message: error?.message ?? "You don't have permission to access the developer portal.",
                    code: error?.code,
                });
                return;
            }
            logger.error('Failed to load developer portal data', { error });
        } finally {
            setLoading(false);
        }
    }, []);

    useEffect(() => {
        loadData();

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
    }, [loadData]);

    // Handlers
    const handleCopyToClipboard = async (text: string, label: string) => {
        const success = await copyToClipboard(text);
        if (success) {
            toast.success(`${label} copied to clipboard`);
        } else {
            toast.error('Failed to copy to clipboard');
        }
    };

    const handleCreateKey = () => {
        router.push('/developer-portal/api-keys/create');
    };

    const handleRevoke = useCallback(
        async (keyId: string, reason: string) => {
            setRevokeModal(prev => ({ ...prev, isLoading: true }));
            try {
                const apiClient = createAdminApiClient();
                const plansClient = createPlansClient(apiClient);
                const response = await plansClient.revokeApiKey(keyId, reason);

                if (response.success) {
                    toast.success('API key revoked successfully');
                    loadData();
                } else {
                    throw new Error('Failed to revoke API key');
                }
            } catch (error) {
                logger.error('Failed to revoke API key', { keyId, error });
                toast.error('Failed to revoke API key');
                throw error;
            } finally {
                setRevokeModal(prev => ({ ...prev, isLoading: false }));
            }
        },
        [loadData]
    );

    const handleUpdateExpiration = useCallback(
        async (keyId: string, expiresAt: string | null) => {
            setExpirationModal(prev => ({ ...prev, isLoading: true }));
            try {
                const apiClient = createAdminApiClient();
                const plansClient = createPlansClient(apiClient);
                const response = await plansClient.updateApiKeyExpiration(keyId, expiresAt);

                if (response.success) {
                    toast.success('Expiration updated successfully');
                    loadData();
                } else {
                    throw new Error('Failed to update expiration');
                }
            } catch (error) {
                logger.error('Failed to update API key expiration', { keyId, error });
                toast.error('Failed to update expiration');
                throw error;
            } finally {
                setExpirationModal(prev => ({ ...prev, isLoading: false }));
            }
        },
        [loadData]
    );

    // Loading state
    if (authLoading) {
        return (
            <div className="flex items-center justify-center p-8">
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
                <span className="ml-2 text-gray-600 dark:text-gray-300">Initializing...</span>
            </div>
        );
    }

    // Access denied state
    if (accessDenied) {
        return (
            <div className="flex items-center justify-center min-h-[60vh]">
                <div className="relative overflow-hidden rounded-[32px] bg-slate-900/40 backdrop-blur-2xl border border-white/5 p-12 shadow-xl text-center max-w-md">
                    <div className="inline-flex p-4 bg-red-500/10 rounded-[24px] border border-red-500/10 text-red-500 mb-6 font-bold">
                        <Shield className="w-12 h-12" />
                    </div>
                    <h2 className="text-2xl font-black text-foreground uppercase tracking-tight mb-2">
                        Access Denied
                    </h2>
                    <p className="text-muted-foreground font-bold mb-6">{accessDenied.message}</p>
                    {accessDenied.code && (
                        <div className="px-4 py-2 bg-white/5 rounded-xl border border-white/5 text-xs font-mono text-muted-foreground">
                            Error: {accessDenied.code}
                        </div>
                    )}
                </div>
            </div>
        );
    }

    // Loading data state
    if (loading) {
        return (
            <div className="flex flex-col items-center justify-center p-20 text-center">
                <div className="relative w-16 h-16 mb-6">
                    <div className="absolute inset-0 border-4 border-[#1fc7d4]/20 rounded-full" />
                    <div className="absolute inset-0 border-4 border-[#1fc7d4] border-t-transparent rounded-full animate-spin" />
                </div>
                <h3 className="text-xl font-black text-foreground uppercase tracking-tight mb-2">Loading</h3>
                <p className="text-muted-foreground font-bold">Preparing developer portal...</p>
            </div>
        );
    }

    return (
        <>
            {/* Header */}
            <PageHeader
                title="Developer Portal"
                subtitle="Manage API keys, documentation, and third-party integrations"
                icon="Code"
                gradient="warning"
                centered={true}
            />

            {/* Tab Navigation */}
            <PageTabs
                tabs={[
                    { id: 'overview', label: 'Overview', prefix: '📊' },
                    { id: 'keys', label: 'API Keys', prefix: '🔑' },
                    { id: 'docs', label: 'Documentation', prefix: '📚' },
                    { id: 'usage', label: 'Usage Analytics', prefix: '📈' },
                ]}
                activeTab={activeTab}
                onTabChange={(id) => setActiveTab(id as TabType)}
                className="mb-8"
            />

            {/* Tab Content */}
            {activeTab === 'overview' && (
                <OverviewTab apiKeys={apiKeys} modules={modules} onCreateKey={handleCreateKey} />
            )}

            {activeTab === 'keys' && (
                <ApiKeysTab
                    apiKeys={apiKeys}
                    onCopyWallet={(wallet) => handleCopyToClipboard(wallet, 'Wallet address')}
                    onCopyKeyPrefix={(prefix) => handleCopyToClipboard(prefix, 'API Key prefix')}
                    onRevoke={(apiKey) =>
                        setRevokeModal({
                            isOpen: true,
                            apiKey,
                            isLoading: false,
                        })
                    }
                    onEditExpiration={(apiKey) =>
                        setExpirationModal({
                            isOpen: true,
                            apiKey,
                            isLoading: false,
                        })
                    }
                    onCreateKey={handleCreateKey}
                />
            )}

            {activeTab === 'docs' && <DocumentationTab modules={modules} />}

            {activeTab === 'usage' && <UsageAnalyticsTab apiKeys={apiKeys} />}

            {/* Revoke Modal */}
            {revokeModal.apiKey && (
                <RevokeKeyModal
                    isOpen={revokeModal.isOpen}
                    onClose={() => setRevokeModal({ isOpen: false, apiKey: null, isLoading: false })}
                    apiKey={{
                        id: revokeModal.apiKey.id,
                        client_name: revokeModal.apiKey.client_name,
                        wallet_address: (revokeModal.apiKey as any).wallet_address,
                    }}
                    onRevoke={handleRevoke}
                    isLoading={revokeModal.isLoading}
                />
            )}

            {/* Edit Expiration Modal */}
            {expirationModal.apiKey && (
                <EditExpirationModal
                    isOpen={expirationModal.isOpen}
                    onClose={() => setExpirationModal({ isOpen: false, apiKey: null, isLoading: false })}
                    apiKey={{
                        id: expirationModal.apiKey.id,
                        client_name: expirationModal.apiKey.client_name,
                        expires_at: expirationModal.apiKey.expires_at,
                    }}
                    onUpdate={handleUpdateExpiration}
                    isLoading={expirationModal.isLoading}
                />
            )}

            {/* New API Key Display Modal */}
            {newApiKey && (
                <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
                    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-md mx-4">
                        <div className="p-6 border-b border-gray-200 dark:border-gray-700">
                            <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                                API Key Created
                            </h2>
                        </div>

                        <div className="p-6">
                            <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4 mb-4">
                                <div className="flex items-start">
                                    <div className="text-yellow-600 dark:text-yellow-400 mt-0.5 mr-2">⚠️</div>
                                    <div>
                                        <h3 className="font-medium text-yellow-800 dark:text-yellow-200">Important</h3>
                                        <p className="text-sm text-yellow-700 dark:text-yellow-300 mt-1">
                                            This is the only time you&apos;ll see your API key. Please copy it and store
                                            it securely.
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
                                        onClick={() => handleCopyToClipboard(newApiKey, 'API Key')}
                                        className="p-1 text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300"
                                    >
                                        📋
                                    </button>
                                </div>
                                <code className="block text-sm font-mono text-gray-900 dark:text-gray-100 bg-white dark:bg-gray-800 p-3 rounded border border-gray-200 dark:border-gray-600 break-all">
                                    {newApiKey}
                                </code>
                            </div>
                        </div>

                        <div className="p-6 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-700/50 flex justify-end rounded-b-lg">
                            <button
                                onClick={() => setNewApiKey(null)}
                                className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                            >
                                I&apos;ve Saved the Key
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </>
    );
};
