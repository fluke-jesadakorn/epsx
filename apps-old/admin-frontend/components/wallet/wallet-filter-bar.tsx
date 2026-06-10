'use client';

import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { cn } from '@/lib/utils';
import { ArrowDownUp, Search } from 'lucide-react';
import type { WalletFilters } from './types';

interface WalletFilterBarProps {
    filters: WalletFilters;
    onFilterChange: (filters: WalletFilters) => void;
}

const TRIGGER_CLS = 'h-10 bg-muted/30 border-border/30 rounded-xl text-sm hover:border-border/50 transition-colors';
const CONTENT_CLS = 'rounded-xl border-border/20 bg-card';

export function WalletFilterBar({ filters, onFilterChange }: WalletFilterBarProps) {
    const toggleSortOrder = () => {
        onFilterChange({ ...filters, sortOrder: filters.sortOrder === 'desc' ? 'asc' : 'desc' });
    };

    return (
        <div className="flex flex-col sm:flex-row gap-3 bg-card p-4 rounded-2xl border border-border/20 shadow-lg">
            {/* Search */}
            <div className="relative flex-1 min-w-0">
                <Search className="absolute left-3.5 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground pointer-events-none" />
                <Input
                    placeholder="Search address, label, or note..."
                    className="pl-10 h-10 bg-muted/30 border-border/30 focus:border-[#1fc7d4]/50 transition-colors rounded-xl placeholder:text-muted-foreground/40 text-sm"
                    value={filters.search}
                    onChange={(e) => onFilterChange({ ...filters, search: e.target.value })}
                />
            </div>

            {/* Controls */}
            <div className="flex items-center gap-2 flex-shrink-0 flex-wrap">
                <Select
                    value={filters.status}
                    onValueChange={(v) => onFilterChange({ ...filters, status: v as WalletFilters['status'] })}
                >
                    <SelectTrigger className={cn('w-[120px]', TRIGGER_CLS)}>
                        <SelectValue placeholder="Status" />
                    </SelectTrigger>
                    <SelectContent className={CONTENT_CLS}>
                        <SelectItem value="all">All Status</SelectItem>
                        <SelectItem value="active">Active</SelectItem>
                        <SelectItem value="disabled">Disabled</SelectItem>
                    </SelectContent>
                </Select>

                <Select
                    value={filters.platform}
                    onValueChange={(v) => onFilterChange({ ...filters, platform: v as WalletFilters['platform'] })}
                >
                    <SelectTrigger className={cn('w-[130px]', TRIGGER_CLS)}>
                        <SelectValue placeholder="Platform" />
                    </SelectTrigger>
                    <SelectContent className={CONTENT_CLS}>
                        <SelectItem value="all">All Platforms</SelectItem>
                        <SelectItem value="analytics">Analytics</SelectItem>
                        <SelectItem value="pay">Pay</SelectItem>
                        <SelectItem value="token">Token</SelectItem>
                        <SelectItem value="markets">Markets</SelectItem>
                    </SelectContent>
                </Select>

                <Select
                    value={filters.sortBy}
                    onValueChange={(v) => onFilterChange({ ...filters, sortBy: v as WalletFilters['sortBy'] })}
                >
                    <SelectTrigger className={cn('w-[140px]', TRIGGER_CLS)}>
                        <SelectValue placeholder="Sort by" />
                    </SelectTrigger>
                    <SelectContent className={CONTENT_CLS}>
                        <SelectItem value="created_at">Date Created</SelectItem>
                        <SelectItem value="last_auth_at">Last Active</SelectItem>
                        <SelectItem value="wallet_address">Address</SelectItem>
                    </SelectContent>
                </Select>

                <button
                    type="button"
                    title={filters.sortOrder === 'desc' ? 'Descending — click to toggle' : 'Ascending — click to toggle'}
                    onClick={toggleSortOrder}
                    className="h-10 w-10 rounded-xl bg-muted/30 border border-border/30 hover:border-[#1fc7d4]/40 hover:bg-muted/50 hover:text-[#1fc7d4] transition-colors flex items-center justify-center text-muted-foreground flex-shrink-0"
                >
                    <ArrowDownUp className={cn('h-4 w-4 transition-transform duration-300', filters.sortOrder === 'asc' && 'rotate-180')} />
                </button>
            </div>
        </div>
    );
}
