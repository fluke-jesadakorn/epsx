'use client';

import { Activity, Key, Plus, Settings, Shield } from 'lucide-react';
import { useRouter } from 'next/navigation';
import React from 'react';

import { StatsCard } from '../shared/stats-card';

import { Button } from '@/components/ui/button';
import type { ApiKeyResponse, Module } from '@/shared/api/plans';

interface OverviewTabProps {
    apiKeys: ApiKeyResponse[];
    modules: Module[];
    onCreateKey: () => void;
}

/**
 * Get status badge color
 * @param status
 */
const getStatusColor = (status: string): string => {
    switch (status) {
        case 'active':
            return 'bg-green-500/10 text-green-400 border border-green-500/10';
        case 'revoked':
            return 'bg-red-500/10 text-red-400 border border-red-500/10';
        case 'expired':
            return 'bg-yellow-500/10 text-yellow-400 border border-yellow-500/10';
        default:
            return 'bg-white/5 text-muted-foreground border border-white/10';
    }
};

/**
 * Overview tab component showing stats and recent API keys
 * @param root0
 * @param root0.apiKeys
 * @param root0.modules
 * @param root0.onCreateKey
 */
export const OverviewTab: React.FC<OverviewTabProps> = ({
    apiKeys,
    modules,
    onCreateKey,
}) => {
    const router = useRouter();

    const totalRequests = apiKeys.reduce((sum, key) => sum + key.total_requests, 0);
    const activeKeys = apiKeys.filter(key => key.status === 'active').length;

    return (
        <div className="space-y-6">
            {/* Stats Overview */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <StatsCard
                    title="Total API Keys"
                    value={apiKeys.length}
                    icon={Key}
                    iconBgColor="bg-cyan-500"
                    iconColor="text-cyan-400"
                />
                <StatsCard
                    title="Active Keys"
                    value={activeKeys}
                    icon={Shield}
                    iconBgColor="bg-green-500"
                    iconColor="text-green-400"
                />
                <StatsCard
                    title="Total Requests"
                    value={totalRequests}
                    icon={Activity}
                    iconBgColor="bg-purple-500"
                    iconColor="text-purple-400"
                />
                <StatsCard
                    title="Available Modules"
                    value={modules.length}
                    icon={Settings}
                    iconBgColor="bg-warning"
                    iconColor="text-yellow-400"
                />
            </div>

            {/* Recent API Keys */}
            <div className="relative overflow-hidden rounded-[32px] bg-slate-900/40 backdrop-blur-2xl border border-white/5 shadow-xl">
                <div className="p-8 border-b border-white/5">
                    <div className="flex items-center justify-between">
                        <div>
                            <h3 className="text-xl font-black text-foreground uppercase tracking-tight mb-1">
                                Recent API Keys
                            </h3>
                            <p className="text-sm font-bold text-muted-foreground">Most recently used and active keys</p>
                        </div>
                        <Button
                            onClick={onCreateKey}
                            className="bg-gradient-to-r from-[#1fc7d4] to-[#7645d9] hover:opacity-90 text-white font-black px-6 py-6 rounded-2xl shadow-lg shadow-cyan-500/20 active:scale-95 transition-all text-xs uppercase tracking-widest"
                        >
                            <Plus className="w-5 h-5 mr-2" />
                            Create New Key
                        </Button>
                    </div>
                </div>
                <div className="divide-y divide-white/5">
                    {apiKeys.length === 0 ? (
                        <div className="p-12 text-center text-muted-foreground font-bold">
                            No API keys found. Create one to get started.
                        </div>
                    ) : (
                        apiKeys.slice(0, 5).map(apiKey => (
                            <div key={apiKey.id} className="p-8 hover:bg-white/[0.02] transition-colors">
                                <div className="flex items-center justify-between">
                                    <div className="flex items-center space-x-5">
                                        <div className="w-12 h-12 flex items-center justify-center bg-white/5 rounded-2xl border border-white/5">
                                            <Key className="w-6 h-6 text-[#1fc7d4]" />
                                        </div>
                                        <div>
                                            <h4 className="font-black text-foreground tracking-tight text-lg">
                                                {apiKey.client_name}
                                            </h4>
                                            <p className="text-sm font-mono text-muted-foreground mt-1">
                                                <span className="text-[#1fc7d4]/70">prefix_</span>{(apiKey as any).key_prefix ?? apiKey.key_preview}
                                            </p>
                                        </div>
                                    </div>
                                    <div className="flex items-center space-x-4">
                                        <span
                                            className={`px-4 py-1.5 rounded-full text-[10px] font-black uppercase tracking-widest ${getStatusColor(apiKey.status)}`}
                                        >
                                            {apiKey.status}
                                        </span>
                                        <div className="text-xs font-black text-muted-foreground uppercase tracking-widest bg-white/5 px-3 py-1.5 rounded-lg border border-white/5">
                                            {apiKey.total_requests.toLocaleString()} REQS
                                        </div>
                                    </div>
                                </div>
                                <div className="mt-4 flex flex-wrap gap-2 ml-[68px]">
                                    {/* Permission Groups */}
                                    {(apiKey as any).permission_groups?.length > 0 && (
                                        <>
                                            {(apiKey as any).permission_groups.map((group: { id: string; name: string }) => (
                                                <span
                                                    key={group.id}
                                                    className="inline-flex items-center px-3 py-1.5 rounded-xl text-[10px] font-black uppercase tracking-widest bg-purple-500/10 text-purple-400 border border-purple-500/10"
                                                >
                                                    {group.name}
                                                </span>
                                            ))}
                                        </>
                                    )}
                                    {/* Legacy modules */}
                                    {apiKey.allowed_modules.map(module => (
                                        <span
                                            key={module.module_id}
                                            className="inline-flex items-center px-3 py-1.5 rounded-xl text-[10px] font-black uppercase tracking-widest bg-cyan-500/10 text-cyan-400 border border-cyan-500/10"
                                        >
                                            {module.module_name}
                                        </span>
                                    ))}
                                </div>
                            </div>
                        ))
                    )}
                </div>
            </div>

            {/* Available Modules */}
            <div className="relative overflow-hidden rounded-[32px] bg-slate-900/40 backdrop-blur-2xl border border-white/5 shadow-xl">
                <div className="p-8 border-b border-white/5">
                    <h3 className="text-xl font-black text-foreground uppercase tracking-tight mb-1">
                        Available Modules
                    </h3>
                    <p className="text-sm font-bold text-muted-foreground">Choose from these modules when creating API keys</p>
                </div>
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 p-8">
                    {modules.length === 0 ? (
                        <div className="col-span-full text-center text-muted-foreground font-bold py-12">
                            No modules available.
                        </div>
                    ) : (
                        modules.map(module => (
                            <div
                                key={module.id}
                                className="relative group p-6 rounded-[24px] bg-white/5 border border-white/5 hover:bg-white/[0.08] transition-all duration-300"
                            >
                                <div className="flex items-start justify-between mb-4">
                                    <div>
                                        <h4 className="font-black text-foreground tracking-tight text-lg mb-1">
                                            {module.display_name}
                                        </h4>
                                        <p className="text-[10px] font-black text-[#1fc7d4] uppercase tracking-widest bg-[#1fc7d4]/10 w-fit px-2 py-0.5 rounded-md">
                                            {module.name}
                                        </p>
                                    </div>
                                    <span className="px-3 py-1 bg-green-500/10 text-green-400 border border-green-500/10 rounded-full text-[10px] font-black uppercase tracking-widest">
                                        {module.status}
                                    </span>
                                </div>
                                <p className="text-sm font-bold text-muted-foreground mb-4 line-clamp-2">
                                    {module.description ?? 'No description available'}
                                </p>
                                <div className="flex items-center text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em] bg-white/5 px-3 py-2 rounded-xl border border-white/5 w-fit">
                                    <span className="opacity-50 mr-2">Category:</span> {module.category}
                                </div>
                            </div>
                        ))
                    )}
                </div>
            </div>
        </div>
    );
};
