'use client'

import { useState, useEffect } from 'react'

import { PancakeCard } from '@/components/ui/PancakeCard'
import { toast } from '@/hooks/use-toast'
import { createPlansClient, type PlanAnalyticsResponse, isApiSuccess } from '@/shared/api/plans'
import { createAdminApiClient } from '@/shared/utils/api-client'

interface PlanAnalyticsModalProps {
  planId: number
  onClose: () => void
}

/**
 *
 * @param root0
 * @param root0.planId
 * @param root0.onClose
 */
export function PlanAnalyticsModal({ planId, onClose }: PlanAnalyticsModalProps) {
  const [analytics, setAnalytics] = useState<PlanAnalyticsResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [selectedPeriod, setSelectedPeriod] = useState('30d')

  useEffect(() => {
    loadAnalytics()
  }, [planId, selectedPeriod])

  const loadAnalytics = async () => {
    const adminClient = createPlansClient(createAdminApiClient())
    try {
      setLoading(true)
      const response = await adminClient.getPlanAnalytics(planId, selectedPeriod)
      
      if (isApiSuccess(response)) {
        setAnalytics(response.data)
      } else {
        toast({
          title: "Error",
          description: response.error || "Failed to load analytics",
          variant: "destructive"
        })
      }
    } catch (_error) {
      toast({
        title: "Error",
        description: "Failed to load analytics",
        variant: "destructive"
      })
    } finally {
      setLoading(false)
    }
  }

  if (loading) {
    return (
      <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center p-4 z-50">
        <PancakeCard className="bg-white dark:bg-gray-800 max-w-6xl w-full max-h-[90vh] overflow-y-auto">
          <div className="p-8 space-y-6 animate-pulse">
            <div className="h-8 bg-gray-300 rounded-xl w-1/3"></div>
            <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
              {Array.from({ length: 4 }).map((_, i) => (
                <div key={i} className="bg-gray-300 rounded-3xl h-32"></div>
              ))}
            </div>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              {Array.from({ length: 4 }).map((_, i) => (
                <div key={i} className="bg-gray-300 rounded-3xl h-48"></div>
              ))}
            </div>
          </div>
        </PancakeCard>
      </div>
    )
  }

  if (!analytics) {
    return (
      <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center p-4 z-50">
        <PancakeCard className="bg-white dark:bg-gray-800 max-w-2xl w-full">
          <div className="p-8 text-center">
            <div className="text-6xl mb-4">📊</div>
            <h3 className="text-xl font-semibold text-gray-600 dark:text-gray-400 mb-2">
              No analytics data available
            </h3>
            <p className="text-gray-500 mb-6">
              Analytics data for this plan is not available at the moment.
            </p>
            <button
              onClick={onClose}
              className="px-6 py-3 bg-gradient-to-r from-emerald-400 to-green-500 text-white rounded-xl font-semibold"
            >
              Close
            </button>
          </div>
        </PancakeCard>
      </div>
    )
  }

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center p-4 z-50">
      <PancakeCard className="bg-white dark:bg-gray-800 max-w-6xl w-full max-h-[90vh] overflow-y-auto">
        <div className="p-8">
          {/* Header */}
          <div className="flex items-center justify-between mb-6">
            <div>
              <h2 className="text-2xl font-bold bg-gradient-to-r from-emerald-600 to-green-600 bg-clip-text text-transparent">
                Plan Analytics: {analytics.plan_name}
              </h2>
              <p className="text-gray-500 dark:text-gray-400 mt-1">
                Analytics period: {analytics.analytics_period}
              </p>
            </div>
            <div className="flex items-center gap-4">
              {/* Period Selector */}
              <select
                value={selectedPeriod}
                onChange={(e) => setSelectedPeriod(e.target.value)}
                className="px-4 py-2 rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              >
                <option value="7d">Last 7 days</option>
                <option value="30d">Last 30 days</option>
                <option value="90d">Last 90 days</option>
                <option value="1y">Last year</option>
              </select>
              
              <button
                onClick={onClose}
                className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 text-2xl"
              >
                ✕
              </button>
            </div>
          </div>

          {/* Key Metrics */}
          <div className="grid grid-cols-1 md:grid-cols-4 gap-6 mb-8">
            <PancakeCard className="bg-gradient-to-br from-blue-50 to-blue-100 dark:from-blue-900/20 dark:to-blue-800/20 border border-blue-200 dark:border-blue-800">
              <div className="p-6">
                <div className="text-blue-600 dark:text-blue-400 font-semibold mb-2">Total Subscribers</div>
                <div className="text-3xl font-bold text-blue-900 dark:text-blue-100 mb-1">
                  {analytics.subscriber_metrics.total_subscribers}
                </div>
                <div className="text-sm text-blue-600 dark:text-blue-400">
                  +{analytics.subscriber_metrics.new_subscribers_this_period} new
                </div>
              </div>
            </PancakeCard>

            <PancakeCard className="bg-gradient-to-br from-green-50 to-green-100 dark:from-green-900/20 dark:to-green-800/20 border border-green-200 dark:border-green-800">
              <div className="p-6">
                <div className="text-green-600 dark:text-green-400 font-semibold mb-2">Total Revenue</div>
                <div className="text-3xl font-bold text-green-900 dark:text-green-100 mb-1">
                  ${analytics.revenue_metrics.total_revenue.toFixed(2)}
                </div>
                <div className="text-sm text-green-600 dark:text-green-400">
                  {analytics.revenue_metrics.revenue_growth_rate > 0 ? '+' : ''}
                  {analytics.revenue_metrics.revenue_growth_rate.toFixed(1)}% growth
                </div>
              </div>
            </PancakeCard>

            <PancakeCard className="bg-gradient-to-br from-purple-50 to-purple-100 dark:from-purple-900/20 dark:to-purple-800/20 border border-purple-200 dark:border-purple-800">
              <div className="p-6">
                <div className="text-purple-600 dark:text-purple-400 font-semibold mb-2">Avg Revenue/User</div>
                <div className="text-3xl font-bold text-purple-900 dark:text-purple-100 mb-1">
                  ${analytics.revenue_metrics.average_revenue_per_user.toFixed(2)}
                </div>
                <div className="text-sm text-purple-600 dark:text-purple-400">per subscriber</div>
              </div>
            </PancakeCard>

            <PancakeCard className="bg-gradient-to-br from-orange-50 to-orange-100 dark:from-orange-900/20 dark:to-orange-800/20 border border-orange-200 dark:border-orange-800">
              <div className="p-6">
                <div className="text-orange-600 dark:text-orange-400 font-semibold mb-2">Efficiency Score</div>
                <div className="text-3xl font-bold text-orange-900 dark:text-orange-100 mb-1">
                  {analytics.performance_metrics.plan_efficiency_score.toFixed(1)}%
                </div>
                <div className="text-sm text-orange-600 dark:text-orange-400">performance</div>
              </div>
            </PancakeCard>
          </div>

          {/* Detailed Analytics */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-8 mb-8">
            {/* Usage Metrics */}
            <PancakeCard className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-white/20">
              <div className="p-6">
                <h3 className="text-xl font-bold text-gray-900 dark:text-white mb-4">Usage Metrics</h3>
                <div className="space-y-4">
                  <div className="flex justify-between items-center">
                    <span className="text-gray-600 dark:text-gray-400">Total API Calls</span>
                    <span className="font-semibold">{analytics.usage_metrics.total_api_calls.toLocaleString()}</span>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-gray-600 dark:text-gray-400">Data Transfer</span>
                    <span className="font-semibold">{analytics.usage_metrics.total_data_transfer_gb.toFixed(2)} GB</span>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-gray-600 dark:text-gray-400">Avg Requests/User</span>
                    <span className="font-semibold">{analytics.usage_metrics.average_requests_per_subscriber.toFixed(0)}</span>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-gray-600 dark:text-gray-400">Resource Utilization</span>
                    <span className="font-semibold">{analytics.usage_metrics.resource_utilization_percentage.toFixed(1)}%</span>
                  </div>
                </div>
              </div>
            </PancakeCard>

            {/* Performance Metrics */}
            <PancakeCard className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-white/20">
              <div className="p-6">
                <h3 className="text-xl font-bold text-gray-900 dark:text-white mb-4">Performance</h3>
                <div className="space-y-4">
                  <div className="flex justify-between items-center">
                    <span className="text-gray-600 dark:text-gray-400">Cost per Request</span>
                    <span className="font-semibold">${analytics.performance_metrics.cost_per_request.toFixed(4)}</span>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-gray-600 dark:text-gray-400">Profit Margin</span>
                    <span className="font-semibold">{analytics.performance_metrics.profit_margin.toFixed(1)}%</span>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-gray-600 dark:text-gray-400">Rate Limit Hits</span>
                    <span className="font-semibold">{analytics.performance_metrics.rate_limit_hit_rate.toFixed(1)}%</span>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-gray-600 dark:text-gray-400">Overage Usage</span>
                    <span className="font-semibold">{analytics.performance_metrics.overage_usage_rate.toFixed(1)}%</span>
                  </div>
                </div>
              </div>
            </PancakeCard>

            {/* Top Endpoints */}
            <PancakeCard className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-white/20">
              <div className="p-6">
                <h3 className="text-xl font-bold text-gray-900 dark:text-white mb-4">Top Endpoints</h3>
                <div className="space-y-3">
                  {analytics.usage_metrics.top_endpoints.slice(0, 5).map((endpoint, index) => (
                    <div key={index} className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
                      <div className="flex-1">
                        <div className="font-medium text-sm">{endpoint.endpoint}</div>
                        <div className="text-xs text-gray-500">{endpoint.avg_response_time_ms}ms avg</div>
                      </div>
                      <div className="text-right">
                        <div className="font-semibold">{endpoint.request_count.toLocaleString()}</div>
                        <div className="text-xs text-gray-500">requests</div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </PancakeCard>

            {/* Recommendations */}
            <PancakeCard className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-white/20">
              <div className="p-6">
                <h3 className="text-xl font-bold text-gray-900 dark:text-white mb-4">Recommendations</h3>
                <div className="space-y-3">
                  {analytics.recommendations.slice(0, 4).map((rec, index) => (
                    <div key={index} className="p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
                      <div className="flex items-center gap-2 mb-1">
                        <span className={`px-2 py-1 text-xs rounded ${
                          rec.priority === 'high' ? 'bg-red-100 text-red-600' :
                          rec.priority === 'medium' ? 'bg-yellow-100 text-yellow-600' :
                          'bg-green-100 text-green-600'
                        }`}>
                          {rec.priority}
                        </span>
                        <span className="font-semibold text-sm">{rec.title}</span>
                      </div>
                      <p className="text-xs text-gray-600 dark:text-gray-400">{rec.description}</p>
                    </div>
                  ))}
                </div>
              </div>
            </PancakeCard>
          </div>

          {/* Close Button */}
          <div className="flex justify-end">
            <button
              onClick={onClose}
              className="px-6 py-3 bg-gradient-to-r from-emerald-400 to-green-500 text-white rounded-xl font-semibold hover:from-emerald-500 hover:to-green-600"
            >
              Close
            </button>
          </div>
        </div>
      </PancakeCard>
    </div>
  )
}