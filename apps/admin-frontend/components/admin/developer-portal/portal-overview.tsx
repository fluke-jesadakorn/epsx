
import { Activity, BarChart3, Key, Plus, Settings, Shield } from 'lucide-react';
import { useRouter } from 'next/navigation';
import React from 'react';

import { Button } from '@/components/ui/button';
import { type ApiKeyResponse as ApiKey, type Module } from '@/shared/api/plans';

import { getAccessLevelColor, getStatusColor } from './utils';

interface PortalOverviewProps {
    apiKeys: ApiKey[];
    modules: Module[];
}

export const PortalOverview: React.FC<PortalOverviewProps> = ({ apiKeys, modules }) => {
    const router = useRouter();

    return (
        <div className="space-y-6">
            {/* Stats Overview */}
            <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
                <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
                    <div className="flex items-center">
                        <div className="p-2 bg-blue-100 rounded-lg">
                            <Key className="w-6 h-6 text-blue-600" />
                        </div>
                        <div className="ml-4">
                            <p className="text-sm font-medium text-gray-600 dark:text-gray-300">
                                Total API Keys
                            </p>
                            <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                                {apiKeys.length}
                            </p>
                        </div>
                    </div>
                </div>

                <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
                    <div className="flex items-center">
                        <div className="p-2 bg-green-100 dark:bg-green-900/50 rounded-lg">
                            <Activity className="w-6 h-6 text-green-600" />
                        </div>
                        <div className="ml-4">
                            <p className="text-sm font-medium text-gray-600 dark:text-gray-300">
                                Active Keys
                            </p>
                            <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                                {apiKeys.filter(key => key.status === 'active').length}
                            </p>
                        </div>
                    </div>
                </div>

                <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
                    <div className="flex items-center">
                        <div className="p-2 bg-purple-100 dark:bg-purple-900/50 rounded-lg">
                            <BarChart3 className="w-6 h-6 text-purple-600" />
                        </div>
                        <div className="ml-4">
                            <p className="text-sm font-medium text-gray-600 dark:text-gray-300">
                                Total Requests
                            </p>
                            <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                                {apiKeys
                                    .reduce((sum, key) => sum + key.total_requests, 0)
                                    .toLocaleString()}
                            </p>
                        </div>
                    </div>
                </div>

                <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
                    <div className="flex items-center">
                        <div className="p-2 bg-orange-100 dark:bg-orange-900/50 rounded-lg">
                            <Settings className="w-6 h-6 text-orange-600" />
                        </div>
                        <div className="ml-4">
                            <p className="text-sm font-medium text-gray-600 dark:text-gray-300">
                                Available Modules
                            </p>
                            <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                                {modules.length}
                            </p>
                        </div>
                    </div>
                </div>
            </div>

            {/* Recent API Keys */}
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow">
                <div className="p-6 border-b">
                    <div className="flex items-center justify-between">
                        <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100">
                            Recent API Keys
                        </h3>
                        <Button onClick={() => router.push('/developer-portal/api-keys/create')} size="sm">
                            <Plus className="w-4 h-4 mr-2" />
                            Create New Key
                        </Button>
                    </div>
                </div>
                <div className="divide-y">
                    {apiKeys.slice(0, 5).map(apiKey => (
                        <div key={apiKey.id} className="p-6">
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
                                            Key: {apiKey.key_preview ?? (apiKey as any).key_prefix}...
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
                                {/* Permission Groups (new system) */}
                                {(apiKey as any).permission_groups?.length > 0 && (
                                    <>
                                        {(apiKey as any).permission_groups.map((group: { id: string; name: string; slug: string }) => (
                                            <span
                                                key={group.id}
                                                className="inline-flex items-center px-2 py-1 rounded-full text-xs bg-purple-100 dark:bg-purple-900/50 text-purple-800 dark:text-purple-200"
                                            >
                                                <Shield className="w-3 h-3 mr-1" />
                                                {group.name}
                                            </span>
                                        ))}
                                    </>
                                )}
                                {/* Legacy modules (if any) */}
                                {apiKey.allowed_modules.map(module => (
                                    <span
                                        key={module.module_id}
                                        className="inline-flex items-center px-2 py-1 rounded-full text-xs bg-blue-100 dark:bg-blue-900/50 text-blue-800 dark:text-blue-200"
                                    >
                                        {module.module_name}
                                        <span
                                            className={`ml-1 ${getAccessLevelColor(module.access_level)}`}
                                        >
                                            ({module.access_level})
                                        </span>
                                    </span>
                                ))}
                            </div>
                        </div>
                    ))}
                </div>
            </div>

            {/* Available Modules */}
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
                        <div key={module.id} className="border border-gray-200 dark:border-gray-600 rounded-lg p-4">
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
                    ))}
                </div>
            </div>
        </div>
    );
};
