'use client';

import { useRouter } from 'next/navigation';
import { type Dispatch, type SetStateAction, useEffect } from 'react';
import { toast } from 'sonner';

import { ExpiryDatePicker } from '@/components/wallet/expiry-date-picker';
import { ReenableWalletModal } from '@/components/wallet/reenable-wallet-modal';
import type { AccessItem, UseWalletAccessReturn } from '@/hooks/use-wallet-access';
import type { useWalletActions } from '@/hooks/use-wallet-actions';
import type { useWalletData } from '@/hooks/use-wallet-detail';

interface WalletModalsProps {
    walletData: ReturnType<typeof useWalletData>;
    walletActions: ReturnType<typeof useWalletActions>;
    showDisableModal: boolean;
    setShowDisableModal: (show: boolean) => void;
    showReenableModal: boolean;
    setShowReenableModal: (show: boolean) => void;
    editingItem: { item: AccessItem; type: 'plan' | 'permission' } | null;
    setEditingItem: Dispatch<SetStateAction<{ item: AccessItem; type: 'plan' | 'permission' } | null>>;
    pendingDrops: AccessItem[];
    setPendingDrops: Dispatch<SetStateAction<AccessItem[]>>;
    accessData: UseWalletAccessReturn;
}

export function WalletModals({
    walletData,
    walletActions,
    showDisableModal,
    setShowDisableModal,
    showReenableModal,
    setShowReenableModal,
    editingItem,
    setEditingItem,
    pendingDrops,
    setPendingDrops,
    accessData
}: WalletModalsProps) {
    const router = useRouter();

    // Redirect to disable page when modal would open
    useEffect(() => {
        if (showDisableModal === true && walletData.wallet?.walletAddress !== undefined && walletData.wallet.walletAddress !== '') {
            setShowDisableModal(false);
            router.push(`/wallet-management/wallets/${encodeURIComponent(walletData.wallet.walletAddress)}/disable`);
        }
    }, [showDisableModal, walletData.wallet?.walletAddress, setShowDisableModal, router]);

    return (
        <>
            {showReenableModal === true && walletData.wallet != null && (
                <ReenableWalletModal
                    walletAddress={walletData.wallet.walletAddress}
                    disableInfo={walletData.wallet.disableInfo}
                    isOpen={true}
                    onClose={() => setShowReenableModal(false)}
                    onConfirm={walletActions.handleReenable}
                    isLoading={walletActions.isLoading}
                />
            )}

            {editingItem !== null && (
                <ExpiryDatePicker
                    itemName={editingItem.item.name}
                    itemType={editingItem.type}
                    isOpen={true}
                    onConfirm={(date) => {
                        const handleConfirm = async () => {
                            try {
                                const expiry = date !== null ? date.toISOString() : undefined;

                                // Check if item is in pending list
                                const isPending = pendingDrops.some(p => p.id === editingItem.item.id);

                                if (isPending === true) {
                                    setPendingDrops(prev => prev.map(p =>
                                        p.id === editingItem.item.id
                                            ? { ...p, expiresAt: expiry }
                                            : p
                                    ));
                                    toast.success(`Updated pending expiry for "${editingItem.item.name}"`);
                                } else if (editingItem.type === 'plan') {
                                    await accessData.assignPlan(editingItem.item.id, expiry);
                                    toast.success(`Updated expiry for "${editingItem.item.name}"`);
                                    void accessData.refresh();
                                }
                            } catch (_err) {
                                toast.error('Failed to update expiry');
                            }
                            setEditingItem(null);
                        };
                        void handleConfirm();
                    }}
                    onCancel={() => setEditingItem(null)}
                />
            )}
        </>
    );
}
