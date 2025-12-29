'use client';

import { Activity, Key, Plus, Settings } from 'lucide-react';
import { useRouter } from 'next/navigation';
import React from 'react';

import { Button } from '@/components/ui/button';

import type { ApiKeyResponse, Module } from '@/shared/api/plans';
import { StatsCard } from '../shared/StatsCard';

interface OverviewTabProps {
    apiKeys: ApiKeyResponse[];
    modules: Module[];
    onCreateKey: () => void;
}

/**
 * Get status badge color
 */
const getStatusColor = (status: string): string => {
    switch (status) {
        case 'active':
            return 'bg-green-100 dark:bg-green-900/50 text-green-800 dark:text-green-200';
        case 'revoked':
            return 'bg-red-100 dark:bg-red-900/50 text-red-800 dark:text-red-200';
        case 'expired':
            return 'bg-yellow-100 dark:bg-yellow-900/50 text-yellow-800 dark:text-yellow-200';
        default:
            return 'bg-gray-100 dark:bg-gray-800 text-gray-800 dark:text-gray-300';
    }
};

/**
 * Overview tab component showing stats and recent API keys
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
                    iconBgColor="bg-blue-100 dark:bg-blue-900/50"
                    iconColor="text-blue-600 dark:text-blue-400"
                />
                <StatsCard
                    title="Active Keys"
                    value={activeKeys}
                    icon={Activity}
                    iconBgColor="bg-green-100 dark:bg-green-900/50"
                    iconColor="text-green-600 dark:text-green-400"
                />
                <StatsCard
                    title="Total Requests"
                    value={totalRequests}
                    icon={Activity}
                    iconBgColor="bg-purple-100 dark:bg-purple-900/50"
                    iconColor="text-purple-600 dark:text-purple-400"
                />
                <StatsCard
                    title="Available Modules"
                    value={modules.length}
                    icon={Settings}
                    iconBgColor="bg-orange-100 dark:bg-orange-900/50"
                    iconColor="text-orange-600 dark:text-orange-400"
                />
            </div>

            {/* Recent API Keys */}
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow">
                <div className="p-6 border-b border-gray-200 dark:border-gray-700">
                    <div className="flex items-center justify-between">
                        <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100">
                            Recent API Keys
                        </h3>
                        <Button onClick={onCreateKey} size="sm">
                            <Plus className="w-4 h-4 mr-2" />
                            Create New Key
                        </Button>
                    </div>
                </div>
                <div className="divide-y divide-gray-200 dark:divide-gray-700">
                    {apiKeys.length === 0 ? (
                        <div className="p-6 text-center text-gray-500 dark:text-gray-400">
                            No API keys found. Create one to get started.
                        </div>
                    ) : (
                        apiKeys.slice(0, 5).map(apiKey => (
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
                                                Key: {(apiKey as any).key_prefix || apiKey.key_preview}...
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
                                    {/* Permission Groups */}
                                    {(apiKey as any).permission_groups?.length > 0 && (
                                        <>
                                            {(apiKey as any).permission_groups.map((group: { id: string; name: string }) => (
                                                <span
                                                    key={group.id}
                                                    className="inline-flex items-center px-2 py-1 rounded-full text-xs bg-purple-100 dark:bg-purple-900/50 text-purple-800 dark:text-purple-200"
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
                                            className="inline-flex items-center px-2 py-1 rounded-full text-xs bg-blue-100 dark:bg-blue-900/50 text-blue-800 dark:text-blue-200"
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
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow">
                <div className="p-6 border-b border-gray-200 dark:border-gray-700">
                    <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100">
                        Available Modules
                    </h3>
                    <p className="text-sm text-gray-600 dark:text-gray-300">
                        Choose from these modules when creating API keys
                    </p>
                </div>
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 p-6">
                    {modules.length === 0 ? (
                        <div className="col-span-full text-center text-gray-500 dark:text-gray-400 py-6">
                            No modules available.
                        </div>
                    ) : (
                        modules.map(module => (
                            <div
                                key={module.id}
                                className="border border-gray-200 dark:border-gray-600 rounded-lg p-4"
                            >
                                <div className="flex items-start justify-between mb-3">
                                    <div>
                                        <h4 className="font-medium text-gray-900 dark:text-gray-100">
                                            {module.display_name}
                                        </h4>
                                        <p className="text-sm text-gray-500 dark:text-gray-400">
                                            {module.name}
                                        </p>
                                    </div>
                                    <span className="px-2 py-1 bg-green-100 dark:bg-green-900/50 text-green-800 dark:text-green-200 rounded-full text-xs font-medium">
                                        {module.status}
                                    </span>
                                </div>
                                <p className="text-sm text-gray-600 dark:text-gray-300 mb-3">
                                    {module.description || 'No description available'}
                                </p>
                                <div className="text-xs text-gray-500 dark:text-gray-400">
                                    <span className="font-medium">Category:</span> {module.category}
                                </div>
                            </div>
                        ))
                    )}
                </div>
            </div>
        </div>
    );
};
