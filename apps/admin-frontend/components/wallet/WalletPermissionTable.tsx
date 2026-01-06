/**
 * Wallet Permission Table Component
 * Displays permissions grouped by platform with source badges
 */
'use client';

import { Clock, Settings, Trash2 } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table';
import { cn } from '@/lib/utils';

import type { PermissionSource, Platform, WalletPermission } from './types';

interface WalletPermissionTableProps {
    permissions: WalletPermission[];
    groupByPlatform?: boolean;
    showActions?: boolean;
    onRevoke?: (permissionId: string) => void;
    className?: string;
}

const PLATFORM_LABELS: Record<Platform, { label: string; emoji: string }> = {
    analytics: { label: 'EPSX Analytics', emoji: '📊' },
    pay: { label: 'EPSX Pay', emoji: '💳' },
    token: { label: 'EPSX Token', emoji: '🪙' },
    markets: { label: 'EPSX Markets', emoji: '📈' },
};

const SOURCE_CONFIG: Record<PermissionSource, { label: string; emoji: string; className: string }> = {
    manual: {
        label: 'Manual',
        emoji: '🔧',
        className: 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400',
    },
    auto: {
        label: 'Auto',
        emoji: '🛒',
        className: 'bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-400',
    },
    system: {
        label: 'System',
        emoji: '⚙️',
        className: 'bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-400',
    },
};

function formatDate(dateString?: string): string {
    if (!dateString) return 'Never';
    const date = new Date(dateString);
    return date.toLocaleDateString('en-US', {
        month: 'short',
        day: 'numeric',
        year: 'numeric',
    });
}

function isExpiringSoon(dateString?: string): boolean {
    if (!dateString) return false;
    const date = new Date(dateString);
    const now = new Date();
    const daysUntilExpiry = (date.getTime() - now.getTime()) / (1000 * 60 * 60 * 24);
    return daysUntilExpiry > 0 && daysUntilExpiry <= 7;
}

function isExpired(dateString?: string): boolean {
    if (!dateString) return false;
    return new Date(dateString) < new Date();
}

function SourceBadge({ source, planName }: { source: PermissionSource; planName?: string }) {
    const config = SOURCE_CONFIG[source];

    return (
        <Badge className={cn('text-xs px-2 py-0.5 gap-1', config.className)}>
            {config.emoji}
            {source === 'auto' && planName ? `Auto: ${planName}` : config.label}
        </Badge>
    );
}

function ExpiryBadge({ expiresAt }: { expiresAt?: string }) {
    if (!expiresAt) {
        return (
            <span className="text-xs text-green-600 dark:text-green-400 flex items-center gap-1">
                ∞ Permanent
            </span>
        );
    }

    const expired = isExpired(expiresAt);
    const expiringSoon = isExpiringSoon(expiresAt);

    // Calculate days remaining
    const now = new Date();
    const expDate = new Date(expiresAt);
    const daysRemaining = Math.ceil((expDate.getTime() - now.getTime()) / (1000 * 60 * 60 * 24));

    return (
        <span className={cn(
            'text-xs flex items-center gap-1',
            expired && 'text-red-600 dark:text-red-400',
            expiringSoon && !expired && 'text-amber-600 dark:text-amber-400',
            !expired && !expiringSoon && 'text-gray-600 dark:text-gray-400'
        )}>
            <Clock className="h-3 w-3" />
            {expired ? (
                <Badge className="text-[10px] px-1.5 py-0 bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400">
                    Expired
                </Badge>
            ) : expiringSoon ? (
                <>
                    <span className="font-medium">{daysRemaining}d left</span>
                    <Badge className="text-[10px] px-1.5 py-0 bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-400">
                        ⚠️ Soon
                    </Badge>
                </>
            ) : (
                <span>{daysRemaining}d ({formatDate(expiresAt)})</span>
            )}
        </span>
    );
}

function PermissionRow({
    permission,
    showPlatform = false,
    showActions = false,
    onRevoke,
}: {
    permission: WalletPermission;
    showPlatform?: boolean;
    showActions?: boolean;
    onRevoke?: (permissionId: string) => void;
}) {
    const expired = isExpired(permission.expiresAt);

    return (
        <TableRow className={cn(
            !permission.isActive && 'opacity-50',
            expired && 'bg-red-50/50 dark:bg-red-900/10'
        )}>
            {showPlatform && (
                <TableCell className="font-medium">
                    <span className="flex items-center gap-2">
                        {PLATFORM_LABELS[permission.platform].emoji}
                        {PLATFORM_LABELS[permission.platform].label}
                    </span>
                </TableCell>
            )}
            <TableCell>
                <code className="px-2 py-1 bg-gray-100 dark:bg-gray-800 rounded text-sm font-mono">
                    {permission.permission}
                </code>
            </TableCell>
            <TableCell>
                <SourceBadge source={permission.source} planName={permission.sourcePlanName} />
            </TableCell>
            <TableCell>
                <ExpiryBadge expiresAt={permission.expiresAt} />
            </TableCell>
            <TableCell>
                <Badge className={cn(
                    'text-xs',
                    permission.isActive && !expired
                        ? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400'
                        : 'bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-400'
                )}>
                    {permission.isActive && !expired ? 'Active' : expired ? 'Expired' : 'Inactive'}
                </Badge>
            </TableCell>
            {showActions && (
                <TableCell>
                    <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => onRevoke?.(permission.id)}
                        className="h-8 px-2 text-red-600 hover:text-red-700 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-900/20"
                        disabled={!permission.isActive || expired}
                    >
                        <Trash2 className="h-4 w-4" />
                    </Button>
                </TableCell>
            )}
        </TableRow>
    );
}

export function WalletPermissionTable({
    permissions,
    groupByPlatform = true,
    showActions = false,
    onRevoke,
    className,
}: WalletPermissionTableProps) {
    if (permissions.length === 0) {
        return (
            <div className="text-center py-8 text-gray-500 dark:text-gray-400">
                <Settings className="h-12 w-12 mx-auto mb-3 opacity-30" />
                <p className="font-medium">No permissions assigned</p>
                <p className="text-sm mt-1">Use the form below to assign permissions</p>
            </div>
        );
    }

    if (groupByPlatform) {
        // Group permissions by platform
        const grouped = permissions.reduce((acc, perm) => {
            if (!acc[perm.platform]) acc[perm.platform] = [];
            acc[perm.platform].push(perm);
            return acc;
        }, {} as Record<Platform, WalletPermission[]>);

        return (
            <div className={cn('space-y-6', className)}>
                {(Object.entries(grouped) as [Platform, WalletPermission[]][]).map(([platform, perms]) => (
                    <div key={platform}>
                        <div className="flex items-center gap-2 mb-3">
                            <span className="text-lg">{PLATFORM_LABELS[platform].emoji}</span>
                            <h4 className="font-semibold text-gray-900 dark:text-white">
                                {PLATFORM_LABELS[platform].label}
                            </h4>
                            <Badge variant="secondary" className="text-xs">
                                {perms.length}
                            </Badge>
                        </div>
                        <div className="rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden">
                            <Table>
                                <TableHeader>
                                    <TableRow className="bg-gray-50 dark:bg-gray-800/50">
                                        <TableHead>Permission</TableHead>
                                        <TableHead>Source</TableHead>
                                        <TableHead>Expires</TableHead>
                                        <TableHead>Status</TableHead>
                                        {showActions && <TableHead className="w-16">Action</TableHead>}
                                    </TableRow>
                                </TableHeader>
                                <TableBody>
                                    {perms.map((perm) => (
                                        <PermissionRow
                                            key={perm.id}
                                            permission={perm}
                                            showActions={showActions}
                                            onRevoke={onRevoke}
                                        />
                                    ))}
                                </TableBody>
                            </Table>
                        </div>
                    </div>
                ))}
            </div>
        );
    }

    // Flat table view
    return (
        <div className={cn('rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden', className)}>
            <Table>
                <TableHeader>
                    <TableRow className="bg-gray-50 dark:bg-gray-800/50">
                        <TableHead>Platform</TableHead>
                        <TableHead>Permission</TableHead>
                        <TableHead>Source</TableHead>
                        <TableHead>Expires</TableHead>
                        <TableHead>Status</TableHead>
                        {showActions && <TableHead className="w-16">Action</TableHead>}
                    </TableRow>
                </TableHeader>
                <TableBody>
                    {permissions.map((perm) => (
                        <PermissionRow
                            key={perm.id}
                            permission={perm}
                            showPlatform
                            showActions={showActions}
                            onRevoke={onRevoke}
                        />
                    ))}
                </TableBody>
            </Table>
        </div>
    );
}
