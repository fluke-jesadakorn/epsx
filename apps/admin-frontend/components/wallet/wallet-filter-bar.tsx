'use client';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Plus, Search } from 'lucide-react';
import type { WalletFilters } from './types';

interface WalletFilterBarProps {
    filters: WalletFilters;
    onFilterChange: (filters: WalletFilters) => void;
    onAddWallet: () => void;
}

export function WalletFilterBar({ filters, onFilterChange, onAddWallet }: WalletFilterBarProps) {
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
                        <SelectItem value="pending">Pending</SelectItem>
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

            <div className="flex items-center gap-2 w-full sm:w-auto">
                <Button
                    onClick={onAddWallet}
                    className="w-full sm:w-auto h-12 px-6 bg-[#1fc7d4] hover:bg-[#1fc7d4]/90 text-white font-bold rounded-2xl shadow-lg shadow-cyan-500/20 active:scale-95 transition-all"
                >
                    <Plus className="h-5 w-5 mr-2" />
                    Add New Wallet
                </Button>
            </div>
        </div>
    );
}
