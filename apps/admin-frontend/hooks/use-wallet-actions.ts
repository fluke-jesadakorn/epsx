import { disableWalletAction, enableWalletAction } from '@/app/wallet-management/plan-actions';
import type { DisableWalletData } from '@/components/wallet/disable-wallet-modal';
import type { ReenableWalletData } from '@/components/wallet/reenable-wallet-modal';
import { logger } from '@/shared/utils/logger';
import { useCallback, useState } from 'react';
import { toast } from 'sonner';

export interface UseWalletActionsContext {
    walletAddress: string;
    onActionComplete: () => Promise<void>;
}

export function useWalletActions({ walletAddress, onActionComplete }: UseWalletActionsContext) {
    const [isLoading, setIsLoading] = useState(false);

    const handleDisable = useCallback(async (data: DisableWalletData) => {
        setIsLoading(true);
        try {
            await disableWalletAction(walletAddress, {
                duration_days: data.duration === 'until_manual' ? null : data.duration,
                reason_category: data.reasonCategory,
                reason_details: data.reasonDetails,
                affected_platforms: data.affectedPlatforms,
                block_login: data.blockLogin,
                pause_subscriptions: data.pauseSubscriptions,
                notify_user: data.notifyUser,
            });
            toast.success('Wallet disabled successfully');
            await onActionComplete();
        } catch (_err) {
            logger.error('Failed to disable wallet:', _err);
            toast.error(_err instanceof Error ? _err.message : 'Failed to disable wallet');
        } finally {
            setIsLoading(false);
        }
    }, [walletAddress, onActionComplete]);

    const handleReenable = useCallback(async (data: ReenableWalletData) => {
        setIsLoading(true);
        try {
            await enableWalletAction(walletAddress, {
                platforms_to_enable: data.platformsToEnable,
                restore_permissions: data.restorePermissions,
                resume_subscriptions: data.resumeSubscriptions,
                resolution_note: data.resolutionNote,
            });
            toast.success('Wallet re-enabled successfully');
            await onActionComplete();
        } catch (_err) {
            logger.error('Failed to re-enable wallet:', _err);
            toast.error(_err instanceof Error ? _err.message : 'Failed to re-enable wallet');
        } finally {
            setIsLoading(false);
        }
    }, [walletAddress, onActionComplete]);

    return { isLoading, handleDisable, handleReenable };
}
