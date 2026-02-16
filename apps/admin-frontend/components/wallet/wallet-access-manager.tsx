'use client';

import React from 'react';

import { ExpiryDatePicker } from './expiry-date-picker';
import {
    AuthorizedColumn,
    AvailableColumn,
    WalletAccessActionBar,
    WalletAccessColumnsActions,
    WalletAccessError,
    WalletAccessHeader,
    WalletAccessInfoBar
} from './wallet-access-sections';

import { useWalletAccess } from '@/hooks/use-wallet-access';
import { useDragAndDrop, usePendingChanges, useSearchAndFilter, useSelection } from '@/hooks/use-wallet-access-manager';
import { cn } from '@/lib/utils';

interface WalletAccessManagerProps {
    walletAddress: string;
    className?: string;
    onSaveComplete?: () => void;
}

export function WalletAccessManager({
    walletAddress,
    className,
    onSaveComplete,
}: WalletAccessManagerProps) {
    const {
        data, isLoading, error, batchAssignPermissions, batchRevokePermissions,
        batchAssignPlans, batchRemovePlans, refresh,
    } = useWalletAccess(walletAddress);

    const pendingCtx = usePendingChanges({
        data, batchAssignPermissions, batchRevokePermissions, batchAssignPlans, batchRemovePlans, onSaveComplete
    });
    const selectionCtx = useSelection();
    const filterCtx = useSearchAndFilter({ data, pendingChanges: pendingCtx.pendingChanges });
    const dragCtx = useDragAndDrop({
        pendingChanges: pendingCtx.pendingChanges,
        stageRemove: pendingCtx.stageRemove,
        setExpiryModalItems: pendingCtx.setExpiryModalItems
    });

    const hasChanges = pendingCtx.pendingChanges.size > 0;

    if (error !== null) {
        return <WalletAccessError error={error} onRetry={() => { void refresh(); }} />;
    }

    return (
        <div className={cn('rounded-xl bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-gray-200 dark:border-gray-700', className)}>
            <WalletAccessHeader
                hasChanges={hasChanges}
                pendingCount={pendingCtx.pendingChanges.size}
                isLoading={isLoading}
                onRefresh={() => { void refresh(); }}
            />

            <WalletAccessActionBar
                hasChanges={hasChanges}
                summary={pendingCtx.changesSummary}
                onDiscard={() => { pendingCtx.discardChanges(); selectionCtx.clearSelections(); }}
                onApply={() => { void pendingCtx.applyChanges(); }}
                isApplying={pendingCtx.isApplying}
            />

            <WalletAccessInfoBar />

            <div className="grid grid-cols-1 md:grid-cols-[1fr_auto_1fr] gap-4 p-4 bg-gray-100 dark:bg-gray-800">
                <AvailableColumn
                    ref={dragCtx.availableRef}
                    items={filterCtx.availableItems}
                    selectedItems={selectionCtx.selectedAvailable}
                    isLoading={isLoading}
                    search={filterCtx.availableSearch}
                    pendingChanges={pendingCtx.pendingChanges}
                    dragState={{ dropTarget: dragCtx.dropTarget, dragSource: dragCtx.dragSource }}
                    onSearchChange={filterCtx.setAvailableSearch}
                    onSelectAll={selectionCtx.handleSelectAllAvailable}
                    onSelectItem={selectionCtx.handleSelectAvailable}
                    onDragStart={(e, item) => dragCtx.handleDragStart(e, item, 'available')}
                    onDragEnd={dragCtx.handleDragEnd}
                    onDragOver={(e) => dragCtx.handleDragOver(e, 'available')}
                    onDragLeave={dragCtx.handleDragLeave}
                    onDrop={(e) => dragCtx.handleDrop(e, 'available')}
                    onItemClick={(item) => pendingCtx.stageAssign(item)}
                />

                <WalletAccessColumnsActions
                    selectedAvailable={selectionCtx.selectedAvailable}
                    selectedAuthorized={selectionCtx.selectedAuthorized}
                    availableItems={filterCtx.availableItems}
                    authorizedItems={filterCtx.authorizedItems}
                    onBulkAssign={pendingCtx.stageBulkAssign}
                    onBulkRemove={pendingCtx.stageBulkRemove}
                    onClearAuthorizedSelection={() => selectionCtx.setSelectedAuthorized(new Set())}
                />

                <AuthorizedColumn
                    ref={dragCtx.authorizedRef}
                    items={filterCtx.authorizedItems}
                    selectedItems={selectionCtx.selectedAuthorized}
                    isLoading={isLoading}
                    search={filterCtx.authorizedSearch}
                    pendingChanges={pendingCtx.pendingChanges}
                    dragState={{ dropTarget: dragCtx.dropTarget, dragSource: dragCtx.dragSource }}
                    onSearchChange={filterCtx.setAuthorizedSearch}
                    onSelectAll={selectionCtx.handleSelectAllAuthorized}
                    onSelectItem={selectionCtx.handleSelectAuthorized}
                    onDragStart={(e, item) => dragCtx.handleDragStart(e, item, 'authorized')}
                    onDragEnd={dragCtx.handleDragEnd}
                    onDragOver={(e) => dragCtx.handleDragOver(e, 'authorized')}
                    onDragLeave={dragCtx.handleDragLeave}
                    onDrop={(e) => dragCtx.handleDrop(e, 'authorized')}
                    onItemClick={(item) => pendingCtx.stageRemove(item)}
                />
            </div>

            <ExpiryDatePicker
                itemName={pendingCtx.expiryModalItems !== null && pendingCtx.expiryModalItems.length > 1
                    ? `${pendingCtx.expiryModalItems.length} items`
                    : pendingCtx.expiryModalItems?.[0]?.name ?? ''}
                itemType={(pendingCtx.expiryModalItems !== null && pendingCtx.expiryModalItems.length > 1
                    ? 'items'
                    : (pendingCtx.expiryModalItems?.[0]?.type ?? 'permission')) as 'permission' | 'plan' | 'items'}
                isOpen={pendingCtx.expiryModalItems !== null}
                onConfirm={pendingCtx.handleExpiryConfirm}
                onCancel={pendingCtx.handleExpiryCancel}
            />
        </div>
    );
}

export default WalletAccessManager;
