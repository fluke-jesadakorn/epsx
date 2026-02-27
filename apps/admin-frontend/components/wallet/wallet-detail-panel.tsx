/**
 * Wallet Detail Panel Component
 * Slide-out panel showing comprehensive wallet information
 */
'use client';

import { Check, ExternalLink, Package, Pencil, Shield, X } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';

import { getExplorerAddressLink } from '@/shared/config/constants';
import { AssignPermissionForm, type AssignPermissionData } from './assign-permission-form';
import type { WalletData } from './types';
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
    isLoading?: boolean;
}

interface LabelNoteSectionProps {
    wallet: WalletData;
    isEditingLabel: boolean;
    isEditingNote: boolean;
    labelValue: string;
    noteValue: string;
    isSaving: boolean;
    onLabelChange: (v: string) => void;
    onNoteChange: (v: string) => void;
    onSaveLabel: () => void;
    onSaveNote: () => void;
    onCancelLabel: () => void;
    onCancelNote: () => void;
    onStartEditLabel: () => void;
    onStartEditNote: () => void;
}

function LabelNoteSection({
    wallet, isEditingLabel, isEditingNote, labelValue, noteValue, isSaving,
    onLabelChange, onNoteChange, onSaveLabel, onSaveNote,
    onCancelLabel, onCancelNote, onStartEditLabel, onStartEditNote,
}: LabelNoteSectionProps) {
    return (
        <div className="p-4 rounded-xl border border-border/20 bg-muted/30">
            <h4 className="text-sm font-semibold text-foreground mb-4 flex items-center gap-2">
                Label & Notes
            </h4>

            {/* Label */}
            <div className="mb-4">
                <label className="text-xs font-medium text-muted-foreground mb-1.5 block">
                    Label
                </label>
                {isEditingLabel ? (
                    <div className="flex items-center gap-2">
                        <Input
                            value={labelValue}
                            onChange={(e) => onLabelChange(e.target.value)}
                            placeholder="e.g., VIP, Whale, Partner..."
                            className="flex-1 h-9"
                            maxLength={30}
                            autoFocus
                        />
                        <Button size="sm" variant="ghost" onClick={onSaveLabel} disabled={isSaving} className="h-9 w-9 p-0">
                            <Check className="h-4 w-4 text-green-600" />
                        </Button>
                        <Button size="sm" variant="ghost" onClick={onCancelLabel} className="h-9 w-9 p-0">
                            <X className="h-4 w-4 text-muted-foreground" />
                        </Button>
                    </div>
                ) : (
                    <div className="flex items-center gap-2 cursor-pointer group" onClick={onStartEditLabel}>
                        {wallet.label !== undefined && wallet.label !== '' ? (
                            <WalletLabelBadge label={wallet.label} size="md" />
                        ) : (
                            <span className="text-sm text-muted-foreground italic">No label</span>
                        )}
                        <Pencil className="h-3.5 w-3.5 text-muted-foreground opacity-0 group-hover:opacity-100 transition-opacity" />
                    </div>
                )}
            </div>

            {/* Note */}
            <div>
                <label className="text-xs font-medium text-muted-foreground mb-1.5 block">
                    Note
                </label>
                {isEditingNote ? (
                    <div className="space-y-2">
                        <Textarea
                            value={noteValue}
                            onChange={(e) => onNoteChange(e.target.value)}
                            placeholder="Add a note about this wallet..."
                            className="resize-none"
                            rows={3}
                            maxLength={500}
                            autoFocus
                        />
                        <div className="flex items-center justify-between">
                            <span className="text-xs text-muted-foreground">{noteValue.length}/500</span>
                            <div className="flex gap-2">
                                <Button size="sm" variant="ghost" onClick={onCancelNote}>Cancel</Button>
                                <Button size="sm" onClick={onSaveNote} disabled={isSaving}>
                                    {isSaving ? 'Saving...' : 'Save'}
                                </Button>
                            </div>
                        </div>
                    </div>
                ) : (
                    <div className="cursor-pointer group p-2 rounded-lg hover:bg-muted/30 transition-colors -mx-2" onClick={onStartEditNote}>
                        {wallet.note !== undefined && wallet.note !== '' ? (
                            <p className="text-sm text-foreground whitespace-pre-wrap">{wallet.note}</p>
                        ) : (
                            <p className="text-sm text-muted-foreground italic flex items-center gap-1">
                                <Pencil className="h-3 w-3" />
                                Click to add a note...
                            </p>
                        )}
                    </div>
                )}
            </div>
        </div>
    );
}

interface ActiveSubscriptionsSectionProps {
    subscriptions: WalletData['subscriptions'];
}

function ActiveSubscriptionsSection({ subscriptions }: ActiveSubscriptionsSectionProps) {
    const active = subscriptions.filter(s => s.status === 'active');
    if (active.length === 0) { return null; }

    return (
        <div>
            <h3 className="font-semibold text-foreground mb-4 flex items-center gap-2">
                <Package className="h-5 w-5 text-[#7645d9]" />
                Active Subscriptions
            </h3>
            <div className="space-y-3">
                {active.map((sub) => (
                    <div key={sub.id} className="p-4 rounded-xl border border-[#7645d9]/20 bg-[#7645d9]/5 transition-colors">
                        <div className="flex items-center justify-between">
                            <div>
                                <p className="font-bold text-foreground">📦 {sub.planName}</p>
                                <p className="text-sm text-muted-foreground mt-1">
                                    {sub.priceDisplay} • Since {formatDate(sub.startedAt)}
                                </p>
                            </div>
                            <div className="px-2.5 py-1 rounded-full bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400 text-xs font-semibold border border-green-200 dark:border-green-800">
                                Active
                            </div>
                        </div>
                        {sub.grantedPermissions.length > 0 && (
                            <div className="mt-3 pt-3 border-t border-[#7645d9]/20">
                                <p className="text-[10px] uppercase tracking-wider font-bold text-muted-foreground mb-2">
                                    Auto-granted Permissions
                                </p>
                                <div className="flex flex-wrap gap-1.5">
                                    {sub.grantedPermissions.map((perm) => (
                                        <code key={perm} className="text-[10px] px-2 py-0.5 bg-card rounded-md border border-border/40 font-mono text-[#7645d9]">
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
    );
}

interface PermissionsSectionProps {
    wallet: WalletData;
    onRevokePermission?: (permissionId: string) => Promise<void>;
    onAssignPermission?: (data: AssignPermissionData) => Promise<void>;
    isAssigning: boolean;
    onAssign: (data: AssignPermissionData) => Promise<void>;
}

function PermissionsSection({ wallet, onRevokePermission, onAssignPermission, isAssigning, onAssign }: PermissionsSectionProps) {
    const wrappedRevoke = onRevokePermission !== undefined
        ? (id: string) => { void onRevokePermission(id); }
        : undefined;

    return (
        <div>
            <h3 className="font-semibold text-foreground mb-4 flex items-center gap-2">
                <Shield className="h-5 w-5 text-[#1fc7d4]" />
                Security & Permissions
            </h3>
            <div className="space-y-4">
                <WalletPermissionTable
                    permissions={wallet.permissions}
                    showActions={onRevokePermission !== undefined}
                    onRevoke={wrappedRevoke}
                />
                {onAssignPermission !== undefined && (
                    <div className="p-4 rounded-xl border border-border/20 bg-card">
                        <AssignPermissionForm
                            walletAddress={wallet.walletAddress}
                            onAssign={onAssign}
                            isLoading={isAssigning}
                        />
                    </div>
                )}
            </div>
        </div>
    );
}

interface AdminActionsSectionProps {
    isDisabled: boolean;
    walletAddress: string;
    onEnable?: () => void;
    onDisable?: () => void;
}

function AdminActionsSection({ isDisabled, walletAddress, onEnable, onDisable }: AdminActionsSectionProps) {
    return (
        <div className="pt-6 border-t border-border/20">
            <h4 className="text-xs font-bold uppercase tracking-widest text-muted-foreground mb-4">
                Administrative Actions
            </h4>
            <div className="flex flex-wrap gap-3">
                {isDisabled ? (
                    <Button
                        variant="outline"
                        onClick={onEnable}
                        className="border-green-300 text-green-700 hover:bg-green-50 dark:border-green-700/50 dark:text-green-400 dark:hover:bg-green-900/20"
                    >
                        Restore Access
                    </Button>
                ) : (
                    <Button
                        variant="outline"
                        onClick={onDisable}
                        className="border-amber-300 text-amber-700 hover:bg-amber-50 dark:border-amber-700/50 dark:text-amber-400 dark:hover:bg-amber-900/20"
                    >
                        Suspend Access
                    </Button>
                )}
                <Button
                    variant="outline"
                    onClick={() => window.open(getExplorerAddressLink(walletAddress), '_blank')}
                    className="gap-2"
                >
                    <ExternalLink className="h-4 w-4" />
                    Explorer
                </Button>
            </div>
        </div>
    );
}

function useLabelNoteEdit(wallet: WalletData | null) {
    const [isEditingLabel, setIsEditingLabel] = useState(false);
    const [isEditingNote, setIsEditingNote] = useState(false);
    const [labelValue, setLabelValue] = useState('');
    const [noteValue, setNoteValue] = useState('');
    const [isSaving, setIsSaving] = useState(false);

    useEffect(() => {
        if (wallet !== null) {
            setLabelValue(wallet.label ?? '');
            setNoteValue(wallet.note ?? '');
        }
    }, [wallet]);

    const saveLabel = useCallback(async () => {
        if (wallet === null) { return; }
        setIsSaving(true);
        try {
            await walletMgmt.updateWalletMetadata(wallet.walletAddress, { label: labelValue.trim() !== '' ? labelValue.trim() : null });
            setIsEditingLabel(false);
        } catch (error) {
            logger.error('Failed to save label:', { error });
        } finally {
            setIsSaving(false);
        }
    }, [wallet, labelValue]);

    const saveNote = useCallback(async () => {
        if (wallet === null) { return; }
        setIsSaving(true);
        try {
            await walletMgmt.updateWalletMetadata(wallet.walletAddress, { note: noteValue.trim() !== '' ? noteValue.trim() : null });
            setIsEditingNote(false);
        } catch (error) {
            logger.error('Failed to save note:', { error });
        } finally {
            setIsSaving(false);
        }
    }, [wallet, noteValue]);

    return { isEditingLabel, isEditingNote, labelValue, noteValue, isSaving, setIsEditingLabel, setIsEditingNote, setLabelValue, setNoteValue, saveLabel, saveNote };
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
 * @param root0.isLoading
 */
export function WalletDetailPanel({
    wallet,
    isOpen,
    onClose,
    onAssignPermission,
    onRevokePermission,
    onDisable,
    onEnable,
    isLoading: _isLoading,
}: WalletDetailPanelProps) {
    const [isAssigning, setIsAssigning] = useState(false);
    const { isEditingLabel, isEditingNote, labelValue, noteValue, isSaving, setIsEditingLabel, setIsEditingNote, setLabelValue, setNoteValue, saveLabel, saveNote } = useLabelNoteEdit(wallet);

    const isDisabled = wallet !== null && wallet.status === 'disabled';

    const handleAssignPermission = async (data: AssignPermissionData) => {
        if (onAssignPermission === undefined) { return; }
        setIsAssigning(true);
        try {
            await onAssignPermission(data);
        } finally {
            setIsAssigning(false);
        }
    };

    return (
        <Sheet open={isOpen} onOpenChange={(open) => { if (!open) { onClose(); } }}>
            <SheetContent className="w-full sm:max-w-2xl overflow-y-auto">
                {wallet === null ? (
                    <div className="flex items-center justify-center h-full">
                        <div className="text-center text-muted-foreground">
                            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-[#1fc7d4] mx-auto mb-4" />
                            <p>Loading wallet details...</p>
                        </div>
                    </div>
                ) : (
                    <>
                        <SheetHeader className="space-y-4">
                            <div className="flex items-start justify-between">
                                <div>
                                    <SheetTitle className="text-xl flex items-center gap-2">
                                        Wallet Details
                                    </SheetTitle>
                                    <SheetDescription className="mt-1">
                                        View and manage wallet permissions
                                    </SheetDescription>
                                </div>
                            </div>
                        </SheetHeader>

                        <div className="mt-6 space-y-8">
                            <WalletHeader wallet={wallet} />

                            <LabelNoteSection
                                wallet={wallet}
                                isEditingLabel={isEditingLabel}
                                isEditingNote={isEditingNote}
                                labelValue={labelValue}
                                noteValue={noteValue}
                                isSaving={isSaving}
                                onLabelChange={setLabelValue}
                                onNoteChange={setNoteValue}
                                onSaveLabel={() => { void saveLabel(); }}
                                onSaveNote={() => { void saveNote(); }}
                                onCancelLabel={() => { setLabelValue(wallet.label ?? ''); setIsEditingLabel(false); }}
                                onCancelNote={() => { setNoteValue(wallet.note ?? ''); setIsEditingNote(false); }}
                                onStartEditLabel={() => setIsEditingLabel(true)}
                                onStartEditNote={() => setIsEditingNote(true)}
                            />

                            <ActiveSubscriptionsSection subscriptions={wallet.subscriptions} />

                            <PermissionsSection
                                wallet={wallet}
                                onRevokePermission={onRevokePermission}
                                onAssignPermission={onAssignPermission}
                                isAssigning={isAssigning}
                                onAssign={handleAssignPermission}
                            />

                            <AdminActionsSection
                                isDisabled={isDisabled}
                                walletAddress={wallet.walletAddress}
                                onEnable={onEnable}
                                onDisable={onDisable}
                            />
                        </div>
                    </>
                )}
            </SheetContent>
        </Sheet>
    );
}
