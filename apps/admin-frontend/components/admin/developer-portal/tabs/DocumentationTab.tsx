'use client';

import { AlertTriangle, BookOpen, Code, Globe, Shield } from 'lucide-react';
import React from 'react';

import type { Module } from '@/shared/api/plans';

interface DocumentationTabProps {
    modules: Module[];
}

/**
 * API Documentation tab showing authentication, endpoints, and rate limits
 */
export const DocumentationTab: React.FC<DocumentationTabProps> = ({ modules }) => {
    return (
        <div className="space-y-6">
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow">
                <div className="p-6 border-b border-gray-200 dark:border-gray-700">
                    <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                        API Documentation
                    </h2>
                    <p className="text-sm text-gray-600 dark:text-gray-300">
                        Complete guide to using our module-based API
                    </p>
                </div>
                <div className="p-6 space-y-6">
                    {/* Authentication */}
                    <div>
                        <h3 className="text-md font-semibold text-gray-900 dark:text-gray-100 mb-3 flex items-center">
                            <Shield className="w-5 h-5 mr-2 text-blue-600 dark:text-blue-400" />
                            Authentication
                        </h3>
                        <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                            <p className="text-sm text-gray-700 dark:text-gray-300 mb-3">
                                Include your API key in the Authorization header:
                            </p>
                            <code className="block bg-gray-900 text-green-400 p-3 rounded text-sm font-mono overflow-x-auto">
                                curl -H &quot;Authorization: Bearer YOUR_API_KEY&quot; \<br />
                                {'  '}https://api.epsx.io/v1/modules/stock-ranking/rankings
                            </code>
                        </div>
                    </div>

                    {/* Base URL */}
                    <div>
                        <h3 className="text-md font-semibold text-gray-900 dark:text-gray-100 mb-3 flex items-center">
                            <Globe className="w-5 h-5 mr-2 text-blue-600 dark:text-blue-400" />
                            Base URL
                        </h3>
                        <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                            <code className="text-sm font-mono text-gray-900 dark:text-gray-100">
                                https://api.epsx.io/v1/
                            </code>
                        </div>
                    </div>

                    {/* Available Endpoints */}
                    <div>
                        <h3 className="text-md font-semibold text-gray-900 dark:text-gray-100 mb-3 flex items-center">
                            <Code className="w-5 h-5 mr-2 text-blue-600 dark:text-blue-400" />
                            Available Endpoints
                        </h3>
                        <div className="space-y-4">
                            {modules.length === 0 ? (
                                <p className="text-sm text-gray-500 dark:text-gray-400">
                                    No modules available. Check back later.
                                </p>
                            ) : (
                                modules.map(module => (
                                    <div
                                        key={module.id}
                                        className="border border-gray-200 dark:border-gray-600 rounded-lg p-4"
                                    >
                                        <h4 className="font-medium text-gray-900 dark:text-gray-100 mb-2">
                                            {module.display_name}
                                        </h4>
                                        <div className="space-y-2 text-sm">
                                            <div className="flex items-center space-x-3">
                                                <span className="px-2 py-1 bg-green-100 dark:bg-green-900/50 text-green-800 dark:text-green-200 rounded text-xs font-mono">
                                                    GET
                                                </span>
                                                <code className="text-gray-700 dark:text-gray-300">
                                                    /modules/{module.name}/status
                                                </code>
                                                <span className="text-gray-500 dark:text-gray-400">
                                                    - Get module status
                                                </span>
                                            </div>
                                            <div className="flex items-center space-x-3">
                                                <span className="px-2 py-1 bg-blue-100 dark:bg-blue-900/50 text-blue-800 dark:text-blue-200 rounded text-xs font-mono">
                                                    GET
                                                </span>
                                                <code className="text-gray-700 dark:text-gray-300">
                                                    /modules/{module.name}/data
                                                </code>
                                                <span className="text-gray-500 dark:text-gray-400">
                                                    - Get module data
                                                </span>
                                            </div>
                                            <div className="flex items-center space-x-3">
                                                <span className="px-2 py-1 bg-purple-100 dark:bg-purple-900/50 text-purple-800 dark:text-purple-200 rounded text-xs font-mono">
                                                    POST
                                                </span>
                                                <code className="text-gray-700 dark:text-gray-300">
                                                    /modules/{module.name}/analyze
                                                </code>
                                                <span className="text-gray-500 dark:text-gray-400">
                                                    - Perform analysis
                                                </span>
                                            </div>
                                        </div>
                                    </div>
                                ))
                            )}
                        </div>
                    </div>

                    {/* Rate Limits */}
                    <div>
                        <h3 className="text-md font-semibold text-gray-900 dark:text-gray-100 mb-3 flex items-center">
                            <AlertTriangle className="w-5 h-5 mr-2 text-blue-600 dark:text-blue-400" />
                            Rate Limits
                        </h3>
                        <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
                            <div className="space-y-2 text-sm text-yellow-800 dark:text-yellow-200">
                                <div>
                                    <strong>Bronze:</strong> 100 requests/hour, 1,000 requests/day
                                </div>
                                <div>
                                    <strong>Silver:</strong> 500 requests/hour, 5,000 requests/day
                                </div>
                                <div>
                                    <strong>Gold:</strong> 2,000 requests/hour, 20,000 requests/day
                                </div>
                                <div>
                                    <strong>Platinum:</strong> 10,000 requests/hour, 100,000 requests/day
                                </div>
                                <div>
                                    <strong>Enterprise:</strong> Unlimited (fair usage policy)
                                </div>
                            </div>
                        </div>
                    </div>

                    {/* Error Codes */}
                    <div>
                        <h3 className="text-md font-semibold text-gray-900 dark:text-gray-100 mb-3 flex items-center">
                            <BookOpen className="w-5 h-5 mr-2 text-blue-600 dark:text-blue-400" />
                            Common Error Codes
                        </h3>
                        <div className="space-y-2 text-sm">
                            <div className="flex items-center space-x-3">
                                <code className="px-2 py-1 bg-red-100 dark:bg-red-900/50 text-red-800 dark:text-red-200 rounded">
                                    401
                                </code>
                                <span className="text-gray-700 dark:text-gray-300">
                                    Unauthorized - Invalid API key
                                </span>
                            </div>
                            <div className="flex items-center space-x-3">
                                <code className="px-2 py-1 bg-red-100 dark:bg-red-900/50 text-red-800 dark:text-red-200 rounded">
                                    403
                                </code>
                                <span className="text-gray-700 dark:text-gray-300">
                                    Forbidden - Insufficient permissions
                                </span>
                            </div>
                            <div className="flex items-center space-x-3">
                                <code className="px-2 py-1 bg-yellow-100 dark:bg-yellow-900/50 text-yellow-800 dark:text-yellow-200 rounded">
                                    429
                                </code>
                                <span className="text-gray-700 dark:text-gray-300">
                                    Too Many Requests - Rate limit exceeded
                                </span>
                            </div>
                            <div className="flex items-center space-x-3">
                                <code className="px-2 py-1 bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-200 rounded">
                                    500
                                </code>
                                <span className="text-gray-700 dark:text-gray-300">
                                    Internal Server Error - Contact support
                                </span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
};
