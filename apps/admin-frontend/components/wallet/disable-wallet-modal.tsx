/**
 * Disable Wallet Modal Component
 * Modal for temporarily disabling a wallet with reason
 */
'use client';

import { AlertTriangle } from 'lucide-react';
import { useState } from 'react';

import type { DisableReasonCategory, Platform } from './types';

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
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';
import { Textarea } from '@/components/ui/textarea';
import { cn } from '@/lib/utils';

interface DisableWalletModalProps {
    walletAddress: string;
    isOpen: boolean;
    onClose: () => void;
    onConfirm: (data: DisableWalletData) => Promise<void>;
    isLoading?: boolean;
}

export interface DisableWalletData {
    walletAddress: string;
    duration: 'until_manual' | number; // number = days
    reasonCategory: DisableReasonCategory;
    reasonDetails: string;
    affectedPlatforms: Platform[];
    blockLogin: boolean;
    pauseSubscriptions: boolean;
    notifyUser: boolean;
}

const DURATION_OPTIONS = [
    { value: '1', label: '24 hours' },
    { value: '7', label: '7 days' },
    { value: '30', label: '30 days' },
    { value: '90', label: '90 days' },
    { value: 'until_manual', label: 'Until manually re-enabled' },
];

const REASON_CATEGORIES: { value: DisableReasonCategory; label: string; emoji: string }[] = [
    { value: 'suspicious_activity', label: 'Suspicious Activity', emoji: '🔍' },
    { value: 'tos_violation', label: 'Terms of Service Violation', emoji: '📜' },
    { value: 'pending_verification', label: 'Pending Verification', emoji: '✅' },
    { value: 'user_request', label: 'User Request', emoji: '👤' },
    { value: 'other', label: 'Other', emoji: '📝' },
];

const ALL_PLATFORMS: { value: Platform; label: string; emoji: string }[] = [
    { value: 'analytics', label: 'EPSX Analytics', emoji: '📊' },
    { value: 'pay', label: 'EPSX Pay', emoji: '💳' },
    { value: 'token', label: 'EPSX Token', emoji: '🪙' },
    { value: 'markets', label: 'EPSX Markets', emoji: '📈' },
];

/**
 *
 * @param root0
 * @param root0.walletAddress
 * @param root0.isOpen
 * @param root0.onClose
 * @param root0.onConfirm
 * @param root0.isLoading
 */
interface DisableModalBodyProps {
    duration: string;
    affectedPlatforms: Platform[];
    reasonCategory: DisableReasonCategory;
    reasonDetails: string;
    blockLogin: boolean;
    pauseSubscriptions: boolean;
    notifyUser: boolean;
    error: string;
    onDurationChange: (v: string) => void;
    onTogglePlatform: (p: Platform) => void;
    onReasonCategoryChange: (v: DisableReasonCategory) => void;
    onReasonDetailsChange: (v: string) => void;
    onBlockLoginChange: (v: boolean) => void;
    onPauseSubscriptionsChange: (v: boolean) => void;
    onNotifyUserChange: (v: boolean) => void;
}

function DisableModalBody({
    duration, affectedPlatforms, reasonCategory, reasonDetails,
    blockLogin, pauseSubscriptions, notifyUser, error,
    onDurationChange, onTogglePlatform, onReasonCategoryChange,
    onReasonDetailsChange, onBlockLoginChange, onPauseSubscriptionsChange, onNotifyUserChange,
}: DisableModalBodyProps) {
    return (
        <div className="space-y-5 py-4">
            <div>
                <Label className="text-sm font-medium">Disable Duration</Label>
                <div className="mt-2 grid grid-cols-2 sm:grid-cols-3 gap-2">
                    {DURATION_OPTIONS.map((option) => (
                        <button
                            key={option.value}
                            onClick={() => onDurationChange(option.value)}
                            className={cn(
                                'px-3 py-2 text-sm rounded-lg border transition-colors',
                                duration === option.value
                                    ? 'border-amber-500 bg-amber-50 text-amber-700 dark:bg-amber-900/20 dark:text-amber-400'
                                    : 'border-border hover:border-border/80'
                            )}
                        >
                            {option.label}
                        </button>
                    ))}
                </div>
            </div>
            <div>
                <Label className="text-sm font-medium">Affected Platforms</Label>
                <div className="mt-2 grid grid-cols-2 gap-2">
                    {ALL_PLATFORMS.map((platform) => (
                        <label
                            key={platform.value}
                            className={cn(
                                'flex items-center gap-3 px-3 py-2 rounded-lg border cursor-pointer',
                                affectedPlatforms.includes(platform.value)
                                    ? 'border-amber-500 bg-amber-50 dark:bg-amber-900/20'
                                    : 'border-border hover:border-border/80'
                            )}
                        >
                            <Checkbox checked={affectedPlatforms.includes(platform.value)} onCheckedChange={() => onTogglePlatform(platform.value)} />
                            <span className="flex items-center gap-2 text-sm">{platform.emoji} {platform.label}</span>
                        </label>
                    ))}
                </div>
            </div>
            <div>
                <Label className="text-sm font-medium">Reason Category</Label>
                <Select value={reasonCategory} onValueChange={(v) => onReasonCategoryChange(v as DisableReasonCategory)}>
                    <SelectTrigger className="mt-1.5"><SelectValue /></SelectTrigger>
                    <SelectContent>
                        {REASON_CATEGORIES.map((cat) => (
                            <SelectItem key={cat.value} value={cat.value}>
                                <span className="flex items-center gap-2">{cat.emoji} {cat.label}</span>
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>
            </div>
            <div>
                <Label className="text-sm font-medium">Details (Required)</Label>
                <Textarea value={reasonDetails} onChange={(e) => onReasonDetailsChange(e.target.value)} placeholder="Provide specific details about why this wallet is being disabled..." className="mt-1.5 min-h-[80px]" />
            </div>
            <div className="space-y-3">
                <label className="flex items-center gap-3 cursor-pointer">
                    <Checkbox checked={blockLogin} onCheckedChange={(c) => onBlockLoginChange(c === true)} />
                    <span className="text-sm">Block login across all platforms</span>
                </label>
                <label className="flex items-center gap-3 cursor-pointer">
                    <Checkbox checked={pauseSubscriptions} onCheckedChange={(c) => onPauseSubscriptionsChange(c === true)} />
                    <span className="text-sm">Pause active subscriptions (billing paused)</span>
                </label>
                <label className="flex items-center gap-3 cursor-pointer">
                    <Checkbox checked={notifyUser} onCheckedChange={(c) => onNotifyUserChange(c === true)} />
                    <span className="text-sm">Send notification to user (if email registered)</span>
                </label>
            </div>
            {error !== '' && <p className="text-sm text-destructive">⚠️ {error}</p>}
            <div className="p-3 rounded-lg bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800">
                <p className="text-sm text-amber-700 dark:text-amber-400">
                    ⚠️ Wallet will be unable to access selected platforms until the disable period ends or an admin re-enables access.
                </p>
            </div>
        </div>
    );
}

export function DisableWalletModal({
    walletAddress,
    isOpen,
    onClose,
    onConfirm,
    isLoading = false,
}: DisableWalletModalProps) {
    const [duration, setDuration] = useState<string>('until_manual');
    const [reasonCategory, setReasonCategory] = useState<DisableReasonCategory>('suspicious_activity');
    const [reasonDetails, setReasonDetails] = useState('');
    const [affectedPlatforms, setAffectedPlatforms] = useState<Platform[]>(['analytics', 'pay', 'token', 'markets']);
    const [blockLogin, setBlockLogin] = useState(true);
    const [pauseSubscriptions, setPauseSubscriptions] = useState(false);
    const [notifyUser, setNotifyUser] = useState(false);
    const [error, setError] = useState('');

    const togglePlatform = (platform: Platform) => {
        setAffectedPlatforms(prev => prev.includes(platform) ? prev.filter(p => p !== platform) : [...prev, platform]);
    };

    const handleSubmit = async () => {
        setError('');
        if (reasonDetails.trim() === '') { setError('Please provide details about the reason'); return; }
        if (affectedPlatforms.length === 0) { setError('Please select at least one platform'); return; }
        try {
            await onConfirm({ walletAddress, duration: duration === 'until_manual' ? 'until_manual' : Number.parseInt(duration, 10), reasonCategory, reasonDetails: reasonDetails.trim(), affectedPlatforms, blockLogin, pauseSubscriptions, notifyUser });
            onClose();
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to disable wallet');
        }
    };

    const handleClose = () => {
        setDuration('until_manual');
        setReasonCategory('suspicious_activity');
        setReasonDetails('');
        setAffectedPlatforms(['analytics', 'pay', 'token', 'markets']);
        setBlockLogin(true);
        setPauseSubscriptions(false);
        setNotifyUser(false);
        setError('');
        onClose();
    };

    return (
        <Dialog open={isOpen} onOpenChange={(open) => { if (!open) { handleClose(); } }}>
            <DialogContent className="sm:max-w-lg">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2 text-amber-600 dark:text-amber-400">
                        <AlertTriangle className="h-5 w-5" />
                        Disable Wallet
                    </DialogTitle>
                    <DialogDescription>
                        Disable access for wallet{' '}
                        <code className="px-1.5 py-0.5 bg-muted rounded text-sm font-mono">
                            {walletAddress.slice(0, 10)}...{walletAddress.slice(-6)}
                        </code>
                    </DialogDescription>
                </DialogHeader>
                <DisableModalBody
                    duration={duration} affectedPlatforms={affectedPlatforms}
                    reasonCategory={reasonCategory} reasonDetails={reasonDetails}
                    blockLogin={blockLogin} pauseSubscriptions={pauseSubscriptions}
                    notifyUser={notifyUser} error={error}
                    onDurationChange={setDuration} onTogglePlatform={togglePlatform}
                    onReasonCategoryChange={setReasonCategory} onReasonDetailsChange={setReasonDetails}
                    onBlockLoginChange={setBlockLogin} onPauseSubscriptionsChange={setPauseSubscriptions}
                    onNotifyUserChange={setNotifyUser}
                />
                <DialogFooter className="gap-2 sm:gap-0">
                    <Button variant="outline" onClick={handleClose} disabled={isLoading}>Cancel</Button>
                    <Button onClick={() => { void handleSubmit(); }} disabled={isLoading} className="bg-amber-600 hover:bg-amber-700 text-white">
                        {isLoading ? <><span className="animate-spin mr-2">⏳</span>Disabling...</> : <><AlertTriangle className="h-4 w-4 mr-2" />Disable Wallet</>}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
