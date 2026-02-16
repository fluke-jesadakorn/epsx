/**
 * Wallet Detail Panel Component
 * Slide-out panel showing comprehensive wallet information
 */
'use client';

import { Check, ExternalLink, Package, Pencil, Shield, X } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';

import { getExplorerAddressLink } from '@/shared/config/constants';
import { AssignPermissionForm, type AssignPermissionData } from './assign-permission-form';
import type { WalletActivityEvent, WalletData } from './types';
import { WalletActivityHistory } from './wallet-activity-history';
import { WalletHeader } from './wallet-header';
import { WalletLabelBadge } from './wallet-label-badge';
import { WalletPermissionTable } from './wallet-permission-table';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
    Sheet,
    SheetContent,
    SheetDescription,
    SheetHeader,
    SheetTitle,
} from '@/components/ui/sheet';
import { Textarea } from '@/components/ui/textarea';
import { walletMgmt } from '@/lib/api/wallet-management-client';
import { logger } from '@/lib/logger';
import { formatDate } from '@/lib/utils/date';

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

/**
 *
 * @param root0
 * @param root0.wallet
 * @param root0.isOpen
 * @param root0.onClose
 * @param root0.onAssignPermission
 * @param root0.onRevokePermission
 * @param root0.onDisable
 * @param root0.onEnable
 * @param root0.activityEvents
 */
export function WalletDetailPanel({
    wallet,
    isOpen,
    onClose,
    onAssignPermission,
    onRevokePermission,
    onDisable,
    onEnable,
    activityEvents = [],
    isLoading: _isLoading,
}: WalletDetailPanelProps) {
    const [isAssigning, setIsAssigning] = useState(false);

    // Label & Note editing state
    const [isEditingLabel, setIsEditingLabel] = useState(false);
    const [isEditingNote, setIsEditingNote] = useState(false);
    const [labelValue, setLabelValue] = useState('');
    const [noteValue, setNoteValue] = useState('');
    const [isSaving, setIsSaving] = useState(false);

    // Sync label/note state when wallet changes
    useEffect(() => {
        if (wallet) {
            setLabelValue(wallet.label ?? '');
            setNoteValue(wallet.note ?? '');
        }
    }, [wallet]);

    const isDisabled = wallet?.status === 'disabled';
    const activeSubscriptions = wallet?.subscriptions?.filter(s => s.status === 'active') ?? [];

    const handleAssignPermission = async (data: AssignPermissionData) => {
        if (!onAssignPermission) { return; }
        setIsAssigning(true);
        try {
            await onAssignPermission(data);
        } finally {
            setIsAssigning(false);
        }
    };

    // Save label
    const handleSaveLabel = useCallback(async () => {
        if (!wallet) { return; }
        setIsSaving(true);
        try {
            await walletMgmt.updateWalletMetadata(wallet.walletAddress, {
                label: labelValue.trim() ?? null,
            });
            setIsEditingLabel(false);
        } catch (error) {
            logger.error('Failed to save label:', { error });
        } finally {
            setIsSaving(false);
        }
    }, [wallet, labelValue]);

    // Save note
    const handleSaveNote = useCallback(async () => {
        if (!wallet) { return; }
        setIsSaving(true);
        try {
            await walletMgmt.updateWalletMetadata(wallet.walletAddress, {
                note: noteValue.trim() ?? null,
            });
            setIsEditingNote(false);
        } catch (error) {
            logger.error('Failed to save note:', { error });
        } finally {
            setIsSaving(false);
        }
    }, [wallet, noteValue]);

    return (
        <Sheet open={isOpen} onOpenChange={(open) => !open && onClose()}>
            <SheetContent className="w-full sm:max-w-2xl overflow-y-auto">
                {!wallet ? (
                    <div className="flex items-center justify-center h-full">
                        <div className="text-center text-gray-500 dark:text-gray-400">
                            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-4" />
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

                            {/* Label & Note Section */}
                            <div className="p-4 rounded-xl border border-gray-200 dark:border-gray-800 bg-gray-50/50 dark:bg-gray-900/30">
                                <h4 className="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-4 flex items-center gap-2">
                                    🏷️ Label & Notes
                                </h4>

                                {/* Label */}
                                <div className="mb-4">
                                    <label className="text-xs font-medium text-gray-500 dark:text-gray-400 mb-1.5 block">
                                        Label
                                    </label>
                                    {isEditingLabel ? (
                                        <div className="flex items-center gap-2">
                                            <Input
                                                value={labelValue}
                                                onChange={(e) => setLabelValue(e.target.value)}
                                                placeholder="e.g., VIP, Whale, Partner..."
                                                className="flex-1 h-9"
                                                maxLength={30}
                                                autoFocus
                                            />
                                            <Button
                                                size="sm"
                                                variant="ghost"
                                                onClick={handleSaveLabel}
                                                disabled={isSaving}
                                                className="h-9 w-9 p-0"
                                            >
                                                <Check className="h-4 w-4 text-green-600" />
                                            </Button>
                                            <Button
                                                size="sm"
                                                variant="ghost"
                                                onClick={() => {
                                                    setLabelValue(wallet.label ?? '');
                                                    setIsEditingLabel(false);
                                                }}
                                                className="h-9 w-9 p-0"
                                            >
                                                <X className="h-4 w-4 text-gray-500" />
                                            </Button>
                                        </div>
                                    ) : (
                                        <div
                                            className="flex items-center gap-2 cursor-pointer group"
                                            onClick={() => setIsEditingLabel(true)}
                                        >
                                            {wallet.label ? (
                                                <WalletLabelBadge label={wallet.label} size="md" />
                                            ) : (
                                                <span className="text-sm text-gray-400 italic">No label</span>
                                            )}
                                            <Pencil className="h-3.5 w-3.5 text-gray-400 opacity-0 group-hover:opacity-100 transition-opacity" />
                                        </div>
                                    )}
                                </div>

                                {/* Note */}
                                <div>
                                    <label className="text-xs font-medium text-gray-500 dark:text-gray-400 mb-1.5 block">
                                        Note
                                    </label>
                                    {isEditingNote ? (
                                        <div className="space-y-2">
                                            <Textarea
                                                value={noteValue}
                                                onChange={(e) => setNoteValue(e.target.value)}
                                                placeholder="Add a note about this wallet..."
                                                className="resize-none"
                                                rows={3}
                                                maxLength={500}
                                                autoFocus
                                            />
                                            <div className="flex items-center justify-between">
                                                <span className="text-xs text-gray-400">
                                                    {noteValue.length}/500
                                                </span>
                                                <div className="flex gap-2">
                                                    <Button
                                                        size="sm"
                                                        variant="ghost"
                                                        onClick={() => {
                                                            setNoteValue(wallet.note ?? '');
                                                            setIsEditingNote(false);
                                                        }}
                                                    >
                                                        Cancel
                                                    </Button>
                                                    <Button
                                                        size="sm"
                                                        onClick={handleSaveNote}
                                                        disabled={isSaving}
                                                    >
                                                        {isSaving ? 'Saving...' : 'Save'}
                                                    </Button>
                                                </div>
                                            </div>
                                        </div>
                                    ) : (
                                        <div
                                            className="cursor-pointer group p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors -mx-2"
                                            onClick={() => setIsEditingNote(true)}
                                        >
                                            {wallet.note ? (
                                                <p className="text-sm text-gray-700 dark:text-gray-300 whitespace-pre-wrap">
                                                    {wallet.note}
                                                </p>
                                            ) : (
                                                <p className="text-sm text-gray-400 italic flex items-center gap-1">
                                                    <Pencil className="h-3 w-3" />
                                                    Click to add a note...
                                                </p>
                                            )}
                                        </div>
                                    )}
                                </div>
                            </div>

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
                                        showActions={Boolean(onRevokePermission)}
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
                                        onClick={() => window.open(getExplorerAddressLink(wallet.walletAddress), '_blank')}
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
