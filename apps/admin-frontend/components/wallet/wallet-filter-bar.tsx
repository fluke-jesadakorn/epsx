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
        <div className="flex flex-col sm:flex-row gap-3 justify-between items-start sm:items-center bg-card p-4 rounded-xl border border-border/20 shadow-sm">
            <div className="flex items-center gap-3 w-full sm:w-auto flex-1">
                <div className="relative w-full sm:max-w-md">
                    <Search className="absolute left-4 top-3 h-4 w-4 text-muted-foreground" />
                    <Input
                        placeholder="Search by address, name, or label..."
                        className="pl-11 h-10 bg-muted/50 border-border/50 focus:border-[#1fc7d4]/50 transition-all rounded-xl placeholder:text-muted-foreground/50 text-sm"
                        value={filters.search}
                        onChange={(e) => onFilterChange({ ...filters, search: e.target.value })}
                    />
                </div>

                <Select
                    value={filters.status}
                    onValueChange={(v) => onFilterChange({ ...filters, status: v as WalletFilters['status'] })}
                >
                    <SelectTrigger className="w-[130px] h-10 bg-muted/50 border-border/50 rounded-xl text-sm">
                        <SelectValue placeholder="Status" />
                    </SelectTrigger>
                    <SelectContent className="bg-card border-border/50 rounded-xl">
                        <SelectItem value="all">All Status</SelectItem>
                        <SelectItem value="active">Active</SelectItem>
                        <SelectItem value="disabled">Disabled</SelectItem>
                    </SelectContent>
                </Select>

                <Select
                    value={filters.platform}
                    onValueChange={(v) => onFilterChange({ ...filters, platform: v as WalletFilters['platform'] })}
                >
                    <SelectTrigger className="w-[130px] h-10 bg-muted/50 border-border/50 rounded-xl text-sm">
                        <SelectValue placeholder="Platform" />
                    </SelectTrigger>
                    <SelectContent className="bg-card border-border/50 rounded-xl">
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
