'use client';

import { useState } from 'react';

import { TrashDropZone } from '@/components/wallet/trash-drop-zone';
import { WalletAccessSection } from '@/components/wallet/wallet-access-section';
import { WalletDetailHeader } from '@/components/wallet/wallet-detail-header';
import { WalletDragOverlay } from '@/components/wallet/wallet-drag-overlay';
import { WalletModals } from '@/components/wallet/wallet-modals';
import type { UseWalletAccessReturn } from '@/hooks/use-wallet-access';
import { useWalletActions } from '@/hooks/use-wallet-actions';
import type { useSubscriptionData, useWalletData } from '@/hooks/use-wallet-detail';
import { useMetadataForm } from '@/hooks/use-wallet-detail';
import { useWalletDetailViewLogic } from '@/hooks/use-wallet-detail-view-logic';
import { Skeleton } from '@/shared/components/ui/skeleton';

import {
    DndContext,
    MouseSensor,
    TouchSensor,
    useSensor,
    useSensors
} from '@dnd-kit/core';

interface WalletDetailViewProps {
    walletAddress: string;
    authLoading: boolean;
    walletData: ReturnType<typeof useWalletData>;
    subscriptionData: ReturnType<typeof useSubscriptionData>;
    access: UseWalletAccessReturn;
}

export function WalletDetailView({
    walletAddress,
    authLoading,
    walletData,
    subscriptionData,
    access: accessData
}: WalletDetailViewProps) {
    // Logic Hook
    const logic = useWalletDetailViewLogic({
        walletAddress,
        walletData,
        accessData
    });

    // DND Sensors
    const sensors = useSensors(
        useSensor(MouseSensor, { activationConstraint: { distance: 10 } }),
        useSensor(TouchSensor, { activationConstraint: { delay: 250, tolerance: 5 } })
    );

    // Metadata Form
    const metadataForm = useMetadataForm(walletData.wallet);

    // Modal States
    const [showDisableModal, setShowDisableModal] = useState(false);
    const [showReenableModal, setShowReenableModal] = useState(false);

    // Wallet actions
    const walletActions = useWalletActions({
        walletAddress,
        onActionComplete: async () => {
            setShowDisableModal(false);
            setShowReenableModal(false);
            await walletData.loadWallet();
        }
    });

    if (authLoading === true || walletData.isLoading === true) {
        return (
            <div className="p-6">
                <div className="max-w-6xl mx-auto space-y-6">
                    <Skeleton className="h-10 w-32" />
                    <Skeleton className="h-48 w-full rounded-xl" />
                    <Skeleton className="h-96 w-full rounded-xl" />
                </div>
            </div>
        );
    }

    if (!walletData.wallet) { return null; }

    return (
        <DndContext
            sensors={sensors}
            onDragStart={logic.handleDragStart}
            onDragEnd={logic.handleDragEnd}
        >
            <div className="p-3 sm:p-6">
                <div className="max-w-6xl mx-auto space-y-6">

                    <WalletDetailHeader walletData={walletData} accessData={accessData} />

                    <WalletAccessSection
                        walletAddress={walletAddress}
                        wallet={walletData.wallet}
                        loadWallet={walletData.loadWallet}
                        metadataForm={metadataForm}
                        subscriptionData={subscriptionData}
                        accessData={accessData}
                        filteredAvailablePlans={logic.filteredAvailablePlans}
                        searchQuery={logic.searchQuery}
                        setSearchQuery={logic.setSearchQuery}
                        handleManagePlan={logic.handleManagePlan}
                        handleCopyAddress={async () => { await logic.handleCopyAddress(); }}
                        assignedSearchQuery={logic.assignedSearchQuery}
                        setAssignedSearchQuery={logic.setAssignedSearchQuery}
                        setEditingItem={logic.setEditingItem}
                        pendingDrops={logic.pendingDrops}
                        setPendingDrops={logic.setPendingDrops}
                        isSavingPending={logic.isSavingPending}
                        handleSavePendingChanges={async () => { await logic.handleSavePendingChanges(); }}
                        hasPending={logic.hasPending}
                    />

                </div>

                <WalletDragOverlay activeDragItem={logic.activeDragItem} />

                <TrashDropZone isDragging={logic.activeDragItem !== null} />

                <WalletModals
                    walletData={walletData}
                    walletActions={walletActions}
                    showDisableModal={showDisableModal}
                    setShowDisableModal={setShowDisableModal}
                    showReenableModal={showReenableModal}
                    setShowReenableModal={setShowReenableModal}
                    editingItem={logic.editingItem}
                    setEditingItem={logic.setEditingItem}
                    pendingDrops={logic.pendingDrops}
                    setPendingDrops={logic.setPendingDrops}
                    accessData={accessData}
                />
            </div>
        </DndContext >
    );
}
