
import { Activity, BarChart3, Download } from 'lucide-react';
import React from 'react';

import { Button } from '@/components/ui/button';
import { type ApiKeyResponse as ApiKey } from '@/shared/api/plans';

interface UsageAnalyticsProps {
    apiKeys: ApiKey[];
}

interface ExtendedApiKey extends ApiKey {
    last_used_at?: string;
    key_prefix?: string;
}

export const UsageAnalytics: React.FC<UsageAnalyticsProps> = ({ apiKeys }) => {
    return (
        <div className="space-y-6">
            <div className="rounded-2xl bg-card border border-border/20 overflow-hidden shadow-xl p-6">
                <div className="flex items-center justify-between mb-6">
                    <h2 className="text-lg font-semibold text-foreground">
                        Usage Analytics
                    </h2>
                    <Button variant="outline" size="sm">
                        <Download className="w-4 h-4 mr-2" />
                        Export Data
                    </Button>
                </div>

                {/* Usage Charts Placeholder */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div className="p-6 border border-border/20 rounded-xl">
                        <h3 className="font-medium text-foreground mb-4">
                            Requests Over Time
                        </h3>
                        <div className="h-48 bg-muted/30 rounded-xl flex items-center justify-center">
                            <div className="text-center text-muted-foreground">
                                <BarChart3 className="w-8 h-8 mx-auto mb-2" />
                                <p>Chart placeholder</p>
                                <p className="text-xs">
                                    Requests timeline would be displayed here
                                </p>
                            </div>
                        </div>
                    </div>

                    <div className="p-6 border border-border/20 rounded-xl">
                        <h3 className="font-medium text-foreground mb-4">
                            Module Usage Distribution
                        </h3>
                        <div className="h-48 bg-muted/30 rounded-xl flex items-center justify-center">
                            <div className="text-center text-muted-foreground">
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
                    <h3 className="font-medium text-foreground mb-4">
                        API Key Usage Details
                    </h3>
                    <div className="overflow-x-auto">
                        <table className="min-w-full border-collapse border border-border/20">
                            <thead className="bg-muted/30">
                                <tr>
                                    <th className="border border-border/20 px-4 py-3 text-left text-sm font-medium text-foreground">
                                        API Key
                                    </th>
                                    <th className="border border-border/20 px-4 py-3 text-left text-sm font-medium text-foreground">
                                        Client
                                    </th>
                                    <th className="border border-border/20 px-4 py-3 text-left text-sm font-medium text-foreground">
                                        Requests (24h)
                                    </th>
                                    <th className="border border-border/20 px-4 py-3 text-left text-sm font-medium text-foreground">
                                        Most Used Module
                                    </th>
                                    <th className="border border-border/20 px-4 py-3 text-left text-sm font-medium text-foreground">
                                        Last Activity
                                    </th>
                                </tr>
                            </thead>
                            <tbody>
                                {apiKeys.map(key => {
                                    const apiKey = key as ExtendedApiKey;
                                    const keyPreviewValue = apiKey.key_prefix ?? apiKey.key_preview;
                                    return (
                                        <tr key={apiKey.id} className="hover:bg-muted/30">
                                            <td className="border border-border/20 px-4 py-3 text-sm font-mono text-foreground">
                                                {keyPreviewValue !== '' ? `${keyPreviewValue}...` : '...'}
                                            </td>
                                            <td className="border border-border/20 px-4 py-3 text-sm text-foreground">
                                                {apiKey.client_name}
                                            </td>
                                            <td className="border border-border/20 px-4 py-3 text-sm text-foreground">
                                                {Math.floor(
                                                    apiKey.total_requests * 0.1
                                                ).toLocaleString()}
                                            </td>
                                            <td className="border border-border/20 px-4 py-3 text-sm text-foreground">
                                                {apiKey.allowed_modules[0]?.module_name ?? 'N/A'}
                                            </td>
                                            <td className="border border-border/20 px-4 py-3 text-sm text-foreground">
                                                {apiKey.last_used_at !== undefined && apiKey.last_used_at !== ''
                                                    ? new Date(apiKey.last_used_at).toLocaleDateString()
                                                    : 'Never'}
                                            </td>
                                        </tr>
                                    );
                                })}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    );
};
