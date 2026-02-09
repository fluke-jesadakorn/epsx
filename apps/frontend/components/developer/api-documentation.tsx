'use client';

import { Button, Card, CardContent, CardHeader, CardTitle } from '@/components/ui';
import { Badge } from '@/components/ui/badge';
import type { AuthUser } from '@/lib/server-actions';
import { copyToClipboard as copyToClipboardUtil } from '@/utils/util';
import { useState } from 'react';
import { toast } from 'sonner';

interface APIDocumentationProps {
  currentUser: AuthUser;
}

interface APIEndpoint {
  method: 'GET' | 'POST' | 'PUT' | 'DELETE';
  endpoint: string;
  description: string;
  requiresAuth: boolean;
  authType: 'api_key' | 'session' | 'none';
  parameters?: { name: string; type: string; required: boolean; description: string }[];
  response: string;
  example: string;
}

export function APIDocumentation({ currentUser }: APIDocumentationProps) {
  const [selectedSection, setSelectedSection] = useState('public');

  // Use environment variable for base URL
  const baseUrl = process.env.NEXT_PUBLIC_BACKEND_URL ?? 'http://localhost:8080';

  const endpoints: Record<string, APIEndpoint[]> = {
    public: [
      {
        method: 'GET',
        endpoint: '/api/public/analytics/rankings',
        description: 'Get stock rankings with EPS analysis. No authentication required.',
        requiresAuth: false,
        authType: 'none',
        parameters: [
          { name: 'page', type: 'number', required: false, description: 'Page number (default: 1)' },
          { name: 'limit', type: 'number', required: false, description: 'Results per page (max: 100)' },
          { name: 'country', type: 'string', required: false, description: 'Filter by country code (e.g., TH, US)' },
          { name: 'sector', type: 'string', required: false, description: 'Filter by sector' },
          { name: 'sort_by', type: 'string', required: false, description: 'Sort field (e.g., eps_growth, market_cap)' },
          { name: 'sort_order', type: 'string', required: false, description: 'Sort order: asc or desc' }
        ],
        response: `{
  "success": true,
  "data": {
    "rankings": [
      {
        "symbol": "PTT",
        "company_name": "PTT Public Company Limited",
        "eps_growth": 0.15,
        "current_price": 35.25,
        "market_cap": 1250000000,
        "sector": "Energy",
        "country": "TH"
      }
    ],
    "pagination": {
      "page": 1,
      "limit": 10,
      "total": 500,
      "total_pages": 50
    }
  }
}`,
        example: `curl -X GET "${baseUrl}/api/public/analytics/rankings?limit=10&country=TH"`
      },
      {
        method: 'GET',
        endpoint: '/api/public/analytics/stock/:symbol',
        description: 'Get detailed analytics for a specific stock by symbol.',
        requiresAuth: false,
        authType: 'none',
        parameters: [
          { name: 'symbol', type: 'string', required: true, description: 'Stock symbol (e.g., PTT, ADVANC)' }
        ],
        response: `{
  "success": true,
  "data": {
    "symbol": "PTT",
    "company_name": "PTT Public Company Limited",
    "current_price": 35.25,
    "eps": 3.45,
    "pe_ratio": 10.2,
    "market_cap": 1250000000,
    "sector": "Energy",
    "country": "TH",
    "last_updated": "2025-12-25T00:00:00Z"
  }
}`,
        example: `curl -X GET "${baseUrl}/api/public/analytics/stock/PTT"`
      }
    ],
    developer: [
      {
        method: 'GET',
        endpoint: '/api/developer-portal/my-keys',
        description: 'List your API keys. Requires session authentication.',
        requiresAuth: true,
        authType: 'session',
        parameters: [
          { name: 'limit', type: 'number', required: false, description: 'Max results (default: 100)' },
          { name: 'status', type: 'string', required: false, description: 'Filter by status: active, revoked' }
        ],
        response: `{
  "success": true,
  "data": {
    "api_keys": [
      {
        "id": "uuid",
        "key_preview": "epsx...abc",
        "client_name": "My Bot",
        "status": "active",
        "total_requests": 1250,
        "created_at": "2025-12-20T10:00:00Z"
      }
    ],
    "total": 1
  }
}`,
        example: `# Session auth via browser cookies
curl -X GET "${baseUrl}/api/developer-portal/my-keys" \\
  --cookie "session=YOUR_SESSION_COOKIE"`
      },
      {
        method: 'POST',
        endpoint: '/api/developer-portal/my-keys',
        description: 'Create a new API key. Requires session authentication.',
        requiresAuth: true,
        authType: 'session',
        parameters: [
          { name: 'client_name', type: 'string', required: true, description: 'Name for the API key' },
          { name: 'client_description', type: 'string', required: false, description: 'Description of usage' },
          { name: 'plan_ids', type: 'array', required: false, description: 'Permission group IDs to assign' },
          { name: 'expires_at', type: 'string', required: false, description: 'ISO 8601 expiration date' }
        ],
        response: `{
  "success": true,
  "data": {
    "full_key": "epsx_live_abc123...",
    "api_key": {
      "id": "uuid",
      "client_name": "My Bot",
      "status": "active"
    }
  }
}`,
        example: `curl -X POST "${baseUrl}/api/developer-portal/my-keys" \\
  -H "Content-Type: application/json" \\
  --cookie "session=YOUR_SESSION_COOKIE" \\
  -d '{
    "client_name": "Trading Bot",
    "expires_at": "2026-12-25T23:59:59Z"
  }'`
      },
      {
        method: 'GET',
        endpoint: '/api/developer-portal/my-groups',
        description: 'Get your assigned permission groups and usage summary.',
        requiresAuth: true,
        authType: 'session',
        parameters: [],
        response: `{
  "success": true,
  "data": {
    "groups": [
      {
        "id": "uuid",
        "name": "API Developer",
        "slug": "api-developer",
        "permissions": ["epsx:analytics:read"]
      }
    ],
    "total_api_keys": 2,
    "total_requests": 5000
  }
}`,
        example: `curl -X GET "${baseUrl}/api/developer-portal/my-groups" \\
  --cookie "session=YOUR_SESSION_COOKIE"`
      }
    ],
    apikey: [
      {
        method: 'GET',
        endpoint: '/api/analytics/rankings',
        description: 'Get stock rankings using API key authentication.',
        requiresAuth: true,
        authType: 'api_key',
        parameters: [
          { name: 'page', type: 'number', required: false, description: 'Page number' },
          { name: 'limit', type: 'number', required: false, description: 'Results per page' }
        ],
        response: `{
  "success": true,
  "data": {
    "rankings": [...],
    "pagination": {...}
  }
}`,
        example: `curl -X GET "${baseUrl}/api/analytics/rankings" \\
  -H "Authorization: Bearer YOUR_API_KEY"`
      }
    ]
  };

  const sections = [
    { id: 'public', name: 'Public API', icon: '🌐', description: 'No auth required' },
    { id: 'developer', name: 'Developer Portal', icon: '🔧', description: 'Session auth' },
    { id: 'apikey', name: 'API Key auth', icon: '🔑', description: 'Bearer token' }
  ];

  const getMethodColor = (method: string) => {
    switch (method) {
      case 'GET': return 'bg-emerald-100 text-emerald-800 dark:bg-emerald-900/30 dark:text-emerald-400';
      case 'POST': return 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400';
      case 'PUT': return 'bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-400';
      case 'DELETE': return 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400';
      default: return 'bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-400';
    }
  };

  const getAuthBadge = (authType: string) => {
    switch (authType) {
      case 'none': return <Badge className="bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400">Public</Badge>;
      case 'session': return <Badge className="bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-400">Session</Badge>;
      case 'api_key': return <Badge className="bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-400">API Key</Badge>;
      default: return null;
    }
  };

  const copyToClipboard = async (text: string) => {
    const success = await copyToClipboardUtil(text);
    if (success) {
      toast.success('Copied to clipboard!');
    }
  };

  return (
    <div className="space-y-6">
      {/* Base URL */}
      <Card className="border-0 bg-gradient-to-r from-emerald-50 to-teal-50 dark:from-emerald-900/20 dark:to-teal-900/20">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <span className="text-2xl">🚀</span>
            API Base URL
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-3">
            <code className="flex-1 bg-white dark:bg-gray-800 p-3 rounded-lg font-mono text-sm border">
              {baseUrl}
            </code>
            <Button
              variant="outline"
              size="sm"
              onClick={() => copyToClipboard(baseUrl)}
            >
              Copy
            </Button>
          </div>
          <p className="text-sm text-gray-600 dark:text-gray-400 mt-3">
            All API requests should be made to this base URL. Public endpoints require no authentication.
          </p>
        </CardContent>
      </Card>

      {/* Authentication Guide */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <span className="text-2xl">🔐</span>
            Authentication Methods
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid md:grid-cols-3 gap-4">
            <div className="p-4 bg-green-50 dark:bg-green-900/20 rounded-xl">
              <h4 className="font-semibold text-green-800 dark:text-green-300 mb-2">🌐 Public</h4>
              <p className="text-sm text-green-700 dark:text-green-400">No authentication needed. Free access to public endpoints.</p>
            </div>
            <div className="p-4 bg-purple-50 dark:bg-purple-900/20 rounded-xl">
              <h4 className="font-semibold text-purple-800 dark:text-purple-300 mb-2">🔧 Session</h4>
              <p className="text-sm text-purple-700 dark:text-purple-400">Sign in with wallet. Used for developer portal management.</p>
            </div>
            <div className="p-4 bg-amber-50 dark:bg-amber-900/20 rounded-xl">
              <h4 className="font-semibold text-amber-800 dark:text-amber-300 mb-2">🔑 API Key</h4>
              <p className="text-sm text-amber-700 dark:text-amber-400 mb-2">Use Bearer token for programmatic access:</p>
              <code className="text-xs bg-white dark:bg-gray-800 px-2 py-1 rounded">
                Authorization: Bearer YOUR_API_KEY
              </code>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Section Navigation */}
      <div className="flex flex-wrap gap-2 p-1 bg-gray-100 dark:bg-gray-800 rounded-xl">
        {sections.map((section) => (
          <button
            key={section.id}
            onClick={() => setSelectedSection(section.id)}
            className={`flex-1 min-w-[150px] px-4 py-3 rounded-lg font-medium text-sm transition-all ${selectedSection === section.id
              ? 'bg-white dark:bg-gray-700 shadow-lg text-emerald-600 dark:text-emerald-400'
              : 'text-gray-600 dark:text-gray-400 hover:bg-white/50 dark:hover:bg-gray-700/50'
              }`}
          >
            <div className="flex items-center justify-center gap-2">
              <span>{section.icon}</span>
              <span>{section.name}</span>
            </div>
            <div className="text-xs opacity-70 mt-1">{section.description}</div>
          </button>
        ))}
      </div>

      {/* Endpoints */}
      <div className="space-y-4">
        {endpoints[selectedSection]?.map((endpoint, idx) => (
          <Card key={idx} className="overflow-hidden">
            <CardHeader className="bg-gray-50 dark:bg-gray-800/50">
              <div className="flex flex-wrap items-center gap-3">
                <Badge className={`font-mono font-bold ${getMethodColor(endpoint.method)}`}>
                  {endpoint.method}
                </Badge>
                <code className="text-sm font-medium bg-white dark:bg-gray-900 px-3 py-1 rounded border">
                  {endpoint.endpoint}
                </code>
                {getAuthBadge(endpoint.authType)}
              </div>
              <p className="text-gray-600 dark:text-gray-400 mt-2">{endpoint.description}</p>
            </CardHeader>
            <CardContent className="space-y-6 pt-6">
              {/* Parameters */}
              {endpoint.parameters && endpoint.parameters.length > 0 && (
                <div>
                  <h4 className="font-semibold mb-3 flex items-center gap-2">
                    <span className="text-blue-500">⚙️</span> Parameters
                  </h4>
                  <div className="overflow-x-auto">
                    <table className="w-full text-sm">
                      <thead>
                        <tr className="border-b dark:border-gray-700">
                          <th className="text-left py-2 px-3">Name</th>
                          <th className="text-left py-2 px-3">Type</th>
                          <th className="text-left py-2 px-3">Required</th>
                          <th className="text-left py-2 px-3">Description</th>
                        </tr>
                      </thead>
                      <tbody>
                        {endpoint.parameters.map((param, paramIdx) => (
                          <tr key={paramIdx} className="border-b dark:border-gray-800">
                            <td className="py-2 px-3">
                              <code className="bg-gray-100 dark:bg-gray-800 px-2 py-0.5 rounded text-xs">
                                {param.name}
                              </code>
                            </td>
                            <td className="py-2 px-3 text-gray-500">{param.type}</td>
                            <td className="py-2 px-3">
                              {param.required ? (
                                <Badge variant="outline" className="text-xs text-red-600 border-red-300">required</Badge>
                              ) : (
                                <span className="text-gray-400 text-xs">optional</span>
                              )}
                            </td>
                            <td className="py-2 px-3 text-gray-600 dark:text-gray-400">{param.description}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>
              )}

              {/* Response */}
              <div>
                <div className="flex items-center justify-between mb-3">
                  <h4 className="font-semibold flex items-center gap-2">
                    <span className="text-green-500">📤</span> Response
                  </h4>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => copyToClipboard(endpoint.response)}
                  >
                    Copy
                  </Button>
                </div>
                <pre className="bg-gray-100 dark:bg-gray-800 p-4 rounded-lg text-xs overflow-x-auto max-h-60">
                  <code>{endpoint.response}</code>
                </pre>
              </div>

              {/* Example */}
              <div>
                <div className="flex items-center justify-between mb-3">
                  <h4 className="font-semibold flex items-center gap-2">
                    <span className="text-purple-500">💻</span> Example Request
                  </h4>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => copyToClipboard(endpoint.example)}
                  >
                    Copy
                  </Button>
                </div>
                <pre className="bg-gray-900 text-green-400 p-4 rounded-lg text-xs overflow-x-auto">
                  <code>{endpoint.example}</code>
                </pre>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Rate Limits */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <span className="text-2xl">⚡</span>
            Rate Limits
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid md:grid-cols-2 gap-4">
            <div className="p-4 bg-blue-50 dark:bg-blue-900/20 rounded-xl">
              <h4 className="font-semibold text-blue-800 dark:text-blue-300">Default Limits</h4>
              <ul className="text-sm text-blue-700 dark:text-blue-400 mt-2 space-y-1">
                <li>• 60 requests per minute</li>
                <li>• 10,000 requests per day</li>
              </ul>
            </div>
            <div className="p-4 bg-purple-50 dark:bg-purple-900/20 rounded-xl">
              <h4 className="font-semibold text-purple-800 dark:text-purple-300">Higher Limits</h4>
              <p className="text-sm text-purple-700 dark:text-purple-400 mt-2">
                Contact admin for enterprise rate limits based on your subscription plan.
              </p>
            </div>
          </div>
          <p className="text-sm text-gray-600 dark:text-gray-400 mt-4">
            Rate limits are enforced per API key. Exceeding limits will result in HTTP 429 responses.
            Check the <code className="bg-gray-100 dark:bg-gray-800 px-1 rounded">X-RateLimit-*</code> headers for current usage.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}