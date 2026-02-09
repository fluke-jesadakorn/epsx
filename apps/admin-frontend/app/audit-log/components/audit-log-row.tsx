'use client';

import React, { useState } from 'react';
import { AuditLogEntry } from '../types';

interface AuditLogRowProps {
    log: AuditLogEntry;
}

export function AuditLogRow({ log }: AuditLogRowProps): React.JSX.Element {
    const [isExpanded, setIsExpanded] = useState(false);

    const getActionIcon = (action: string): string => {
        const iconMap: Record<string, string> = {
            permission: '🔐',
            wallet: '👛',
            plan: '💳',
            login: '🔑',
            auth: '🔑',
            create: '➕',
            delete: '🗑️',
            remove: '🗑️',
            update: '✏️',
            edit: '✏️',
            disable: '🚫',
            enable: '✅',
        };

        const entry = Object.entries(iconMap).find(([key]) => action.includes(key));
        return entry !== undefined ? entry[1] : '📝';
    };

    const getActionColor = (action: string): string => {
        if (action.includes('create') || action.includes('enable')) {
            return 'text-emerald-600 bg-emerald-100 dark:bg-emerald-900/30';
        }
        if (action.includes('delete') || action.includes('disable') || action.includes('remove')) {
            return 'text-red-600 bg-red-100 dark:bg-red-900/30';
        }
        if (action.includes('update') || action.includes('edit')) {
            return 'text-blue-600 bg-blue-100 dark:bg-blue-900/30';
        }
        if (action.includes('permission')) {
            return 'text-purple-600 bg-purple-100 dark:bg-purple-900/30';
        }
        return 'text-muted-foreground bg-muted';
    };

    const formatAddress = (address: string): string => {
        if (address.length > 16) {
            return `${address.slice(0, 6)}...${address.slice(-4)}`;
        }
        return address;
    };

    const formatTime = (dateStr: string): string => {
        const date = new Date(dateStr);
        const now = new Date();
        const diffMs = now.getTime() - date.getTime();
        const diffMins = Math.floor(diffMs / 60000);
        const diffHours = Math.floor(diffMs / 3600000);
        const diffDays = Math.floor(diffMs / 86400000);

        if (diffMins < 1) { return 'Just now'; }
        if (diffMins < 60) { return `${diffMins}m ago`; }
        if (diffHours < 24) { return `${diffHours}h ago`; }
        if (diffDays < 7) { return `${diffDays}d ago`; }
        return date.toLocaleDateString();
    };

    return (
        <div
            className="p-4 hover:bg-muted/50 transition-colors cursor-pointer"
            onClick={() => setIsExpanded(!isExpanded)}
        >
            {/* Desktop Layout */}
            <div className="hidden md:grid grid-cols-12 gap-4 items-center">
                <div className="col-span-2 text-sm text-muted-foreground">
                    {formatTime(log.timestamp)}
                </div>
                <div className="col-span-2">
                    <span className={`inline-flex items-center gap-1 px-2 py-1 rounded-lg text-sm font-medium ${getActionColor(log.action)}`}>
                        {getActionIcon(log.action)} {log.action.replace(/_/g, ' ')}
                    </span>
                </div>
                <div className="col-span-3">
                    <code className="text-sm bg-muted px-2 py-1 rounded font-mono">
                        {formatAddress(log.wallet_address ?? 'System')}
                    </code>
                </div>
                <div className="col-span-3 text-sm text-muted-foreground">
                    <span className="font-medium">{log.resource_type}</span>
                    <span className="mx-1">→</span>
                    <code className="bg-muted px-1.5 py-0.5 rounded text-xs">
                        {formatAddress(log.resource_id ?? 'N/A')}
                    </code>
                </div>
                <div className="col-span-2 text-right">
                    <span className="text-muted-foreground text-sm">{isExpanded ? '▼' : '▶'}</span>
                </div>
            </div>

            {/* Mobile Layout */}
            <div className="md:hidden space-y-2">
                <div className="flex items-center justify-between">
                    <span className={`inline-flex items-center gap-1 px-2 py-1 rounded-lg text-sm font-medium ${getActionColor(log.action)}`}>
                        {getActionIcon(log.action)} {log.action.replace(/_/g, ' ')}
                    </span>
                    <span className="text-sm text-muted-foreground">{formatTime(log.timestamp)}</span>
                </div>
                <div className="text-sm text-muted-foreground">
                    <span className="font-medium">Actor:</span>{' '}
                    <code className="bg-muted px-1.5 py-0.5 rounded text-xs">
                        {formatAddress(log.wallet_address ?? 'System')}
                    </code>
                </div>
            </div>

            {/* Expanded Details */}
            {isExpanded && (
                <div className="mt-4 pt-4 border-t border-border">
                    <div className="bg-muted/50 rounded-xl p-4 space-y-3">
                        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 text-sm">
                            <div>
                                <span className="text-muted-foreground">Full Actor Address:</span>
                                <code className="block mt-1 bg-muted px-2 py-1 rounded text-xs font-mono break-all">
                                    {log.wallet_address ?? 'System'}
                                </code>
                            </div>
                            <div>
                                <span className="text-muted-foreground">Full Target ID:</span>
                                <code className="block mt-1 bg-muted px-2 py-1 rounded text-xs font-mono break-all">
                                    {log.resource_id ?? 'N/A'}
                                </code>
                            </div>
                            <div>
                                <span className="text-muted-foreground">Timestamp:</span>
                                <p className="mt-1">{new Date(log.timestamp).toLocaleString()}</p>
                            </div>
                            {typeof log.ip_address === 'string' && log.ip_address !== '' && (
                                <div>
                                    <span className="text-muted-foreground">IP Address:</span>
                                    <p className="mt-1">{log.ip_address}</p>
                                </div>
                            )}
                        </div>
                        {log.details && Object.keys(log.details).length > 0 && (
                            <div>
                                <span className="text-muted-foreground text-sm">Details:</span>
                                <pre className="mt-1 bg-muted px-3 py-2 rounded-lg text-xs overflow-x-auto">
                                    {JSON.stringify(log.details, null, 2)}
                                </pre>
                            </div>
                        )}
                    </div>
                </div>
            )}
        </div>
    );
}
