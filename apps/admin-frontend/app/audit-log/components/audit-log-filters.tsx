'use client';

import { Download, RefreshCw, Search } from 'lucide-react';
import { ACTION_CATEGORIES, ActionType } from '../types';

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
        <div className="bg-card rounded-2xl p-4 shadow-xl border border-border/20">
            <div className="flex flex-col lg:flex-row gap-4">
                {/* Search */}
                <div className="relative flex-1">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-muted-foreground" />
                    <input
                        type="text"
                        placeholder="Search by actor, action, or target..."
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        className="w-full pl-10 pr-4 py-3 bg-muted border border-border rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
                    />
                </div>

                {/* Category Filter */}
                <div className="flex gap-2 overflow-x-auto pb-2 lg:pb-0">
                    {Object.entries(ACTION_CATEGORIES).map(([key, { label, icon }]) => (
                        <button
                            key={key}
                            onClick={() => setSelectedCategory(key as ActionType)}
                            className={`px-4 py-2 rounded-xl font-medium whitespace-nowrap transition-all ${selectedCategory === key
                                ? 'bg-gradient-to-r from-indigo-500 to-purple-500 text-white shadow-lg'
                                : 'bg-muted text-foreground hover:bg-muted/80'
                                }`}
                        >
                            {icon} {label}
                        </button>
                    ))}
                </div>
            </div>

            {/* Date Range & Actions */}
            <div className="flex flex-col sm:flex-row gap-4 mt-4">
                <div className="flex gap-2 flex-1">
                    <input
                        type="date"
                        value={dateFrom}
                        onChange={(e) => setDateFrom(e.target.value)}
                        className="flex-1 px-4 py-2 bg-muted border border-border rounded-xl"
                        placeholder="From"
                    />
                    <span className="self-center text-muted-foreground">to</span>
                    <input
                        type="date"
                        value={dateTo}
                        onChange={(e) => setDateTo(e.target.value)}
                        className="flex-1 px-4 py-2 bg-muted border border-border rounded-xl"
                        placeholder="To"
                    />
                </div>

                <div className="flex gap-2">
                    <button
                        onClick={fetchLogs}
                        disabled={isLoadingLogs}
                        className="px-4 py-2 bg-indigo-100 dark:bg-indigo-900/50 text-indigo-600 dark:text-indigo-300 rounded-xl hover:bg-indigo-200 dark:hover:bg-indigo-900 transition-colors flex items-center gap-2"
                    >
                        <RefreshCw className={`w-4 h-4 ${isLoadingLogs ? 'animate-spin' : ''}`} />
                        Refresh
                    </button>
                    <button
                        onClick={handleExport}
                        className="px-4 py-2 bg-emerald-100 dark:bg-emerald-900/50 text-emerald-600 dark:text-emerald-300 rounded-xl hover:bg-emerald-200 dark:hover:bg-emerald-900 transition-colors flex items-center gap-2"
                    >
                        <Download className="w-4 h-4" />
                        Export
                    </button>
                </div>
            </div>
        </div>
    );
}
