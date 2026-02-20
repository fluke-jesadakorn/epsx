'use client';

import { AlertCircle, Copy, Key, Shield } from 'lucide-react';
import { useRouter, useSearchParams } from 'next/navigation';
import React, { useCallback, useEffect, useState } from 'react';
import { toast } from 'sonner';

import { PageHeader } from '@/components/shared';
import { EditExpirationModal } from './modals/edit-expiration-modal';
import { RevokeKeyModal } from './modals/revoke-key-modal';
import { ApiKeysTab } from './tabs/api-keys-tab';
import { DocumentationTab } from './tabs/documentation-tab';
import { OverviewTab } from './tabs/overview-tab';
import { UsageAnalyticsTab } from './tabs/usage-analytics-tab';

import { logger } from '@/lib/logger';
import { createPlansClient, type ApiKeyResponse, type Module } from '@/shared/api/plans';
import { useSharedAuth } from '@/shared/components/auth';
import { isApiError, type ApiResponse } from '@/shared/types/api';
import { copyToClipboard, createAdminApiClient } from '@/shared/utils';

type TabType = 'overview' | 'keys' | 'docs' | 'usage';

interface AccessDeniedState {
    message: string;
    code?: string;
}

interface ModalState {
    isOpen: boolean;
    apiKey: ApiKeyResponse | null;
    isLoading: boolean;
}

/**
 * Access Denied Display Component
 */
const AccessDeniedView: React.FC<{ accessDenied: AccessDeniedState }> = ({ accessDenied }) => (
    <div className="flex items-center justify-center min-h-[60vh]">
        <div className="relative overflow-hidden rounded-[32px] bg-white dark:bg-card backdrop-blur-2xl border border-gray-200 dark:border-border p-12 shadow-xl text-center max-w-md">
            <div className="inline-flex p-4 bg-red-500/10 rounded-[24px] border border-red-500/10 text-red-500 mb-6 font-bold">
                <Shield className="w-12 h-12" />
            </div>
            <h2 className="text-2xl font-black text-foreground uppercase tracking-tight mb-2">
                Access Denied
            </h2>
            <p className="text-muted-foreground font-bold mb-6">{accessDenied.message}</p>
            {accessDenied.code !== undefined && (
                <div className="px-4 py-2 bg-white dark:bg-white/[0.04] rounded-xl border border-gray-200 dark:border-border text-xs font-mono text-muted-foreground">
                    Error: {accessDenied.code}
                </div>
            )}
        </div>
    </div>
);

/**
 * Loading State Display Component
 */
const LoadingView: React.FC = () => (
    <div className="flex flex-col items-center justify-center p-20 text-center">
        <div className="relative w-16 h-16 mb-6">
            <div className="absolute inset-0 border-4 border-[#1fc7d4]/20 rounded-full" />
            <div className="absolute inset-0 border-4 border-[#1fc7d4] border-t-transparent rounded-full animate-spin" />
        </div>
        <h3 className="text-xl font-black text-foreground uppercase tracking-tight mb-2">Loading</h3>
        <p className="text-muted-foreground font-bold">Preparing developer portal...</p>
    </div>
);

const DeveloperPortalContent = ({
    activeTab, apiKeys, modules, onCopyToClipboard, onRevokeKey, onEditExpiration, onCreateKey
}: {
    activeTab: TabType;
    apiKeys: ApiKeyResponse[];
    modules: Module[];
    onCopyToClipboard: (text: string, label: string) => void;
    onRevokeKey: (key: ApiKeyResponse) => void;
    onEditExpiration: (key: ApiKeyResponse) => void;
    onCreateKey: () => void;
}) => {
    switch (activeTab) {
        case 'overview': {
            return <OverviewTab apiKeys={apiKeys} modules={modules} onCreateKey={onCreateKey} />;
        }
        case 'keys': {
            return (
                <ApiKeysTab
                    apiKeys={apiKeys}
                    onCopyWallet={(wallet) => onCopyToClipboard(wallet, 'Wallet address')}
                    onCopyKeyPrefix={(prefix) => onCopyToClipboard(prefix, 'API Key prefix')}
                    onRevoke={onRevokeKey}
                    onEditExpiration={onEditExpiration}
                    onCreateKey={onCreateKey}
                />
            );
        }
        case 'docs': {
            return <DocumentationTab modules={modules} />;
        }
        case 'usage': {
            return <UsageAnalyticsTab apiKeys={apiKeys} />;
        }
        default: {
            return null;
        }
    }
};

const DeveloperPortalModals = ({
    revokeModal, expirationModal, newApiKey, onRevokeModalClose, onRevoke,
    onExpirationModalClose, onUpdateExpiration, onNewApiKeyClose, onCopyToClipboard
}: {
    revokeModal: ModalState;
    expirationModal: ModalState;
    newApiKey: string | null;
    onRevokeModalClose: () => void;
    onRevoke: (id: string, reason: string) => Promise<void>;
    onExpirationModalClose: () => void;
    onUpdateExpiration: (id: string, exp: string | null) => Promise<void>;
    onNewApiKeyClose: () => void;
    onCopyToClipboard: (text: string, label: string) => void;
}) => (
    <>
        {revokeModal.apiKey !== null && (
            <RevokeKeyModal
                isOpen={revokeModal.isOpen}
                onClose={onRevokeModalClose}
                apiKey={{
                    id: revokeModal.apiKey.id,
                    client_name: revokeModal.apiKey.client_name,
                    wallet_address: (revokeModal.apiKey as { wallet_address?: string }).wallet_address ?? '',
                }}
                onRevoke={onRevoke}
                isLoading={revokeModal.isLoading}
            />
        )}
        {expirationModal.apiKey !== null && (
            <EditExpirationModal
                isOpen={expirationModal.isOpen}
                onClose={onExpirationModalClose}
                apiKey={{
                    id: expirationModal.apiKey.id,
                    client_name: expirationModal.apiKey.client_name,
                    expires_at: expirationModal.apiKey.expires_at ?? undefined,
                }}
                onUpdate={onUpdateExpiration}
                isLoading={expirationModal.isLoading}
            />
        )}
        {newApiKey !== null && (
            <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
                <div className="bg-white dark:bg-gray-800 rounded-2xl shadow-2xl w-full max-w-md overflow-hidden animate-in zoom-in-95 duration-200">
                    <div className="p-6 border-b border-gray-100 dark:border-gray-700">
                        <h2 className="text-xl font-black text-gray-900 dark:text-gray-100 uppercase tracking-tight flex items-center gap-2">
                            <Key className="w-5 h-5 text-amber-500" />
                            API Key Created
                        </h2>
                    </div>
                    <div className="p-6 space-y-6">
                        <div className="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-xl p-4 flex gap-3">
                            <AlertCircle className="w-5 h-5 text-amber-600 dark:text-amber-400 shrink-0" />
                            <p className="text-sm text-amber-800 dark:text-amber-200 font-medium">
                                Save this key now! This is the only time it will be shown. Keep it secure.
                            </p>
                        </div>
                        <div className="space-y-2">
                            <label className="text-xs font-black uppercase text-gray-500 tracking-wider">Your API Key</label>
                            <div className="relative group">
                                <code className="block w-full p-4 pr-12 bg-gray-50 dark:bg-card rounded-xl border border-gray-200 dark:border-gray-700 font-mono text-sm break-all">
                                    {newApiKey}
                                </code>
                                <button
                                    onClick={() => {
                                        onCopyToClipboard(newApiKey, 'API Key');
                                    }}
                                    className="absolute right-3 top-3 p-2 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg hover:border-primary transition-all shadow-sm"
                                >
                                    <Copy className="w-4 h-4" />
                                </button>
                            </div>
                        </div>
                    </div>
                    <div className="p-6 bg-gray-50 dark:bg-card/50 flex justify-end">
                        <button
                            onClick={onNewApiKeyClose}
                            className="px-6 py-2 bg-primary text-primary-foreground rounded-xl font-bold hover:opacity-90 transition-all shadow-lg shadow-primary/20"
                        >
                            I&apos;ve Saved the Key
                        </button>
                    </div>
                </div>
            </div>
        )}
    </>
);

function useDeveloperPortalData(setAccessDenied: (s: AccessDeniedState | null) => void) {
    const [apiKeys, setApiKeys] = useState<ApiKeyResponse[]>([]);
    const [modules, setModules] = useState<Module[]>([]);
    const [loading, setLoading] = useState(true);

    const handleApiResponse = useCallback(<T,>(res: ApiResponse<T>) => {
        if (res.success && res.data !== null) {
            return res.data;
        }
        if (res.error !== null && (res.error.code === 'PERMISSION_DENIED' || res.error.status === 403)) {
            setAccessDenied({ message: res.error.message, code: res.error.code });
        }
        return null;
    }, [setAccessDenied]);

    const loadData = useCallback(async () => {
        try {
            setLoading(true);
            setAccessDenied(null);
            const apiClient = createAdminApiClient();
            const plansClient = createPlansClient(apiClient);
            const [keysRes, modulesRes] = await Promise.all([
                plansClient.listApiKeys(),
                plansClient.getModules({ status: 'active' })
            ]);
            const keysData = handleApiResponse(keysRes);
            if (keysData !== null && typeof keysData === 'object' && 'api_keys' in keysData) {
                setApiKeys((keysData as { api_keys: ApiKeyResponse[] }).api_keys);
            }
            const modulesData = handleApiResponse(modulesRes);
            if (modulesData !== null && typeof modulesData === 'object' && 'modules' in modulesData) {
                setModules((modulesData as { modules: Module[] }).modules);
            }
        } catch (error) {
            if (isApiError(error) && (error.status === 403 || error.code === 'PERMISSION_DENIED')) {
                setAccessDenied({ message: error.message, code: error.code });
            } else {
                logger.error('Failed to load developer portal data', { error });
            }
        } finally {
            setLoading(false);
        }
    }, [handleApiResponse, setAccessDenied]);

    return { apiKeys, modules, loading, loadData };
}

/**
 * Developer Portal Page
 */
export const DeveloperPortalPage: React.FC = () => {
    const router = useRouter();
    const searchParams = useSearchParams();
    const { isLoading: authLoading } = useSharedAuth();
    const [accessDenied, setAccessDenied] = useState<AccessDeniedState | null>(null);
    const { apiKeys, modules, loading, loadData } = useDeveloperPortalData(setAccessDenied);
    const activeTab = (searchParams.get('tab') ?? 'overview') as TabType;
    const [newApiKey, setNewApiKey] = useState<string | null>(null);
    const [revokeModal, setRevokeModal] = useState<ModalState>({ isOpen: false, apiKey: null, isLoading: false });
    const [expirationModal, setExpirationModal] = useState<ModalState>({ isOpen: false, apiKey: null, isLoading: false });

    useEffect(() => {
        void loadData();
        const urlParams = new URLSearchParams(window.location.search);
        if (urlParams.get('success') === 'true') {
            const clientName = urlParams.get('client_name');
            const newKey = urlParams.get('new_key');
            if (clientName !== null) { toast.success(`API key for "${clientName}" created successfully!`); }
            if (newKey !== null && newKey !== 'key-created') { setNewApiKey(newKey); }
            window.history.replaceState({}, '', window.location.pathname);
        }
    }, [loadData]);

    const handleCopy = useCallback((text: string, label: string) => {
        const handleAsync = async () => {
            const success = await copyToClipboard(text);
            if (success) { toast.success(`${label} copied to clipboard`); }
            else { toast.error('Failed to copy to clipboard'); }
        };
        void handleAsync();
    }, []);

    const handleRevoke = async (keyId: string, reason: string) => {
        setRevokeModal(prev => ({ ...prev, isLoading: true }));
        try {
            const plansClient = createPlansClient(createAdminApiClient());
            const response = await plansClient.revokeApiKey(keyId, reason);
            if (response.success) {
                toast.success('API key revoked successfully');
                void loadData();
            } else {
                toast.error(response.error?.message ?? 'Failed to revoke API key');
            }
        } catch (err) {
            logger.error('Failed to revoke API key', { keyId, error: err });
            toast.error('Failed to revoke API key');
        } finally {
            setRevokeModal(prev => ({ ...prev, isLoading: false }));
        }
    };

    const handleUpdateExpiration = async (keyId: string, exp: string | null) => {
        setExpirationModal(prev => ({ ...prev, isLoading: true }));
        try {
            const plansClient = createPlansClient(createAdminApiClient());
            const res = await plansClient.updateApiKeyExpiration(keyId, exp);
            if (res.success) {
                toast.success('Expiration updated successfully');
                void loadData();
            } else {
                toast.error(res.error?.message ?? 'Failed to update expiration');
            }
        } catch (err) {
            logger.error('Failed to update API key expiration', { keyId, error: err });
            toast.error('Failed to update expiration');
        } finally {
            setExpirationModal(prev => ({ ...prev, isLoading: false }));
        }
    };

    if (authLoading) { return <div className="p-8 text-center italic">Initializing...</div>; }
    if (accessDenied !== null) { return <AccessDeniedView accessDenied={accessDenied} />; }
    if (loading) { return <LoadingView />; }

    return (
        <div className="space-y-8 pb-20">
            <PageHeader title="Developer Portal" subtitle="Manage API keys, documentation, and third-party integrations" icon="Code" gradient="warning" centered={true} />
            <DeveloperPortalContent activeTab={activeTab} apiKeys={apiKeys} modules={modules} onCopyToClipboard={handleCopy} onRevokeKey={(apiKey) => setRevokeModal({ isOpen: true, apiKey, isLoading: false })} onEditExpiration={(apiKey) => setExpirationModal({ isOpen: true, apiKey, isLoading: false })} onCreateKey={() => router.push('/developer-portal/api-keys/create')} />
            <DeveloperPortalModals revokeModal={revokeModal} expirationModal={expirationModal} newApiKey={newApiKey} onRevokeModalClose={() => setRevokeModal({ isOpen: false, apiKey: null, isLoading: false })} onRevoke={handleRevoke} onExpirationModalClose={() => setExpirationModal({ isOpen: false, apiKey: null, isLoading: false })} onUpdateExpiration={handleUpdateExpiration} onNewApiKeyClose={() => setNewApiKey(null)} onCopyToClipboard={handleCopy} />
        </div>
    );
};
