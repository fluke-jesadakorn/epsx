'use client';

import { Activity, BarChart3, Download } from 'lucide-react';
import React from 'react';

import { Button } from '@/components/ui/button';
import type { ApiKeyResponse } from '@/shared/api/plans';

interface ExtendedApiKey extends ApiKeyResponse {
    key_prefix?: string;
    last_used_at?: string;
}

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
            <div className="bg-card rounded-lg shadow p-6">
                <div className="flex items-center justify-between mb-6">
                    <h2 className="text-lg font-semibold text-foreground">
                        Usage Analytics
                    </h2>
                    <Button variant="outline" size="sm">
                        <Download className="w-4 h-4 mr-2" />
                        Export Data
                    </Button>
                </div>

                {/* Usage Charts */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div className="p-6 border border-border/40 rounded-lg">
                        <h3 className="font-medium text-foreground mb-4">
                            Requests Over Time
                        </h3>
                        <div className="h-48 bg-gray-50 dark:bg-muted rounded flex items-center justify-center">
                            <div className="text-center text-muted-foreground">
                                <BarChart3 className="w-8 h-8 mx-auto mb-2" />
                                <p>Chart placeholder</p>
                                <p className="text-xs">Requests timeline would be displayed here</p>
                            </div>
                        </div>
                    </div>

                    <div className="p-6 border border-border/40 rounded-lg">
                        <h3 className="font-medium text-foreground mb-4">
                            Module Usage Distribution
                        </h3>
                        <div className="h-48 bg-gray-50 dark:bg-muted rounded flex items-center justify-center">
                            <div className="text-center text-muted-foreground">
                                <Activity className="w-8 h-8 mx-auto mb-2" />
                                <p>Chart placeholder</p>
                                <p className="text-xs">Module usage breakdown would be displayed here</p>
                            </div>
                        </div>
                    </div>
                </div>

                {/* Usage Table */}
                <div className="mt-6">
                    <h3 className="font-medium text-foreground mb-4">
                        API Key Usage Details
                    </h3>
                    <div className="overflow-x-auto">
                        <table className="min-w-full border-collapse border border-border/40">
                            <thead className="bg-gray-50 dark:bg-muted">
                                <tr>
                                    <th className="border border-border/40 px-4 py-3 text-left text-sm font-medium text-foreground">
                                        API Key
                                    </th>
                                    <th className="border border-border/40 px-4 py-3 text-left text-sm font-medium text-foreground">
                                        Client
                                    </th>
                                    <th className="border border-border/40 px-4 py-3 text-left text-sm font-medium text-foreground">
                                        Total Requests
                                    </th>
                                    <th className="border border-border/40 px-4 py-3 text-left text-sm font-medium text-foreground">
                                        Status
                                    </th>
                                    <th className="border border-border/40 px-4 py-3 text-left text-sm font-medium text-foreground">
                                        Last Activity
                                    </th>
                                </tr>
                            </thead>
                            <tbody>
                                {apiKeys.length === 0 ? (
                                    <tr>
                                        <td
                                            colSpan={5}
                                            className="border border-border/40 px-4 py-8 text-center text-muted-foreground"
                                        >
                                            No API keys found
                                        </td>
                                    </tr>
                                ) : (
                                    apiKeys.map(key => {
                                        const apiKey = key as ExtendedApiKey;
                                        return (
                                            <tr key={apiKey.id} className="hover:bg-muted/30">
                                                <td className="border border-border/40 px-4 py-3 text-sm font-mono text-foreground">
                                                    {apiKey.key_prefix ?? apiKey.key_preview}...
                                                </td>
                                                <td className="border border-border/40 px-4 py-3 text-sm text-foreground">
                                                    {apiKey.client_name}
                                                </td>
                                                <td className="border border-border/40 px-4 py-3 text-sm text-foreground">
                                                    {apiKey.total_requests.toLocaleString()}
                                                </td>
                                                <td className="border border-border/40 px-4 py-3 text-sm">
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
                                                <td className="border border-border/40 px-4 py-3 text-sm text-foreground">
                                                    {apiKey.last_used_at !== undefined
                                                        ? new Date(apiKey.last_used_at).toLocaleDateString()
                                                        : 'Never'}
                                                </td>
                                            </tr>
                                        );
                                    })
                                )}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    );
};
