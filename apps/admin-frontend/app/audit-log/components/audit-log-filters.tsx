'use client';

import { Download, RefreshCw, Search } from 'lucide-react';
import type { ActionType } from '../types';
import { ACTION_CATEGORIES } from '../types';

interface AuditLogFiltersProps {
    searchQuery: string;
    setSearchQuery: (q: string) => void;
    selectedCategory: ActionType;
    setSelectedCategory: (c: ActionType) => void;
    dateFrom: string;
    setDateFrom: (d: string) => void;
    dateTo: string;
    setDateTo: (d: string) => void;
    fetchLogs: () => void;
    handleExport: () => void;
    isLoadingLogs: boolean;
}

export function AuditLogFilters({
    searchQuery,
    setSearchQuery,
    selectedCategory,
    setSelectedCategory,
    dateFrom,
    setDateFrom,
    dateTo,
    setDateTo,
    fetchLogs,
    handleExport,
    isLoadingLogs,
}: AuditLogFiltersProps) {
    return (
        <div className="rounded-xl border border-border/20 bg-card p-4 shadow-xl">
            <div className="flex flex-col lg:flex-row gap-3">
                {/* Search */}
                <div className="relative flex-1">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
                    <input
                        type="text"
                        placeholder="Search by actor, action, or target..."
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        className="w-full pl-9 pr-4 py-2.5 bg-muted/50 border border-border/50 rounded-xl text-sm focus:ring-1 focus:ring-[#7645d9] focus:border-transparent transition-all"
                    />
                </div>

                {/* Category Filter */}
                <div className="flex gap-2 overflow-x-auto pb-1 lg:pb-0">
                    {Object.entries(ACTION_CATEGORIES).map(([key, { label, icon }]) => (
                        <button
                            key={key}
                            onClick={() => setSelectedCategory(key as ActionType)}
                            className={`px-3 py-2 rounded-lg font-medium text-sm whitespace-nowrap transition-all ${selectedCategory === key
                                ? 'bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white shadow-md'
                                : 'bg-muted/50 text-muted-foreground hover:text-foreground border border-border/40 hover:border-[#7645d9]/30'
                                }`}
                        >
                            {icon} {label}
                        </button>
                    ))}
                </div>
            </div>

            {/* Date Range & Actions */}
            <div className="flex flex-col sm:flex-row gap-3 mt-3">
                <div className="flex gap-2 flex-1">
                    <input
                        type="date"
                        value={dateFrom}
                        onChange={(e) => setDateFrom(e.target.value)}
                        className="flex-1 px-3 py-2 bg-muted/50 border border-border/50 rounded-xl text-sm"
                        placeholder="From"
                    />
                    <span className="self-center text-muted-foreground text-sm">to</span>
                    <input
                        type="date"
                        value={dateTo}
                        onChange={(e) => setDateTo(e.target.value)}
                        className="flex-1 px-3 py-2 bg-muted/50 border border-border/50 rounded-xl text-sm"
                        placeholder="To"
                    />
                </div>

                <div className="flex gap-2">
                    <button
                        onClick={fetchLogs}
                        disabled={isLoadingLogs}
                        className="px-4 py-2 bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white rounded-xl hover:opacity-90 transition-all text-sm font-semibold flex items-center gap-2 disabled:opacity-50"
                    >
                        <RefreshCw className={`w-4 h-4 ${isLoadingLogs ? 'animate-spin' : ''}`} />
                        Refresh
                    </button>
                    <button
                        onClick={handleExport}
                        className="px-4 py-2 bg-card border border-border/40 hover:border-[#31d0aa]/40 text-foreground rounded-xl transition-all text-sm font-semibold flex items-center gap-2"
                    >
                        <Download className="w-4 h-4 text-[#31d0aa]" />
                        Export
                    </button>
                </div>
            </div>
        </div>
    );
}
