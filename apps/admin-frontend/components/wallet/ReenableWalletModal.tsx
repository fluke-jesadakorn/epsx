/**
 * Re-enable Wallet Modal Component
 * Modal for re-enabling a disabled wallet
 */
'use client';

import { Unlock } from 'lucide-react';
import { useState } from 'react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { cn } from '@/lib/utils';

import type { Platform, WalletDisableInfo } from './types';

interface ReenableWalletModalProps {
    walletAddress: string;
    disableInfo: WalletDisableInfo;
    isOpen: boolean;
    onClose: () => void;
    onConfirm: (data: ReenableWalletData) => Promise<void>;
    isLoading?: boolean;
}

export interface ReenableWalletData {
    walletAddress: string;
    platformsToEnable: Platform[];
    restorePermissions: boolean;
    resumeSubscriptions: boolean;
    resolutionNote: string;
}

const ALL_PLATFORMS: { value: Platform; label: string; emoji: string }[] = [
    { value: 'analytics', label: 'EPSX Analytics', emoji: '📊' },
    { value: 'pay', label: 'EPSX Pay', emoji: '💳' },
    { value: 'token', label: 'EPSX Token', emoji: '🪙' },
    { value: 'markets', label: 'EPSX Markets', emoji: '📈' },
];

const REASON_LABELS: Record<string, string> = {
    suspicious_activity: 'Suspicious Activity',
    tos_violation: 'Terms of Service Violation',
    pending_verification: 'Pending Verification',
    user_request: 'User Request',
    other: 'Other',
};

function formatDate(timestamp: string): string {
    return new Date(timestamp).toLocaleDateString('en-US', {
        month: 'short',
        day: 'numeric',
        year: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
    });
}

export function ReenableWalletModal({
    walletAddress,
    disableInfo,
    isOpen,
    onClose,
    onConfirm,
    isLoading = false,
}: ReenableWalletModalProps) {
    const [platformsToEnable, setPlatformsToEnable] = useState<Platform[]>(
        disableInfo.affectedPlatforms
    );
    const [restorePermissions, setRestorePermissions] = useState(true);
    const [resumeSubscriptions, setResumeSubscriptions] = useState(true);
    const [resolutionNote, setResolutionNote] = useState('');
    const [error, setError] = useState('');

    const togglePlatform = (platform: Platform) => {
        setPlatformsToEnable(prev =>
            prev.includes(platform)
                ? prev.filter(p => p !== platform)
                : [...prev, platform]
        );
    };

    const handleSubmit = async () => {
        setError('');

        if (!resolutionNote.trim()) {
            setError('Please provide a resolution note');
            return;
        }

        if (platformsToEnable.length === 0) {
            setError('Please select at least one platform to re-enable');
            return;
        }

        try {
            await onConfirm({
                walletAddress,
                platformsToEnable,
                restorePermissions,
                resumeSubscriptions,
                resolutionNote: resolutionNote.trim(),
            });
            onClose();
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to re-enable wallet');
        }
    };

    const handleClose = () => {
        setResolutionNote('');
        setError('');
        onClose();
    };

    return (
        <Dialog open={isOpen} onOpenChange={(open) => !open && handleClose()}>
            <DialogContent className="sm:max-w-lg">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2 text-green-600 dark:text-green-400">
                        <Unlock className="h-5 w-5" />
                        Re-enable Wallet Access
                    </DialogTitle>
                    <DialogDescription>
                        Restore access for wallet{' '}
                        <code className="px-1.5 py-0.5 bg-gray-100 dark:bg-gray-800 rounded text-sm font-mono">
                            {walletAddress.slice(0, 10)}...{walletAddress.slice(-6)}
                        </code>
                    </DialogDescription>
                </DialogHeader>

                <div className="space-y-5 py-4">
                    {/* Current Disable Info */}
                    <div className="p-4 rounded-xl bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800">
                        <h4 className="text-sm font-medium text-amber-800 dark:text-amber-300 mb-2">
                            Current Disable Information
                        </h4>
                        <div className="space-y-1.5 text-sm text-amber-700 dark:text-amber-400">
                            <p>
                                <strong>Disabled by:</strong> {disableInfo.disabledBy}
                            </p>
                            <p>
                                <strong>Disabled on:</strong> {formatDate(disableInfo.disabledAt)}
                            </p>
                            <p>
                                <strong>Reason:</strong> {REASON_LABELS[disableInfo.reasonCategory] || disableInfo.reasonCategory}
                            </p>
                            <p>
                                <strong>Details:</strong> {disableInfo.reasonDetails}
                            </p>
                            {disableInfo.expiresAt && (
                                <p>
                                    <strong>Scheduled expiry:</strong> {formatDate(disableInfo.expiresAt)}
                                </p>
                            )}
                        </div>
                    </div>

                    {/* Platforms to Re-enable */}
                    <div>
                        <Label className="text-sm font-medium">Platforms to Re-enable</Label>
                        <div className="mt-2 grid grid-cols-2 gap-2">
                            {ALL_PLATFORMS.map((platform) => {
                                const wasAffected = disableInfo.affectedPlatforms.includes(platform.value);

                                return (
                                    <label
                                        key={platform.value}
                                        className={cn(
                                            'flex items-center gap-3 px-3 py-2 rounded-lg border cursor-pointer',
                                            platformsToEnable.includes(platform.value)
                                                ? 'border-green-500 bg-green-50 dark:bg-green-900/20'
                                                : 'border-gray-200 dark:border-gray-700 hover:border-gray-300',
                                            !wasAffected && 'opacity-50'
                                        )}
                                    >
                                        <Checkbox
                                            checked={platformsToEnable.includes(platform.value)}
                                            onCheckedChange={() => togglePlatform(platform.value)}
                                            disabled={!wasAffected}
                                        />
                                        <span className="flex items-center gap-2 text-sm">
                                            {platform.emoji} {platform.label}
                                            {!wasAffected && (
                                                <Badge variant="secondary" className="text-xs">
                                                    Not Affected
                                                </Badge>
                                            )}
                                        </span>
                                    </label>
                                );
                            })}
                        </div>
                    </div>

                    {/* Options */}
                    <div className="space-y-3">
                        <label className="flex items-center gap-3 cursor-pointer">
                            <Checkbox
                                checked={restorePermissions}
                                onCheckedChange={(checked) => setRestorePermissions(checked === true)}
                            />
                            <span className="text-sm">Restore all previous permissions</span>
                        </label>
                        <label className="flex items-center gap-3 cursor-pointer">
                            <Checkbox
                                checked={resumeSubscriptions}
                                onCheckedChange={(checked) => setResumeSubscriptions(checked === true)}
                            />
                            <span className="text-sm">Resume paused subscriptions</span>
                        </label>
                    </div>

                    {/* Resolution Note */}
                    <div>
                        <Label className="text-sm font-medium">Resolution Note (Required)</Label>
                        <Textarea
                            value={resolutionNote}
                            onChange={(e) => setResolutionNote(e.target.value)}
                            placeholder="e.g., Investigation complete - false positive, user verified identity..."
                            className="mt-1.5 min-h-[80px]"
                        />
                    </div>

                    {error && (
                        <p className="text-sm text-red-600 dark:text-red-400">
                            ⚠️ {error}
                        </p>
                    )}
                </div>

                <DialogFooter className="gap-2 sm:gap-0">
                    <Button variant="outline" onClick={handleClose} disabled={isLoading}>
                        Cancel
                    </Button>
                    <Button
                        onClick={handleSubmit}
                        disabled={isLoading}
                        className="bg-green-600 hover:bg-green-700 text-white"
                    >
                        {isLoading ? (
                            <>
                                <span className="animate-spin mr-2">⏳</span>
                                Re-enabling...
                            </>
                        ) : (
                            <>
                                <Unlock className="h-4 w-4 mr-2" />
                                Re-enable Access
                            </>
                        )}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
