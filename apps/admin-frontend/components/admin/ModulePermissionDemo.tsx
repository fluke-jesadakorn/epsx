'use client';

import React from 'react';
import { useModuleAuth, withModuleAccess, ModuleAccessStatus } from '@/auth/module-ctx';
import { Shield, Lock, CheckCircle, XCircle, AlertTriangle } from 'lucide-react';

// Example component that demonstrates module-based permission checks
export const ModulePermissionDemo: React.FC = () => {
  const { 
    moduleAccess: _moduleAccess, 
    hasModuleAccess, 
    getAccessLevel, 
    canPerformAction, 
    hasFeatureAccess,
    getQuotaStatus 
  } = useModuleAuth();

  const modules = ['stock-ranking', 'portfolio-analysis', 'market-data', 'trading-signals'];
  const actions = [
    'view_rankings',
    'ai_insights', 
    'custom_algorithms',
    'bulk_operations',
    'real_time_data',
    'premium_data'
  ];

  return (
    <div className="p-6 max-w-4xl mx-auto">
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-900 mb-2">
          Module Permission System Demo
        </h1>
        <p className="text-gray-600">
          This component demonstrates how to use the new module-based permission system.
        </p>
      </div>

      {/* User's Module Access Overview */}
      <div className="bg-white rounded-lg shadow p-6 mb-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4 flex items-center">
          <Shield className="w-5 h-5 mr-2 text-blue-600" />
          Your Module Access
        </h2>
        
        <div className="mb-4">
          <ModuleAccessStatus />
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {modules.map(module => {
            const hasAccess = hasModuleAccess(module);
            const accessLevel = getAccessLevel(module);
            const quotaStatus = getQuotaStatus(module);
            
            return (
              <div key={module} className="border rounded-lg p-4">
                <div className="flex items-center justify-between mb-2">
                  <h3 className="font-medium text-gray-900">
                    {module.replace('-', ' ').replace(/\b\w/g, l => l.toUpperCase())}
                  </h3>
                  {hasAccess ? (
                    <CheckCircle className="w-5 h-5 text-green-500" />
                  ) : (
                    <XCircle className="w-5 h-5 text-red-500" />
                  )}
                </div>
                
                {hasAccess ? (
                  <div className="space-y-2">
                    <div className="flex items-center justify-between text-sm">
                      <span className="text-gray-600">Access Level:</span>
                      <span className={`px-2 py-1 rounded text-xs font-medium ${
                        accessLevel === 'bronze' ? 'bg-amber-100 text-amber-800' :
                        accessLevel === 'silver' ? 'bg-gray-100 text-gray-800' :
                        accessLevel === 'gold' ? 'bg-yellow-100 text-yellow-800' :
                        accessLevel === 'platinum' ? 'bg-purple-100 text-purple-800' :
                        'bg-blue-100 text-blue-800'
                      }`}>
                        {accessLevel?.toUpperCase()}
                      </span>
                    </div>
                    
                    {quotaStatus && (
                      <div className="text-xs text-gray-500">
                        <div>Rate Limit: {quotaStatus.rate_limit_per_minute}/min</div>
                        {quotaStatus.daily_limit && quotaStatus.daily_limit > 0 && (
                          <div>Daily Limit: {quotaStatus.daily_limit}</div>
                        )}
                      </div>
                    )}
                  </div>
                ) : (
                  <div className="text-sm text-gray-500">
                    No access granted
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </div>

      {/* Action Permissions Matrix */}
      <div className="bg-white rounded-lg shadow p-6 mb-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4 flex items-center">
          <Lock className="w-5 h-5 mr-2 text-blue-600" />
          Action Permissions Matrix
        </h2>
        
        <div className="overflow-x-auto">
          <table className="min-w-full border-collapse">
            <thead>
              <tr className="border-b">
                <th className="text-left p-3 font-medium text-gray-900">Action</th>
                {modules.map(module => (
                  <th key={module} className="text-center p-3 font-medium text-gray-900">
                    <div className="text-xs">
                      {module.split('-').map(word => word.charAt(0).toUpperCase() + word.slice(1)).join(' ')}
                    </div>
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {actions.map(action => (
                <tr key={action} className="border-b hover:bg-gray-50">
                  <td className="p-3 font-medium text-gray-700">
                    {action.replace('_', ' ').replace(/\b\w/g, l => l.toUpperCase())}
                  </td>
                  {modules.map(module => {
                    const canPerform = canPerformAction(module, action);
                    const hasModule = hasModuleAccess(module);
                    
                    return (
                      <td key={`${module}-${action}`} className="p-3 text-center">
                        {!hasModule ? (
                          <div className="text-gray-300" title="No module access">
                            <XCircle className="w-4 h-4 mx-auto" />
                          </div>
                        ) : canPerform ? (
                          <div className="text-green-500" title="Action allowed">
                            <CheckCircle className="w-4 h-4 mx-auto" />
                          </div>
                        ) : (
                          <div className="text-red-500" title="Action not allowed">
                            <XCircle className="w-4 h-4 mx-auto" />
                          </div>
                        )}
                      </td>
                    );
                  })}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* Feature Access Examples */}
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4 flex items-center">
          <AlertTriangle className="w-5 h-5 mr-2 text-blue-600" />
          Feature Access Examples
        </h2>
        
        <div className="space-y-4">
          <div className="p-4 border rounded-lg">
            <h3 className="font-medium text-gray-900 mb-2">Stock Ranking Features</h3>
            <div className="grid grid-cols-2 gap-2 text-sm">
              {[
                'basic_rankings',
                'ai_insights', 
                'pattern_recognition',
                'custom_algorithms',
                'real_time_updates',
                'bulk_operations'
              ].map(feature => {
                const hasFeature = hasFeatureAccess('stock-ranking', feature);
                return (
                  <div key={feature} className="flex items-center justify-between">
                    <span className="text-gray-600">
                      {feature.replace('_', ' ').replace(/\b\w/g, l => l.toUpperCase())}
                    </span>
                    {hasFeature ? (
                      <CheckCircle className="w-4 h-4 text-green-500" />
                    ) : (
                      <XCircle className="w-4 h-4 text-red-500" />
                    )}
                  </div>
                );
              })}
            </div>
          </div>
          
          <div className="p-4 border rounded-lg">
            <h3 className="font-medium text-gray-900 mb-2">Market Data Features</h3>
            <div className="grid grid-cols-2 gap-2 text-sm">
              {[
                'delayed_quotes',
                'real_time_quotes',
                'technical_indicators',
                'level2_data',
                'options_data',
                'international_data'
              ].map(feature => {
                const hasFeature = hasFeatureAccess('market-data', feature);
                return (
                  <div key={feature} className="flex items-center justify-between">
                    <span className="text-gray-600">
                      {feature.replace('_', ' ').replace(/\b\w/g, l => l.toUpperCase())}
                    </span>
                    {hasFeature ? (
                      <CheckCircle className="w-4 h-4 text-green-500" />
                    ) : (
                      <XCircle className="w-4 h-4 text-red-500" />
                    )}
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

// Example of using the withModuleAccess HOC
const StockRankingComponent: React.FC = () => {
  return (
    <div className="p-4 bg-blue-50 rounded-lg">
      <h3 className="font-semibold text-blue-900 mb-2">Stock Ranking Module</h3>
      <p className="text-blue-800">
        This component is only visible to users with stock-ranking module access.
      </p>
    </div>
  );
};

// Wrap component with module access protection
export const ProtectedStockRankingComponent = withModuleAccess(
  StockRankingComponent,
  'stock-ranking',
  'view_rankings'
);

// Example of conditional rendering based on permissions
export const ConditionalFeatureComponent: React.FC = () => {
  const { hasModuleAccess, canPerformAction } = useModuleAuth();
  
  return (
    <div className="space-y-4">
      {/* Basic feature - shown to all users with module access */}
      {hasModuleAccess('stock-ranking') && (
        <div className="p-4 bg-green-50 rounded-lg">
          <h3 className="font-semibold text-green-900">Basic Rankings</h3>
          <p className="text-green-800">Available to all stock-ranking users</p>
        </div>
      )}
      
      {/* Premium feature - shown only to users with specific action permissions */}
      {canPerformAction('stock-ranking', 'ai_insights') && (
        <div className="p-4 bg-purple-50 rounded-lg">
          <h3 className="font-semibold text-purple-900">AI Insights</h3>
          <p className="text-purple-800">Available to Silver+ users</p>
        </div>
      )}
      
      {/* Enterprise feature - shown only to users with highest permissions */}
      {canPerformAction('stock-ranking', 'bulk_operations') && (
        <div className="p-4 bg-orange-50 rounded-lg">
          <h3 className="font-semibold text-orange-900">Bulk Operations</h3>
          <p className="text-orange-800">Available to Platinum+ users</p>
        </div>
      )}
    </div>
  );
};
