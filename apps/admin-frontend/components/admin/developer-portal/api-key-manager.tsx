'use client';

import { Copy, Plus, Trash2 } from 'lucide-react';
import { useRouter } from 'next/navigation';
import React from 'react';

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
import { type ApiKeyResponse as ApiKey } from '@/shared/api/plans';

import { maskKeyPrefix, truncateWallet } from './utils';

interface PermissionGroup {
    id: string;
    name: string;
}

interface ExtendedApiKey extends ApiKey {
    wallet_address?: string;
    key_prefix?: string;
    permission_groups?: PermissionGroup[];
    last_used_at?: string;
}

interface ApiKeyManagerProps {
    apiKeys: ApiKey[];
    showKeyValue: string | null;
    onRevoke: (id: string, name: string) => void;
    onCopy: (text: string, label: string) => void;
}

const STATUS_CLASSES: Record<string, string> = {
    active: 'bg-[#31d0aa]/10 text-[#31d0aa] hover:bg-[#31d0aa]/20 border border-[#31d0aa]/20',
    revoked: 'bg-red-500/10 text-red-400 hover:bg-red-500/20 border border-red-500/20',
    expired: 'bg-[#ffb237]/10 text-[#ffb237] hover:bg-[#ffb237]/20 border border-[#ffb237]/20',
};

const STATUS_VARIANTS: Record<string, 'default' | 'destructive' | 'secondary'> = {
    active: 'default',
    revoked: 'destructive',
    expired: 'secondary',
};

const KeyScope: React.FC<{ apiKey: ExtendedApiKey }> = ({ apiKey }) => {
    if (apiKey.permission_groups !== undefined && apiKey.permission_groups.length > 0) {
        return (
            <>
                {apiKey.permission_groups.map((group) => (
                    <Badge key={group.id} variant="secondary" className="text-xs bg-[#7645d9]/10 text-[#7645d9] hover:bg-[#7645d9]/20 border border-[#7645d9]/20">
                        {group.name}
                    </Badge>
                ))}
            </>
        );
    }
    return (
        <>
            {apiKey.allowed_modules.map((module) => (
                <Badge key={module.module_id} variant="outline" className="text-xs">
                    {module.module_name}
                </Badge>
            ))}
        </>
    );
};

const ApiKeyRow: React.FC<{
    keyData: ApiKey;
    showKeyValue: string | null;
    onRevoke: (id: string, name: string) => void;
    onCopy: (text: string, label: string) => void;
}> = ({ keyData, showKeyValue, onRevoke, onCopy }) => {
    const apiKey = keyData as ExtendedApiKey;
    const walletAddress = apiKey.wallet_address ?? '';
    const keyPrefixValue = apiKey.key_prefix ?? apiKey.key_preview;
    const statusVariant = STATUS_VARIANTS[apiKey.status] ?? 'secondary';
    const statusClass = STATUS_CLASSES[apiKey.status] ?? '';

    return (
        <TableRow key={apiKey.id}>
            <TableCell>
                <div className="flex flex-col">
                    <span className="font-medium text-foreground">
                        {apiKey.client_name}
                    </span>
                    <span
                        className="text-xs text-muted-foreground font-mono cursor-pointer hover:text-[#1fc7d4]"
                        title={walletAddress !== '' ? walletAddress : 'Unknown'}
                        onClick={() => onCopy(walletAddress, 'Wallet address')}
                    >
                        {truncateWallet(apiKey.wallet_address)}
                    </span>
                </div>
            </TableCell>
            <TableCell>
                <div className="flex items-center space-x-2">
                    <span className="font-mono text-sm text-muted-foreground">
                        {showKeyValue === apiKey.id
                            ? keyPrefixValue
                            : maskKeyPrefix(keyPrefixValue)}
                    </span>
                    <button
                        onClick={() => onCopy(`${keyPrefixValue}...`, 'API Key Prefix')}
                        className="p-1 text-muted-foreground hover:text-foreground"
                    >
                        <Copy className="w-3 h-3" />
                    </button>
                </div>
            </TableCell>
            <TableCell>
                <div className="flex flex-wrap gap-1">
                    <KeyScope apiKey={apiKey} />
                </div>
            </TableCell>
            <TableCell>
                <div className="text-sm text-muted-foreground">
                    {apiKey.expires_at !== null && apiKey.expires_at !== '' ? (
                        new Date(apiKey.expires_at).toLocaleDateString()
                    ) : (
                        <span className="text-muted-foreground/60">Never</span>
                    )}
                </div>
            </TableCell>
            <TableCell>
                <Badge variant={statusVariant} className={statusClass}>
                    {apiKey.status}
                </Badge>
            </TableCell>
            <TableCell className="text-right">
                {apiKey.status === 'active' && (
                    <Button
                        size="sm"
                        variant="ghost"
                        className="h-8 w-8 p-0 text-red-400 hover:text-red-300 hover:bg-red-500/10"
                        onClick={() => onRevoke(apiKey.id, apiKey.client_name)}
                    >
                        <Trash2 className="w-4 h-4" />
                        <span className="sr-only">Revoke</span>
                    </Button>
                )}
            </TableCell>
        </TableRow>
    );
};

export const ApiKeyManager: React.FC<ApiKeyManagerProps> = ({
    apiKeys,
    showKeyValue,
    onRevoke,
    onCopy,
}) => {
    const router = useRouter();
    const goCreate = () => {
        router.push('/developer-portal/api-keys/create');
    };

    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h2 className="text-lg font-semibold text-foreground">
                        API Key Management
                    </h2>
                    <p className="text-sm text-muted-foreground">
                        Create and manage API keys for third-party integrations
                    </p>
                </div>
                <Button onClick={goCreate} className="bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white rounded-xl">
                    <Plus className="w-4 h-4 mr-2" />
                    Create API Key
                </Button>
            </div>

            <div className="rounded-2xl bg-card border border-border/20 overflow-hidden shadow-xl">
                <div className="h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" />
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
                        {apiKeys.length === 0 ? (
                            <TableRow>
                                <TableCell colSpan={6} className="h-24 text-center">
                                    No API keys found.
                                </TableCell>
                            </TableRow>
                        ) : (
                            apiKeys.map((key) => (
                                <ApiKeyRow
                                    key={key.id}
                                    keyData={key}
                                    showKeyValue={showKeyValue}
                                    onRevoke={onRevoke}
                                    onCopy={onCopy}
                                />
                            ))
                        )}
                    </TableBody>
                </Table>
            </div>
        </div>
    );
};
