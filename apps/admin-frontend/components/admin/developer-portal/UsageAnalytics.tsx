
import { Activity, BarChart3, Download } from 'lucide-react';
import React from 'react';

import { Button } from '@/components/ui/button';
import { type ApiKeyResponse as ApiKey } from '@/shared/api/plans';

interface UsageAnalyticsProps {
    apiKeys: ApiKey[];
}

export const UsageAnalytics: React.FC<UsageAnalyticsProps> = ({ apiKeys }) => {
    return (
        <div className="space-y-6">
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
                <div className="flex items-center justify-between mb-6">
                    <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                        Usage Analytics
                    </h2>
                    <Button variant="outline" size="sm">
                        <Download className="w-4 h-4 mr-2" />
                        Export Data
                    </Button>
                </div>

                {/* Usage Charts Placeholder */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div className="p-6 border border-gray-200 dark:border-gray-600 rounded-lg">
                        <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-4">
                            Requests Over Time
                        </h3>
                        <div className="h-48 bg-gray-50 dark:bg-gray-700 rounded flex items-center justify-center">
                            <div className="text-center text-gray-500 dark:text-gray-400">
                                <BarChart3 className="w-8 h-8 mx-auto mb-2" />
                                <p>Chart placeholder</p>
                                <p className="text-xs">
                                    Requests timeline would be displayed here
                                </p>
                            </div>
                        </div>
                    </div>

                    <div className="p-6 border border-gray-200 dark:border-gray-600 rounded-lg">
                        <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-4">
                            Module Usage Distribution
                        </h3>
                        <div className="h-48 bg-gray-50 dark:bg-gray-700 rounded flex items-center justify-center">
                            <div className="text-center text-gray-500 dark:text-gray-400">
                                <Activity className="w-8 h-8 mx-auto mb-2" />
                                <p>Chart placeholder</p>
                                <p className="text-xs">
                                    Module usage breakdown would be displayed here
                                </p>
                            </div>
                        </div>
                    </div>
                </div>

                {/* Usage Table */}
                <div className="mt-6">
                    <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-4">
                        API Key Usage Details
                    </h3>
                    <div className="overflow-x-auto">
                        <table className="min-w-full border-collapse border border-gray-200 dark:border-gray-600">
                            <thead className="bg-gray-50 dark:bg-gray-700">
                                <tr>
                                    <th className="border border-gray-200 dark:border-gray-600 px-4 py-3 text-left text-sm font-medium text-gray-900 dark:text-gray-100">
                                        API Key
                                    </th>
                                    <th className="border border-gray-200 dark:border-gray-600 px-4 py-3 text-left text-sm font-medium text-gray-900 dark:text-gray-100">
                                        Client
                                    </th>
                                    <th className="border border-gray-200 dark:border-gray-600 px-4 py-3 text-left text-sm font-medium text-gray-900 dark:text-gray-100">
                                        Requests (24h)
                                    </th>
                                    <th className="border border-gray-200 dark:border-gray-600 px-4 py-3 text-left text-sm font-medium text-gray-900 dark:text-gray-100">
                                        Most Used Module
                                    </th>
                                    <th className="border border-gray-200 dark:border-gray-600 px-4 py-3 text-left text-sm font-medium text-gray-900 dark:text-gray-100">
                                        Last Activity
                                    </th>
                                </tr>
                            </thead>
                            <tbody>
                                {apiKeys.map(apiKey => (
                                    <tr key={apiKey.id} className="hover:bg-gray-50 dark:hover:bg-gray-700">
                                        <td className="border border-gray-200 dark:border-gray-600 px-4 py-3 text-sm font-mono text-gray-900 dark:text-gray-100">
                                            {apiKey.key_preview || (apiKey as any).key_prefix}...
                                        </td>
                                        <td className="border border-gray-200 dark:border-gray-600 px-4 py-3 text-sm text-gray-900 dark:text-gray-100">
                                            {apiKey.client_name}
                                        </td>
                                        <td className="border border-gray-200 dark:border-gray-600 px-4 py-3 text-sm text-gray-900 dark:text-gray-100">
                                            {Math.floor(
                                                apiKey.total_requests * 0.1
                                            ).toLocaleString()}
                                        </td>
                                        <td className="border border-gray-200 dark:border-gray-600 px-4 py-3 text-sm text-gray-900 dark:text-gray-100">
                                            {apiKey.allowed_modules[0]?.module_name || 'N/A'}
                                        </td>
                                        <td className="border border-gray-200 dark:border-gray-600 px-4 py-3 text-sm text-gray-900 dark:text-gray-100">
                                            {apiKey.last_used_at
                                                ? new Date(apiKey.last_used_at).toLocaleDateString()
                                                : 'Never'}
                                        </td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    );
};
