'use client';

import { logger } from '@/lib/logger';
import { createPlansClient, type ApiKeyResponse, type Module, type Plan } from '@/shared/api/plans';
import { createAdminApiClient } from '@/shared/utils';
import type React from 'react';
import { useCallback, useEffect, useState } from 'react';
import { toast } from 'sonner';

interface UseDeveloperPortalDataReturn {
    apiKeys: ApiKeyResponse[];
    modules: Module[];
    availablePlans: Plan[];
    loading: boolean;
    accessDenied: { message: string; code?: string } | null;
    loadData: () => Promise<void>;
    setApiKeys: React.Dispatch<React.SetStateAction<ApiKeyResponse[]>>;
    handleRevokeApiKey: (keyId: string, keyName: string) => void;
}

export const useDeveloperPortalData = (): UseDeveloperPortalDataReturn => {
    const [apiKeys, setApiKeys] = useState<ApiKeyResponse[]>([]);
    const [modules, setModules] = useState<Module[]>([]);
    const [availablePlans, setAvailablePlans] = useState<Plan[]>([]);
    const [loading, setLoading] = useState(true);
    const [accessDenied, setAccessDenied] = useState<{ message: string; code?: string } | null>(null);

    const handleAccessDenied = useCallback((err: { status?: number; code?: string; message?: string }) => {
        setAccessDenied({
            message: err.message ?? "You don't have permission to access the developer portal.",
            code: err.code
        });
    }, []);

    const isAccessDenied = (err: { status?: number; code?: string }) =>
        err.status === 403 || err.code === 'PERMISSION_DENIED';

    const loadApiKeys = async (plansClient: ReturnType<typeof createPlansClient>): Promise<boolean> => {
        try {
            const keysRes = await plansClient.listApiKeys();
            if (keysRes.success === true) {
                setApiKeys(keysRes.data.api_keys);
            }
            return true;
        } catch (error) {
            const err = error as { status?: number; code?: string; message?: string };
            if (isAccessDenied(err)) {
                handleAccessDenied(err);
                return false;
            }
            if (err.status !== 404) {
                logger.warn('Failed to load API keys', { error });
            }
            setApiKeys([]);
            return true;
        }
    };

    const loadModules = async (plansClient: ReturnType<typeof createPlansClient>) => {
        try {
            const modulesRes = await plansClient.getModules();
            if (modulesRes.success === true) {
                setModules(modulesRes.data.modules);
            }
        } catch (error) {
            logger.warn('Failed to load modules', { error });
            setModules([]);
        }
    };

    const loadPlans = async (plansClient: ReturnType<typeof createPlansClient>) => {
        try {
            const plansRes = await plansClient.listPlans({ is_active: true });
            if (plansRes.success === true) {
                setAvailablePlans(plansRes.data.data);
            }
        } catch (error) {
            logger.warn('Failed to load available plans', { error });
        }
    };

    const loadData = useCallback(async () => {
        try {
            setLoading(true);
            setAccessDenied(null);
            const apiClient = createAdminApiClient();
            const plansClient = createPlansClient(apiClient);

            const keysOk = await loadApiKeys(plansClient);
            if (!keysOk) { return; }
            await loadModules(plansClient);
            await loadPlans(plansClient);
        } catch (error) {
            const err = error as { status?: number; code?: string; message?: string };
            if (isAccessDenied(err)) {
                handleAccessDenied(err);
                return;
            }
            logger.error('Failed to load developer portal data', { error });
        } finally {
            setLoading(false);
        }
    // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [handleAccessDenied]);

    const handleRevokeApiKey = useCallback((keyId: string, keyName: string) => {
        const doRevoke = async () => {
            try {
                const apiClient = createAdminApiClient();
                const plansClient = createPlansClient(apiClient);
                const response = await plansClient.revokeApiKey(keyId);
                if (response.success === true) {
                    toast.success('API key revoked successfully');
                    void loadData();
                } else {
                    toast.error('Failed to revoke API key');
                }
            } catch (error) {
                logger.error('Failed to revoke API key', { keyId, error });
                toast.error('Failed to revoke API key');
            }
        };

        toast(`Revoke API key for "${keyName}"?`, {
            action: {
                label: 'Revoke',
                onClick: () => { void doRevoke(); }
            },
        });
    }, [loadData]);

    return {
        apiKeys,
        modules,
        availablePlans,
        loading,
        accessDenied,
        loadData,
        setApiKeys,
        handleRevokeApiKey,
    };
};

export const useDeveloperPortalParams = (setNewApiKey: (key: string | null) => void) => {
    useEffect(() => {
        // Check for URL parameters (success message, new API key)
        const urlParams = new URLSearchParams(window.location.search);
        if (urlParams.get('success') === 'true') {
            const clientNameValue = urlParams.get('client_name');
            const newKeyValue = urlParams.get('new_key');

            if (clientNameValue !== null && clientNameValue !== '') {
                toast.success(`API key for "${clientNameValue}" created successfully!`);
            }

            if (newKeyValue !== null && newKeyValue !== '' && newKeyValue !== 'key-created') {
                setNewApiKey(newKeyValue);
            }

            // Clean up URL
            window.history.replaceState({}, '', window.location.pathname);
        }
    }, [setNewApiKey]);
};
