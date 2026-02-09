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
}

interface ApiKeyManagerProps {
    apiKeys: ApiKey[];
    showKeyValue: string | null;
    onRevoke: (id: string, name: string) => void;
    onCopy: (text: string, label: string) => void;
}

// eslint-disable-next-line max-lines-per-function, complexity
export const ApiKeyManager: React.FC<ApiKeyManagerProps> = ({
    apiKeys,
    showKeyValue,
    onRevoke,
    onCopy,
}) => {
    const router = useRouter();

    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                        API Key Management
                    </h2>
                    <p className="text-sm text-gray-600 dark:text-gray-300">
                        Create and manage API keys for third-party integrations
                    </p>
                </div>
                <Button onClick={() => router.push('/developer-portal/api-keys/create')}>
                    <Plus className="w-4 h-4 mr-2" />
                    Create API Key
                </Button>
            </div>

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
                        {apiKeys.length === 0 ? (
                            <TableRow>
                                <TableCell colSpan={6} className="h-24 text-center">
                                    No API keys found.
                                </TableCell>
                            </TableRow>
                        ) : (
                            apiKeys.map((key) => {
                                const apiKey = key as ExtendedApiKey;
                                return (
                                <TableRow key={apiKey.id}>
                                    <TableCell>
                                        <div className="flex flex-col">
                                            <span className="font-medium text-gray-900 dark:text-gray-100">
                                                {apiKey.client_name}
                                            </span>
                                            <span
                                                className="text-xs text-gray-500 dark:text-gray-400 font-mono cursor-pointer hover:text-blue-600"
                                                title={apiKey.wallet_address ?? 'Unknown'}
                                                onClick={() => onCopy(apiKey.wallet_address ?? '', 'Wallet address')}
                                            >
                                                {truncateWallet(apiKey.wallet_address)}
                                            </span>
                                        </div>
                                    </TableCell>
                                    <TableCell>
                                        <div className="flex items-center space-x-2">
                                            <span className="font-mono text-sm text-gray-600 dark:text-gray-300">
                                                {showKeyValue === apiKey.id ? (apiKey.key_preview ?? apiKey.key_prefix) : maskKeyPrefix(apiKey.key_preview ?? apiKey.key_prefix ?? '')}
                                            </span>
                                            <button
                                                onClick={() =>
                                                    onCopy(
                                                        `${apiKey.key_preview ?? apiKey.key_prefix ?? ''}...`,
                                                        'API Key Prefix'
                                                    )
                                                }
                                                className="p-1 text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300"
                                            >
                                                <Copy className="w-3 h-3" />
                                            </button>
                                        </div>
                                    </TableCell>
                                    <TableCell>
                                        <div className="flex flex-wrap gap-1">
                                            {apiKey.permission_groups && apiKey.permission_groups.length > 0 ? (
                                                apiKey.permission_groups.map((group) => (
                                                    <Badge key={group.id} variant="secondary" className="text-xs bg-purple-100 dark:bg-purple-900/50 text-purple-800 dark:text-purple-200 hover:bg-purple-200 border-none">
                                                        {group.name}
                                                    </Badge>
                                                ))
                                            ) : (
                                                apiKey.allowed_modules.map((module) => (
                                                    <Badge key={module.module_id} variant="outline" className="text-xs">
                                                        {module.module_name}
                                                    </Badge>
                                                ))
                                            )}
                                        </div>
                                    </TableCell>
                                    <TableCell>
                                        <div className="text-sm text-gray-600 dark:text-gray-300">
                                            {apiKey.expires_at ? (
                                                new Date(apiKey.expires_at).toLocaleDateString()
                                            ) : (
                                                <span className="text-gray-400">Never</span>
                                            )}
                                        </div>
                                    </TableCell>
                                    <TableCell>
                                        <Badge
                                            variant={apiKey.status === 'active' ? 'default' : apiKey.status === 'revoked' ? 'destructive' : 'secondary'}
                                            className={`
                          ${apiKey.status === 'active' ? 'bg-green-100 dark:bg-green-900/50 text-green-800 dark:text-green-200 hover:bg-green-200 border-none' : ''}
                          ${apiKey.status === 'revoked' ? 'bg-red-100 dark:bg-red-900/50 text-red-800 dark:text-red-200 hover:bg-red-200 border-none' : ''}
                          ${apiKey.status === 'expired' ? 'bg-yellow-100 dark:bg-yellow-900/50 text-yellow-800 dark:text-yellow-200 hover:bg-yellow-200 border-none' : ''}
                        `}
                                        >
                                            {apiKey.status}
                                        </Badge>
                                    </TableCell>
                                    <TableCell className="text-right">
                                        {apiKey.status === 'active' && (
                                            <Button
                                                size="sm"
                                                variant="ghost"
                                                className="h-8 w-8 p-0 text-red-600 hover:text-red-700 hover:bg-red-50 dark:hover:bg-red-900/20"
                                                onClick={() => onRevoke(apiKey.id, apiKey.client_name)}
                                            >
                                                <Trash2 className="w-4 h-4" />
                                                <span className="sr-only">Revoke</span>
                                            </Button>
                                        )}
                                    </TableCell>
                                </TableRow>
                                );
                            })
                        )}
                    </TableBody>
                </Table>
            </div>
        </div>
    );
};
