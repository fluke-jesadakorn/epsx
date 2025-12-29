'use client';

import { Plus, Search } from 'lucide-react';
import React, { useMemo, useState } from 'react';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table';

import type { ApiKeyResponse } from '@/shared/api/plans';
import { ApiKeyRow } from '../shared/ApiKeyRow';

interface ApiKeysTabProps {
    apiKeys: ApiKeyResponse[];
    onCopyWallet: (wallet: string) => void;
    onCopyKeyPrefix: (prefix: string) => void;
    onRevoke: (apiKey: ApiKeyResponse) => void;
    onEditExpiration: (apiKey: ApiKeyResponse) => void;
    onCreateKey: () => void;
}

type StatusFilter = 'all' | 'active' | 'revoked' | 'expired';

/**
 * API Keys management tab with filtering and actions
 */
export const ApiKeysTab: React.FC<ApiKeysTabProps> = ({
    apiKeys,
    onCopyWallet,
    onCopyKeyPrefix,
    onRevoke,
    onEditExpiration,
    onCreateKey,
}) => {
    const [searchQuery, setSearchQuery] = useState('');
    const [statusFilter, setStatusFilter] = useState<StatusFilter>('all');

    // Filter API keys based on search and status
    const filteredApiKeys = useMemo(() => {
        return apiKeys.filter(key => {
            // Status filter
            if (statusFilter !== 'all' && key.status !== statusFilter) {
                return false;
            }

            // Search filter (client name or wallet address)
            if (searchQuery.trim()) {
                const query = searchQuery.toLowerCase();
                const matchesClientName = key.client_name.toLowerCase().includes(query);
                const walletAddress = (key as any).wallet_address?.toLowerCase() || '';
                const matchesWallet = walletAddress.includes(query);
                const keyPrefix = ((key as any).key_prefix || key.key_preview || '').toLowerCase();
                const matchesKeyPrefix = keyPrefix.includes(query);

                return matchesClientName || matchesWallet || matchesKeyPrefix;
            }

            return true;
        });
    }, [apiKeys, searchQuery, statusFilter]);

    // Status counts for badges
    const statusCounts = useMemo(() => {
        const counts = { all: apiKeys.length, active: 0, revoked: 0, expired: 0 };
        apiKeys.forEach(key => {
            if (key.status === 'active') counts.active++;
            else if (key.status === 'revoked') counts.revoked++;
            else if (key.status === 'expired') counts.expired++;
        });
        return counts;
    }, [apiKeys]);

    return (
        <div className="space-y-6">
            {/* Header */}
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
                <div>
                    <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                        User API Key Management
                    </h2>
                    <p className="text-sm text-gray-600 dark:text-gray-300">
                        Manage API keys for third-party integrations
                    </p>
                </div>
                <Button onClick={onCreateKey}>
                    <Plus className="w-4 h-4 mr-2" />
                    Create API Key
                </Button>
            </div>

            {/* Filters */}
            <div className="flex flex-col sm:flex-row gap-4">
                {/* Search */}
                <div className="relative flex-1 max-w-md">
                    <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
                    <Input
                        type="text"
                        placeholder="Search by client name, wallet, or key prefix..."
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        className="pl-10"
                    />
                </div>

                {/* Status Filter Tabs */}
                <div className="flex bg-gray-100 dark:bg-gray-800 rounded-lg p-1">
                    {(['all', 'active', 'revoked', 'expired'] as StatusFilter[]).map((status) => (
                        <button
                            key={status}
                            onClick={() => setStatusFilter(status)}
                            className={`px-3 py-1.5 rounded-md text-sm font-medium transition-colors ${statusFilter === status
                                    ? 'bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 shadow-sm'
                                    : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100'
                                }`}
                        >
                            <span className="capitalize">{status}</span>
                            <span className="ml-1.5 text-xs text-gray-500 dark:text-gray-400">
                                ({statusCounts[status]})
                            </span>
                        </button>
                    ))}
                </div>
            </div>

            {/* Table */}
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow overflow-hidden">
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead>User / Client</TableHead>
                            <TableHead>API Key</TableHead>
                            <TableHead>Scope</TableHead>
                            <TableHead>Expires</TableHead>
                            <TableHead>Status</TableHead>
                            <TableHead className="text-right">Actions</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {filteredApiKeys.length === 0 ? (
                            <TableRow>
                                <TableCell colSpan={6} className="h-24 text-center">
                                    {searchQuery || statusFilter !== 'all'
                                        ? 'No API keys match your filters.'
                                        : 'No API keys found.'}
                                </TableCell>
                            </TableRow>
                        ) : (
                            filteredApiKeys.map((apiKey) => (
                                <ApiKeyRow
                                    key={apiKey.id}
                                    apiKey={apiKey}
                                    onCopyWallet={onCopyWallet}
                                    onCopyKeyPrefix={onCopyKeyPrefix}
                                    onRevoke={onRevoke}
                                    onEditExpiration={onEditExpiration}
                                />
                            ))
                        )}
                    </TableBody>
                </Table>
            </div>

            {/* Results summary */}
            <div className="text-sm text-gray-500 dark:text-gray-400">
                Showing {filteredApiKeys.length} of {apiKeys.length} API keys
            </div>
        </div>
    );
};
