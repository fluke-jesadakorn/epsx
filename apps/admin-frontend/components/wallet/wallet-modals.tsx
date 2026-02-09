'use client';

import type { Dispatch, SetStateAction } from 'react';
import { toast } from 'sonner';

import { DisableWalletModal } from '@/components/wallet/disable-wallet-modal';
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
    return (
        <>
            {showDisableModal === true && (
                <DisableWalletModal
                    walletAddress={walletData.wallet?.walletAddress ?? ''}
                    isOpen={true}
                    onClose={() => setShowDisableModal(false)}
                    onConfirm={walletActions.handleDisable}
                    isLoading={walletActions.isLoading}
                />
            )}
            {showReenableModal === true && walletData.wallet?.disableInfo !== undefined && (
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
