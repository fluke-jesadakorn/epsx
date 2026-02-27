'use client';

import { Copy, Trash2 } from 'lucide-react';
import React from 'react';

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
 * @param address
 */
const truncateWallet = (address: string | undefined): string => {
    if (address === undefined || address === '' || address.length < 12) { return address ?? 'Unknown'; }
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
};

/**
 * Mask API key prefix for display
 * @param prefix
 */
const maskKeyPrefix = (prefix: string): string => {
    if (!prefix) { return '***'; }
    if (prefix.length <= 8) { return `${prefix}...`; }
    const start = prefix.slice(0, 4);
    const end = prefix.slice(-3);
    return `${start}...${end}`;
};

/**
 * Get status badge styling
 * @param status
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
            return 'bg-muted/30 text-gray-800 dark:text-muted-foreground';
    }
};

/**
 * Reusable table row component for API key display
 * @param param0 Component props
 * @param param0.apiKey The API key to display
 * @param param0.onCopyWallet Callback for copying wallet address
 * @param param0.onCopyKeyPrefix Callback for copying key prefix
 * @param param0.onRevoke Callback for revoking key
 * @param param0.onEditExpiration Callback for editing expiration
 */
interface ExtendedApiKey extends ApiKeyResponse {
    wallet_address?: string;
    key_prefix?: string;
    permission_groups?: Array<{ id: string; name: string }>;
}

function ScopeCell({ apiKey, permissionGroups }: { apiKey: ApiKeyResponse; permissionGroups: Array<{ id: string; name: string }> }) {
    return (
        <TableCell className="py-6 px-6">
            <div className="flex flex-wrap gap-2 max-w-[240px]">
                {permissionGroups.length > 0 ? (
                    permissionGroups.slice(0, 3).map((group) => (
                        <span key={group.id} className="inline-flex items-center px-3 py-1 rounded-xl text-[10px] font-black uppercase tracking-widest bg-purple-500/10 text-purple-400 border border-purple-500/10 shadow-[0_0_15px_rgba(168,85,247,0.05)]">
                            {group.name}
                        </span>
                    ))
                ) : apiKey.allowed_modules.length > 0 ? (
                    apiKey.allowed_modules.slice(0, 3).map((module) => (
                        <span key={module.module_id} className="inline-flex items-center px-3 py-1 rounded-xl text-[10px] font-black uppercase tracking-widest bg-cyan-500/10 text-cyan-400 border border-cyan-500/10 shadow-[0_0_15px_rgba(31,199,212,0.05)]">
                            {module.module_name}
                        </span>
                    ))
                ) : (
                    <span className="text-[10px] font-black text-muted-foreground/30 uppercase tracking-widest">No scope</span>
                )}
                {(permissionGroups.length > 3 || apiKey.allowed_modules.length > 3) && (
                    <span className="inline-flex items-center px-3 py-1 rounded-xl text-[10px] font-black uppercase tracking-widest bg-muted/30 text-muted-foreground border border-border/20">
                        +{Math.max(permissionGroups.length, apiKey.allowed_modules.length) - 3}
                    </span>
                )}
            </div>
        </TableCell>
    );
}

export const ApiKeyRow: React.FC<ApiKeyRowProps> = ({
    apiKey,
    onCopyWallet,
    onCopyKeyPrefix,
    onRevoke,
    onEditExpiration,
}) => {
    const extendedKey = apiKey as ExtendedApiKey;
    const walletAddress = extendedKey.wallet_address ?? '';
    const keyPrefix = extendedKey.key_prefix ?? apiKey.key_preview;
    const permissionGroups = extendedKey.permission_groups ?? [];

    return (
        <TableRow className="hover:bg-gray-50 dark:hover:bg-white/[0.02] border-border/20 transition-colors">
            <TableCell className="py-6 px-6">
                <div className="flex flex-col">
                    <span className="font-black text-foreground tracking-tight text-base">{apiKey.client_name}</span>
                    <span className="text-[10px] font-black text-muted-foreground uppercase tracking-widest mt-1 cursor-pointer hover:text-[#1fc7d4] transition-colors" title={walletAddress} onClick={() => onCopyWallet(walletAddress)}>
                        {truncateWallet(walletAddress)}
                    </span>
                </div>
            </TableCell>
            <TableCell className="py-6 px-6">
                <div className="flex items-center space-x-3">
                    <div className="px-3 py-1.5 bg-muted/30 border border-border/20 rounded-lg font-mono text-xs font-bold text-[#1fc7d4]">{maskKeyPrefix(keyPrefix)}</div>
                    <button onClick={() => onCopyKeyPrefix(keyPrefix)} className="p-2 text-muted-foreground/50 hover:text-[#1fc7d4] hover:bg-muted/30 rounded-lg transition-all" title="Copy key prefix">
                        <Copy className="w-4 h-4" />
                    </button>
                </div>
            </TableCell>
            <ScopeCell apiKey={apiKey} permissionGroups={permissionGroups} />
            <TableCell className="py-6 px-6">
                <button onClick={() => onEditExpiration(apiKey)} className="flex flex-col items-start group" title="Click to edit expiration">
                    <span className="text-xs font-black text-foreground group-hover:text-[#1fc7d4] transition-colors">
                        {apiKey.expires_at !== null ? new Date(apiKey.expires_at).toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' }) : 'Never'}
                    </span>
                    <span className="text-[10px] font-black text-muted-foreground/50 uppercase tracking-widest">
                        {apiKey.expires_at !== null ? 'Fixed Date' : 'Permanent'}
                    </span>
                </button>
            </TableCell>
            <TableCell className="py-6 px-6">
                <span className={`px-4 py-1.5 rounded-full text-[10px] font-black uppercase tracking-widest ${getStatusBadgeClass(apiKey.status)}`}>{apiKey.status}</span>
            </TableCell>
            <TableCell className="py-6 px-6 text-right">
                {apiKey.status === 'active' && (
                    <Button size="sm" variant="ghost" className="h-10 w-10 p-0 text-red-400 hover:text-red-300 hover:bg-red-500/10 rounded-xl border border-transparent hover:border-red-500/20 active:scale-90 transition-all" onClick={() => onRevoke(apiKey)} title="Revoke API Key">
                        <Trash2 className="w-5 h-5" />
                        <span className="sr-only">Revoke</span>
                    </Button>
                )}
            </TableCell>
        </TableRow>
    );
};
