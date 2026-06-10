import { AlertTriangle, Code, Globe, Shield } from 'lucide-react';
import React from 'react';

import { type Plan } from '@/shared/api/plans';

export interface Module {
    id: string;
    name: string;
    display_name: string;
}

interface DocumentationViewerProps {
    modules: Module[];
    availablePlans?: Plan[];
}

const AuthSection: React.FC = () => (
    <div>
        <h3 className="text-md font-semibold text-foreground mb-3 flex items-center">
            <Shield className="w-5 h-5 mr-2 text-blue-600 dark:text-blue-400" />
            Authentication
        </h3>
        <div className="bg-gray-50 dark:bg-muted rounded-lg p-4">
            <p className="text-sm text-gray-700 dark:text-muted-foreground mb-3">
                Include your API key in the Authorization header:
            </p>
            <code className="block bg-gray-900 text-green-400 p-3 rounded text-sm font-mono">
                curl -H &quot;Authorization: Bearer YOUR_API_KEY&quot; \
                <br />{' '}
                https://api.epsx.com/modules/stock-ranking/rankings
            </code>
        </div>
    </div>
);

const BaseUrlSection: React.FC = () => (
    <div>
        <h3 className="text-md font-semibold text-foreground mb-3 flex items-center">
            <Globe className="w-5 h-5 mr-2 text-blue-600" />
            Base URL
        </h3>
        <div className="bg-gray-50 dark:bg-muted rounded-lg p-4">
            <code className="text-sm font-mono text-foreground">
                https://api.epsx.com
            </code>
        </div>
    </div>
);

const EndpointSection: React.FC<{ modules: Module[] }> = ({ modules }) => (
    <div>
        <h3 className="text-md font-semibold text-foreground mb-3 flex items-center">
            <Code className="w-5 h-5 mr-2 text-blue-600" />
            Available Endpoints
        </h3>
        <div className="space-y-4">
            {modules.map(module => (
                <div key={module.id} className="border border-border/40 rounded-lg p-4">
                    <h4 className="font-medium text-foreground mb-2">
                        {module.display_name}
                    </h4>
                    <div className="space-y-2 text-sm">
                        <div className="flex items-center space-x-3">
                            <span className="px-2 py-1 bg-green-100 text-green-800 rounded text-xs font-mono">
                                GET
                            </span>
                            <code className="text-gray-700 dark:text-muted-foreground">
                                /modules/{module.name}/status
                            </code>
                            <span className="text-muted-foreground">
                                - Get module status
                            </span>
                        </div>
                        <div className="flex items-center space-x-3">
                            <span className="px-2 py-1 bg-blue-100 text-blue-800 rounded text-xs font-mono">
                                GET
                            </span>
                            <code className="text-gray-700 dark:text-muted-foreground">
                                /modules/{module.name}/data
                            </code>
                            <span className="text-muted-foreground">
                                - Get module data
                            </span>
                        </div>
                        <div className="flex items-center space-x-3">
                            <span className="px-2 py-1 bg-purple-100 text-purple-800 rounded text-xs font-mono">
                                POST
                            </span>
                            <code className="text-gray-700 dark:text-muted-foreground">
                                /modules/{module.name}/analyze
                            </code>
                            <span className="text-muted-foreground">
                                - Perform analysis
                            </span>
                        </div>
                    </div>
                </div>
            ))}
        </div>
    </div>
);

const RateLimitSection: React.FC<{ availablePlans: Plan[] }> = ({ availablePlans }) => (
    <div>
        <h3 className="text-md font-semibold text-foreground mb-3 flex items-center">
            <AlertTriangle className="w-5 h-5 mr-2 text-blue-600" />
            Access Tiers & Rate Limits
        </h3>
        <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
            <div className="space-y-2 text-sm text-yellow-800 dark:text-yellow-200">
                {availablePlans.length > 0 ? (
                    availablePlans.map(plan => (
                        <div key={plan.id}>
                            <strong>{plan.name}:</strong> {plan.description}
                        </div>
                    ))
                ) : (
                    <p>No access tiers configuration found.</p>
                )}
            </div>
        </div>
    </div>
);

const ErrorCodeSection: React.FC = () => (
    <div>
        <h3 className="text-md font-semibold text-foreground mb-3">
            Common Error Codes
        </h3>
        <div className="space-y-2 text-sm">
            {[
                { code: '401', color: 'red', text: 'Unauthorized - Invalid API key' },
                { code: '403', color: 'red', text: 'Forbidden - Insufficient permissions' },
                { code: '429', color: 'yellow', text: 'Too Many Requests - Rate limit exceeded' },
                { code: '500', color: 'gray', text: 'Internal Server Error - Contact support' }
            ].map(err => (
                <div key={err.code} className="flex items-center space-x-3">
                    <code className={`px-2 py-1 bg-${err.color}-100 text-${err.color}-800 rounded`}>
                        {err.code}
                    </code>
                    <span className="text-gray-700 dark:text-muted-foreground">
                        {err.text}
                    </span>
                </div>
            ))}
        </div>
    </div>
);

export const DocumentationViewer: React.FC<DocumentationViewerProps> = ({ modules, availablePlans = [] }) => {
    return (
        <div className="space-y-6">
            <div className="bg-card rounded-lg shadow">
                <div className="p-6 border-b border-border/40">
                    <h2 className="text-lg font-semibold text-foreground">
                        API Documentation
                    </h2>
                    <p className="text-sm text-muted-foreground">
                        Complete guide to using our module-based API
                    </p>
                </div>
                <div className="p-6 space-y-6">
                    <AuthSection />
                    <BaseUrlSection />
                    <EndpointSection modules={modules} />
                    <RateLimitSection availablePlans={availablePlans} />
                    <ErrorCodeSection />
                </div>
            </div>
        </div>
    );
};
