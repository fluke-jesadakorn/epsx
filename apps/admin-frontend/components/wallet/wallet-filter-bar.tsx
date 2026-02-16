'use client';

import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Search } from 'lucide-react';
import type { WalletFilters } from './types';

interface WalletFilterBarProps {
    filters: WalletFilters;
    onFilterChange: (filters: WalletFilters) => void;
}

export function WalletFilterBar({ filters, onFilterChange }: WalletFilterBarProps) {
    return (
        <div className="flex flex-col sm:flex-row gap-4 justify-between items-start sm:items-center bg-slate-900/40 backdrop-blur-2xl p-4 rounded-[32px] border border-white/5 shadow-xl">
            <div className="flex items-center gap-3 w-full sm:w-auto flex-1">
                <div className="relative w-full sm:max-w-md">
                    <Search className="absolute left-4 top-3 h-4 w-4 text-muted-foreground" />
                    <Input
                        placeholder="Search by address, name, or label..."
                        className="pl-11 h-12 bg-white/5 border-white/5 focus:bg-white/10 transition-all rounded-2xl placeholder:text-muted-foreground/50 font-medium"
                        value={filters.search}
                        onChange={(e) => onFilterChange({ ...filters, search: e.target.value })}
                    />
                </div>

                <Select
                    value={filters.status}
                    onValueChange={(v) => onFilterChange({ ...filters, status: v as WalletFilters['status'] })}
                >
                    <SelectTrigger className="w-[140px] h-12 bg-white/5 border-white/5 rounded-2xl font-bold text-sm">
                        <SelectValue placeholder="Status" />
                    </SelectTrigger>
                    <SelectContent className="bg-slate-900 border-white/10 rounded-2xl">
                        <SelectItem value="all">All Status</SelectItem>
                        <SelectItem value="active">Active</SelectItem>
                        <SelectItem value="disabled">Disabled</SelectItem>
                    </SelectContent>
                </Select>

                <Select
                    value={filters.platform}
                    onValueChange={(v) => onFilterChange({ ...filters, platform: v as WalletFilters['platform'] })}
                >
                    <SelectTrigger className="w-[140px] h-12 bg-white/5 border-white/5 rounded-2xl font-bold text-sm">
                        <SelectValue placeholder="Platform" />
                    </SelectTrigger>
                    <SelectContent className="bg-slate-900 border-white/10 rounded-2xl">
                        <SelectItem value="all">Platforms</SelectItem>
                        <SelectItem value="analytics">Analytics</SelectItem>
                        <SelectItem value="pay">Pay</SelectItem>
                        <SelectItem value="token">Token</SelectItem>
                        <SelectItem value="markets">Markets</SelectItem>
                    </SelectContent>
                </Select>
            </div>
        </div>
    );
}
