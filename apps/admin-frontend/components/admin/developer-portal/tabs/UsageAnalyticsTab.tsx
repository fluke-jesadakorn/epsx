'use client';

import { Activity, BarChart3, Download } from 'lucide-react';
import React from 'react';

import { Button } from '@/components/ui/button';
import type { ApiKeyResponse } from '@/shared/api/plans';

interface UsageAnalyticsTabProps {
    apiKeys: ApiKeyResponse[];
}

/**
 * Usage Analytics tab showing API usage charts and statistics
 * @param root0
 * @param root0.apiKeys
 */
export const UsageAnalyticsTab: React.FC<UsageAnalyticsTabProps> = ({ apiKeys }) => {
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

                {/* Usage Charts */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div className="p-6 border border-gray-200 dark:border-gray-600 rounded-lg">
                        <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-4">
                            Requests Over Time
                        </h3>
                        <div className="h-48 bg-gray-50 dark:bg-gray-700 rounded flex items-center justify-center">
                            <div className="text-center text-gray-500 dark:text-gray-400">
                                <BarChart3 className="w-8 h-8 mx-auto mb-2" />
                                <p>Chart placeholder</p>
                                <p className="text-xs">Requests timeline would be displayed here</p>
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
                                <p className="text-xs">Module usage breakdown would be displayed here</p>
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
                                        Total Requests
                                    </th>
                                    <th className="border border-gray-200 dark:border-gray-600 px-4 py-3 text-left text-sm font-medium text-gray-900 dark:text-gray-100">
                                        Status
                                    </th>
                                    <th className="border border-gray-200 dark:border-gray-600 px-4 py-3 text-left text-sm font-medium text-gray-900 dark:text-gray-100">
                                        Last Activity
                                    </th>
                                </tr>
                            </thead>
                            <tbody>
                                {apiKeys.length === 0 ? (
                                    <tr>
                                        <td
                                            colSpan={5}
                                            className="border border-gray-200 dark:border-gray-600 px-4 py-8 text-center text-gray-500 dark:text-gray-400"
                                        >
                                            No API keys found
                                        </td>
                                    </tr>
                                ) : (
                                    apiKeys.map(apiKey => (
                                        <tr key={apiKey.id} className="hover:bg-gray-50 dark:hover:bg-gray-700">
                                            <td className="border border-gray-200 dark:border-gray-600 px-4 py-3 text-sm font-mono text-gray-900 dark:text-gray-100">
                                                {(apiKey as any).key_prefix || apiKey.key_preview}...
                                            </td>
                                            <td className="border border-gray-200 dark:border-gray-600 px-4 py-3 text-sm text-gray-900 dark:text-gray-100">
                                                {apiKey.client_name}
                                            </td>
                                            <td className="border border-gray-200 dark:border-gray-600 px-4 py-3 text-sm text-gray-900 dark:text-gray-100">
                                                {apiKey.total_requests.toLocaleString()}
                                            </td>
                                            <td className="border border-gray-200 dark:border-gray-600 px-4 py-3 text-sm">
                                                <span
                                                    className={`px-2 py-1 rounded-full text-xs font-medium ${apiKey.status === 'active'
                                                            ? 'bg-green-100 dark:bg-green-900/50 text-green-800 dark:text-green-200'
                                                            : apiKey.status === 'revoked'
                                                                ? 'bg-red-100 dark:bg-red-900/50 text-red-800 dark:text-red-200'
                                                                : 'bg-yellow-100 dark:bg-yellow-900/50 text-yellow-800 dark:text-yellow-200'
                                                        }`}
                                                >
                                                    {apiKey.status}
                                                </span>
                                            </td>
                                            <td className="border border-gray-200 dark:border-gray-600 px-4 py-3 text-sm text-gray-900 dark:text-gray-100">
                                                {apiKey.last_used_at
                                                    ? new Date(apiKey.last_used_at).toLocaleDateString()
                                                    : 'Never'}
                                            </td>
                                        </tr>
                                    ))
                                )}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    );
};
