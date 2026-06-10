'use client';

import React, { useState } from 'react';
import type { AuditLogEntry } from '../types';
import { AuditDetailView } from './audit-detail-view';

interface AuditLogRowProps {
    log: AuditLogEntry;
}

export function AuditLogRow({ log }: AuditLogRowProps): React.JSX.Element {
    const [isExpanded, setIsExpanded] = useState(false);
    const rawAction = log.action_raw ?? log.action;
    const rawResource = log.resource_type_raw ?? log.resource_type;

    const getActionIcon = (action: string): string => {
        const iconMap: Record<string, string> = {
            grant: '🔓',
            revoke: '🔒',
            bulk_assign: '📦',
            bulk_remove: '📦',
            assign: '🔗',
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
        const colorMap: Array<[string[], string]> = [
            [['create', 'enable', 'grant'], 'text-emerald-600 bg-emerald-100 dark:bg-emerald-900/30'],
            [['delete', 'disable', 'remove', 'revoke'], 'text-red-600 bg-red-100 dark:bg-red-900/30'],
            [['update', 'edit', 'assign'], 'text-blue-600 bg-blue-100 dark:bg-blue-900/30'],
            [['permission', 'bulk'], 'text-purple-600 bg-purple-100 dark:bg-purple-900/30'],
        ];
        const match = colorMap.find(([keys]) => keys.some(k => action.includes(k)));
        return match !== undefined ? match[1] : 'text-muted-foreground bg-muted';
    };

    const fmtAddr = (address: string): string => {
        if (address.length > 16) {
            return `${address.slice(0, 6)}...${address.slice(-4)}`;
        }
        return address;
    };

    const fmtTime = (dateStr: string): string => {
        const diffMs = Date.now() - new Date(dateStr).getTime();
        const mins = Math.floor(diffMs / 60000);
        const hrs = Math.floor(diffMs / 3600000);
        const days = Math.floor(diffMs / 86400000);
        if (mins < 1) { return 'Just now'; }
        if (mins < 60) { return `${mins}m ago`; }
        if (hrs < 24) { return `${hrs}h ago`; }
        if (days < 7) { return `${days}d ago`; }
        return new Date(dateStr).toLocaleDateString();
    };

    return (
        <div
            className="p-4 hover:bg-muted/50 transition-colors cursor-pointer"
            onClick={() => setIsExpanded(!isExpanded)}
        >
            {/* Desktop Layout */}
            <div className="hidden md:grid grid-cols-12 gap-4 items-center">
                <div className="col-span-2 text-sm text-muted-foreground">
                    {fmtTime(log.timestamp)}
                </div>
                <div className="col-span-2">
                    <span className={`inline-flex items-center gap-1 px-2 py-1 rounded-lg text-sm font-medium ${getActionColor(rawAction)}`}>
                        {getActionIcon(rawAction)} {rawAction.replace(/_/g, ' ')}
                    </span>
                </div>
                <div className="col-span-3">
                    <code className="text-sm bg-muted px-2 py-1 rounded font-mono">
                        {fmtAddr(log.wallet_address ?? 'System')}
                    </code>
                </div>
                <div className="col-span-3 text-sm text-muted-foreground">
                    <span className="font-medium">{rawResource}</span>
                    <span className="mx-1">&rarr;</span>
                    <code className="bg-muted px-1.5 py-0.5 rounded text-xs">
                        {fmtAddr(log.resource_id ?? 'N/A')}
                    </code>
                </div>
                <div className="col-span-2 text-right">
                    <span className="text-muted-foreground text-sm">{isExpanded ? '▼' : '▶'}</span>
                </div>
            </div>

            {/* Mobile Layout */}
            <div className="md:hidden space-y-2">
                <div className="flex items-center justify-between">
                    <span className={`inline-flex items-center gap-1 px-2 py-1 rounded-lg text-sm font-medium ${getActionColor(rawAction)}`}>
                        {getActionIcon(rawAction)} {rawAction.replace(/_/g, ' ')}
                    </span>
                    <span className="text-sm text-muted-foreground">{fmtTime(log.timestamp)}</span>
                </div>
                <div className="text-sm text-muted-foreground">
                    <span className="font-medium">Actor:</span>{' '}
                    <code className="bg-muted px-1.5 py-0.5 rounded text-xs">
                        {fmtAddr(log.wallet_address ?? 'System')}
                    </code>
                </div>
            </div>

            {/* Expanded Details */}
            {isExpanded && <AuditDetailView log={log} />}
        </div>
    );
}
