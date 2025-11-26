'use client';

import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle, Button } from '@/components/ui';
import { Badge } from '@/components/ui/badge';
import type { AuthUser } from '@/lib/server-actions';

interface APIDocumentationProps {
  currentUser: AuthUser;
}

interface APIEndpoint {
  method: 'GET' | 'POST' | 'PUT' | 'DELETE';
  endpoint: string;
  description: string;
  requiresAuth: boolean;
  parameters?: { name: string; type: string; required: boolean; description: string }[];
  response: string;
  example: string;
  tier: 'basic' | 'premium' | 'admin';
}

export function APIDocumentation({ currentUser }: APIDocumentationProps) {
  const [selectedSection, setSelectedSection] = useState('analytics');

  const endpoints: Record<string, APIEndpoint[]> = {
    analytics: [
      {
        method: 'GET',
        endpoint: '/api/analytics/rankings',
        description: 'Get stock rankings with EPS analysis',
        requiresAuth: true,
        tier: 'basic',
        parameters: [
          { name: 'page', type: 'number', required: false, description: 'Page number (default: 1)' },
          { name: 'limit', type: 'number', required: false, description: 'Results per page (max: 100)' },
          { name: 'country', type: 'string', required: false, description: 'Filter by country code' },
          { name: 'sector', type: 'string', required: false, description: 'Filter by sector' },
          { name: 'min_eps', type: 'number', required: false, description: 'Minimum EPS value' }
        ],
        response: `{
  "data": [
    {
      "symbol": "AAPL",
      "company_name": "Apple Inc.",
      "eps_growth": 0.15,
      "current_price": 150.25,
      "rank": 1,
      "country": "US",
      "sector": "Technology"
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 10,
    "total": 1000,
    "total_pages": 100
  }
}`,
        example: `curl -X GET "https://api.epsx.io/api/analytics/rankings?limit=10&country=US" \\
  -H "Authorization: Bearer YOUR_API_KEY"`
      },
      {
        method: 'GET',
        endpoint: '/api/analytics/stock/{symbol}',
        description: 'Get detailed analytics for a specific stock',
        requiresAuth: true,
        tier: 'premium',
        parameters: [
          { name: 'symbol', type: 'string', required: true, description: 'Stock symbol (e.g., AAPL)' },
          { name: 'period', type: 'string', required: false, description: 'Time period: 1y, 3y, 5y (default: 1y)' }
        ],
        response: `{
  "symbol": "AAPL",
  "company_name": "Apple Inc.",
  "current_data": {
    "price": 150.25,
    "eps": 6.15,
    "pe_ratio": 24.4,
    "growth_rate": 0.15
  },
  "historical_data": [
    {
      "date": "2024-01-01",
      "eps": 5.89,
      "price": 142.50
    }
  ],
  "predictions": {
    "next_quarter_eps": 6.45,
    "confidence": 0.82
  }
}`,
        example: `curl -X GET "https://api.epsx.io/api/analytics/stock/AAPL?period=1y" \\
  -H "Authorization: Bearer YOUR_API_KEY"`
      }
    ],
    webhooks: [
      {
        method: 'POST',
        endpoint: '/api/webhooks/rankings-update',
        description: 'Register webhook for ranking updates',
        requiresAuth: true,
        tier: 'premium',
        parameters: [
          { name: 'url', type: 'string', required: true, description: 'Your webhook URL' },
          { name: 'events', type: 'array', required: true, description: 'Events to subscribe to' },
          { name: 'filters', type: 'object', required: false, description: 'Optional filters' }
        ],
        response: `{
  "webhook_id": "wh_123456789",
  "url": "https://your-app.com/webhook",
  "events": ["ranking_updated", "new_stock_added"],
  "status": "active",
  "created_at": "2024-01-15T10:30:00Z"
}`,
        example: `curl -X POST "https://api.epsx.io/api/webhooks/rankings-update" \\
  -H "Authorization: Bearer YOUR_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "url": "https://your-app.com/webhook",
    "events": ["ranking_updated"]
  }'`
      }
    ]
  };

  const sections = [
    { id: 'analytics', name: 'Analytics API', icon: '📊' },
    { id: 'webhooks', name: 'Webhooks', icon: '🔔' }
  ];

  const getTierColor = (tier: string) => {
    switch (tier) {
      case 'basic': return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300';
      case 'premium': return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300';
      case 'admin': return 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-300';
      default: return 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-300';
    }
  };

  const hasAccess = (tier: string) => {
    if (currentUser.role === 'admin') return true;
    if (tier === 'basic') return true;
    if (tier === 'premium') return currentUser.role === 'premium' || currentUser.role === 'admin';
    return false;
  };

  return (
    <div className="space-y-6">
      {/* Base URL */}
      <Card>
        <CardHeader>
          <CardTitle>API Base URL</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="bg-gray-100 dark:bg-gray-700 p-3 rounded font-mono text-sm">
            https://api.epsx.io
          </div>
          <p className="text-sm text-gray-600 dark:text-gray-400 mt-2">
            All API requests should be made to this base URL with your API key in the Authorization header.
          </p>
        </CardContent>
      </Card>

      {/* Authentication */}
      <Card>
        <CardHeader>
          <CardTitle>Authentication</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="mb-4">Include your API key in the Authorization header:</p>
          <div className="bg-gray-100 dark:bg-gray-700 p-3 rounded font-mono text-sm">
            Authorization: Bearer YOUR_API_KEY
          </div>
        </CardContent>
      </Card>

      {/* Section Navigation */}
      <div className="flex space-x-4 border-b border-gray-200 dark:border-gray-700">
        {sections.map((section) => (
          <button
            key={section.id}
            onClick={() => setSelectedSection(section.id)}
            className={`pb-4 px-2 border-b-2 font-medium text-sm ${
              selectedSection === section.id
                ? 'border-emerald-500 text-emerald-600 dark:text-emerald-400'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-300'
            }`}
          >
            <span className="mr-2">{section.icon}</span>
            {section.name}
          </button>
        ))}
      </div>

      {/* Endpoints */}
      <div className="space-y-6">
        {endpoints[selectedSection]?.map((endpoint, idx) => (
          <Card key={idx} className={!hasAccess(endpoint.tier) ? 'opacity-60' : ''}>
            <CardHeader>
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-3">
                  <Badge variant="outline" className="font-mono">
                    {endpoint.method}
                  </Badge>
                  <code className="text-sm bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded">
                    {endpoint.endpoint}
                  </code>
                </div>
                <div className="flex items-center space-x-2">
                  <Badge className={getTierColor(endpoint.tier)}>
                    {endpoint.tier}
                  </Badge>
                  {!hasAccess(endpoint.tier) && (
                    <Badge variant="secondary">
                      Upgrade Required
                    </Badge>
                  )}
                </div>
              </div>
              <p className="text-gray-600 dark:text-gray-400">{endpoint.description}</p>
            </CardHeader>
            <CardContent className="space-y-6">
              {/* Parameters */}
              {endpoint.parameters && endpoint.parameters.length > 0 && (
                <div>
                  <h4 className="font-semibold mb-3">Parameters</h4>
                  <div className="space-y-2">
                    {endpoint.parameters.map((param, paramIdx) => (
                      <div key={paramIdx} className="flex items-start space-x-3 text-sm">
                        <code className="bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded text-xs">
                          {param.name}
                        </code>
                        <div className="flex-1">
                          <div className="flex items-center space-x-2">
                            <span className="text-gray-500">{param.type}</span>
                            {param.required && (
                              <Badge variant="outline" className="text-xs">required</Badge>
                            )}
                          </div>
                          <p className="text-gray-600 dark:text-gray-400 mt-1">
                            {param.description}
                          </p>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Response */}
              <div>
                <h4 className="font-semibold mb-3">Response</h4>
                <pre className="bg-gray-100 dark:bg-gray-700 p-4 rounded text-xs overflow-x-auto">
                  <code>{endpoint.response}</code>
                </pre>
              </div>

              {/* Example */}
              <div>
                <h4 className="font-semibold mb-3">Example Request</h4>
                <pre className="bg-gray-900 text-green-400 p-4 rounded text-xs overflow-x-auto">
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
          <CardTitle>Rate Limits</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid md:grid-cols-3 gap-4 text-sm">
            <div className="p-4 bg-green-50 dark:bg-green-900/20 rounded">
              <h4 className="font-semibold text-green-800 dark:text-green-300">Basic</h4>
              <p className="text-green-700 dark:text-green-400">100 requests/hour</p>
            </div>
            <div className="p-4 bg-blue-50 dark:bg-blue-900/20 rounded">
              <h4 className="font-semibold text-blue-800 dark:text-blue-300">Premium</h4>
              <p className="text-blue-700 dark:text-blue-400">1,000 requests/hour</p>
            </div>
            <div className="p-4 bg-purple-50 dark:bg-purple-900/20 rounded">
              <h4 className="font-semibold text-purple-800 dark:text-purple-300">Admin</h4>
              <p className="text-purple-700 dark:text-purple-400">Unlimited</p>
            </div>
          </div>
          <p className="text-sm text-gray-600 dark:text-gray-400 mt-4">
            Rate limits are enforced per API key. Exceeding limits will result in HTTP 429 responses.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}