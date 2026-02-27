import { Activity, BarChart3, Key, Plus, Settings, Shield } from 'lucide-react';
import { useRouter } from 'next/navigation';
import React from 'react';

import { Button } from '@/components/ui/button';
import { type ApiKeyResponse as ApiKey, type Module } from '@/shared/api/plans';

import { getStatusColor } from './utils';

interface PortalOverviewProps {
    apiKeys: ApiKey[];
    modules: Module[];
}

const StatCard: React.FC<{
    icon: React.ReactNode;
    title: string;
    value: string | number;
    iconBgClass?: string;
    iconColorClass?: string;
}> = ({ icon, title, value, iconBgClass = 'bg-[#1fc7d4]/10', iconColorClass = 'text-[#1fc7d4]' }) => (
    <div className="rounded-2xl bg-card border border-border/20 shadow-xl p-6">
        <div className="flex items-center">
            <div className={`p-2 rounded-xl ${iconBgClass}`}>
                {React.cloneElement(icon as React.ReactElement<{ className?: string }>, { className: `w-6 h-6 ${iconColorClass}` })}
            </div>
            <div className="ml-4">
                <p className="text-sm font-medium text-muted-foreground">
                    {title}
                </p>
                <p className="text-2xl font-bold text-foreground">
                    {value}
                </p>
            </div>
        </div>
    </div>
);

const RecentApiKeyItem: React.FC<{ apiKey: ApiKey }> = ({ apiKey }) => {
    const keyPreviewString = apiKey.key_prefix ?? apiKey.key_preview;
    return (
        <div className="p-6">
            <div className="flex items-center justify-between">
                <div className="flex items-center space-x-4">
                    <div className="p-2 bg-muted/30 rounded-xl">
                        <Key className="w-5 h-5 text-muted-foreground" />
                    </div>
                    <div>
                        <h4 className="font-medium text-foreground">
                            {apiKey.client_name}
                        </h4>
                        <p className="text-sm text-muted-foreground">
                            Key: {keyPreviewString !== '' ? `${keyPreviewString}...` : ''}
                        </p>
                    </div>
                </div>
                <div className="flex items-center space-x-3">
                    <span
                        className={`px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(apiKey.status)}`}
                    >
                        {apiKey.status}
                    </span>
                    <div className="text-sm text-muted-foreground">
                        {apiKey.total_requests.toLocaleString()} requests
                    </div>
                </div>
            </div>
            <div className="mt-3 flex flex-wrap gap-2">
                {apiKey.permission_groups !== undefined &&
                    apiKey.permission_groups.length > 0 &&
                    apiKey.permission_groups.map((group) => (
                        <span
                            key={group.id}
                            className="inline-flex items-center px-2 py-1 rounded-full text-xs bg-[#7645d9]/10 text-[#7645d9] border border-[#7645d9]/20"
                        >
                            <Shield className="w-3 h-3 mr-1" />
                            {group.name}
                        </span>
                    ))}
                {apiKey.allowed_modules.map(module => (
                    <span
                        key={module.module_id}
                        className="inline-flex items-center px-2 py-1 rounded-full text-xs bg-[#1fc7d4]/10 text-[#1fc7d4] border border-[#1fc7d4]/20"
                    >
                        {module.module_name}
                    </span>
                ))}
            </div>
        </div>
    );
};

const ModuleCard: React.FC<{ module: Module }> = ({ module }) => (
    <div className="border border-border/20 rounded-xl p-4">
        <div className="flex items-start justify-between mb-3">
            <div>
                <h4 className="font-medium text-foreground">
                    {module.display_name}
                </h4>
                <p className="text-sm text-muted-foreground">{module.name}</p>
            </div>
            <span className="px-2 py-1 bg-[#31d0aa]/10 text-[#31d0aa] border border-[#31d0aa]/20 rounded-full text-xs font-medium">
                {module.status}
            </span>
        </div>
        <p className="text-sm text-muted-foreground mb-3">
            {module.description ?? 'No description available'}
        </p>
        <div className="text-xs text-muted-foreground">
            <span className="font-medium">Category:</span>{' '}
            {module.category}
        </div>
    </div>
);

export const PortalOverview: React.FC<PortalOverviewProps> = ({ apiKeys, modules }) => {
    const router = useRouter();

    const handleCreateKey = () => {
        router.push('/developer-portal/api-keys/create');
    };

    const totalRequests = apiKeys
        .reduce((sum, key) => sum + key.total_requests, 0)
        .toLocaleString();

    return (
        <div className="space-y-6">
            <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
                <StatCard icon={<Key />} title="Total API Keys" value={apiKeys.length} />
                <StatCard
                    icon={<Activity />}
                    title="Active Keys"
                    value={apiKeys.filter(key => key.status === 'active').length}
                    iconBgClass="bg-[#31d0aa]/10"
                    iconColorClass="text-[#31d0aa]"
                />
                <StatCard
                    icon={<BarChart3 />}
                    title="Total Requests"
                    value={totalRequests}
                    iconBgClass="bg-[#7645d9]/10"
                    iconColorClass="text-[#7645d9]"
                />
                <StatCard
                    icon={<Settings />}
                    title="Available Modules"
                    value={modules.length}
                    iconBgClass="bg-[#ffb237]/10"
                    iconColorClass="text-[#ffb237]"
                />
            </div>

            <div className="rounded-2xl bg-card border border-border/20 overflow-hidden shadow-xl">
                <div className="h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" />
                <div className="p-6 border-b border-border/20">
                    <div className="flex items-center justify-between">
                        <h3 className="text-lg font-medium text-foreground">
                            Recent API Keys
                        </h3>
                        <Button onClick={handleCreateKey} size="sm" className="bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white rounded-xl">
                            <Plus className="w-4 h-4 mr-2" />
                            Create New Key
                        </Button>
                    </div>
                </div>
                <div className="divide-y divide-border/20 text-sm">
                    {apiKeys.length === 0 ? (
                        <div className="p-6 text-center text-muted-foreground">No API keys found.</div>
                    ) : (
                        apiKeys.slice(0, 5).map(apiKey => (
                            <RecentApiKeyItem key={apiKey.id} apiKey={apiKey} />
                        ))
                    )}
                </div>
            </div>

            <div className="rounded-2xl bg-card border border-border/20 overflow-hidden shadow-xl">
                <div className="h-[3px] bg-gradient-to-r from-[#7645d9] to-[#ed4b9e]" />
                <div className="p-6 border-b border-border/20">
                    <h3 className="text-lg font-medium text-foreground">
                        Available Modules
                    </h3>
                    <p className="text-sm text-muted-foreground">
                        Choose from these modules when creating API keys
                    </p>
                </div>
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 p-6">
                    {modules.map(module => (
                        <ModuleCard key={module.id} module={module} />
                    ))}
                </div>
            </div>
        </div>
    );
};
