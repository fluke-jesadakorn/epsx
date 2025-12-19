/**
 * Wallet Detail Panel Component
 * Slide-out panel showing comprehensive wallet information
 */
'use client';

import { ExternalLink, Package, Shield } from 'lucide-react';
import { useState } from 'react';

import { Button } from '@/components/ui/button';
import {
    Sheet,
    SheetContent,
    SheetDescription,
    SheetHeader,
    SheetTitle,
} from '@/components/ui/sheet';
import { formatDate } from '@/lib/utils/date';

import { AssignPermissionForm, type AssignPermissionData } from './AssignPermissionForm';
import { WalletActivityHistory } from './WalletActivityHistory';
import { WalletHeader } from './WalletHeader';
import { WalletPermissionTable } from './WalletPermissionTable';
import type { WalletActivityEvent, WalletData } from './types';

interface WalletDetailPanelProps {
    wallet: WalletData | null;
    isOpen: boolean;
    onClose: () => void;
    onAssignPermission?: (data: AssignPermissionData) => Promise<void>;
    onRevokePermission?: (permissionId: string) => Promise<void>;
    onDisable?: () => void;
    onEnable?: () => void;
    activityEvents?: WalletActivityEvent[];
    isLoading?: boolean;
}

export function WalletDetailPanel({
    wallet,
    isOpen,
    onClose,
    onAssignPermission,
    onRevokePermission,
    onDisable,
    onEnable,
    activityEvents = [],
}: WalletDetailPanelProps) {
    const [isAssigning, setIsAssigning] = useState(false);

    const isDisabled = wallet?.status === 'disabled';
    const activeSubscriptions = wallet?.subscriptions?.filter(s => s.status === 'active') ?? [];

    const handleAssignPermission = async (data: AssignPermissionData) => {
        if (!onAssignPermission) return;
        setIsAssigning(true);
        try {
            await onAssignPermission(data);
        } finally {
            setIsAssigning(false);
        }
    };

    return (
        <Sheet open={isOpen} onOpenChange={(open) => !open && onClose()}>
            <SheetContent className="w-full sm:max-w-2xl overflow-y-auto">
                {!wallet ? (
                    <div className="flex items-center justify-center h-full">
                        <div className="text-center text-gray-500 dark:text-gray-400">
                            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-4"></div>
                            <p>Loading wallet details...</p>
                        </div>
                    </div>
                ) : (
                    <>
                        <SheetHeader className="space-y-4">
                            <div className="flex items-start justify-between">
                                <div>
                                    <SheetTitle className="text-xl flex items-center gap-2">
                                        <span className="text-2xl">👛</span>
                                        Wallet Details
                                    </SheetTitle>
                                    <SheetDescription className="mt-1">
                                        View and manage wallet permissions
                                    </SheetDescription>
                                </div>
                            </div>
                        </SheetHeader>

                        <div className="mt-6 space-y-8">
                            {/* Header Section */}
                            <WalletHeader wallet={wallet} />

                            {/* Active Subscriptions */}
                            {activeSubscriptions.length > 0 && (
                                <div>
                                    <h3 className="font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
                                        <Package className="h-5 w-5 text-purple-600" />
                                        Active Subscriptions
                                    </h3>
                                    <div className="space-y-3">
                                        {activeSubscriptions.map((sub) => (
                                            <div
                                                key={sub.id}
                                                className="p-4 rounded-xl border border-purple-200 dark:border-purple-800 bg-purple-50/50 dark:bg-purple-900/10 transition-colors"
                                            >
                                                <div className="flex items-center justify-between">
                                                    <div>
                                                        <p className="font-bold text-gray-900 dark:text-white">
                                                            📦 {sub.planName}
                                                        </p>
                                                        <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                                                            {sub.priceDisplay} • Since {formatDate(sub.startedAt)}
                                                        </p>
                                                    </div>
                                                    <div className="px-2.5 py-1 rounded-full bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400 text-xs font-semibold border border-green-200 dark:border-green-800">
                                                        Active
                                                    </div>
                                                </div>
                                                {sub.grantedPermissions.length > 0 && (
                                                    <div className="mt-3 pt-3 border-t border-purple-100 dark:border-purple-800/50">
                                                        <p className="text-[10px] uppercase tracking-wider font-bold text-gray-400 dark:text-gray-500 mb-2">
                                                            Auto-granted Permissions
                                                        </p>
                                                        <div className="flex flex-wrap gap-1.5">
                                                            {sub.grantedPermissions.map((perm) => (
                                                                <code
                                                                    key={perm}
                                                                    className="text-[10px] px-2 py-0.5 bg-white dark:bg-gray-800 rounded-md border border-gray-200 dark:border-gray-700 font-mono text-purple-600 dark:text-purple-400"
                                                                >
                                                                    {perm}
                                                                </code>
                                                            ))}
                                                        </div>
                                                    </div>
                                                )}
                                            </div>
                                        ))}
                                    </div>
                                </div>
                            )}

                            {/* Permissions Section */}
                            <div>
                                <h3 className="font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
                                    <Shield className="h-5 w-5 text-blue-600" />
                                    Security & Permissions
                                </h3>
                                <div className="space-y-4">
                                    <WalletPermissionTable
                                        permissions={wallet.permissions}
                                        showActions={!!onRevokePermission}
                                        onRevoke={onRevokePermission}
                                    />

                                    {onAssignPermission && (
                                        <div className="p-4 rounded-xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-black/20">
                                            <AssignPermissionForm
                                                walletAddress={wallet.walletAddress}
                                                onAssign={handleAssignPermission}
                                                isLoading={isAssigning}
                                            />
                                        </div>
                                    )}
                                </div>
                            </div>

                            {/* Activity History */}
                            <div>
                                <h3 className="font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
                                    📜 Activity History
                                </h3>
                                <WalletActivityHistory events={activityEvents} />
                            </div>

                            {/* Quick Actions */}
                            <div className="pt-6 border-t border-gray-100 dark:border-gray-800">
                                <h4 className="text-xs font-bold uppercase tracking-widest text-gray-400 dark:text-gray-500 mb-4">
                                    Administrative Actions
                                </h4>
                                <div className="flex flex-wrap gap-3">
                                    {isDisabled ? (
                                        <Button
                                            variant="outline"
                                            onClick={onEnable}
                                            className="border-green-300 text-green-700 hover:bg-green-50 dark:border-green-700/50 dark:text-green-400 dark:hover:bg-green-900/20"
                                        >
                                            🔓 Restore Access
                                        </Button>
                                    ) : (
                                        <Button
                                            variant="outline"
                                            onClick={onDisable}
                                            className="border-amber-300 text-amber-700 hover:bg-amber-50 dark:border-amber-700/50 dark:text-amber-400 dark:hover:bg-amber-900/20"
                                        >
                                            ⚠️ Suspend Access
                                        </Button>
                                    )}
                                    <Button
                                        variant="outline"
                                        onClick={() => window.open(`https://bscscan.com/address/${wallet.walletAddress}`, '_blank')}
                                        className="gap-2"
                                    >
                                        <ExternalLink className="h-4 w-4" />
                                        Explorer
                                    </Button>
                                </div>
                            </div>
                        </div>
                    </>
                )}
            </SheetContent>
        </Sheet>
    );
}
