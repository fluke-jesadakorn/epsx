'use client';

import {
    fetchWalletDetailAction,
    updateWalletMetadataAction
} from '@/app/wallet-management/plan-actions';
import type { WalletData } from '@/components/wallet/types';
import type { SubscriptionResponse } from '@/shared/api/plans';
import { createPlansClient, isApiSuccess } from '@/shared/api/plans';
import { createAdminApiClient } from '@/shared/utils/api-client';
import { logger } from '@/shared/utils/logger';
import { useRouter } from 'next/navigation';
import { useCallback, useEffect, useState } from 'react';
import { toast } from 'sonner';

interface UseWalletDataContext {
    walletAddress: string;
    router: ReturnType<typeof useRouter>;
}

export function useWalletData(ctx: UseWalletDataContext) {
    const [wallet, setWallet] = useState<WalletData | null>(null);
    const [isLoading, setIsLoading] = useState(true);
    const [isRefreshing, setIsRefreshing] = useState(false);

    const loadWallet = useCallback(async () => {
        if (ctx.walletAddress === '') { return; }

        try {
            setIsRefreshing(true);
            const data = await fetchWalletDetailAction(ctx.walletAddress);
            setWallet(data);
        } catch (_err) {
            logger.error('Failed to load wallet:', _err);
            toast.error('Failed to load wallet details');
            ctx.router.push('/wallet-management');
        } finally {
            setIsLoading(false);
            setIsRefreshing(false);
        }
    }, [ctx.walletAddress, ctx.router]);

    useEffect(() => {
        void loadWallet();
    }, [loadWallet]);

    return { wallet, setWallet, isLoading, isRefreshing, loadWallet };
}

export function useMetadataForm(wallet: WalletData | null) {
    const [metadataForm, setMetadataForm] = useState({ label: '', note: '' });
    const [hasChanges, setHasChanges] = useState(false);
    const [isSaving, setIsSaving] = useState(false);

    useEffect(() => {
        if (wallet !== null) {
            setMetadataForm({
                label: wallet.label ?? '',
                note: wallet.note ?? ''
            });
            setHasChanges(false);
        }
    }, [wallet]);

    const handleSave = async (walletAddress: string, onSuccess: () => Promise<void>) => {
        setIsSaving(true);
        try {
            await updateWalletMetadataAction(walletAddress, {
                label: metadataForm.label,
                note: metadataForm.note
            });
            toast.success('Wallet details updated');
            setHasChanges(false);
            await onSuccess();
        } catch (_err) {
            logger.error('Failed to update wallet:', _err);
            toast.error('Failed to update wallet details');
        } finally {
            setIsSaving(false);
        }
    };

    return { metadataForm, setMetadataForm, hasChanges, setHasChanges, isSaving, handleSave };
}

export function useSubscriptionData(walletAddress: string) {
    const [activeSub, setActiveSub] = useState<SubscriptionResponse | null>(null);
    const [isLoading, setIsLoading] = useState(false);

    const loadSubscription = useCallback(async () => {
        if (walletAddress === '') { return; }
        setIsLoading(true);
        try {
            const client = createPlansClient(createAdminApiClient());
            const res = await client.getSubscriptions({ limit: 100 });
            if (isApiSuccess(res)) {
                const sub = res.data.subscriptions.find((s: SubscriptionResponse) =>
                    s.user_id === walletAddress && s.status === 'active'
                );
                if (sub !== undefined) { setActiveSub(sub); }
            }
        } catch (_e) {
            logger.error('Failed to load subscription details', _e);
        } finally {
            setIsLoading(false);
        }
    }, [walletAddress]);

    useEffect(() => {
        void loadSubscription();
    }, [loadSubscription]);

    return { activeSub, isLoading, refreshSubscription: loadSubscription };
}
