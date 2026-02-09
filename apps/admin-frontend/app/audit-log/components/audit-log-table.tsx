'use client';

import { ChevronLeft, ChevronRight, RefreshCw } from 'lucide-react';
import React from 'react';
import { AuditLogEntry } from '../types';
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
        <div className="bg-card rounded-2xl shadow-xl border border-border/20 overflow-hidden">
            {isLoadingLogs ? (
                <div className="p-8 text-center">
                    <RefreshCw className="w-8 h-8 animate-spin mx-auto mb-4 text-indigo-500" />
                    <p className="text-muted-foreground">Loading audit logs...</p>
                </div>
            ) : error ? (
                <div className="p-8 text-center">
                    <p className="text-red-500 mb-4">{error}</p>
                    <button
                        onClick={fetchLogs}
                        className="px-4 py-2 bg-indigo-500 text-white rounded-xl hover:bg-indigo-600"
                    >
                        Retry
                    </button>
                </div>
            ) : logs.length === 0 ? (
                <div className="p-8 text-center">
                    <div className="text-6xl mb-4">📭</div>
                    <p className="text-muted-foreground">No audit logs found</p>
                </div>
            ) : (
                <>
                    {/* Table Header */}
                    <div className="hidden md:grid grid-cols-12 gap-4 p-4 bg-muted/50 border-b border-border font-medium text-sm text-muted-foreground">
                        <div className="col-span-2">Time</div>
                        <div className="col-span-2">Action</div>
                        <div className="col-span-3">Actor</div>
                        <div className="col-span-3">Target</div>
                        <div className="col-span-2 text-right">Details</div>
                    </div>

                    {/* Log Entries */}
                    <div className="divide-y divide-border">
                        {logs.map((log) => (
                            <AuditLogRow key={log.id} log={log} />
                        ))}
                    </div>
                </>
            )}

            {/* Pagination */}
            {logs.length > 0 && (
                <div className="p-4 bg-muted/50 border-t border-border flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">
                        Page {page} of {totalPages}
                    </span>
                    <div className="flex gap-2">
                        <button
                            onClick={() => setPage(p => Math.max(1, p - 1))}
                            disabled={page === 1}
                            className="p-2 rounded-lg bg-card border border-border disabled:opacity-50 hover:bg-muted"
                        >
                            <ChevronLeft className="w-5 h-5" />
                        </button>
                        <button
                            onClick={() => setPage(p => Math.min(totalPages, p + 1))}
                            disabled={page === totalPages}
                            className="p-2 rounded-lg bg-card border border-border disabled:opacity-50 hover:bg-muted"
                        >
                            <ChevronRight className="w-5 h-5" />
                        </button>
                    </div>
                </div>
            )}
        </div>
    );
}
