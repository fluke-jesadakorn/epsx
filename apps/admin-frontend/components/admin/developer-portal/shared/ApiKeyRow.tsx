'use client';

import { Copy, Trash2 } from 'lucide-react';
import React from 'react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { TableCell, TableRow } from '@/components/ui/table';

import type { ApiKeyResponse } from '@/shared/api/plans';

interface ApiKeyRowProps {
    apiKey: ApiKeyResponse;
    onCopyWallet: (wallet: string) => void;
    onCopyKeyPrefix: (prefix: string) => void;
    onRevoke: (apiKey: ApiKeyResponse) => void;
    onEditExpiration: (apiKey: ApiKeyResponse) => void;
}

/**
 * Truncate wallet address for display
 */
const truncateWallet = (address: string): string => {
    if (!address || address.length < 12) return address || 'Unknown';
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
};

/**
 * Mask API key prefix for display
 */
const maskKeyPrefix = (prefix: string): string => {
    if (!prefix) return '***';
    if (prefix.length <= 8) return `${prefix}...`;
    const start = prefix.slice(0, 4);
    const end = prefix.slice(-3);
    return `${start}...${end}`;
};

/**
 * Get status badge styling
 */
const getStatusBadgeClass = (status: string): string => {
    switch (status) {
        case 'active':
            return 'bg-green-100 dark:bg-green-900/50 text-green-800 dark:text-green-200 hover:bg-green-200 border-none';
        case 'revoked':
            return 'bg-red-100 dark:bg-red-900/50 text-red-800 dark:text-red-200 hover:bg-red-200 border-none';
        case 'expired':
            return 'bg-yellow-100 dark:bg-yellow-900/50 text-yellow-800 dark:text-yellow-200 hover:bg-yellow-200 border-none';
        default:
            return 'bg-gray-100 dark:bg-gray-800 text-gray-800 dark:text-gray-300';
    }
};

/**
 * Reusable table row component for API key display
 */
export const ApiKeyRow: React.FC<ApiKeyRowProps> = ({
    apiKey,
    onCopyWallet,
    onCopyKeyPrefix,
    onRevoke,
    onEditExpiration,
}) => {
    const walletAddress = (apiKey as any).wallet_address || '';
    const keyPrefix = (apiKey as any).key_prefix || apiKey.key_preview || '';
    const permissionGroups = (apiKey as any).permission_groups || [];

    return (
        <TableRow>
            {/* User / Client */}
            <TableCell>
                <div className="flex flex-col">
                    <span className="font-medium text-gray-900 dark:text-gray-100">
                        {apiKey.client_name}
                    </span>
                    <span
                        className="text-xs text-gray-500 dark:text-gray-400 font-mono cursor-pointer hover:text-blue-600 dark:hover:text-blue-400"
                        title={walletAddress}
                        onClick={() => onCopyWallet(walletAddress)}
                    >
                        {truncateWallet(walletAddress)}
                    </span>
                </div>
            </TableCell>

            {/* API Key */}
            <TableCell>
                <div className="flex items-center space-x-2">
                    <span className="font-mono text-sm text-gray-600 dark:text-gray-300">
                        {maskKeyPrefix(keyPrefix)}
                    </span>
                    <button
                        onClick={() => onCopyKeyPrefix(keyPrefix)}
                        className="p-1 text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300"
                        title="Copy key prefix"
                    >
                        <Copy className="w-3 h-3" />
                    </button>
                </div>
            </TableCell>

            {/* Scope / Permission Groups */}
            <TableCell>
                <div className="flex flex-wrap gap-1 max-w-[200px]">
                    {permissionGroups.length > 0 ? (
                        permissionGroups.slice(0, 3).map((group: { id: string; name: string }) => (
                            <Badge
                                key={group.id}
                                variant="secondary"
                                className="text-xs bg-purple-100 dark:bg-purple-900/50 text-purple-800 dark:text-purple-200 hover:bg-purple-200 border-none"
                            >
                                {group.name}
                            </Badge>
                        ))
                    ) : apiKey.allowed_modules.length > 0 ? (
                        apiKey.allowed_modules.slice(0, 3).map((module) => (
                            <Badge key={module.module_id} variant="outline" className="text-xs">
                                {module.module_name}
                            </Badge>
                        ))
                    ) : (
                        <span className="text-xs text-gray-400">No scope</span>
                    )}
                    {(permissionGroups.length > 3 || apiKey.allowed_modules.length > 3) && (
                        <Badge variant="outline" className="text-xs">
                            +{Math.max(permissionGroups.length, apiKey.allowed_modules.length) - 3}
                        </Badge>
                    )}
                </div>
            </TableCell>

            {/* Expiration */}
            <TableCell>
                <button
                    onClick={() => onEditExpiration(apiKey)}
                    className="text-sm text-gray-600 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400 hover:underline"
                    title="Click to edit expiration"
                >
                    {apiKey.expires_at ? (
                        new Date(apiKey.expires_at).toLocaleDateString()
                    ) : (
                        <span className="text-gray-400 dark:text-gray-500">Never</span>
                    )}
                </button>
            </TableCell>

            {/* Status */}
            <TableCell>
                <Badge className={getStatusBadgeClass(apiKey.status)}>
                    {apiKey.status}
                </Badge>
            </TableCell>

            {/* Actions */}
            <TableCell className="text-right">
                {apiKey.status === 'active' && (
                    <Button
                        size="sm"
                        variant="ghost"
                        className="h-8 w-8 p-0 text-red-600 hover:text-red-700 hover:bg-red-50 dark:hover:bg-red-900/20"
                        onClick={() => onRevoke(apiKey)}
                        title="Revoke API Key"
                    >
                        <Trash2 className="w-4 h-4" />
                        <span className="sr-only">Revoke</span>
                    </Button>
                )}
            </TableCell>
        </TableRow>
    );
};
