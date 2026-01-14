/**
 * Wallet Detail Page
 * Full page view for comprehensive wallet information and management
 */
'use client';

import { AlertTriangle, ArrowLeft, Clock, Copy, Edit, ExternalLink, Loader2, Package, RefreshCw, Save } from 'lucide-react';
import Link from 'next/link';
import { useParams, useRouter } from 'next/navigation';
import { useCallback, useEffect, useState } from 'react';
import { toast } from 'sonner';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Skeleton } from '@/components/ui/skeleton';
import { Textarea } from '@/components/ui/textarea';
import { DisableWalletModal, type DisableWalletData } from '@/components/wallet/DisableWalletModal';
import { ReenableWalletModal, type ReenableWalletData } from '@/components/wallet/ReenableWalletModal';
import type { WalletActivityEvent, WalletData, WalletStatus } from '@/components/wallet/types';
import { WalletAccessManager } from '@/components/wallet/WalletAccessManager';
import { WalletActivityTimeline } from '@/components/wallet/WalletActivityTimeline';
import { walletMgmt } from '@/lib/api/wallet-management-client';
import { cn, copyToClipboard } from '@/lib/utils';
import { useSharedAuth } from '@/shared/components/auth/Provider';

const STATUS_CONFIG: Record<WalletStatus, { label: string; emoji: string; className: string }> = {
    active: {
        label: 'Active',
        emoji: '🟢',
        className: 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400 border-green-200 dark:border-green-800',
    },
    disabled: {
        label: 'Disabled',
        emoji: '⚠️',
        className: 'bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-400 border-amber-200 dark:border-amber-800',
    },
    pending: {
        label: 'Pending',
        emoji: '⏳',
        className: 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400 border-blue-200 dark:border-blue-800',
    },
};

function formatDate(timestamp: string): string {
    const date = new Date(timestamp);
    return date.toLocaleDateString('en-US', {
        month: 'short',
        day: 'numeric',
        year: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
    });
}

function formatTimeAgo(timestamp: string): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diffInHours = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60));

    if (diffInHours < 1) { return 'Just now'; }
    if (diffInHours < 24) { return `${diffInHours} hours ago`; }

    const diffInDays = Math.floor(diffInHours / 24);
    if (diffInDays === 1) { return 'Yesterday'; }
    if (diffInDays < 30) { return `${diffInDays} days ago`; }

    return date.toLocaleDateString();
}

/**
 *
 */
export default function WalletDetailPage() {
    const router = useRouter();
    const params = useParams();
    const walletAddress = decodeURIComponent(params['address'] as string);

    const { isAuthenticated, isLoading: authLoading } = useSharedAuth();

    // State
    const [wallet, setWallet] = useState<WalletData | null>(null);
    const [isLoading, setIsLoading] = useState(true);
    const [isRefreshing, setIsRefreshing] = useState(false);
    const [activityEvents, setActivityEvents] = useState<WalletActivityEvent[]>([]);
    const [copied, setCopied] = useState(false);

    // Metadata Editing State
    const [isEditingMetadata, setIsEditingMetadata] = useState(false);
    const [metadataForm, setMetadataForm] = useState({ label: '', note: '' });
    const [isSavingMetadata, setIsSavingMetadata] = useState(false);

    // Modals
    const [showDisableModal, setShowDisableModal] = useState(false);
    const [showReenableModal, setShowReenableModal] = useState(false);
    const [isActionLoading, setIsActionLoading] = useState(false);

    // Load wallet data
    const loadWallet = useCallback(async () => {
        if (!walletAddress) { return; }

        try {
            setIsRefreshing(true);
            const [walletData, events] = await Promise.all([
                walletMgmt.fetchWalletDetail(walletAddress),
                walletMgmt.fetchActivityHistory(walletAddress),
            ]);
            setWallet(walletData);
            setActivityEvents(events);

            // Initial metadata form state
            if (walletData) {
                setMetadataForm({
                    label: walletData.label || '',
                    note: walletData.note || '',
                });
            }

        } catch (err) {
            console.error('Failed to load wallet:', err);
            toast.error('Failed to load wallet details');
            router.push('/wallet-management');
        } finally {
            setIsLoading(false);
            setIsRefreshing(false);
        }
    }, [walletAddress, router]);

    useEffect(() => {
        if (isAuthenticated && !authLoading) {
            loadWallet();
        }
    }, [isAuthenticated, authLoading, loadWallet]);

    const handleCopyAddress = async () => {
        if (!wallet) { return; }
        const success = await copyToClipboard(wallet.walletAddress);
        if (success) {
            setCopied(true);
            toast.success('Address copied!');
            setTimeout(() => setCopied(false), 2000);
        }
    };

    const handleSaveMetadata = async () => {
        if (!wallet) { return; }

        setIsSavingMetadata(true);
        try {
            await walletMgmt.updateWalletMetadata(wallet.walletAddress, {
                label: metadataForm.label || null,
                note: metadataForm.note || null,
            });

            toast.success('Wallet metadata updated');
            setIsEditingMetadata(false);
            await loadWallet();
        } catch (err) {
            console.error('Failed to update metadata:', err);
            toast.error('Failed to save changes');
        } finally {
            setIsSavingMetadata(false);
        }
    };

    const startEditing = () => {
        if (wallet) {
            setMetadataForm({
                label: wallet.label || '',
                note: wallet.note || '',
            });
            setIsEditingMetadata(true);
        }
    };

    const cancelEditing = () => {
        setIsEditingMetadata(false);
        if (wallet) {
            setMetadataForm({
                label: wallet.label || '',
                note: wallet.note || '',
            });
        }
    };

    const handleDisableWallet = async (data: DisableWalletData) => {
        if (!wallet) { return; }
        setIsActionLoading(true);
        try {
            await walletMgmt.disableWallet(wallet.walletAddress, {
                duration_days: data.duration === 'until_manual' ? null : data.duration,
                reason_category: data.reasonCategory,
                reason_details: data.reasonDetails,
                affected_platforms: data.affectedPlatforms,
                block_login: data.blockLogin,
                pause_subscriptions: data.pauseSubscriptions,
                notify_user: data.notifyUser,
            });
            toast.success('Wallet disabled successfully');
            setShowDisableModal(false);
            await loadWallet();
        } catch (err) {
            console.error('Failed to disable wallet:', err);
            toast.error(err instanceof Error ? err.message : 'Failed to disable wallet');
        } finally {
            setIsActionLoading(false);
        }
    };

    const handleReenableWallet = async (data: ReenableWalletData) => {
        if (!wallet) { return; }
        setIsActionLoading(true);
        try {
            await walletMgmt.enableWallet(wallet.walletAddress, {
                platforms_to_enable: data.platformsToEnable,
                restore_permissions: data.restorePermissions,
                resume_subscriptions: data.resumeSubscriptions,
                resolution_note: data.resolutionNote,
            });
            toast.success('Wallet re-enabled successfully');
            setShowReenableModal(false);
            await loadWallet();
        } catch (err) {
            console.error('Failed to re-enable wallet:', err);
            toast.error(err instanceof Error ? err.message : 'Failed to re-enable wallet');
        } finally {
            setIsActionLoading(false);
        }
    };

    const statusConfig = wallet ? STATUS_CONFIG[wallet.status] : STATUS_CONFIG.active;
    const isDisabled = wallet?.status === 'disabled';
    const activeSubscriptions = wallet?.subscriptions?.filter(s => s.status === 'active') ?? [];

    // Auth check
    if (authLoading) {
        return (
            <div className="min-h-screen bg-gradient-to-br from-blue-50 via-purple-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
                <div className="max-w-4xl mx-auto">
                    <div className="animate-pulse space-y-6">
                        <div className="h-8 bg-gray-300 dark:bg-gray-700 rounded w-1/3"></div>
                        <div className="h-64 bg-gray-200 dark:bg-gray-800 rounded-2xl"></div>
                    </div>
                </div>
            </div>
        );
    }

    if (!isAuthenticated) {
        return (
            <div className="min-h-screen bg-gradient-to-br from-blue-50 via-purple-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
                <div className="flex items-center justify-center h-full">
                    <div className="text-center max-w-md">
                        <div className="text-4xl mb-4">🔐</div>
                        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                            Authentication Required
                        </h3>
                        <p className="text-gray-600 dark:text-gray-400">
                            Please connect your wallet to access wallet details.
                        </p>
                    </div>
                </div>
            </div>
        );
    }

    if (isLoading) {
        return (
            <div className="min-h-screen bg-gradient-to-br from-blue-50 via-purple-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
                <div className="max-w-4xl mx-auto space-y-6">
                    {/* Back button skeleton */}
                    <Skeleton className="h-10 w-32" />

                    {/* Header skeleton */}
                    <div className="rounded-xl bg-white/80 dark:bg-gray-800/80 p-6">
                        <Skeleton className="h-6 w-96 mb-3" />
                        <div className="flex gap-3">
                            <Skeleton className="h-6 w-20" />
                            <Skeleton className="h-6 w-40" />
                        </div>
                    </div>

                    {/* Content skeleton */}
                    <div className="rounded-xl bg-white/80 dark:bg-gray-800/80 p-6">
                        <Skeleton className="h-6 w-32 mb-4" />
                        <div className="space-y-3">
                            <Skeleton className="h-12 w-full" />
                            <Skeleton className="h-12 w-full" />
                            <Skeleton className="h-12 w-full" />
                        </div>
                    </div>
                </div>
            </div>
        );
    }

    if (!wallet) {
        return (
            <div className="min-h-screen bg-gradient-to-br from-blue-50 via-purple-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
                <div className="max-w-4xl mx-auto text-center py-20">
                    <div className="text-4xl mb-4">🔍</div>
                    <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                        Wallet Not Found
                    </h3>
                    <p className="text-gray-600 dark:text-gray-400 mb-4">
                        The wallet address you're looking for doesn't exist.
                    </p>
                    <Link href="/wallet-management">
                        <Button>Back to Wallet Management</Button>
                    </Link>
                </div>
            </div>
        );
    }

    return (
        <div className="min-h-screen bg-gradient-to-br from-blue-50 via-purple-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-3 sm:p-6">
            <div className="max-w-4xl mx-auto space-y-6">
                {/* Header */}
                <div className="flex items-center gap-4">
                    <Link
                        href="/wallet-management"
                        className="p-2 rounded-xl bg-white/80 dark:bg-gray-800/80 hover:bg-white dark:hover:bg-gray-800 transition-colors border border-gray-200 dark:border-gray-700"
                    >
                        <ArrowLeft className="h-5 w-5" />
                    </Link>
                    <div className="flex-1">
                        <h1 className="text-2xl font-bold text-gray-900 dark:text-white flex items-center gap-2">
                            <span className="text-2xl">👛</span>
                            Wallet Details
                        </h1>
                        <p className="text-sm text-gray-600 dark:text-gray-400">
                            View and manage wallet permissions
                        </p>
                    </div>
                    <Button
                        variant="outline"
                        onClick={loadWallet}
                        disabled={isRefreshing}
                        className="gap-2"
                    >
                        <RefreshCw className={cn('h-4 w-4', isRefreshing && 'animate-spin')} />
                        Refresh
                    </Button>
                </div>

                {/* Address Card */}
                <div className="rounded-xl bg-gradient-to-r from-blue-50 to-purple-50 dark:from-blue-900/20 dark:to-purple-900/20 p-5 border border-blue-200 dark:border-blue-800 bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm">
                    {/* Address */}
                    <div className="flex items-center gap-2 mb-3">
                        <code className="text-sm sm:text-base font-mono font-semibold text-gray-900 dark:text-white break-all">
                            {wallet.walletAddress}
                        </code>
                        <button
                            onClick={handleCopyAddress}
                            className="p-1.5 rounded-lg hover:bg-white/50 dark:hover:bg-gray-800/50 text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 transition-colors"
                            title="Copy address"
                        >
                            <Copy className="h-4 w-4" />
                        </button>
                        {copied && (
                            <span className="text-xs text-green-600 dark:text-green-400">Copied!</span>
                        )}
                    </div>

                    {/* Status & Info */}
                    <div className="flex flex-wrap items-center gap-3">
                        <Badge className={cn('px-3 py-1 border', statusConfig.className)}>
                            {statusConfig.emoji} {statusConfig.label}
                        </Badge>
                        <span className="text-sm text-gray-600 dark:text-gray-400">
                            <Clock className="h-3.5 w-3.5 inline mr-1" />
                            Created {formatTimeAgo(wallet.createdAt)}
                        </span>
                        {wallet.lastAuthAt && (
                            <span className="text-sm text-gray-600 dark:text-gray-400">
                                Last active {formatTimeAgo(wallet.lastAuthAt)}
                            </span>
                        )}
                    </div>

                    {/* Disable Info */}
                    {wallet.disableInfo && (
                        <div className="mt-4 p-3 rounded-lg bg-amber-100/50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800">
                            <p className="text-sm font-medium text-amber-800 dark:text-amber-300">
                                ⚠️ This wallet is temporarily disabled
                            </p>
                            <p className="text-sm text-amber-700 dark:text-amber-400 mt-1">
                                <strong>Reason:</strong> {wallet.disableInfo.reasonDetails}
                            </p>
                            <p className="text-xs text-amber-600 dark:text-amber-500 mt-1">
                                Disabled by {wallet.disableInfo.disabledBy} on {formatDate(wallet.disableInfo.disabledAt)}
                                {wallet.disableInfo.expiresAt && ` • Expires ${formatDate(wallet.disableInfo.expiresAt)}`}
                            </p>
                        </div>
                    )}
                </div>

                {/* Active Subscriptions */}
                {activeSubscriptions.length > 0 && (
                    <div className="rounded-xl bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-gray-200 dark:border-gray-700 p-5">
                        <h3 className="font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
                            <Package className="h-5 w-5 text-purple-600" />
                            Active Subscriptions
                        </h3>
                        <div className="space-y-3">
                            {activeSubscriptions.map((sub) => (
                                <div
                                    key={sub.id}
                                    className="p-4 rounded-xl border border-purple-200 dark:border-purple-800 bg-purple-50/50 dark:bg-purple-900/10"
                                >
                                    <div className="flex items-center justify-between">
                                        <div>
                                            <p className="font-medium text-gray-900 dark:text-white">
                                                📦 {sub.planName}
                                            </p>
                                            <p className="text-sm text-gray-600 dark:text-gray-400">
                                                {sub.priceDisplay} • Since {formatDate(sub.startedAt)}
                                            </p>
                                        </div>
                                        <Badge className="bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400">
                                            Active
                                        </Badge>
                                    </div>
                                    {sub.grantedPermissions.length > 0 && (
                                        <div className="mt-2 pt-2 border-t border-purple-200 dark:border-purple-700">
                                            <p className="text-xs text-gray-500 dark:text-gray-400 mb-1">
                                                Auto-grants:
                                            </p>
                                            <div className="flex flex-wrap gap-1">
                                                {sub.grantedPermissions.map((perm) => (
                                                    <code
                                                        key={perm}
                                                        className="text-xs px-1.5 py-0.5 bg-white dark:bg-gray-800 rounded border border-gray-200 dark:border-gray-700"
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

                {/* Access Permissions - Unified Manager */}
                <WalletAccessManager
                    walletAddress={wallet.walletAddress}
                    onSaveComplete={loadWallet}
                />

                {/* Quick Actions & Metadata */}
                <div className="rounded-xl bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 p-6 flex items-center justify-between">
                    <div className="flex flex-col gap-1">
                        <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                            Quick Actions & Metadata
                        </h3>

                        {/* Metadata Display */}
                        {!isEditingMetadata && (
                            <div className="flex items-center gap-4 mt-1">
                                <div className="flex items-center gap-3 bg-gray-50 dark:bg-gray-900/50 rounded-full px-4 py-1.5 border border-gray-100 dark:border-gray-800">
                                    <div className="flex items-center gap-2">
                                        <span className="text-sm text-gray-500 font-medium">Label</span>
                                        {wallet.label ? (
                                            <span className="px-2 py-0.5 bg-white dark:bg-gray-800 rounded-full text-sm border border-gray-200 dark:border-gray-700 shadow-sm">
                                                {wallet.label}
                                            </span>
                                        ) : (
                                            <span className="text-sm text-gray-400 italic">None</span>
                                        )}
                                    </div>
                                    <div className="w-px h-4 bg-gray-300 dark:bg-gray-700" />
                                    <div className="flex items-center gap-2">
                                        <span className="text-sm text-gray-500 font-medium">Note</span>
                                        <span className="text-sm text-gray-700 dark:text-gray-300 max-w-[300px] truncate">
                                            {wallet.note || <span className="text-gray-400 italic">No notes</span>}
                                        </span>
                                    </div>
                                </div>
                            </div>
                        )}

                        {/* Inline Metadata Editor */}
                        {isEditingMetadata && (
                            <div className="mt-2 bg-gray-50 dark:bg-gray-900/40 p-4 rounded-lg border border-gray-200 dark:border-gray-700 space-y-4 animate-in fade-in slide-in-from-top-2 w-full max-w-2xl">
                                <div className="space-y-2">
                                    <Label htmlFor="edit-label">Wallet Label</Label>
                                    <Input
                                        id="edit-label"
                                        placeholder="e.g. VIP, Whale (max 20 chars)"
                                        value={metadataForm.label}
                                        onChange={(e) => setMetadataForm(prev => ({ ...prev, label: e.target.value.slice(0, 20) }))}
                                        className="bg-white dark:bg-gray-800"
                                    />
                                </div>
                                <div className="space-y-2">
                                    <Label htmlFor="edit-note">Internal Note</Label>
                                    <Textarea
                                        id="edit-note"
                                        placeholder="Add details about this wallet..."
                                        value={metadataForm.note}
                                        onChange={(e) => setMetadataForm(prev => ({ ...prev, note: e.target.value.slice(0, 500) }))}
                                        className="bg-white dark:bg-gray-800 resize-none h-24"
                                    />
                                    <p className="text-xs text-right text-gray-500">
                                        {metadataForm.note.length}/500
                                    </p>
                                </div>
                                <div className="flex justify-end gap-2 pt-2">
                                    <Button
                                        variant="ghost"
                                        size="sm"
                                        onClick={cancelEditing}
                                        disabled={isSavingMetadata}
                                        className="h-8"
                                    >
                                        Cancel
                                    </Button>
                                    <Button
                                        size="sm"
                                        onClick={handleSaveMetadata}
                                        disabled={isSavingMetadata}
                                        className="h-8 gap-2"
                                    >
                                        {isSavingMetadata ? (
                                            <Loader2 className="h-3.5 w-3.5 animate-spin" />
                                        ) : (
                                            <Save className="h-3.5 w-3.5" />
                                        )}
                                        Save Changes
                                    </Button>
                                </div>
                            </div>
                        )}
                    </div>

                    {!isEditingMetadata && (
                        <div className="flex items-center gap-3">
                            <Button
                                variant="outline"
                                onClick={startEditing}
                                className="h-10 px-4 border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 text-gray-700 dark:text-gray-200"
                            >
                                <Edit className="h-4 w-4 mr-2 text-blue-500" />
                                Edit Label/Note
                            </Button>

                            {isDisabled ? (
                                <Button
                                    variant="outline"
                                    onClick={() => setShowReenableModal(true)}
                                    className="h-10 px-4 border-green-200 dark:border-green-800 hover:bg-green-50 dark:hover:bg-green-900/20 text-green-700 dark:text-green-400"
                                >
                                    🔓 Re-enable Wallet
                                </Button>
                            ) : (
                                <Button
                                    variant="outline"
                                    onClick={() => setShowDisableModal(true)}
                                    className="h-10 px-4 border-amber-200 dark:border-amber-800 hover:bg-amber-50 dark:hover:bg-amber-900/20 text-amber-700 dark:text-amber-400"
                                >
                                    <AlertTriangle className="h-4 w-4 mr-2" />
                                    Disable Wallet
                                </Button>
                            )}

                            <Button
                                variant="outline"
                                onClick={() => window.open(`https://bscscan.com/address/${wallet.walletAddress}`, '_blank')}
                                className="h-10 px-4 border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 text-gray-700 dark:text-gray-200"
                            >
                                <ExternalLink className="h-4 w-4 mr-2" />
                                View on BSCScan
                            </Button>
                        </div>
                    )}
                </div>

                {/* Activity History */}
                {activityEvents.length > 0 && (
                    <div className="rounded-xl bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-gray-200 dark:border-gray-700 p-5">
                        <h3 className="font-semibold text-gray-900 dark:text-white mb-4">
                            📜 Activity History
                        </h3>
                        <WalletActivityTimeline events={activityEvents} maxItems={10} />
                    </div>
                )}

                {/* Back Button */}
                <div className="flex gap-4">
                    <Link href="/wallet-management" className="flex-1">
                        <Button variant="outline" className="w-full">
                            <ArrowLeft className="h-4 w-4 mr-2" />
                            Back to Wallet Management
                        </Button>
                    </Link>
                </div>
            </div>

            {/* Disable Modal */}
            {showDisableModal && (
                <DisableWalletModal
                    walletAddress={wallet.walletAddress}
                    isOpen={true}
                    onClose={() => setShowDisableModal(false)}
                    onConfirm={handleDisableWallet}
                    isLoading={isActionLoading}
                />
            )}

            {/* Re-enable Modal */}
            {showReenableModal && wallet.disableInfo && (
                <ReenableWalletModal
                    walletAddress={wallet.walletAddress}
                    disableInfo={wallet.disableInfo}
                    isOpen={true}
                    onClose={() => setShowReenableModal(false)}
                    onConfirm={handleReenableWallet}
                    isLoading={isActionLoading}
                />
            )}
        </div>
    );
}
