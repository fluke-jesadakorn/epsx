'use client';

import React from 'react';
import type { AuditLogEntry } from '../types';

interface Props {
    log: AuditLogEntry;
}

type DetailShape = 'before_after' | 'permission' | 'assignment' | 'payment' | 'flat';

function detectShape(details: Record<string, unknown> | null): DetailShape {
    if (details === null || Object.keys(details).length === 0) {
        return 'flat';
    }
    if ('before' in details || 'after' in details) {
        return 'before_after';
    }
    if ('performed_by' in details && ('previous_state' in details || 'new_state' in details)) {
        return 'permission';
    }
    if ('old_value' in details || 'new_value' in details) {
        return 'assignment';
    }
    if ('old_status' in details || 'new_status' in details) {
        return 'payment';
    }
    return 'flat';
}

function formatVal(v: unknown): string {
    if (v === null || v === undefined) {
        return '-';
    }
    if (typeof v === 'boolean') {
        return v ? 'Yes' : 'No';
    }
    if (typeof v === 'object') {
        return JSON.stringify(v, null, 2);
    }
    return String(v);
}

function formatKey(k: string): string {
    return k.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase());
}

function truncAddr(addr: string): string {
    if (addr.length > 16) {
        return `${addr.slice(0, 6)}...${addr.slice(-4)}`;
    }
    return addr;
}

function ResultBadge({ result }: { result: string }): React.JSX.Element {
    const cls =
        result === 'success' ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-400' :
        result === 'denied' ? 'bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400' :
        'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400';
    return <span className={`px-2 py-0.5 rounded text-xs font-medium ${cls}`}>{result}</span>;
}

function ResourceBadge({ type }: { type: string }): React.JSX.Element {
    return <span className="px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400">{type}</span>;
}

function ValDisplay({ v }: { v: unknown }): React.JSX.Element {
    if (v === null || v === undefined) {
        return <span className="text-muted-foreground italic">-</span>;
    }
    if (typeof v === 'boolean') {
        const cls = v
            ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30'
            : 'bg-red-100 text-red-700 dark:bg-red-900/30';
        return <span className={`px-2 py-0.5 rounded text-xs font-medium ${cls}`}>{v ? 'true' : 'false'}</span>;
    }
    if (typeof v === 'object') {
        return <pre className="text-xs bg-muted px-2 py-1 rounded overflow-x-auto whitespace-pre-wrap">{JSON.stringify(v, null, 2)}</pre>;
    }
    return <span className="text-sm">{String(v)}</span>;
}

function MetaField({ label, value }: { label: string; value: unknown }): React.JSX.Element | null {
    if (value === null || value === undefined || value === '') {
        return null;
    }
    return (
        <div>
            <span className="text-xs text-muted-foreground">{label}</span>
            <div className="mt-0.5"><ValDisplay v={value} /></div>
        </div>
    );
}

function DiffView({ before, after }: { before: Record<string, unknown> | null; after: Record<string, unknown> | null }): React.JSX.Element {
    const bObj: Record<string, unknown> = before !== null && typeof before === 'object' ? before : {};
    const aObj: Record<string, unknown> = after !== null && typeof after === 'object' ? after : {};
    const allKeys = [...new Set([...Object.keys(bObj), ...Object.keys(aObj)])];

    if (allKeys.length === 0) {
        return <p className="text-sm text-muted-foreground italic">No changes recorded</p>;
    }

    return (
        <div className="overflow-x-auto">
            <table className="w-full text-sm">
                <thead>
                    <tr className="border-b border-border">
                        <th className="text-left py-1.5 px-2 text-xs text-muted-foreground font-medium w-1/4">Field</th>
                        <th className="text-left py-1.5 px-2 text-xs text-red-500 font-medium w-[37.5%]">Before</th>
                        <th className="text-left py-1.5 px-2 text-xs text-emerald-500 font-medium w-[37.5%]">After</th>
                    </tr>
                </thead>
                <tbody>
                    {allKeys.map(k => {
                        const bv = formatVal(bObj[k]);
                        const av = formatVal(aObj[k]);
                        const changed = bv !== av;
                        return (
                            <tr key={k} className={changed ? 'bg-amber-50/50 dark:bg-amber-900/10' : ''}>
                                <td className="py-1.5 px-2 font-medium text-xs">{formatKey(k)}</td>
                                <td className={`py-1.5 px-2 text-xs ${changed ? 'bg-red-50/50 dark:bg-red-900/10' : ''}`}>
                                    <code className="break-all">{bv}</code>
                                </td>
                                <td className={`py-1.5 px-2 text-xs ${changed ? 'bg-emerald-50/50 dark:bg-emerald-900/10' : ''}`}>
                                    <code className="break-all">{av}</code>
                                </td>
                            </tr>
                        );
                    })}
                </tbody>
            </table>
        </div>
    );
}

function PermissionView({ details }: { details: Record<string, unknown> }): React.JSX.Element {
    return (
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <MetaField label="Permission" value={details.permission_string ?? details.group_name} />
            <MetaField label="Performed By" value={details.performed_by} />
            <MetaField label="Previous State" value={details.previous_state} />
            <MetaField label="New State" value={details.new_state} />
            <MetaField label="Reason" value={details.reason} />
        </div>
    );
}

function AssignmentView({ details }: { details: Record<string, unknown> }): React.JSX.Element {
    return (
        <div className="space-y-3">
            <div className="flex items-center gap-3 text-sm">
                <div className="flex-1 bg-red-50/50 dark:bg-red-900/10 rounded px-3 py-2">
                    <span className="text-xs text-muted-foreground block mb-1">Old Value</span>
                    <ValDisplay v={details.old_value ?? '-'} />
                </div>
                <span className="text-muted-foreground font-bold text-lg shrink-0">&rarr;</span>
                <div className="flex-1 bg-emerald-50/50 dark:bg-emerald-900/10 rounded px-3 py-2">
                    <span className="text-xs text-muted-foreground block mb-1">New Value</span>
                    <ValDisplay v={details.new_value ?? '-'} />
                </div>
            </div>
            <MetaField label="Reason" value={details.reason} />
        </div>
    );
}

function PaymentView({ details }: { details: Record<string, unknown> }): React.JSX.Element {
    const rest = { ...details };
    delete rest.old_status;
    delete rest.new_status;
    delete rest.reason;
    const restKeys = Object.keys(rest).filter(k => rest[k] !== null && rest[k] !== undefined);

    return (
        <div className="space-y-3">
            <div className="flex items-center gap-3 text-sm">
                <StatusBadge status={String(details.old_status ?? '-')} />
                <span className="text-muted-foreground font-bold text-lg">&rarr;</span>
                <StatusBadge status={String(details.new_status ?? '-')} />
            </div>
            <MetaField label="Reason" value={details.reason} />
            {restKeys.length > 0 && (
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-2 border-t border-border">
                    {restKeys.map(k => <MetaField key={k} label={formatKey(k)} value={rest[k]} />)}
                </div>
            )}
        </div>
    );
}

function StatusBadge({ status }: { status: string }): React.JSX.Element {
    const cls =
        status === 'confirmed' || status === 'completed' || status === 'active'
            ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30'
            : status === 'pending' || status === 'processing'
            ? 'bg-amber-100 text-amber-700 dark:bg-amber-900/30'
            : status === 'failed' || status === 'cancelled' || status === 'disabled'
            ? 'bg-red-100 text-red-700 dark:bg-red-900/30'
            : 'bg-muted text-muted-foreground';
    return <span className={`px-3 py-1 rounded-lg text-sm font-medium ${cls}`}>{status}</span>;
}

function FlatView({ details }: { details: Record<string, unknown> }): React.JSX.Element {
    const entries = Object.entries(details).filter(([, v]) => v !== null && v !== undefined && v !== '');
    if (entries.length === 0) {
        return <p className="text-sm text-muted-foreground italic">No additional details</p>;
    }
    return (
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            {entries.map(([k, v]) => <MetaField key={k} label={formatKey(k)} value={v} />)}
        </div>
    );
}

function ChangesSection({ shape, details }: { shape: DetailShape; details: Record<string, unknown> }): React.JSX.Element {
    return (
        <div>
            <h4 className="text-xs font-medium text-muted-foreground uppercase tracking-wider mb-2">Changes</h4>
            {shape === 'before_after' && (
                <DiffView
                    before={details.before as Record<string, unknown> | null}
                    after={details.after as Record<string, unknown> | null}
                />
            )}
            {shape === 'permission' && <PermissionView details={details} />}
            {shape === 'assignment' && <AssignmentView details={details} />}
            {shape === 'payment' && <PaymentView details={details} />}
            {shape === 'flat' && <FlatView details={details} />}
        </div>
    );
}

export function AuditDetailView({ log }: Props): React.JSX.Element {
    const details = log.details ?? log.additional_data ?? null;
    const shape = detectShape(details);
    const actionLabel = (log.action_raw ?? log.action).replace(/_/g, ' ');
    const resourceLabel = log.resource_type_raw ?? log.resource_type;

    return (
        <div className="mt-4 pt-4 border-t border-border">
            <div className="bg-muted/50 rounded-xl p-4 space-y-4">
                {/* Header */}
                <div className="flex flex-wrap items-center gap-2">
                    <ResultBadge result={log.result} />
                    <ResourceBadge type={resourceLabel} />
                    <span className="text-sm font-medium capitalize">{actionLabel}</span>
                </div>

                {/* Meta grid */}
                <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 text-sm">
                    <div>
                        <span className="text-xs text-muted-foreground">Actor</span>
                        <code className="block mt-0.5 bg-muted px-2 py-1 rounded text-xs font-mono break-all">
                            {log.wallet_address !== null ? truncAddr(log.wallet_address) : 'System'}
                        </code>
                    </div>
                    <div>
                        <span className="text-xs text-muted-foreground">Target</span>
                        <code className="block mt-0.5 bg-muted px-2 py-1 rounded text-xs font-mono break-all">
                            {log.resource_id !== null ? truncAddr(log.resource_id) : '-'}
                        </code>
                    </div>
                    <div>
                        <span className="text-xs text-muted-foreground">Timestamp</span>
                        <p className="mt-0.5 text-xs">{new Date(log.timestamp).toLocaleString()}</p>
                    </div>
                    {typeof log.ip_address === 'string' && log.ip_address !== '' && (
                        <div>
                            <span className="text-xs text-muted-foreground">IP Address</span>
                            <p className="mt-0.5 text-xs">{log.ip_address}</p>
                        </div>
                    )}
                </div>

                {/* Changes section */}
                {details !== null && Object.keys(details).length > 0 && (
                    <ChangesSection shape={shape} details={details} />
                )}
            </div>
        </div>
    );
}
