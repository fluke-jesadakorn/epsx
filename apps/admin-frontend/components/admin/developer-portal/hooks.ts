'use client';

import { logger } from '@/lib/logger';
import { createPlansClient, type ApiKeyResponse, type Module, type Plan } from '@/shared/api/plans';
import { createAdminApiClient } from '@/shared/utils';
import React, { useCallback, useEffect, useState } from 'react';
import { toast } from 'sonner';

interface UseDeveloperPortalDataReturn {
    apiKeys: ApiKeyResponse[];
    modules: Module[];
    availablePlans: Plan[];
    loading: boolean;
    accessDenied: { message: string; code?: string } | null;
    loadData: () => Promise<void>;
    setApiKeys: React.Dispatch<React.SetStateAction<ApiKeyResponse[]>>;
    handleRevokeApiKey: (keyId: string, keyName: string) => Promise<void>;
}

export const useDeveloperPortalData = (): UseDeveloperPortalDataReturn => {
    const [apiKeys, setApiKeys] = useState<ApiKeyResponse[]>([]);
    const [modules, setModules] = useState<Module[]>([]);
    const [availablePlans, setAvailablePlans] = useState<Plan[]>([]);
    const [loading, setLoading] = useState(true);
    const [accessDenied, setAccessDenied] = useState<{ message: string; code?: string } | null>(null);

    const loadData = useCallback(async () => {
        try {
            setLoading(true);
            setAccessDenied(null);
            const apiClient = createAdminApiClient();
            const plansClient = createPlansClient(apiClient);

            // Load API Keys using PlansApi (Admin)
            try {
                const keysRes = await plansClient.listApiKeys();
                if (keysRes.success && keysRes.data) {
                    setApiKeys(keysRes.data.api_keys);
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
                    setModules(modulesRes.data.modules);
                }
            } catch (error) {
                logger.warn('Failed to load modules', { error });
                setModules([]);
            }

            // Load Available Plans
            try {
                const plansRes = await plansClient.listPlans({ is_active: true });
                if (plansRes.success && plansRes.data) {
                    setAvailablePlans(plansRes.data.data);
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
    }, []);

    const handleRevokeApiKey = useCallback(async (keyId: string, keyName: string) => {
        // eslint-disable-next-line no-alert
        const reason = window.confirm(
            `Are you sure you want to revoke the API key for "${keyName}"?`
        ) ? 'Revoked by admin' : null;

        if (reason === null) {
            return;
        }

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
        } catch (error) {
            logger.error('Failed to revoke API key', { keyId, error });
            toast.error('Failed to revoke API key');
        }
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

            if (clientNameValue) {
                toast.success(`API key for "${clientNameValue}" created successfully!`);
            }

            if (newKeyValue && newKeyValue !== 'key-created') {
                setNewApiKey(newKeyValue);
            }

            // Clean up URL
            window.history.replaceState({}, '', window.location.pathname);
        }
    }, [setNewApiKey]);
};
