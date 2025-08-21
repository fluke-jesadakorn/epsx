/**
 * Analytics Usage Server Component - Server-rendered usage details
 * Shows detailed usage patterns and module-specific analytics
 */

import { Activity, Users, TrendingUp, AlertTriangle } from 'lucide-react'
import type { ModuleUsageData, TimeSeriesData } from '@/lib/actions/analytics-actions'
import { adminCardVariants, cn } from '@/design-system'

interface AnalyticsUsageServerProps {
  moduleData: ModuleUsageData[]
  timeSeriesData: TimeSeriesData[]
  selectedModule: string
}

export function AnalyticsUsageServer({ 
  moduleData, 
  timeSeriesData, 
  selectedModule 
}: AnalyticsUsageServerProps) {
  const formatNumber = (num: number) => {
    return new Intl.NumberFormat('en-US').format(num)
  }

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(amount)
  }

  // Filter data if specific module is selected
  const filteredModuleData = selectedModule === 'all' 
    ? moduleData 
    : moduleData.filter(m => m.moduleName === selectedModule)

  // Calculate usage statistics
  const totalQuota = filteredModuleData.reduce((sum, m) => sum + m.quota, 0)
  const totalQuotaUsed = filteredModuleData.reduce((sum, m) => sum + m.quotaUsed, 0)
  const overallQuotaUsage = totalQuota > 0 ? (totalQuotaUsed / totalQuota) * 100 : 0

  const highUsageModules = filteredModuleData.filter(m => m.quotaPercentage > 80)
  const warningModules = filteredModuleData.filter(m => m.quotaPercentage > 90)

  return (
    <div className="space-y-6">
      {/* Usage Summary */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
          <div className="flex items-center justify-between">
            <div>
              <p className="text-2xl font-bold text-info-600">{overallQuotaUsage.toFixed(1)}%</p>
              <p className="text-sm text-muted-foreground">Overall Usage</p>
            </div>
            <Activity className="h-8 w-8 text-info-500" />
          </div>
        </div>
        
        <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
          <div className="flex items-center justify-between">
            <div>
              <p className="text-2xl font-bold text-success-600">{formatNumber(totalQuotaUsed)}</p>
              <p className="text-sm text-muted-foreground">Total Used</p>
            </div>
            <TrendingUp className="h-8 w-8 text-success-500" />
          </div>
        </div>

        <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
          <div className="flex items-center justify-between">
            <div>
              <p className="text-2xl font-bold text-primary-600">{formatNumber(totalQuota)}</p>
              <p className="text-sm text-muted-foreground">Total Quota</p>
            </div>
            <Users className="h-8 w-8 text-primary-500" />
          </div>
        </div>

        <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
          <div className="flex items-center justify-between">
            <div>
              <p className="text-2xl font-bold text-error-600">{warningModules.length}</p>
              <p className="text-sm text-muted-foreground">Critical Usage</p>
            </div>
            <AlertTriangle className="h-8 w-8 text-error-500" />
          </div>
        </div>
      </div>

      {/* Usage Alerts */}
      {warningModules.length > 0 && (
        <div className="bg-error-50 border border-error-200 rounded-lg p-4">
          <div className="flex items-center gap-2 mb-3">
            <AlertTriangle className="h-5 w-5 text-error-600" />
            <h3 className="font-semibold text-error-900">Critical Usage Alert</h3>
          </div>
          <div className="space-y-2">
            {warningModules.map(module => (
              <div key={module.moduleName} className="flex justify-between items-center text-sm">
                <span className="text-error-800">{module.moduleName}</span>
                <span className="font-semibold text-error-900">
                  {module.quotaPercentage.toFixed(1)}% used
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Module Details Table */}
      <div className={cn(adminCardVariants({ variant: 'pancake' }), 'overflow-hidden')}>
        <div className="px-6 py-4 border-b border-neutral-200">
          <h3 className="text-lg font-semibold text-foreground">
            Module Usage Details 
            {selectedModule !== 'all' && (
              <span className="text-info-600 ml-2">({selectedModule})</span>
            )}
          </h3>
        </div>
        
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-neutral-200">
            <thead className="bg-neutral-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-neutral-500 uppercase tracking-wider">
                  Module
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-neutral-500 uppercase tracking-wider">
                  Requests
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-neutral-500 uppercase tracking-wider">
                  Users
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-neutral-500 uppercase tracking-wider">
                  Revenue
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-neutral-500 uppercase tracking-wider">
                  Quota Usage
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-neutral-500 uppercase tracking-wider">
                  Efficiency
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-neutral-200">
              {filteredModuleData.map((module) => {
                const efficiency = module.requests > 0 ? (module.revenue / module.requests * 1000).toFixed(4) : '0.0000'
                const quotaColor = module.quotaPercentage > 90 ? 'text-error-600' : 
                                 module.quotaPercentage > 80 ? 'text-warning-600' : 'text-success-600'
                
                return (
                  <tr key={module.moduleName} className="hover:bg-neutral-50">
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="font-medium text-neutral-900">{module.moduleName}</div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-neutral-900">
                      {formatNumber(module.requests)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-neutral-900">
                      {formatNumber(module.users)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap font-medium text-success-600">
                      {formatCurrency(module.revenue)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex items-center">
                        <div className="w-16 bg-neutral-200 rounded-full h-2 mr-2">
                          <div 
                            className={`h-2 rounded-full ${
                              module.quotaPercentage > 90 ? 'bg-error-500' :
                              module.quotaPercentage > 80 ? 'bg-warning-500' : 'bg-success-500'
                            }`}
                            style={{ width: `${Math.min(module.quotaPercentage, 100)}%` }}
                          />
                        </div>
                        <span className={`text-sm font-medium ${quotaColor}`}>
                          {module.quotaPercentage.toFixed(1)}%
                        </span>
                      </div>
                      <div className="text-xs text-neutral-500 mt-1">
                        {formatNumber(module.quotaUsed)} / {formatNumber(module.quota)}
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-neutral-600">
                      ${efficiency}/1K requests
                    </td>
                  </tr>
                )
              })}
            </tbody>
          </table>
        </div>
      </div>

      {/* Usage Recommendations */}
      <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <h3 className="text-lg font-semibold mb-4">Usage Recommendations</h3>
        <div className="space-y-3">
          {highUsageModules.length > 0 && (
            <div className="p-4 bg-warning-50 border border-warning-200 rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <AlertTriangle className="h-4 w-4 text-warning-600" />
                <span className="font-medium text-warning-900">High Usage Detected</span>
              </div>
              <p className="text-warning-800 text-sm">
                {highUsageModules.length} module(s) are using more than 80% of their quota. 
                Consider upgrading or optimizing usage patterns.
              </p>
            </div>
          )}
          
          {overallQuotaUsage < 50 && (
            <div className="p-4 bg-success-50 border border-success-200 rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <TrendingUp className="h-4 w-4 text-success-600" />
                <span className="font-medium text-success-900">Healthy Usage</span>
              </div>
              <p className="text-success-800 text-sm">
                Your overall usage is well within limits. You have room to grow without 
                additional quotas.
              </p>
            </div>
          )}

          <div className="p-4 bg-info-50 border border-info-200 rounded-lg">
            <div className="flex items-center gap-2 mb-2">
              <Users className="h-4 w-4 text-info-600" />
              <span className="font-medium text-info-900">Optimization Tips</span>
            </div>
            <ul className="text-info-800 text-sm space-y-1 list-disc list-inside">
              <li>Monitor peak usage times to optimize resource allocation</li>
              <li>Consider caching frequently requested data to reduce API calls</li>
              <li>Review user patterns to identify optimization opportunities</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  )
}