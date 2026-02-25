'use client';

import { ChevronLeft, ChevronRight, RefreshCw } from 'lucide-react';
import React from 'react';
import type { AuditLogEntry } from '../types';
import { AuditLogRow } from './audit-log-row';

interface AuditLogTableProps {
    isLoadingLogs: boolean;
    error: string | null;
    logs: AuditLogEntry[];
    page: number;
    totalPages: number;
    setPage: React.Dispatch<React.SetStateAction<number>>;
    fetchLogs: () => void;
}

export function AuditLogTable({
    isLoadingLogs,
    error,
    logs,
    page,
    totalPages,
    setPage,
    fetchLogs,
}: AuditLogTableProps) {
    return (
        <div className="rounded-2xl border border-border/20 overflow-hidden bg-card shadow-xl">
            <div className="h-[3px] bg-gradient-to-r from-[#7645d9] to-[#1fc7d4]" />
            {isLoadingLogs ? (
                <div className="p-8 text-center">
                    <RefreshCw className="w-8 h-8 animate-spin mx-auto mb-4 text-[#7645d9]" />
                    <p className="text-muted-foreground text-sm">Loading audit logs...</p>
                </div>
            ) : error ? (
                <div className="p-8 text-center">
                    <p className="text-destructive mb-4 text-sm">{error}</p>
                    <button
                        onClick={fetchLogs}
                        className="px-4 py-2 bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white rounded-xl hover:opacity-90 text-sm font-semibold"
                    >
                        Retry
                    </button>
                </div>
            ) : logs.length === 0 ? (
                <div className="p-8 text-center">
                    <div className="text-5xl mb-3">📭</div>
                    <p className="text-muted-foreground text-sm">No audit logs found</p>
                </div>
            ) : (
                <>
                    {/* Table Header */}
                    <div className="hidden md:grid grid-cols-12 gap-4 px-4 py-3 bg-muted/30 border-b border-border/30 text-[10px] font-bold text-muted-foreground uppercase tracking-[0.15em]">
                        <div className="col-span-2">Time</div>
                        <div className="col-span-2">Action</div>
                        <div className="col-span-3">Actor</div>
                        <div className="col-span-3">Target</div>
                        <div className="col-span-2 text-right">Details</div>
                    </div>

                    {/* Log Entries */}
                    <div className="divide-y divide-border/30">
                        {logs.map((log) => (
                            <AuditLogRow key={log.id} log={log} />
                        ))}
                    </div>
                </>
            )}

            {/* Pagination */}
            {logs.length > 0 && (
                <div className="px-4 py-3 bg-muted/20 border-t border-border/30 flex items-center justify-between">
                    <span className="text-xs text-muted-foreground">
                        Page {page} of {totalPages}
                    </span>
                    <div className="flex gap-2">
                        <button
                            onClick={() => setPage(p => Math.max(1, p - 1))}
                            disabled={page === 1}
                            className="p-1.5 rounded-lg bg-card border border-border/40 disabled:opacity-40 hover:bg-muted transition-colors"
                        >
                            <ChevronLeft className="w-4 h-4" />
                        </button>
                        <button
                            onClick={() => setPage(p => Math.min(totalPages, p + 1))}
                            disabled={page === totalPages}
                            className="p-1.5 rounded-lg bg-card border border-border/40 disabled:opacity-40 hover:bg-muted transition-colors"
                        >
                            <ChevronRight className="w-4 h-4" />
                        </button>
                    </div>
                </div>
            )}
        </div>
    );
}
