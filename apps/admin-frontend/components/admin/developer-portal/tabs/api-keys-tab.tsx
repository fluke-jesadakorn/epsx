'use client';

import { Plus, Search } from 'lucide-react';
import React, { useMemo, useState } from 'react';

import { ApiKeyRow } from '../shared/api-key-row';

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
 * @param root0
 * @param root0.apiKeys
 * @param root0.onCopyWallet
 * @param root0.onCopyKeyPrefix
 * @param root0.onRevoke
 * @param root0.onEditExpiration
 * @param root0.onCreateKey
 */
// eslint-disable-next-line max-lines-per-function
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
                const walletAddress = (key as any).wallet_address?.toLowerCase() ?? '';
                const matchesWallet = walletAddress.includes(query);
                const keyPrefix = ((key as any).key_prefix ?? key.key_preview ?? '').toLowerCase();
                const matchesKeyPrefix = keyPrefix.includes(query);

                return matchesClientName ?? matchesWallet ?? matchesKeyPrefix;
            }

            return true;
        });
    }, [apiKeys, searchQuery, statusFilter]);

    // Status counts for badges
    const statusCounts = useMemo(() => {
        const counts = { all: apiKeys.length, active: 0, revoked: 0, expired: 0 };
        apiKeys.forEach(key => {
            if (key.status === 'active') { counts.active++; }
            else if (key.status === 'revoked') { counts.revoked++; }
            else if (key.status === 'expired') { counts.expired++; }
        });
        return counts;
    }, [apiKeys]);

    return (
        <div className="space-y-8">
            {/* Header */}
            <div className="relative overflow-hidden rounded-[32px] bg-white dark:bg-card backdrop-blur-2xl border border-gray-200 dark:border-border p-8 shadow-xl">
                <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-6">
                    <div>
                        <h2 className="text-2xl font-black text-foreground uppercase tracking-tight mb-2">
                            User API Key Management
                        </h2>
                        <p className="text-sm font-bold text-muted-foreground">
                            Manage API keys for third-party integrations and system access
                        </p>
                    </div>
                    <Button
                        onClick={onCreateKey}
                        className="bg-gradient-to-r from-[#1fc7d4] to-[#7645d9] hover:opacity-90 text-white font-black px-6 py-6 rounded-2xl shadow-lg shadow-cyan-500/20 active:scale-95 transition-all text-xs uppercase tracking-widest"
                    >
                        <Plus className="w-5 h-5 mr-2" />
                        Create API Key
                    </Button>
                </div>

                {/* Filters Row */}
                <div className="mt-8 flex flex-col lg:flex-row gap-4">
                    {/* Search */}
                    <div className="relative flex-1">
                        <Search className="absolute left-4 top-1/2 transform -translate-y-1/2 w-5 h-5 text-muted-foreground/50" />
                        <Input
                            type="text"
                            placeholder="Search by client name, wallet, or prefix..."
                            value={searchQuery}
                            onChange={(e) => setSearchQuery(e.target.value)}
                            className="pl-12 bg-white dark:bg-white/[0.04] border-gray-200 dark:border-border rounded-2xl h-12 text-sm font-bold placeholder:text-muted-foreground/30 focus:ring-[#1fc7d4]/20 focus:border-[#1fc7d4]/50"
                        />
                    </div>

                    {/* Status Filter Tabs */}
                    <div className="flex bg-white dark:bg-white/[0.04] border border-gray-200 dark:border-border rounded-2xl p-1 w-fit">
                        {(['all', 'active', 'revoked', 'expired'] as StatusFilter[]).map((status) => (
                            <button
                                key={status}
                                onClick={() => setStatusFilter(status)}
                                className={`px-6 py-2 rounded-xl text-xs font-black uppercase tracking-widest transition-all duration-300 ${statusFilter === status
                                    ? 'bg-gradient-to-r from-[#1fc7d4]/20 to-[#7645d9]/20 text-[#1fc7d4] border border-[#1fc7d4]/30 shadow-[0_0_20px_rgba(31,199,212,0.1)]'
                                    : 'text-muted-foreground hover:text-foreground hover:bg-gray-100 dark:hover:bg-white/5 border border-transparent'
                                    }`}
                            >
                                {status}
                                <span className="ml-2 opacity-50 font-mono">
                                    {statusCounts[status]}
                                </span>
                            </button>
                        ))}
                    </div>
                </div>
            </div>

            {/* Table Container */}
            <div className="relative overflow-hidden rounded-[32px] bg-white dark:bg-card backdrop-blur-2xl border border-gray-200 dark:border-border p-1 shadow-xl">
                <div className="overflow-hidden rounded-[28px] bg-card">
                    <Table>
                        <TableHeader className="bg-white dark:bg-white/[0.04]">
                            <TableRow className="hover:bg-transparent border-gray-200 dark:border-border">
                                <TableHead className="py-5 px-6 text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground">User / Client</TableHead>
                                <TableHead className="py-5 px-6 text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground">API Key</TableHead>
                                <TableHead className="py-5 px-6 text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground">Scope</TableHead>
                                <TableHead className="py-5 px-6 text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground">Expires</TableHead>
                                <TableHead className="py-5 px-6 text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground">Status</TableHead>
                                <TableHead className="py-5 px-6 text-right text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground">Actions</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {filteredApiKeys.length === 0 ? (
                                <TableRow className="hover:bg-transparent">
                                    <TableCell colSpan={6} className="h-64 text-center">
                                        <div className="flex flex-col items-center justify-center space-y-3">
                                            <div className="p-4 bg-white dark:bg-white/[0.04] rounded-full border border-gray-200 dark:border-border text-muted-foreground/30">
                                                <Search className="w-8 h-8" />
                                            </div>
                                            <p className="text-muted-foreground font-bold">
                                                {searchQuery ?? statusFilter !== 'all'
                                                    ? 'No API keys match your current filters.'
                                                    : 'No API keys have been created yet.'}
                                            </p>
                                        </div>
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
            </div>

            {/* Results summary footer */}
            <div className="flex items-center justify-between px-2">
                <div className="text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em] bg-white dark:bg-white/[0.04] px-4 py-2 rounded-xl border border-gray-200 dark:border-border">
                    Showing {filteredApiKeys.length} of {apiKeys.length} API keys
                </div>
                {filteredApiKeys.length < apiKeys.length && (
                    <button
                        onClick={() => { setSearchQuery(''); setStatusFilter('all'); }}
                        className="text-[10px] font-black text-[#1fc7d4] uppercase tracking-widest hover:underline"
                    >
                        Clear all filters
                    </button>
                )}
            </div>
        </div>
    );
};
