'use client';

import { Key, Activity, BarChart3, Clock, AlertTriangle, Globe } from 'lucide-react';
import { memo } from 'react';

import { Card, CardContent } from '@/components/ui/card';

interface ApiKey {
  id: string;
  client_name: string;
  status: 'active' | 'revoked' | 'expired';
  total_requests: number;
  last_used_at?: string;
  created_at: string;
  expires_at?: string;
}

interface Module {
  id: string;
  name: string;
  description: string;
  endpoints: Array<{
    path: string;
    method: string;
    description: string;
  }>;
}

interface DeveloperOverviewTabProps {
  apiKeys: ApiKey[];
  modules: Module[];
}

function DeveloperOverviewTab({ apiKeys, modules }: DeveloperOverviewTabProps) {
  const activeKeys = apiKeys.filter(key => key.status === 'active');
  const totalRequests = apiKeys.reduce((sum, key) => sum + key.total_requests, 0);
  const recentlyUsedKeys = apiKeys.filter(key => {
    if (!key.last_used_at) {return false;}
    const lastUsed = new Date(key.last_used_at);
    const dayAgo = new Date(Date.now() - 24 * 60 * 60 * 1000);
    return lastUsed > dayAgo;
  });

  return (
    <div className="space-y-6">
      {/* Stats Overview */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        <Card>
          <CardContent className="p-6">
            <div className="flex items-center">
              <div className="p-2 bg-blue-100 rounded-lg">
                <Key className="w-6 h-6 text-blue-600" />
              </div>
              <div className="ml-4">
                <p className="text-sm font-medium text-gray-600 dark:text-gray-300">
                  Total API Keys
                </p>
                <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                  {apiKeys.length}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <div className="flex items-center">
              <div className="p-2 bg-green-100 dark:bg-green-900/50 rounded-lg">
                <Activity className="w-6 h-6 text-green-600" />
              </div>
              <div className="ml-4">
                <p className="text-sm font-medium text-gray-600 dark:text-gray-300">
                  Active Keys
                </p>
                <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                  {activeKeys.length}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <div className="flex items-center">
              <div className="p-2 bg-purple-100 dark:bg-purple-900/50 rounded-lg">
                <BarChart3 className="w-6 h-6 text-purple-600" />
              </div>
              <div className="ml-4">
                <p className="text-sm font-medium text-gray-600 dark:text-gray-300">
                  Total Requests
                </p>
                <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                  {totalRequests.toLocaleString()}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <div className="flex items-center">
              <div className="p-2 bg-orange-100 dark:bg-orange-900/50 rounded-lg">
                <Clock className="w-6 h-6 text-orange-600" />
              </div>
              <div className="ml-4">
                <p className="text-sm font-medium text-gray-600 dark:text-gray-300">
                  Active Today
                </p>
                <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                  {recentlyUsedKeys.length}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Quick Actions */}
      <Card>
        <CardContent className="p-6">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
            Quick Actions
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="p-4 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer">
              <div className="flex items-center space-x-3">
                <Key className="w-5 h-5 text-blue-600" />
                <div>
                  <h4 className="font-medium text-gray-900 dark:text-gray-100">
                    Create API Key
                  </h4>
                  <p className="text-sm text-gray-600 dark:text-gray-300">
                    Generate a new API key for integration
                  </p>
                </div>
              </div>
            </div>

            <div className="p-4 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer">
              <div className="flex items-center space-x-3">
                <BarChart3 className="w-5 h-5 text-green-600" />
                <div>
                  <h4 className="font-medium text-gray-900 dark:text-gray-100">
                    View Analytics
                  </h4>
                  <p className="text-sm text-gray-600 dark:text-gray-300">
                    Monitor usage and performance metrics
                  </p>
                </div>
              </div>
            </div>

            <div className="p-4 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer">
              <div className="flex items-center space-x-3">
                <Globe className="w-5 h-5 text-purple-600" />
                <div>
                  <h4 className="font-medium text-gray-900 dark:text-gray-100">
                    API Documentation
                  </h4>
                  <p className="text-sm text-gray-600 dark:text-gray-300">
                    Browse available endpoints and examples
                  </p>
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Available Modules */}
      <Card>
        <CardContent className="p-6">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
            Available Modules
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {modules.map(module => (
              <div 
                key={module.id} 
                className="p-4 border border-gray-200 dark:border-gray-700 rounded-lg"
              >
                <h4 className="font-medium text-gray-900 dark:text-gray-100 mb-2">
                  {module.name}
                </h4>
                <p className="text-sm text-gray-600 dark:text-gray-300 mb-3">
                  {module.description}
                </p>
                <div className="flex items-center text-xs text-gray-500 dark:text-gray-400">
                  <Globe className="w-3 h-3 mr-1" />
                  {module.endpoints.length} endpoints available
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Recent Activity Alert */}
      {activeKeys.length === 0 && (
        <Card className="border-yellow-200 bg-yellow-50 dark:bg-yellow-900/20">
          <CardContent className="p-6">
            <div className="flex items-center">
              <AlertTriangle className="w-5 h-5 text-yellow-600 mr-3" />
              <div>
                <h4 className="font-medium text-yellow-800 dark:text-yellow-200">
                  No Active API Keys
                </h4>
                <p className="text-sm text-yellow-700 dark:text-yellow-300">
                  Create your first API key to start integrating with our services.
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}

export default memo(DeveloperOverviewTab);