'use client'

import { useParams, useRouter } from 'next/navigation'
import { useEffect, useState } from 'react'

import { PancakeCard } from '@/components/ui/PancakeCard'
import { toast } from '@/hooks/use-toast'
import { createPlansClient, isApiSuccess, type PlanAnalyticsResponse } from '@/shared/api/plans'
import { useSharedAuth } from '@/shared/components/auth/Provider'
import { createAdminApiClient } from '@/shared/utils/api-client'

/**
 *
 */
export default function PlanAnalyticsPage() {
  const router = useRouter()
  const params = useParams()
  const { user, isLoading: authLoading, isAuthenticated } = useSharedAuth()
  const [analytics, setAnalytics] = useState<PlanAnalyticsResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [selectedPeriod, setSelectedPeriod] = useState('30d')

  useEffect(() => {
    if (!authLoading && (!isAuthenticated || !user)) {
      router.push('/plans')
    }
  }, [authLoading, isAuthenticated, user, router])

  useEffect(() => {
    loadAnalytics()
  }, [params.id, selectedPeriod])

  const loadAnalytics = async () => {
    if (!params.id) { return }

    const adminClient = createPlansClient(createAdminApiClient())
    try {
      setLoading(true)
      const response = await adminClient.getPlanAnalytics(params.id as string, selectedPeriod)

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

  if (authLoading || loading) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
        <div className="max-w-6xl mx-auto">
          <PancakeCard className="bg-white dark:bg-gray-800">
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
      </div>
    )
  }

  if (!analytics) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
        <div className="max-w-2xl mx-auto">
          <PancakeCard className="bg-white dark:bg-gray-800">
            <div className="p-8 text-center">
              <div className="text-6xl mb-4">📊</div>
              <h3 className="text-xl font-semibold text-gray-600 dark:text-gray-400 mb-2">
                No analytics data available
              </h3>
              <p className="text-gray-500 mb-6">
                Analytics data for this plan is not available at the moment.
              </p>
              <button
                onClick={() => router.push('/plans')}
                className="px-6 py-3 bg-gradient-to-r from-emerald-400 to-green-500 text-white rounded-xl font-semibold"
              >
                Back to Plans
              </button>
            </div>
          </PancakeCard>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen p-6">
      <div className="max-w-6xl mx-auto">
        <div className="bg-card rounded-3xl border border-border/50 shadow-sm overflow-hidden">
          <div className="p-8">
            {/* Header */}
            <div className="flex items-center justify-between mb-8">
              <div>
                <h1 className="text-2xl font-bold bg-gradient-to-r from-primary via-secondary to-primary bg-clip-text text-transparent">
                  Plan Analytics: {analytics.plan_name}
                </h1>
                <p className="text-sm text-muted-foreground mt-1">
                  Analytics period: {analytics.analytics_period}
                </p>
              </div>
              <div className="flex items-center gap-4">
                <select
                  value={selectedPeriod}
                  onChange={(e) => setSelectedPeriod(e.target.value)}
                  className="px-4 py-2 rounded-xl border border-border bg-card text-foreground focus:ring-2 focus:ring-primary outline-none transition-all"
                >
                  <option value="7d">Last 7 days</option>
                  <option value="30d">Last 30 days</option>
                  <option value="90d">Last 90 days</option>
                  <option value="1y">Last year</option>
                </select>

                <button
                  onClick={() => router.push('/plans')}
                  className="p-2 rounded-xl text-muted-foreground hover:text-foreground hover:bg-muted transition-all"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </div>
            </div>

            {/* Key Metrics */}
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
              <div className="p-6 rounded-3xl bg-primary/5 border border-primary/20 shadow-sm">
                <div className="text-primary font-semibold mb-2">Total Subscribers</div>
                <div className="text-3xl font-bold text-foreground mb-1">
                  {analytics.subscriber_metrics.total_subscribers}
                </div>
                <div className="text-sm text-success font-medium">
                  +{analytics.subscriber_metrics.new_subscribers_this_period} new
                </div>
              </div>

              <div className="p-6 rounded-3xl bg-success/5 border border-success/20 shadow-sm">
                <div className="text-success font-semibold mb-2">Total Revenue</div>
                <div className="text-3xl font-bold text-foreground mb-1">
                  ${analytics.revenue_metrics.total_revenue.toFixed(2)}
                </div>
                <div className="text-sm text-success font-medium">
                  {analytics.revenue_metrics.revenue_growth_rate > 0 ? '+' : ''}
                  {analytics.revenue_metrics.revenue_growth_rate.toFixed(1)}% growth
                </div>
              </div>

              <div className="p-6 rounded-3xl bg-secondary/5 border border-secondary/20 shadow-sm">
                <div className="text-secondary font-semibold mb-2">Avg Revenue/User</div>
                <div className="text-3xl font-bold text-foreground mb-1">
                  ${analytics.revenue_metrics.average_revenue_per_user.toFixed(2)}
                </div>
                <div className="text-sm text-muted-foreground">per subscriber</div>
              </div>

              <div className="p-6 rounded-3xl bg-warning/5 border border-warning/20 shadow-sm">
                <div className="text-warning font-semibold mb-2">Efficiency Score</div>
                <div className="text-3xl font-bold text-foreground mb-1">
                  {analytics.performance_metrics.plan_efficiency_score.toFixed(1)}%
                </div>
                <div className="text-sm text-muted-foreground">performance</div>
              </div>
            </div>

            {/* Detailed Analytics */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-8 mb-8">
              {/* Usage Metrics */}
              <div className="p-6 rounded-3xl bg-card border border-border/50 shadow-sm">
                <h3 className="text-lg font-bold text-foreground mb-6">Usage Metrics</h3>
                <div className="space-y-4">
                  <div className="flex justify-between items-center pb-3 border-b border-border/30">
                    <span className="text-muted-foreground">Total API Calls</span>
                    <span className="font-semibold font-mono">{analytics.usage_metrics.total_api_calls.toLocaleString()}</span>
                  </div>
                  <div className="flex justify-between items-center pb-3 border-b border-border/30">
                    <span className="text-muted-foreground">Data Transfer</span>
                    <span className="font-semibold font-mono">{analytics.usage_metrics.total_data_transfer_gb.toFixed(2)} GB</span>
                  </div>
                  <div className="flex justify-between items-center pb-3 border-b border-border/30">
                    <span className="text-muted-foreground">Avg Requests/User</span>
                    <span className="font-semibold font-mono">{analytics.usage_metrics.average_requests_per_subscriber.toFixed(0)}</span>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-muted-foreground">Resource Utilization</span>
                    <span className="font-semibold font-mono">{analytics.usage_metrics.resource_utilization_percentage.toFixed(1)}%</span>
                  </div>
                </div>
              </div>

              {/* Performance Metrics */}
              <div className="p-6 rounded-3xl bg-card border border-border/50 shadow-sm">
                <h3 className="text-lg font-bold text-foreground mb-6">Performance</h3>
                <div className="space-y-4">
                  <div className="flex justify-between items-center pb-3 border-b border-border/30">
                    <span className="text-muted-foreground">Cost per Request</span>
                    <span className="font-semibold font-mono">${analytics.performance_metrics.cost_per_request.toFixed(4)}</span>
                  </div>
                  <div className="flex justify-between items-center pb-3 border-b border-border/30">
                    <span className="text-muted-foreground">Profit Margin</span>
                    <span className="font-semibold font-mono">{analytics.performance_metrics.profit_margin.toFixed(1)}%</span>
                  </div>
                  <div className="flex justify-between items-center pb-3 border-b border-border/30">
                    <span className="text-muted-foreground">Rate Limit Hits</span>
                    <span className="font-semibold font-mono">{analytics.performance_metrics.rate_limit_hit_rate.toFixed(1)}%</span>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-muted-foreground">Overage Usage</span>
                    <span className="font-semibold font-mono">{analytics.performance_metrics.overage_usage_rate.toFixed(1)}%</span>
                  </div>
                </div>
              </div>

              {/* Top Endpoints */}
              <div className="p-6 rounded-3xl bg-card border border-border/50 shadow-sm">
                <h3 className="text-lg font-bold text-foreground mb-6">Top Endpoints</h3>
                <div className="space-y-3">
                  {analytics.usage_metrics.top_endpoints.slice(0, 5).map((endpoint, index) => (
                    <div key={index} className="flex items-center justify-between p-4 bg-muted/30 rounded-2xl border border-border/30">
                      <div className="flex-1">
                        <div className="font-semibold text-sm text-foreground">{endpoint.endpoint}</div>
                        <div className="text-xs text-muted-foreground">{endpoint.avg_response_time_ms}ms avg</div>
                      </div>
                      <div className="text-right">
                        <div className="font-bold text-foreground">{endpoint.request_count.toLocaleString()}</div>
                        <div className="text-xs text-muted-foreground">requests</div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Recommendations */}
              <div className="p-6 rounded-3xl bg-card border border-border/50 shadow-sm">
                <h3 className="text-lg font-bold text-foreground mb-6">Recommendations</h3>
                <div className="space-y-3">
                  {analytics.recommendations.slice(0, 4).map((rec, index) => (
                    <div key={index} className="p-4 bg-primary/5 rounded-2xl border border-primary/10 transition-all hover:bg-primary/10">
                      <div className="flex items-center gap-2 mb-2">
                        <span className={`px-2.5 py-1 text-[10px] font-bold uppercase tracking-wider rounded-lg ${rec.priority === 'high' ? 'bg-destructive/10 text-destructive' :
                          rec.priority === 'medium' ? 'bg-warning/10 text-warning' :
                            'bg-success/10 text-success'
                          }`}>
                          {rec.priority}
                        </span>
                        <span className="font-bold text-sm text-foreground">{rec.title}</span>
                      </div>
                      <p className="text-xs text-muted-foreground leading-relaxed">{rec.description}</p>
                    </div>
                  ))}
                </div>
              </div>
            </div>

            {/* Footer */}
            <div className="flex justify-end pt-4 border-t border-border/30">
              <button
                onClick={() => router.push('/plans')}
                className="px-8 py-3 bg-secondary text-secondary-foreground rounded-xl font-bold hover:opacity-90 transition-all shadow-lg shadow-secondary/10"
              >
                Back to Plans
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
