/**
 * Assign Permission Form Component
 * Form for manually assigning permissions to a wallet
 */
'use client';

import { Plus } from 'lucide-react';
import React, { useState } from 'react';

import type { Platform } from './types';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';
import { cn } from '@/lib/utils';

interface AssignPermissionFormProps {
    walletAddress: string;
    onAssign: (data: AssignPermissionData) => Promise<void>;
    isLoading?: boolean;
    className?: string;
}

export interface AssignPermissionData {
    walletAddress: string;
    platform: Platform;
    permission: string;
    duration: 'permanent' | 'custom';
    expiresAt?: string;
    reason: string;
}

// Permission options per platform
const PERMISSION_OPTIONS: Record<Platform, { value: string; label: string }[]> = {
    analytics: [
        { value: 'epsx:analytics:read', label: 'Analytics Read' },
        { value: 'epsx:analytics:export', label: 'Analytics Export' },
        { value: 'epsx:rankings:view', label: 'Rankings View' },
        { value: 'epsx:rankings:export', label: 'Rankings Export' },
        { value: 'epsx:analytics:premium', label: 'Premium Analytics' },
    ],
    pay: [
        { value: 'epsx-pay:transfers:read', label: 'View Transfers' },
        { value: 'epsx-pay:transfers:write', label: 'Create Transfers' },
        { value: 'epsx-pay:settings:manage', label: 'Manage Settings' },
    ],
    token: [
        { value: 'epsx-token:balance:view', label: 'View Balance' },
        { value: 'epsx-token:staking:manage', label: 'Manage Staking' },
        { value: 'epsx-token:governance:vote', label: 'Governance Voting' },
    ],
    markets: [
        { value: 'epsx-markets:data:view', label: 'View Market Data' },
        { value: 'epsx-markets:alerts:manage', label: 'Manage Alerts' },
        { value: 'epsx-markets:premium:access', label: 'Premium Access' },
    ],
};

const PLATFORM_OPTIONS: { value: Platform; label: string; emoji: string }[] = [
    { value: 'analytics', label: 'EPSX Analytics', emoji: '📊' },
    { value: 'pay', label: 'EPSX Pay', emoji: '💳' },
    { value: 'token', label: 'EPSX Token', emoji: '🪙' },
    { value: 'markets', label: 'EPSX Markets', emoji: '📈' },
];

const DURATION_OPTIONS = [
    { value: 'permanent', label: 'Permanent' },
    { value: '7', label: '7 days' },
    { value: '30', label: '30 days' },
    { value: '90', label: '90 days' },
    { value: '365', label: '1 year' },
    { value: 'custom', label: 'Custom date' },
];

/**
 *
 * @param root0
 * @param root0.walletAddress
 * @param root0.onAssign
 * @param root0.isLoading
 * @param root0.className
 */
interface FormFieldsProps {
    platform: Platform;
    permission: string;
    duration: string;
    customDate: string;
    reason: string;
    availablePermissions: { value: string; label: string }[];
    onPlatformChange: (v: Platform) => void;
    onPermissionChange: (v: string) => void;
    onDurationChange: (v: string) => void;
    onCustomDateChange: (v: string) => void;
    onReasonChange: (v: string) => void;
}

function PermissionFormFields({
    platform, permission, duration, customDate, reason,
    availablePermissions, onPlatformChange, onPermissionChange,
    onDurationChange, onCustomDateChange, onReasonChange,
}: FormFieldsProps) {
    return (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
                <Label htmlFor="platform" className="text-sm font-medium">Platform</Label>
                <Select value={platform} onValueChange={(v) => onPlatformChange(v as Platform)}>
                    <SelectTrigger className="mt-1.5 bg-card"><SelectValue /></SelectTrigger>
                    <SelectContent>
                        {PLATFORM_OPTIONS.map((opt) => (
                            <SelectItem key={opt.value} value={opt.value}>
                                <span className="flex items-center gap-2">{opt.emoji} {opt.label}</span>
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>
            </div>
            <div>
                <Label htmlFor="permission" className="text-sm font-medium">Permission</Label>
                <Select value={permission} onValueChange={onPermissionChange}>
                    <SelectTrigger className="mt-1.5 bg-card"><SelectValue placeholder="Select permission" /></SelectTrigger>
                    <SelectContent>
                        {availablePermissions.map((opt) => (
                            <SelectItem key={opt.value} value={opt.value}>{opt.label}</SelectItem>
                        ))}
                    </SelectContent>
                </Select>
            </div>
            <div>
                <Label htmlFor="duration" className="text-sm font-medium">Duration</Label>
                <Select value={duration} onValueChange={onDurationChange}>
                    <SelectTrigger className="mt-1.5 bg-card"><SelectValue /></SelectTrigger>
                    <SelectContent>
                        {DURATION_OPTIONS.map((opt) => (
                            <SelectItem key={opt.value} value={opt.value}>{opt.label}</SelectItem>
                        ))}
                    </SelectContent>
                </Select>
            </div>
            {duration === 'custom' && (
                <div>
                    <Label htmlFor="customDate" className="text-sm font-medium">Expiry Date</Label>
                    <Input
                        type="date"
                        id="customDate"
                        value={customDate}
                        onChange={(e) => onCustomDateChange(e.target.value)}
                        min={new Date().toISOString().split('T')[0]}
                        className="mt-1.5 bg-card"
                    />
                </div>
            )}
            <div className="md:col-span-2">
                <Label htmlFor="reason" className="text-sm font-medium">Reason</Label>
                <Input
                    id="reason"
                    value={reason}
                    onChange={(e) => onReasonChange(e.target.value)}
                    placeholder="e.g., Admin grant for premium support"
                    className="mt-1.5 bg-card"
                />
            </div>
        </div>
    );
}

export function AssignPermissionForm({
    walletAddress,
    onAssign,
    isLoading = false,
    className,
}: AssignPermissionFormProps) {
    const [platform, setPlatform] = useState<Platform>('analytics');
    const [permission, setPermission] = useState<string>('');
    const [duration, setDuration] = useState<string>('permanent');
    const [customDate, setCustomDate] = useState<string>('');
    const [reason, setReason] = useState<string>('');
    const [error, setError] = useState<string>('');

    const availablePermissions = PERMISSION_OPTIONS[platform];

    const handlePlatformChange = (newPlatform: Platform) => {
        setPlatform(newPlatform);
        setPermission('');
    };

    const calculateExpiresAt = (): string | undefined => {
        if (duration === 'permanent') { return undefined; }
        if (duration === 'custom') { return customDate !== '' ? customDate : undefined; }
        const days = parseInt(duration);
        const date = new Date();
        date.setDate(date.getDate() + days);
        return date.toISOString();
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setError('');
        if (permission === '') { setError('Please select a permission'); return; }
        if (reason.trim() === '') { setError('Please provide a reason for this assignment'); return; }
        if (duration === 'custom' && customDate === '') { setError('Please select an expiry date'); return; }
        try {
            await onAssign({
                walletAddress, platform, permission,
                duration: duration === 'permanent' ? 'permanent' : 'custom',
                expiresAt: calculateExpiresAt(),
                reason: reason.trim(),
            });
            setPermission('');
            setReason('');
            setDuration('permanent');
            setCustomDate('');
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to assign permission');
        }
    };

    return (
        <form onSubmit={(e) => { void handleSubmit(e); }} className={cn('space-y-4', className)}>
            <div className="rounded-xl border-2 border-dashed border-blue-300 dark:border-blue-700 bg-blue-50/50 dark:bg-blue-900/10 p-4">
                <h4 className="font-semibold text-foreground mb-4 flex items-center gap-2">
                    <Plus className="h-5 w-5 text-blue-600" />
                    Assign Permission
                </h4>
                <PermissionFormFields
                    platform={platform}
                    permission={permission}
                    duration={duration}
                    customDate={customDate}
                    reason={reason}
                    availablePermissions={availablePermissions}
                    onPlatformChange={handlePlatformChange}
                    onPermissionChange={setPermission}
                    onDurationChange={setDuration}
                    onCustomDateChange={setCustomDate}
                    onReasonChange={setReason}
                />
                {error !== '' && (
                    <p className="mt-3 text-sm text-red-600 dark:text-red-400">⚠️ {error}</p>
                )}
                <div className="mt-4 flex justify-end">
                    <Button type="submit" disabled={isLoading} className="bg-gradient-to-r from-blue-500 to-purple-500 text-white border-0">
                        {isLoading ? (
                            <><span className="animate-spin mr-2">⏳</span>Assigning...</>
                        ) : (
                            <><Plus className="h-4 w-4 mr-2" />Assign Permission</>
                        )}
                    </Button>
                </div>
            </div>
        </form>
    );
}
