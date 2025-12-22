/**
 * Wallet Detail Page
 * Full page view for comprehensive wallet information and management
 */
'use client';

import { ArrowLeft, Clock, Copy, ExternalLink, Package, RefreshCw, Save, Shield } from 'lucide-react';
import Link from 'next/link';
import { useParams, useRouter } from 'next/navigation';
import { useCallback, useEffect, useState } from 'react';
import { toast } from 'sonner';

import { PermissionTransferList } from '@/components/groups/PermissionTransferList';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Skeleton } from '@/components/ui/skeleton';
import { DisableWalletModal, type DisableWalletData } from '@/components/wallet/DisableWalletModal';
import { ReenableWalletModal, type ReenableWalletData } from '@/components/wallet/ReenableWalletModal';
import { WalletActivityTimeline } from '@/components/wallet/WalletActivityTimeline';
import type { WalletActivityEvent, WalletData, WalletStatus } from '@/components/wallet/types';
import { walletMgmt } from '@/lib/api/wallet-management-client';
import { cn } from '@/lib/utils';
import { useSharedAuth } from '@/shared/components/auth/Provider';

// All available permissions in the system
const ALL_AVAILABLE_PERMISSIONS = [
    'epsx:analytics:view',
    'epsx:analytics:advanced',
    'epsx:trading:basic',
    'epsx:trading:advanced',
    'epsx:trading:pro',
    'epsx:data:export',
    'epsx:api:read',
    'epsx:api:write',
    'epsx:notifications:manage',
    'epsx:markets:view',
    'epsx:markets:alerts',
    'epsx:pay:basic',
    'epsx:pay:advanced',
    'epsx:token:view',
    'epsx:token:transfer',
];

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

    if (diffInHours < 1) return 'Just now';
    if (diffInHours < 24) return `${diffInHours} hours ago`;

    const diffInDays = Math.floor(diffInHours / 24);
    if (diffInDays === 1) return 'Yesterday';
    if (diffInDays < 30) return `${diffInDays} days ago`;

    return date.toLocaleDateString();
}

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
    const [isSavingPermissions, setIsSavingPermissions] = useState(false);

    // Permission management state
    const [selectedPermissions, setSelectedPermissions] = useState<string[]>([]);
    const [hasPermissionChanges, setHasPermissionChanges] = useState(false);

    // Modals
    const [showDisableModal, setShowDisableModal] = useState(false);
    const [showReenableModal, setShowReenableModal] = useState(false);
    const [isActionLoading, setIsActionLoading] = useState(false);

    // Load wallet data
    const loadWallet = useCallback(async () => {
        if (!walletAddress) return;

        try {
            setIsRefreshing(true);
            const [walletData, events] = await Promise.all([
                walletMgmt.fetchWalletDetail(walletAddress),
                walletMgmt.fetchActivityHistory(walletAddress),
            ]);
            setWallet(walletData);
            setActivityEvents(events);
            // Initialize selected permissions from wallet data
            const currentPermissions = walletData.permissions.map(p => p.permission);
            setSelectedPermissions(currentPermissions);
            setHasPermissionChanges(false);
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
        if (!wallet) return;
        await navigator.clipboard.writeText(wallet.walletAddress);
        setCopied(true);
        toast.success('Address copied!');
        setTimeout(() => setCopied(false), 2000);
    };

    // Handle permission changes from PermissionTransferList
    const handlePermissionsChange = useCallback((newPermissions: string[]) => {
        setSelectedPermissions(newPermissions);
        // Check if permissions have changed from original
        const originalPermissions = wallet?.permissions.map(p => p.permission) ?? [];
        const hasChanged =
            newPermissions.length !== originalPermissions.length ||
            newPermissions.some(p => !originalPermissions.includes(p)) ||
            originalPermissions.some(p => !newPermissions.includes(p));
        setHasPermissionChanges(hasChanged);
    }, [wallet]);

    // Save permission changes
    const handleSavePermissions = async () => {
        if (!wallet || !hasPermissionChanges) return;

        setIsSavingPermissions(true);
        try {
            const originalPermissions = wallet.permissions.map(p => p.permission);
            const toAdd = selectedPermissions.filter(p => !originalPermissions.includes(p));
            const toRemove = originalPermissions.filter(p => !selectedPermissions.includes(p));

            // TODO: Call API to update permissions
            console.log('Permissions to add:', toAdd);
            console.log('Permissions to remove:', toRemove);

            // Simulate API call
            await new Promise(resolve => setTimeout(resolve, 1000));

            toast.success(`Permissions updated: +${toAdd.length} / -${toRemove.length}`);
            setHasPermissionChanges(false);
            await loadWallet();
        } catch (err) {
            toast.error('Failed to save permission changes');
        } finally {
            setIsSavingPermissions(false);
        }
    };

    const handleDisableWallet = async (data: DisableWalletData) => {
        setIsActionLoading(true);
        try {
            // TODO: Call API to disable wallet
            console.log('Disable wallet:', data);
            await new Promise(resolve => setTimeout(resolve, 1000));
            toast.success('Wallet disabled successfully');
            setShowDisableModal(false);
            await loadWallet();
        } catch (err) {
            toast.error('Failed to disable wallet');
        } finally {
            setIsActionLoading(false);
        }
    };

    const handleReenableWallet = async (data: ReenableWalletData) => {
        setIsActionLoading(true);
        try {
            // TODO: Call API to re-enable wallet
            console.log('Re-enable wallet:', data);
            await new Promise(resolve => setTimeout(resolve, 1000));
            toast.success('Wallet re-enabled successfully');
            setShowReenableModal(false);
            await loadWallet();
        } catch (err) {
            toast.error('Failed to re-enable wallet');
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

                {/* Permissions - Drag and Drop Transfer List */}
                <div className="rounded-xl bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-gray-200 dark:border-gray-700 p-5">
                    <div className="flex items-center justify-between mb-4">
                        <h3 className="font-semibold text-gray-900 dark:text-white flex items-center gap-2">
                            <Shield className="h-5 w-5 text-blue-600" />
                            Access Permissions
                        </h3>
                        {hasPermissionChanges && (
                            <Button
                                onClick={handleSavePermissions}
                                disabled={isSavingPermissions}
                                className="gap-2 bg-green-600 hover:bg-green-700 text-white"
                            >
                                {isSavingPermissions ? (
                                    <RefreshCw className="h-4 w-4 animate-spin" />
                                ) : (
                                    <Save className="h-4 w-4" />
                                )}
                                Save Changes
                            </Button>
                        )}
                    </div>
                    <PermissionTransferList
                        available={ALL_AVAILABLE_PERMISSIONS}
                        selected={selectedPermissions}
                        onChange={handlePermissionsChange}
                    />
                </div>

                {/* Quick Actions */}
                <div className="rounded-xl bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-gray-200 dark:border-gray-700 p-5">
                    <h3 className="font-semibold text-gray-900 dark:text-white mb-4">
                        Quick Actions
                    </h3>
                    <div className="flex flex-wrap gap-3">
                        {isDisabled ? (
                            <Button
                                variant="outline"
                                onClick={() => setShowReenableModal(true)}
                                className="border-green-300 text-green-700 hover:bg-green-50 dark:border-green-700 dark:text-green-400 dark:hover:bg-green-900/20"
                            >
                                🔓 Re-enable Access
                            </Button>
                        ) : (
                            <Button
                                variant="outline"
                                onClick={() => setShowDisableModal(true)}
                                className="border-amber-300 text-amber-700 hover:bg-amber-50 dark:border-amber-700 dark:text-amber-400 dark:hover:bg-amber-900/20"
                            >
                                ⚠️ Temporarily Disable
                            </Button>
                        )}
                        <Button
                            variant="outline"
                            onClick={() => window.open(`https://bscscan.com/address/${wallet.walletAddress}`, '_blank')}
                        >
                            <ExternalLink className="h-4 w-4 mr-2" />
                            View on BSCScan
                        </Button>
                    </div>
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
