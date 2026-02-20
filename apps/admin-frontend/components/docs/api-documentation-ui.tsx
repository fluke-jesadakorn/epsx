'use client';

import {
    AlertTriangle,
    BookOpen,
    ChevronDown,
    ChevronRight,
    Code,
    Copy,
    ExternalLink,
    Globe,
    Key,
    Shield,
} from 'lucide-react';
import React from 'react';
import { Button } from '@/components/ui/button';
import { EndpointExample, ModuleDocumentation } from './api-documentation-types';

interface ApiHeaderProps {
    onRequestAccess: () => void;
}

export function ApiHeader({ onRequestAccess }: ApiHeaderProps) {
    return (
        <div className="mb-8">
            <div className="flex items-center mb-4">
                <BookOpen className="w-8 h-8 text-blue-600 mr-3" />
                <h1 className="text-3xl font-bold text-gray-900">API Documentation</h1>
            </div>
            <p className="text-lg text-gray-600 mb-6">
                Complete guide to integrating with the EPSX module-based API platform. Access financial data, analytics, and market tools
                programmatically.
            </p>

            {/* Quick Start Banner */}
            <div className="bg-gradient-to-r from-blue-50 to-indigo-50 border border-blue-200 rounded-lg p-6">
                <div className="flex items-start">
                    <Key className="w-6 h-6 text-blue-600 mt-1 mr-3" />
                    <div>
                        <h3 className="font-semibold text-blue-900 mb-2">Need an API Key?</h3>
                        <p className="text-blue-800 mb-3">Contact our team to get access to the developer portal and create your API keys.</p>
                        <Button size="sm" className="bg-blue-600 hover:bg-blue-700" onClick={onRequestAccess}>
                            <ExternalLink className="w-4 h-4 mr-2" />
                            Request Access
                        </Button>
                    </div>
                </div>
            </div>
        </div>
    );
}

interface AuthenticationSectionProps {
    onCopy: (text: string, label: string) => void;
}

export function AuthenticationSection({ onCopy }: AuthenticationSectionProps) {
    return (
        <section className="mb-8">
            <h2 className="text-2xl font-bold text-gray-900 mb-4 flex items-center">
                <Shield className="w-6 h-6 text-blue-600 mr-2" />
                Authentication
            </h2>

            <div className="bg-white border rounded-lg p-6">
                <p className="text-gray-600 mb-4">All API requests require authentication using an API key in the Authorization header:</p>

                <div className="bg-gray-900 rounded-lg p-4 mb-4">
                    <div className="flex items-center justify-between mb-2">
                        <span className="text-gray-400 text-sm">cURL Example</span>
                        <button
                            onClick={() =>
                                onCopy(
                                    'curl -H "Authorization: Bearer YOUR_API_KEY" https://api.epsx.com/stock-ranking/rankings',
                                    'cURL example'
                                )
                            }
                            className="text-gray-400 hover:text-white"
                        >
                            <Copy className="w-4 h-4" />
                        </button>
                    </div>
                    <code className="text-green-400 text-sm font-mono">
                        curl -H &quot;Authorization: Bearer YOUR_API_KEY&quot; \
                        <br />     https://api.epsx.com/stock-ranking/rankings
                    </code>
                </div>

                <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
                    <div className="flex items-start">
                        <AlertTriangle className="w-5 h-5 text-yellow-600 mt-0.5 mr-2" />
                        <div>
                            <h4 className="font-medium text-yellow-800">Security Best Practices</h4>
                            <ul className="text-sm text-yellow-700 mt-1 space-y-1">
                                <li>• Never expose your API key in client-side code</li>
                                <li>• Use environment variables to store your API key</li>
                                <li>• Rotate your API keys regularly</li>
                                <li>• Monitor your API usage and set up alerts</li>
                            </ul>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    );
}

interface BaseUrlSectionProps {
    onCopy: (text: string, label: string) => void;
}

export function BaseUrlSection({ onCopy }: BaseUrlSectionProps) {
    return (
        <section className="mb-8">
            <h2 className="text-2xl font-bold text-gray-900 mb-4 flex items-center">
                <Globe className="w-6 h-6 text-blue-600 mr-2" />
                Base URL
            </h2>

            <div className="bg-white border rounded-lg p-6">
                <div className="flex items-center justify-between">
                    <code className="text-lg font-mono text-gray-900 bg-gray-100 px-3 py-2 rounded">https://api.epsx.com</code>
                    <button
                        onClick={() => onCopy('https://api.epsx.com', 'Base URL')}
                        className="text-gray-400 hover:text-gray-600"
                    >
                        <Copy className="w-5 h-5" />
                    </button>
                </div>
                <p className="text-gray-600 mt-3">All API endpoints are relative to this base URL. HTTPS is required for all requests.</p>
            </div>
        </section>
    );
}

export function RateLimitsSection() {
    return (
        <section className="mb-8">
            <h2 className="text-2xl font-bold text-gray-900 mb-4">Rate Limits</h2>

            <div className="bg-white border rounded-lg p-6">
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    {[
                        { level: 'Bronze', requests: '100/hour', daily: '1,000/day', color: 'amber' },
                        { level: 'Silver', requests: '500/hour', daily: '5,000/day', color: 'gray' },
                        { level: 'Gold', requests: '2,000/hour', daily: '20,000/day', color: 'yellow' },
                        { level: 'Platinum', requests: '10,000/hour', daily: '100,000/day', color: 'purple' },
                        { level: 'Enterprise', requests: 'Unlimited', daily: 'Fair usage', color: 'blue' },
                    ].map((tier) => (
                        <div key={tier.level} className="border rounded-lg p-4">
                            <h3 className={`font-semibold mb-2 text-${tier.color}-600`}>{tier.level}</h3>
                            <div className="space-y-1 text-sm text-gray-600">
                                <div>Requests: {tier.requests}</div>
                                <div>Daily limit: {tier.daily}</div>
                            </div>
                        </div>
                    ))}
                </div>

                <div className="mt-6 bg-blue-50 border border-blue-200 rounded-lg p-4">
                    <h4 className="font-medium text-blue-900 mb-2">Rate Limit Headers</h4>
                    <div className="space-y-1 text-sm text-blue-800 font-mono">
                        <div>X-RateLimit-Limit: 500</div>
                        <div>X-RateLimit-Remaining: 487</div>
                        <div>X-RateLimit-Reset: 1642680000</div>
                    </div>
                </div>
            </div>
        </section>
    );
}

interface ParametersTableProps {
    parameters: EndpointExample['parameters'];
}

export function ParametersTable({ parameters }: ParametersTableProps) {
    if (!parameters ?? parameters.length === 0) {
        return null;
    }

    return (
        <div>
            <h5 className="font-semibold text-gray-900 mb-2">Parameters</h5>
            <div className="overflow-x-auto">
                <table className="min-w-full text-sm">
                    <thead className="bg-gray-100">
                        <tr>
                            <th className="text-left p-2 font-medium text-gray-700">Name</th>
                            <th className="text-left p-2 font-medium text-gray-700">Type</th>
                            <th className="text-left p-2 font-medium text-gray-700">Required</th>
                            <th className="text-left p-2 font-medium text-gray-700">Description</th>
                        </tr>
                    </thead>
                    <tbody>
                        {parameters.map((param, paramIndex) => (
                            <tr key={paramIndex} className="border-t">
                                <td className="p-2 font-mono text-blue-600">{param.name}</td>
                                <td className="p-2 text-gray-600">{param.type}</td>
                                <td className="p-2">
                                    <span
                                        className={`px-2 py-1 rounded text-xs ${
                                            param.required ? 'bg-red-100 text-red-800' : 'bg-gray-100 text-gray-600'
                                        }`}
                                    >
                                        {param.required ? 'Required' : 'Optional'}
                                    </span>
                                </td>
                                <td className="p-2 text-gray-600">{param.description}</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        </div>
    );
}

interface EndpointDetailsProps {
    endpoint: EndpointExample;
    onCopy: (text: string, label: string) => void;
}

export function EndpointDetails({ endpoint, onCopy }: EndpointDetailsProps) {
    return (
        <div className="border-t bg-gray-50 p-4 space-y-4">
            <ParametersTable parameters={endpoint.parameters} />

            {/* Example Response */}
            <div>
                <div className="flex items-center justify-between mb-2">
                    <h5 className="font-semibold text-gray-900">Example Response</h5>
                    <button onClick={() => onCopy(endpoint.response, 'Example response')} className="text-gray-400 hover:text-gray-600">
                        <Copy className="w-4 h-4" />
                    </button>
                </div>
                <div className="bg-gray-900 rounded-lg p-4 overflow-x-auto">
                    <pre className="text-green-400 text-sm font-mono whitespace-pre-wrap">{endpoint.response}</pre>
                </div>
            </div>

            {/* cURL Example */}
            <div>
                <div className="flex items-center justify-between mb-2">
                    <h5 className="font-semibold text-gray-900">cURL Example</h5>
                    <button
                        onClick={() => {
                            const curlExample = `curl -X ${endpoint.method} \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  https://api.epsx.com/v1${endpoint.path}`;
                            onCopy(curlExample, 'cURL example');
                        }}
                        className="text-gray-400 hover:text-gray-600"
                    >
                        <Copy className="w-4 h-4" />
                    </button>
                </div>
                <div className="bg-gray-900 rounded-lg p-4 overflow-x-auto">
                    <code className="text-green-400 text-sm font-mono whitespace-pre-wrap">
                        {`curl -X ${endpoint.method} \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  https://api.epsx.com/v1${endpoint.path}`}
                    </code>
                </div>
            </div>
        </div>
    );
}

interface EndpointCardProps {
    endpoint: EndpointExample;
    endpointKey: string;
    isExpanded: boolean;
    onToggle: (key: string) => void;
    onCopy: (text: string, label: string) => void;
    getMethodColor: (method: string) => string;
    getAccessLevelColor: (level: string) => string;
}

export function EndpointCard({
    endpoint,
    endpointKey,
    isExpanded,
    onToggle,
    onCopy,
    getMethodColor,
    getAccessLevelColor,
}: EndpointCardProps) {
    return (
        <div className="bg-white border rounded-lg overflow-hidden">
            <button
                onClick={() => onToggle(endpointKey)}
                className="w-full p-4 text-left hover:bg-gray-50 transition-colors"
            >
                <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-3">
                        <span className={`px-2 py-1 rounded text-xs font-mono font-semibold ${getMethodColor(endpoint.method)}`}>
                            {endpoint.method}
                        </span>
                        <code className="text-sm font-mono text-gray-700">{endpoint.path}</code>
                        <span className={`text-xs font-medium ${getAccessLevelColor(endpoint.accessLevel)}`}>{endpoint.accessLevel}</span>
                    </div>
                    {isExpanded ? (
                        <ChevronDown className="w-4 h-4 text-gray-400" />
                    ) : (
                        <ChevronRight className="w-4 h-4 text-gray-400" />
                    )}
                </div>
                <p className="text-sm text-gray-600 mt-2">{endpoint.description}</p>
            </button>

            {isExpanded && <EndpointDetails endpoint={endpoint} onCopy={onCopy} />}
        </div>
    );
}

interface ModuleCardProps {
    module: ModuleDocumentation;
    isExpanded: boolean;
    onToggle: (moduleName: string) => void;
    expandedEndpoint: string | null;
    onToggleEndpoint: (key: string) => void;
    onCopy: (text: string, label: string) => void;
    getMethodColor: (method: string) => string;
    getAccessLevelColor: (level: string) => string;
}

export function ModuleCard({
    module,
    isExpanded,
    onToggle,
    expandedEndpoint,
    onToggleEndpoint,
    onCopy,
    getMethodColor,
    getAccessLevelColor,
}: ModuleCardProps) {
    return (
        <div className="bg-white border rounded-lg overflow-hidden">
            <button onClick={() => onToggle(module.name)} className="w-full p-6 text-left hover:bg-gray-50 transition-colors">
                <div className="flex items-center justify-between">
                    <div>
                        <h3 className="text-xl font-semibold text-gray-900 mb-1">{module.displayName}</h3>
                        <p className="text-gray-600">{module.description}</p>
                        <div className="mt-2 flex items-center space-x-4">
                            <span className="text-sm text-gray-500">Category: {module.category}</span>
                            <span className="text-sm text-gray-500">{module.endpoints.length} endpoints</span>
                        </div>
                    </div>
                    {isExpanded ? (
                        <ChevronDown className="w-5 h-5 text-gray-400" />
                    ) : (
                        <ChevronRight className="w-5 h-5 text-gray-400" />
                    )}
                </div>
            </button>

            {isExpanded && (
                <div className="border-t bg-gray-50">
                    <div className="p-6 space-y-4">
                        {module.endpoints.map((endpoint, index) => {
                            const endpointKey = `${module.name}-${index}`;
                            const isEndpointExpanded = expandedEndpoint === endpointKey;

                            return (
                                <EndpointCard
                                    key={index}
                                    endpoint={endpoint}
                                    endpointKey={endpointKey}
                                    isExpanded={isEndpointExpanded}
                                    onToggle={onToggleEndpoint}
                                    onCopy={onCopy}
                                    getMethodColor={getMethodColor}
                                    getAccessLevelColor={getAccessLevelColor}
                                />
                            );
                        })}
                    </div>
                </div>
            )}
        </div>
    );
}

interface ModulesListProps {
    modules: ModuleDocumentation[];
    expandedModule: string | null;
    expandedEndpoint: string | null;
    onToggleModule: (moduleName: string) => void;
    onToggleEndpoint: (key: string) => void;
    onCopy: (text: string, label: string) => void;
    getMethodColor: (method: string) => string;
    getAccessLevelColor: (level: string) => string;
}

export function ModulesList({
    modules,
    expandedModule,
    expandedEndpoint,
    onToggleModule,
    onToggleEndpoint,
    onCopy,
    getMethodColor,
    getAccessLevelColor,
}: ModulesListProps) {
    return (
        <section className="mb-8">
            <h2 className="text-2xl font-bold text-gray-900 mb-4 flex items-center">
                <Code className="w-6 h-6 text-blue-600 mr-2" />
                API Modules
            </h2>

            <div className="space-y-4">
                {modules.map((module) => (
                    <ModuleCard
                        key={module.name}
                        module={module}
                        isExpanded={expandedModule === module.name}
                        onToggle={onToggleModule}
                        expandedEndpoint={expandedEndpoint}
                        onToggleEndpoint={onToggleEndpoint}
                        onCopy={onCopy}
                        getMethodColor={getMethodColor}
                        getAccessLevelColor={getAccessLevelColor}
                    />
                ))}
            </div>
        </section>
    );
}

export function ErrorCodesSection() {
    return (
        <section className="mb-8">
            <h2 className="text-2xl font-bold text-gray-900 mb-4">Error Codes</h2>

            <div className="bg-white border rounded-lg p-6">
                <div className="space-y-4">
                    {[
                        { code: 200, status: 'OK', description: 'Request successful' },
                        { code: 400, status: 'Bad Request', description: 'Invalid request parameters' },
                        { code: 401, status: 'Unauthorized', description: 'Invalid or missing API key' },
                        { code: 403, status: 'Forbidden', description: 'Insufficient permissions for this resource' },
                        { code: 404, status: 'Not Found', description: 'Resource not found' },
                        { code: 429, status: 'Too Many Requests', description: 'Rate limit exceeded' },
                        { code: 500, status: 'Internal Server Error', description: 'Server error - contact support' },
                    ].map((error) => (
                        <div key={error.code} className="flex items-center space-x-4 p-3 border rounded">
                            <code
                                className={`px-2 py-1 rounded text-sm font-mono ${
                                    error.code < 300
                                        ? 'bg-green-100 text-green-800'
                                        : error.code < 400
                                          ? 'bg-blue-100 text-blue-800'
                                          : error.code < 500
                                            ? 'bg-yellow-100 text-yellow-800'
                                            : 'bg-red-100 text-red-800'
                                }`}
                            >
                                {error.code}
                            </code>
                            <div>
                                <span className="font-medium text-gray-900">{error.status}</span>
                                <span className="text-gray-500 ml-2">- {error.description}</span>
                            </div>
                        </div>
                    ))}
                </div>
            </div>
        </section>
    );
}

export function SupportSection() {
    return (
        <section>
            <h2 className="text-2xl font-bold text-gray-900 mb-4">Support</h2>

            <div className="bg-white border rounded-lg p-6">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div>
                        <h3 className="font-semibold text-gray-900 mb-2">Need Help?</h3>
                        <p className="text-gray-600 mb-4">Our developer support team is here to help you integrate successfully.</p>
                        <div className="space-y-2 text-sm">
                            <div>📧 Email: api-support@epsx.com</div>
                            <div>💬 Discord: EPSX Developers</div>
                            <div>📚 Knowledge Base: docs.epsx.com</div>
                        </div>
                    </div>

                    <div>
                        <h3 className="font-semibold text-gray-900 mb-2">Status & Updates</h3>
                        <p className="text-gray-600 mb-4">Stay informed about API updates and maintenance.</p>
                        <div className="space-y-2 text-sm">
                            <div>🟢 API Status: status.epsx.com</div>
                            <div>📢 Changelog: github.com/epsx/api-changelog</div>
                            <div>🔔 Developer Newsletter: Subscribe for updates</div>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    );
}
