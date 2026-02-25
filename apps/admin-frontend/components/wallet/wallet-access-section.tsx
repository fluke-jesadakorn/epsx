'use client';

import type { WalletData } from '@/components/wallet/types';
import {
    WalletAssignedPlansCard,
    WalletAvailablePlansCard,
    WalletMetadataCard,
    WalletSubscriptionCard
} from '@/components/wallet/wallet-detail-sections';
import type { AccessItem, UseWalletAccessReturn } from '@/hooks/use-wallet-access';
import type { useMetadataForm, useSubscriptionData } from '@/hooks/use-wallet-detail';
import { ShieldCheck } from 'lucide-react';

interface WalletAccessSectionProps {
    walletAddress: string;
    wallet: WalletData;
    loadWallet: () => Promise<void>;
    metadataForm: ReturnType<typeof useMetadataForm>;
    subscriptionData: ReturnType<typeof useSubscriptionData>;
    accessData: UseWalletAccessReturn;
    filteredAvailablePlans: AccessItem[];
    searchQuery: string;
    setSearchQuery: (query: string) => void;
    handleManagePlan: (planId: string) => void;
    handleCopyAddress: () => Promise<void>;
    assignedSearchQuery: string;
    setAssignedSearchQuery: (query: string) => void;
    setEditingItem: (editing: { item: AccessItem; type: 'plan' | 'permission' } | null) => void;
    pendingDrops: AccessItem[];
    setPendingDrops: (drops: AccessItem[] | ((prev: AccessItem[]) => AccessItem[])) => void;
    isSavingPending: boolean;
    handleSavePendingChanges: () => Promise<void>;
    hasPending: boolean;
}

export function WalletAccessSection({
    walletAddress,
    wallet,
    loadWallet,
    metadataForm,
    subscriptionData,
    accessData,
    filteredAvailablePlans,
    searchQuery,
    setSearchQuery,
    handleManagePlan,
    handleCopyAddress,
    assignedSearchQuery,
    setAssignedSearchQuery,
    setEditingItem,
    pendingDrops,
    setPendingDrops,
    isSavingPending,
    handleSavePendingChanges,
    hasPending
}: WalletAccessSectionProps) {
    const handleCopyAddressAction = () => {
        const run = async () => { await handleCopyAddress(); };
        void run();
    };

    const handleSavePendingChangesAction = () => {
        const run = async () => { await handleSavePendingChanges(); };
        void run();
    };

    const handleSaveMetadata = () => {
        const run = async () => { await metadataForm.handleSave(walletAddress, loadWallet); };
        void run();
    };

    return (
        <div className="pt-4 border-t border-gray-200 dark:border-slate-700 space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h2 className="text-xl font-bold text-gray-900 dark:text-white flex items-center gap-2">
                        <ShieldCheck className="h-5 w-5 text-purple-500 dark:text-purple-400" />
                        Wallet Identity & Access Management
                    </h2>
                    <p className="text-sm text-gray-500 dark:text-slate-500 mt-1">Manage wallet identification, subscription plans, and access plans</p>
                </div>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
                {/* LEFT COLUMN: Selection (Available Plans) */}
                <div className="lg:col-span-5 flex flex-col gap-4">
                    <WalletAvailablePlansCard
                        plans={filteredAvailablePlans}
                        searchQuery={searchQuery}
                        setSearchQuery={setSearchQuery}
                        onManagePlan={handleManagePlan}
                    />
                </div>

                {/* RIGHT COLUMN: Management (Details, Current Plan, Assigned Plans) */}
                <div className="lg:col-span-7 flex flex-col gap-6">
                    {/* Section 1: Wallet Details */}
                    <WalletMetadataCard
                        wallet={wallet}
                        metadataForm={metadataForm.metadataForm}
                        setMetadataForm={metadataForm.setMetadataForm}
                        hasChanges={metadataForm.hasChanges}
                        setHasChanges={metadataForm.setHasChanges}
                        isSaving={metadataForm.isSaving}
                        onSave={handleSaveMetadata}
                        onDiscard={() => {
                            metadataForm.setMetadataForm({ label: wallet.label ?? '', note: wallet.note ?? '' });
                            metadataForm.setHasChanges(false);
                        }}
                        onCopyAddress={handleCopyAddressAction}
                    />

                    {/* Section 2: Current Subscription (Detailed) */}
                    {subscriptionData.activeSub !== null && (
                        <WalletSubscriptionCard subscription={subscriptionData.activeSub} />
                    )}

                    {/* Section 3: Assigned Plans */}
                    <WalletAssignedPlansCard
                        authorizedPlans={accessData.data.authorizedPlans}
                        pendingDrops={pendingDrops}
                        searchQuery={assignedSearchQuery}
                        setSearchQuery={setAssignedSearchQuery}
                        onEdit={(item: AccessItem) => setEditingItem({ item, type: 'plan' })}
                        onManage={(item: AccessItem) => handleManagePlan(item.id)}
                        onDelete={(id: string) => {
                            const pendingItem = pendingDrops.find(p => p.id === id);
                            if (pendingItem !== undefined) {
                                setPendingDrops((prev: AccessItem[]) => prev.filter((p: AccessItem) => p.id !== id));
                            } else {
                                const plan = accessData.data.authorizedPlans.find(g => g.id === id);
                                // eslint-disable-next-line no-alert
                                if (window.confirm(`Are you sure you want to remove access to "${plan?.name ?? 'this plan'}"?`)) {
                                    const handleRemove = async () => {
                                        try {
                                            await accessData.removePlan(id);
                                            void accessData.refresh();
                                        } catch (_err) {
                                            // Handle error if needed
                                        }
                                    };
                                    void handleRemove();
                                }
                            }
                        }}
                        onDiscard={() => setPendingDrops([])}
                        onSave={handleSavePendingChangesAction}
                        isSaving={isSavingPending}
                        hasPending={hasPending}
                    />
                </div>
            </div>
        </div>
    );
}
