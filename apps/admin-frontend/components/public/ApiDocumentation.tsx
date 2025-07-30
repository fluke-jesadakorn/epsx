'use client';

import React, { useState } from 'react';
import { 
  Code, 
  Shield, 
  Globe, 
  Key, 
  AlertTriangle, 
  BookOpen, 
  ExternalLink,
  Copy,
  ChevronRight,
  ChevronDown
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { toast } from 'react-hot-toast';

interface EndpointExample {
  method: string;
  path: string;
  description: string;
  parameters?: Array<{
    name: string;
    type: string;
    required: boolean;
    description: string;
  }>;
  response: string;
  accessLevel: string;
}

interface ModuleDocumentation {
  name: string;
  displayName: string;
  description: string;
  category: string;
  endpoints: EndpointExample[];
}

const moduleDocumentation: ModuleDocumentation[] = [
  {
    name: 'stock-ranking',
    displayName: 'Stock Ranking',
    description: 'Advanced stock ranking and analysis tools with AI-powered insights',
    category: 'Analytics',
    endpoints: [
      {
        method: 'GET',
        path: '/api/v1/stock-ranking/rankings',
        description: 'Get basic stock rankings with filtering options',
        accessLevel: 'Bronze+',
        parameters: [
          { name: 'sector', type: 'string', required: false, description: 'Filter by sector (e.g., "technology")' },
          { name: 'limit', type: 'integer', required: false, description: 'Number of results (max 100)' },
          { name: 'sort', type: 'string', required: false, description: 'Sort by: rank, performance, volume' }
        ],
        response: `{
  "data": [
    {
      "symbol": "AAPL",
      "rank": 1,
      "score": 95.2,
      "sector": "Technology",
      "performance": {
        "1d": 0.025,
        "1w": -0.012,
        "1m": 0.087
      }
    }
  ],
  "total": 50,
  "quota_remaining": 95
}`
      },
      {
        method: 'GET',
        path: '/api/v1/stock-ranking/rankings/ai-insights',
        description: 'Get AI-powered insights and predictions',
        accessLevel: 'Silver+',
        parameters: [
          { name: 'symbol', type: 'string', required: true, description: 'Stock symbol (e.g., "AAPL")' },
          { name: 'horizon', type: 'string', required: false, description: 'Prediction horizon: 1d, 1w, 1m' }
        ],
        response: `{
  "symbol": "AAPL",
  "insights": {
    "sentiment": "bullish",
    "confidence": 0.87,
    "price_target": 185.50,
    "key_factors": [
      "Strong earnings growth",
      "Positive analyst sentiment"
    ]
  },
  "quota_remaining": 18
}`
      },
      {
        method: 'POST',
        path: '/api/v1/stock-ranking/rankings/custom',
        description: 'Create custom ranking algorithm',
        accessLevel: 'Gold+',
        parameters: [
          { name: 'algorithm', type: 'object', required: true, description: 'Custom algorithm configuration' },
          { name: 'name', type: 'string', required: true, description: 'Algorithm name' }
        ],
        response: `{
  "algorithm_id": "custom_123",
  "name": "My Custom Algorithm",
  "status": "created",
  "estimated_runtime": "2-5 minutes"
}`
      }
    ]
  },
  {
    name: 'market-data',
    displayName: 'Market Data',
    description: 'Real-time and historical market data with technical indicators',
    category: 'Data',
    endpoints: [
      {
        method: 'GET',
        path: '/api/v1/market-data/quotes/{symbol}',
        description: 'Get current quote for a symbol (15-minute delay for Bronze)',
        accessLevel: 'Bronze+',
        parameters: [
          { name: 'symbol', type: 'string', required: true, description: 'Stock symbol in path (e.g., "AAPL")' }
        ],
        response: `{
  "symbol": "AAPL",
  "price": 182.45,
  "change": 2.15,
  "change_percent": 1.19,
  "volume": 45678901,
  "timestamp": "2024-01-15T15:30:00Z",
  "delay_minutes": 15
}`
      },
      {
        method: 'GET',
        path: '/api/v1/market-data/quotes/{symbol}/live',
        description: 'Get real-time quote (no delay)',
        accessLevel: 'Silver+',
        parameters: [
          { name: 'symbol', type: 'string', required: true, description: 'Stock symbol in path' }
        ],
        response: `{
  "symbol": "AAPL",
  "price": 182.47,
  "bid": 182.46,
  "ask": 182.48,
  "volume": 45678901,
  "timestamp": "2024-01-15T15:30:05Z",
  "real_time": true
}`
      },
      {
        method: 'GET',
        path: '/api/v1/market-data/indicators/{symbol}/rsi',
        description: 'Get Relative Strength Index (RSI) indicator',
        accessLevel: 'Silver+',
        parameters: [
          { name: 'symbol', type: 'string', required: true, description: 'Stock symbol in path' },
          { name: 'period', type: 'integer', required: false, description: 'RSI period (default: 14)' }
        ],
        response: `{
  "symbol": "AAPL",
  "indicator": "RSI",
  "value": 67.2,
  "signal": "neutral",
  "period": 14,
  "timestamp": "2024-01-15T15:30:00Z"
}`
      }
    ]
  },
  {
    name: 'portfolio-analysis',
    displayName: 'Portfolio Analysis',
    description: 'Comprehensive portfolio management and risk analysis tools',
    category: 'Analytics',
    endpoints: [
      {
        method: 'POST',
        path: '/api/v1/portfolio-analysis/portfolios',
        description: 'Create a new portfolio for analysis',
        accessLevel: 'Bronze+',
        parameters: [
          { name: 'name', type: 'string', required: true, description: 'Portfolio name' },
          { name: 'holdings', type: 'array', required: true, description: 'Array of stock holdings' }
        ],
        response: `{
  "portfolio_id": "port_123",
  "name": "My Portfolio",
  "total_value": 125000.00,
  "holdings_count": 8,
  "created_at": "2024-01-15T15:30:00Z"
}`
      },
      {
        method: 'GET',
        path: '/api/v1/portfolio-analysis/portfolios/{id}/risk',
        description: 'Get comprehensive risk analysis',
        accessLevel: 'Silver+',
        parameters: [
          { name: 'id', type: 'string', required: true, description: 'Portfolio ID in path' },
          { name: 'timeframe', type: 'string', required: false, description: 'Analysis timeframe: 1m, 3m, 6m, 1y' }
        ],
        response: `{
  "portfolio_id": "port_123",
  "risk_metrics": {
    "beta": 1.12,
    "sharpe_ratio": 1.47,
    "max_drawdown": -0.085,
    "var_95": -0.032
  },
  "risk_level": "moderate"
}`
      }
    ]
  },
  {
    name: 'trading-signals',
    displayName: 'Trading Signals',
    description: 'AI-powered trading signals and strategy management',
    category: 'Trading',
    endpoints: [
      {
        method: 'GET',
        path: '/api/v1/trading-signals/signals',
        description: 'Get current trading signals',
        accessLevel: 'Silver+',
        parameters: [
          { name: 'symbols', type: 'string', required: false, description: 'Comma-separated symbols (e.g., "AAPL,MSFT")' },
          { name: 'signal_type', type: 'string', required: false, description: 'Filter by: buy, sell, hold' }
        ],
        response: `{
  "signals": [
    {
      "symbol": "AAPL",
      "signal": "buy",
      "confidence": 0.87,
      "price_target": 185.50,
      "stop_loss": 175.00,
      "generated_at": "2024-01-15T15:30:00Z"
    }
  ],
  "quota_remaining": 15
}`
      },
      {
        method: 'POST',
        path: '/api/v1/trading-signals/strategies/{id}/backtest',
        description: 'Run backtesting on a trading strategy',
        accessLevel: 'Gold+',
        parameters: [
          { name: 'id', type: 'string', required: true, description: 'Strategy ID in path' },
          { name: 'start_date', type: 'string', required: true, description: 'Backtest start date (YYYY-MM-DD)' },
          { name: 'end_date', type: 'string', required: true, description: 'Backtest end date (YYYY-MM-DD)' }
        ],
        response: `{
  "backtest_id": "bt_456",
  "strategy_id": "strat_123",
  "results": {
    "total_return": 0.247,
    "sharpe_ratio": 1.83,
    "max_drawdown": -0.065,
    "win_rate": 0.68
  },
  "status": "completed"
}`
      }
    ]
  }
];

export const ApiDocumentation: React.FC = () => {
  const [expandedModule, setExpandedModule] = useState<string | null>(null);
  const [expandedEndpoint, setExpandedEndpoint] = useState<string | null>(null);

  const copyToClipboard = (text: string, label: string) => {
    navigator.clipboard.writeText(text).then(() => {
      toast.success(`${label} copied to clipboard`);
    }).catch(() => {
      toast.error('Failed to copy to clipboard');
    });
  };

  const getMethodColor = (method: string) => {
    switch (method.toUpperCase()) {
      case 'GET': return 'bg-green-100 text-green-800';
      case 'POST': return 'bg-blue-100 text-blue-800';
      case 'PUT': return 'bg-yellow-100 text-yellow-800';
      case 'DELETE': return 'bg-red-100 text-red-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  const getAccessLevelColor = (level: string) => {
    if (level.includes('Bronze')) return 'text-amber-600';
    if (level.includes('Silver')) return 'text-gray-500';
    if (level.includes('Gold')) return 'text-yellow-500';
    if (level.includes('Platinum')) return 'text-purple-600';
    return 'text-blue-600';
  };

  return (
    <div className="max-w-6xl mx-auto p-6">
      {/* Header */}
      <div className="mb-8">
        <div className="flex items-center mb-4">
          <BookOpen className="w-8 h-8 text-blue-600 mr-3" />
          <h1 className="text-3xl font-bold text-gray-900">API Documentation</h1>
        </div>
        <p className="text-lg text-gray-600 mb-6">
          Complete guide to integrating with the EPSX module-based API platform. 
          Access financial data, analytics, and trading tools programmatically.
        </p>
        
        {/* Quick Start Banner */}
        <div className="bg-gradient-to-r from-blue-50 to-indigo-50 border border-blue-200 rounded-lg p-6">
          <div className="flex items-start">
            <Key className="w-6 h-6 text-blue-600 mt-1 mr-3" />
            <div>
              <h3 className="font-semibold text-blue-900 mb-2">Need an API Key?</h3>
              <p className="text-blue-800 mb-3">
                Contact our team to get access to the developer portal and create your API keys.
              </p>
              <Button size="sm" className="bg-blue-600 hover:bg-blue-700">
                <ExternalLink className="w-4 h-4 mr-2" />
                Request Access
              </Button>
            </div>
          </div>
        </div>
      </div>

      {/* Authentication Section */}
      <section className="mb-8">
        <h2 className="text-2xl font-bold text-gray-900 mb-4 flex items-center">
          <Shield className="w-6 h-6 text-blue-600 mr-2" />
          Authentication
        </h2>
        
        <div className="bg-white border rounded-lg p-6">
          <p className="text-gray-600 mb-4">
            All API requests require authentication using an API key in the Authorization header:
          </p>
          
          <div className="bg-gray-900 rounded-lg p-4 mb-4">
            <div className="flex items-center justify-between mb-2">
              <span className="text-gray-400 text-sm">cURL Example</span>
              <button
                onClick={() => copyToClipboard(
                  'curl -H "Authorization: Bearer YOUR_API_KEY" https://api.epsx.com/v1/stock-ranking/rankings',
                  'cURL example'
                )}
                className="text-gray-400 hover:text-white"
              >
                <Copy className="w-4 h-4" />
              </button>
            </div>
            <code className="text-green-400 text-sm font-mono">
              curl -H "Authorization: Bearer YOUR_API_KEY" \
              <br />     https://api.epsx.com/v1/stock-ranking/rankings
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

      {/* Base URL Section */}
      <section className="mb-8">
        <h2 className="text-2xl font-bold text-gray-900 mb-4 flex items-center">
          <Globe className="w-6 h-6 text-blue-600 mr-2" />
          Base URL
        </h2>
        
        <div className="bg-white border rounded-lg p-6">
          <div className="flex items-center justify-between">
            <code className="text-lg font-mono text-gray-900 bg-gray-100 px-3 py-2 rounded">
              https://api.epsx.com/v1/
            </code>
            <button
              onClick={() => copyToClipboard('https://api.epsx.com/v1/', 'Base URL')}
              className="text-gray-400 hover:text-gray-600"
            >
              <Copy className="w-5 h-5" />
            </button>
          </div>
          <p className="text-gray-600 mt-3">
            All API endpoints are relative to this base URL. HTTPS is required for all requests.
          </p>
        </div>
      </section>

      {/* Rate Limits Section */}
      <section className="mb-8">
        <h2 className="text-2xl font-bold text-gray-900 mb-4">Rate Limits</h2>
        
        <div className="bg-white border rounded-lg p-6">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {[
              { level: 'Bronze', requests: '100/hour', daily: '1,000/day', color: 'amber' },
              { level: 'Silver', requests: '500/hour', daily: '5,000/day', color: 'gray' },
              { level: 'Gold', requests: '2,000/hour', daily: '20,000/day', color: 'yellow' },
              { level: 'Platinum', requests: '10,000/hour', daily: '100,000/day', color: 'purple' },
              { level: 'Enterprise', requests: 'Unlimited', daily: 'Fair usage', color: 'blue' }
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

      {/* Modules Documentation */}
      <section className="mb-8">
        <h2 className="text-2xl font-bold text-gray-900 mb-4 flex items-center">
          <Code className="w-6 h-6 text-blue-600 mr-2" />
          API Modules
        </h2>
        
        <div className="space-y-4">
          {moduleDocumentation.map((module) => (
            <div key={module.name} className="bg-white border rounded-lg overflow-hidden">
              <button
                onClick={() => setExpandedModule(expandedModule === module.name ? null : module.name)}
                className="w-full p-6 text-left hover:bg-gray-50 transition-colors"
              >
                <div className="flex items-center justify-between">
                  <div>
                    <h3 className="text-xl font-semibold text-gray-900 mb-1">{module.displayName}</h3>
                    <p className="text-gray-600">{module.description}</p>
                    <div className="mt-2 flex items-center space-x-4">
                      <span className="text-sm text-gray-500">Category: {module.category}</span>
                      <span className="text-sm text-gray-500">{module.endpoints.length} endpoints</span>
                    </div>
                  </div>
                  {expandedModule === module.name ? (
                    <ChevronDown className="w-5 h-5 text-gray-400" />
                  ) : (
                    <ChevronRight className="w-5 h-5 text-gray-400" />
                  )}
                </div>
              </button>
              
              {expandedModule === module.name && (
                <div className="border-t bg-gray-50">
                  <div className="p-6 space-y-4">
                    {module.endpoints.map((endpoint, index) => {
                      const endpointKey = `${module.name}-${index}`;
                      const isExpanded = expandedEndpoint === endpointKey;
                      
                      return (
                        <div key={index} className="bg-white border rounded-lg overflow-hidden">
                          <button
                            onClick={() => setExpandedEndpoint(isExpanded ? null : endpointKey)}
                            className="w-full p-4 text-left hover:bg-gray-50 transition-colors"
                          >
                            <div className="flex items-center justify-between">
                              <div className="flex items-center space-x-3">
                                <span className={`px-2 py-1 rounded text-xs font-mono font-semibold ${getMethodColor(endpoint.method)}`}>
                                  {endpoint.method}
                                </span>
                                <code className="text-sm font-mono text-gray-700">{endpoint.path}</code>
                                <span className={`text-xs font-medium ${getAccessLevelColor(endpoint.accessLevel)}`}>
                                  {endpoint.accessLevel}
                                </span>
                              </div>
                              {isExpanded ? (
                                <ChevronDown className="w-4 h-4 text-gray-400" />
                              ) : (
                                <ChevronRight className="w-4 h-4 text-gray-400" />
                              )}
                            </div>
                            <p className="text-sm text-gray-600 mt-2">{endpoint.description}</p>
                          </button>
                          
                          {isExpanded && (
                            <div className="border-t bg-gray-50 p-4">
                              <div className="space-y-4">
                                {/* Parameters */}
                                {endpoint.parameters && endpoint.parameters.length > 0 && (
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
                                          {endpoint.parameters.map((param, paramIndex) => (
                                            <tr key={paramIndex} className="border-t">
                                              <td className="p-2 font-mono text-blue-600">{param.name}</td>
                                              <td className="p-2 text-gray-600">{param.type}</td>
                                              <td className="p-2">
                                                <span className={`px-2 py-1 rounded text-xs ${
                                                  param.required 
                                                    ? 'bg-red-100 text-red-800' 
                                                    : 'bg-gray-100 text-gray-600'
                                                }`}>
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
                                )}
                                
                                {/* Example Response */}
                                <div>
                                  <div className="flex items-center justify-between mb-2">
                                    <h5 className="font-semibold text-gray-900">Example Response</h5>
                                    <button
                                      onClick={() => copyToClipboard(endpoint.response, 'Example response')}
                                      className="text-gray-400 hover:text-gray-600"
                                    >
                                      <Copy className="w-4 h-4" />
                                    </button>
                                  </div>
                                  <div className="bg-gray-900 rounded-lg p-4 overflow-x-auto">
                                    <pre className="text-green-400 text-sm font-mono whitespace-pre-wrap">
                                      {endpoint.response}
                                    </pre>
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
                                        copyToClipboard(curlExample, 'cURL example');
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
                            </div>
                          )}
                        </div>
                      );
                    })}
                  </div>
                </div>
              )}
            </div>
          ))}
        </div>
      </section>

      {/* Error Codes Section */}
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
              { code: 500, status: 'Internal Server Error', description: 'Server error - contact support' }
            ].map((error) => (
              <div key={error.code} className="flex items-center space-x-4 p-3 border rounded">
                <code className={`px-2 py-1 rounded text-sm font-mono ${
                  error.code < 300 ? 'bg-green-100 text-green-800' :
                  error.code < 400 ? 'bg-blue-100 text-blue-800' :
                  error.code < 500 ? 'bg-yellow-100 text-yellow-800' :
                  'bg-red-100 text-red-800'
                }`}>
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

      {/* Support Section */}
      <section>
        <h2 className="text-2xl font-bold text-gray-900 mb-4">Support</h2>
        
        <div className="bg-white border rounded-lg p-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <h3 className="font-semibold text-gray-900 mb-2">Need Help?</h3>
              <p className="text-gray-600 mb-4">
                Our developer support team is here to help you integrate successfully.
              </p>
              <div className="space-y-2 text-sm">
                <div>📧 Email: api-support@epsx.com</div>
                <div>💬 Discord: EPSX Developers</div>
                <div>📚 Knowledge Base: docs.epsx.com</div>
              </div>
            </div>
            
            <div>
              <h3 className="font-semibold text-gray-900 mb-2">Status & Updates</h3>
              <p className="text-gray-600 mb-4">
                Stay informed about API updates and maintenance.
              </p>
              <div className="space-y-2 text-sm">
                <div>🟢 API Status: status.epsx.com</div>
                <div>📢 Changelog: github.com/epsx/api-changelog</div>
                <div>🔔 Developer Newsletter: Subscribe for updates</div>
              </div>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
};
