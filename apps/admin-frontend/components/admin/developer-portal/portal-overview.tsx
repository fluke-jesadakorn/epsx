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
}> = ({ icon, title, value, iconBgClass = 'bg-blue-100', iconColorClass = 'text-blue-600' }) => (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <div className="flex items-center">
            <div className={`p-2 rounded-lg ${iconBgClass}`}>
                {React.cloneElement(icon as React.ReactElement<any>, { className: `w-6 h-6 ${iconColorClass}` })}
            </div>
            <div className="ml-4">
                <p className="text-sm font-medium text-gray-600 dark:text-gray-300">
                    {title}
                </p>
                <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                    {value}
                </p>
            </div>
        </div>
    </div>
);

const RecentApiKeyItem: React.FC<{ apiKey: ApiKey }> = ({ apiKey }) => {
    const keyPreviewString = apiKey.key_preview ?? apiKey.key_prefix ?? '';
    return (
        <div className="p-6">
            <div className="flex items-center justify-between">
                <div className="flex items-center space-x-4">
                    <div className="p-2 bg-gray-100 dark:bg-gray-700 rounded-lg">
                        <Key className="w-5 h-5 text-gray-600 dark:text-gray-300" />
                    </div>
                    <div>
                        <h4 className="font-medium text-gray-900 dark:text-gray-100">
                            {apiKey.client_name}
                        </h4>
                        <p className="text-sm text-gray-500 dark:text-gray-400">
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
                    <div className="text-sm text-gray-500 dark:text-gray-400">
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
                            className="inline-flex items-center px-2 py-1 rounded-full text-xs bg-purple-100 dark:bg-purple-900/50 text-purple-800 dark:text-purple-200"
                        >
                            <Shield className="w-3 h-3 mr-1" />
                            {group.name}
                        </span>
                    ))}
                {apiKey.allowed_modules.map(module => (
                    <span
                        key={module.module_id}
                        className="inline-flex items-center px-2 py-1 rounded-full text-xs bg-blue-100 dark:bg-blue-900/50 text-blue-800 dark:text-blue-200"
                    >
                        {module.module_name}
                    </span>
                ))}
            </div>
        </div>
    );
};

const ModuleCard: React.FC<{ module: Module }> = ({ module }) => (
    <div className="border border-gray-200 dark:border-gray-600 rounded-lg p-4">
        <div className="flex items-start justify-between mb-3">
            <div>
                <h4 className="font-medium text-gray-900 dark:text-gray-100">
                    {module.display_name}
                </h4>
                <p className="text-sm text-gray-500 dark:text-gray-400">{module.name}</p>
            </div>
            <span className="px-2 py-1 bg-green-100 dark:bg-green-900/50 text-green-800 dark:text-green-200 rounded-full text-xs font-medium">
                {module.status}
            </span>
        </div>
        <p className="text-sm text-gray-600 dark:text-gray-300 mb-3">
            {module.description ?? 'No description available'}
        </p>
        <div className="text-xs text-gray-500 dark:text-gray-400">
            <span className="font-medium">Category:</span>{' '}
            {module.category}
        </div>
    </div>
);

export const PortalOverview: React.FC<PortalOverviewProps> = ({ apiKeys, modules }) => {
    const router = useRouter();

    const handleCreateKey = () => {
        void router.push('/developer-portal/api-keys/create');
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
                    iconBgClass="bg-green-100 dark:bg-green-900/50"
                    iconColorClass="text-green-600"
                />
                <StatCard
                    icon={<BarChart3 />}
                    title="Total Requests"
                    value={totalRequests}
                    iconBgClass="bg-purple-100 dark:bg-purple-900/50"
                    iconColorClass="text-purple-600"
                />
                <StatCard
                    icon={<Settings />}
                    title="Available Modules"
                    value={modules.length}
                    iconBgClass="bg-orange-100 dark:bg-orange-900/50"
                    iconColorClass="text-orange-600"
                />
            </div>

            <div className="bg-white dark:bg-gray-800 rounded-lg shadow">
                <div className="p-6 border-b">
                    <div className="flex items-center justify-between">
                        <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100">
                            Recent API Keys
                        </h3>
                        <Button onClick={handleCreateKey} size="sm">
                            <Plus className="w-4 h-4 mr-2" />
                            Create New Key
                        </Button>
                    </div>
                </div>
                <div className="divide-y text-sm">
                    {apiKeys.length === 0 ? (
                        <div className="p-6 text-center text-gray-500">No API keys found.</div>
                    ) : (
                        apiKeys.slice(0, 5).map(apiKey => (
                            <RecentApiKeyItem key={apiKey.id} apiKey={apiKey} />
                        ))
                    )}
                </div>
            </div>

            <div className="bg-white dark:bg-gray-800 rounded-lg shadow">
                <div className="p-6 border-b">
                    <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100">
                        Available Modules
                    </h3>
                    <p className="text-sm text-gray-600 dark:text-gray-300">
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
