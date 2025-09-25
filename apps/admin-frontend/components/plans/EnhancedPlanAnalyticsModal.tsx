'use client'

import { useState, useEffect, useCallback, useRef } from 'react'
import { PancakeCard } from '@/components/ui/PancakeCard'
import { toast } from '@/hooks/use-toast'
import { PermissionErrorBoundary } from '@/components/error-boundaries/PermissionErrorBoundary'
import { PermissionErrorUI } from '@/components/ui/PermissionErrorUI'
import { enhancedPermissionAuthority } from '@/lib/api/enhanced-backend-permission-authority'
import { permissionErrorAnalytics } from '@/lib/analytics/permission-error-analytics'
import type { ApiError, PermissionValidationResult } from '@/types/api'

interface PlanAnalyticsData {
  plan_id: number
  plan_name: string
  analytics_period: string
  subscriber_metrics: {
    total_subscribers: number
    new_subscribers_this_period: number
    churned_subscribers_this_period: number
    subscriber_growth_rate: number
    subscriber_retention_rate: number
    avg_subscription_duration_days: number
  }
  revenue_metrics: {
    total_revenue: number
    recurring_revenue: number
    one_time_revenue: number
    revenue_growth_rate: number
    average_revenue_per_user: number
    revenue_per_subscriber: number
    refunds_total: number
    net_revenue: number
  }
  usage_metrics: {
    total_api_calls: number
    total_data_transfer_gb: number
    average_requests_per_subscriber: number
    resource_utilization_percentage: number
    top_endpoints: EndpointUsage[]
    peak_usage_hours: number[]
    geographic_distribution: GeographicUsage[]
  }
  performance_metrics: {
    plan_efficiency_score: number
    cost_per_request: number
    profit_margin: number
    rate_limit_hit_rate: number
    overage_usage_rate: number
    support_ticket_rate: number
    average_response_time_ms: number
    uptime_percentage: number
  }
  ai_insights: {
    optimization_opportunities: AIInsight[]
    performance_predictions: PerformancePrediction[]
    cost_optimization_suggestions: CostOptimization[]
    growth_forecasts: GrowthForecast[]
  }
  recommendations: AnalyticsRecommendation[]
  health_score: number
  risk_factors: RiskFactor[]
  competitive_analysis?: CompetitiveAnalysis
}

interface EndpointUsage {
  endpoint: string
  request_count: number
  avg_response_time_ms: number
  error_rate: number
  cost_impact: number
}

interface GeographicUsage {
  region: string
  subscriber_count: number
  revenue_contribution: number
  avg_latency_ms: number
}

interface AIInsight {
  id: string
  type: 'performance' | 'cost' | 'user_experience' | 'growth' | 'retention'
  title: string
  description: string
  confidence: number
  impact_score: number
  recommended_action: string
  potential_savings: number
  implementation_effort: 'low' | 'medium' | 'high'
}

interface PerformancePrediction {
  metric: string
  current_value: number
  predicted_30d: number
  predicted_90d: number
  confidence: number
  trend: 'increasing' | 'decreasing' | 'stable'
}

interface CostOptimization {
  area: string
  current_cost: number
  optimized_cost: number
  savings_potential: number
  implementation_steps: string[]
}

interface GrowthForecast {
  period: '30d' | '90d' | '1y'
  subscriber_forecast: number
  revenue_forecast: number
  confidence_interval: [number, number]
}

interface AnalyticsRecommendation {
  id: string
  type: 'pricing' | 'feature' | 'performance' | 'marketing' | 'retention'
  priority: 'low' | 'medium' | 'high' | 'critical'
  title: string
  description: string
  expected_impact: number
  implementation_cost: number
  roi_estimate: number
}

interface RiskFactor {
  type: 'churn' | 'performance' | 'cost' | 'compliance' | 'competition'
  severity: 'low' | 'medium' | 'high' | 'critical'
  description: string
  mitigation_strategy: string
}

interface CompetitiveAnalysis {
  market_position: string
  price_competitiveness: number
  feature_gap_analysis: string[]
  competitive_advantages: string[]
}

interface EnhancedPlanAnalyticsModalState {
  analytics: PlanAnalyticsData | null
  loading: boolean
  validating: boolean
  validationResult: PermissionValidationResult | null
  selectedPeriod: '7d' | '30d' | '90d' | '1y'
  selectedTab: 'overview' | 'usage' | 'performance' | 'insights' | 'recommendations'
  realtimeData: boolean
  exportInProgress: boolean
}

interface EnhancedPlanAnalyticsModalProps {
  planId: number
  currentUserId: string
  onClose: () => void
  component?: string
  enableAnalytics?: boolean
  enableRealtimeUpdates?: boolean
}

function EnhancedPlanAnalyticsModalCore({
  planId,
  currentUserId,
  onClose,
  component = 'EnhancedPlanAnalyticsModal',
  enableAnalytics = true,
  enableRealtimeUpdates = true
}: EnhancedPlanAnalyticsModalProps) {
  const [state, setState] = useState<EnhancedPlanAnalyticsModalState>({
    analytics: null,
    loading: true,
    validating: false,
    validationResult: null,
    selectedPeriod: '30d',
    selectedTab: 'overview',
    realtimeData: false,
    exportInProgress: false
  })

  const analytics = permissionErrorAnalytics
  const realtimeIntervalRef = useRef<NodeJS.Timeout>()
  const abortControllerRef = useRef<AbortController>()

  // Enhanced permission validation for analytics access
  const validateAnalyticsPermissions = useCallback(async () => {
    setState(prev => ({ ...prev, validating: true }))
    
    try {
      const permissionsToValidate = [
        'admin:analytics:view',
        'admin:plans:analytics',
        'admin:insights:view',
        'admin:performance:view'
      ]

      const results = await Promise.allSettled(
        permissionsToValidate.map(permission =>
          enhancedPermissionAuthority.validatePermission(
            currentUserId,
            permission,
            {
              component,
              context: { 
                action: 'view_plan_analytics',
                resource_type: 'plan_analytics',
                plan_id: planId,
                security_level: 'standard',
                requires_data_access: true
              }
            }
          )
        )
      )

      const validationResult: PermissionValidationResult = {
        hasPermission: results.every(result => 
          result.status === 'fulfilled' && result.value.success && result.value.data?.hasPermission
        ),
        permissions: permissionsToValidate.reduce((acc, permission, index) => {
          const result = results[index]
          acc[permission] = result.status === 'fulfilled' && result.value.success && result.value.data?.hasPermission || false
          return acc
        }, {} as Record<string, boolean>),
        context: {
          user_id: currentUserId,
          component,
          timestamp: Date.now(),
          validation_method: 'backend_authority'
        },
        metadata: {
          total_permissions: permissionsToValidate.length,
          granted_permissions: results.filter(r => 
            r.status === 'fulfilled' && r.value.success && r.value.data?.hasPermission
          ).length,
          security_level: 'standard',
          resource_id: planId.toString()
        }
      }

      setState(prev => ({ ...prev, validationResult, validating: false }))

      if (!validationResult.hasPermission) {
        const deniedPermissions = permissionsToValidate.filter(p => !validationResult.permissions[p])
        if (enableAnalytics) {
          analytics.trackPermissionDenied(
            currentUserId,
            deniedPermissions[0] || 'admin:analytics:view',
            { component, context: 'plan_analytics_access', plan_id: planId }
          )
        }
      }

      return validationResult

    } catch (error) {
      console.error('Analytics permission validation failed:', error)
      setState(prev => ({ ...prev, validating: false }))
      
      const apiError: ApiError = {
        success: false,
        error: {
          type: 'VALIDATION_ERROR',
          code: 'ANALYTICS_PERMISSION_CHECK_FAILED',
          message: 'Failed to validate analytics access permissions',
          user_message: 'Unable to verify your permissions for plan analytics. Please refresh and try again.'
        }
      }
      
      if (enableAnalytics) {
        analytics.trackError(apiError, { 
          component, 
          context: { user_id: currentUserId, plan_id: planId, action: 'analytics_validation' }
        })
      }
      
      throw apiError
    }
  }, [currentUserId, component, planId, enableAnalytics, analytics])

  // Load comprehensive analytics data
  const loadAnalyticsData = useCallback(async (period: string = state.selectedPeriod) => {
    abortControllerRef.current?.abort()
    abortControllerRef.current = new AbortController()
    
    try {
      setState(prev => ({ ...prev, loading: true }))

      // Validate permissions first
      const validationResult = await validateAnalyticsPermissions()
      if (!validationResult.hasPermission) {
        setState(prev => ({ ...prev, loading: false }))
        return
      }

      const response = await fetch(`/api/admin/plans/${planId}/analytics`, {
        method: 'POST',
        credentials: 'include',
        signal: abortControllerRef.current.signal,
        headers: {
          'Content-Type': 'application/json',
          'x-component': component
        },
        body: JSON.stringify({
          period,
          include_ai_insights: true,
          include_predictions: true,
          include_recommendations: true,
          include_competitive_analysis: true
        })
      })

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}))
        const apiError: ApiError = {
          success: false,
          error: {
            type: response.status === 403 ? 'PERMISSION_DENIED' : 'DATA_FETCH_ERROR',
            code: 'PLAN_ANALYTICS_FETCH_FAILED',
            message: errorData.message || 'Failed to load plan analytics',
            user_message: 'Unable to load analytics data. Please check your permissions and try again.'
          }
        }
        
        if (enableAnalytics) {
          analytics.trackError(apiError, { 
            component, 
            context: { user_id: currentUserId, plan_id: planId, http_status: response.status }
          })
        }
        throw apiError
      }

      const analyticsData = await response.json()

      setState(prev => ({
        ...prev,
        analytics: analyticsData,
        loading: false
      }))

      if (enableAnalytics) {
        analytics.trackEvent('plan_analytics_loaded', {
          component,
          user_id: currentUserId,
          plan_id: planId,
          period,
          health_score: analyticsData.health_score,
          recommendations_count: analyticsData.recommendations?.length || 0,
          ai_insights_count: analyticsData.ai_insights?.optimization_opportunities?.length || 0
        })
      }

    } catch (error) {
      console.error('Failed to load analytics data:', error)
      setState(prev => ({ ...prev, loading: false }))
      
      if (error.name !== 'AbortError') {
        toast({
          title: "Error",
          description: error.message || "Failed to load analytics data",
          variant: "destructive"
        })

        if (enableAnalytics && !(error as ApiError).error) {
          analytics.trackError(error as ApiError, { 
            component, 
            context: { user_id: currentUserId, plan_id: planId, action: 'load_analytics' }
          })
        }
      }
    }
  }, [planId, currentUserId, component, enableAnalytics, state.selectedPeriod, validateAnalyticsPermissions, analytics])

  // Export analytics data
  const exportAnalyticsData = useCallback(async (format: 'pdf' | 'csv' | 'xlsx') => {
    setState(prev => ({ ...prev, exportInProgress: true }))
    
    try {
      const response = await fetch(`/api/admin/plans/${planId}/analytics/export`, {
        method: 'POST',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
          'x-component': component
        },
        body: JSON.stringify({
          format,
          period: state.selectedPeriod,
          include_charts: true,
          include_recommendations: true
        })
      })

      if (!response.ok) {
        throw new Error('Export failed')
      }

      const blob = await response.blob()
      const url = window.URL.createObjectURL(blob)
      const link = document.createElement('a')
      link.href = url
      link.download = `plan_${planId}_analytics_${state.selectedPeriod}.${format}`
      document.body.appendChild(link)
      link.click()
      document.body.removeChild(link)
      window.URL.revokeObjectURL(url)

      toast({
        title: "Success",
        description: `Analytics report exported as ${format.toUpperCase()}`,
      })

      if (enableAnalytics) {
        analytics.trackEvent('analytics_exported', {
          component,
          user_id: currentUserId,
          plan_id: planId,
          format,
          period: state.selectedPeriod
        })
      }

    } catch (error) {
      console.error('Export failed:', error)
      toast({
        title: "Error",
        description: "Failed to export analytics data",
        variant: "destructive"
      })
    } finally {
      setState(prev => ({ ...prev, exportInProgress: false }))
    }
  }, [planId, component, state.selectedPeriod, currentUserId, enableAnalytics, analytics])

  // Real-time updates
  useEffect(() => {
    if (state.realtimeData && enableRealtimeUpdates) {
      realtimeIntervalRef.current = setInterval(() => {
        loadAnalyticsData()
      }, 30000) // Update every 30 seconds
    } else {
      if (realtimeIntervalRef.current) {
        clearInterval(realtimeIntervalRef.current)
      }
    }

    return () => {
      if (realtimeIntervalRef.current) {
        clearInterval(realtimeIntervalRef.current)
      }
    }
  }, [state.realtimeData, enableRealtimeUpdates, loadAnalyticsData])

  // Initialize
  useEffect(() => {
    loadAnalyticsData()

    return () => {
      abortControllerRef.current?.abort()
      if (realtimeIntervalRef.current) {
        clearInterval(realtimeIntervalRef.current)
      }
    }
  }, [loadAnalyticsData])

  // Period change handler
  useEffect(() => {
    if (state.selectedPeriod) {
      loadAnalyticsData(state.selectedPeriod)
    }
  }, [state.selectedPeriod, loadAnalyticsData])

  // Permission denial UI
  if (state.validationResult && !state.validationResult.hasPermission) {
    return (
      <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center p-4 z-50">
        <div className="bg-white dark:bg-gray-800 rounded-2xl p-8 max-w-2xl w-full">
          <PermissionErrorUI
            title="Analytics Access Required"
            message="You need analytics permissions to view plan performance data"
            missingPermissions={Object.keys(state.validationResult.permissions).filter(
              p => !state.validationResult!.permissions[p]
            )}
            requiredLevel="admin"
            onRetry={validateAnalyticsPermissions}
            showEscalation={true}
            context={{
              component,
              user_id: currentUserId,
              plan_id: planId,
              required_permissions: ['admin:analytics:view', 'admin:plans:analytics']
            }}
          />
        </div>
      </div>
    )
  }

  if (state.loading) {
    return (
      <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center p-4 z-50">
        <PancakeCard className="bg-white dark:bg-gray-800 max-w-7xl w-full max-h-[95vh] overflow-y-auto">
          <div className="p-8 space-y-8">
            <div className="flex items-center justify-between animate-pulse">
              <div className="h-8 bg-gray-300 rounded-xl w-1/3"></div>
              <div className="h-8 bg-gray-300 rounded-xl w-24"></div>
            </div>
            <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
              {Array.from({ length: 4 }).map((_, i) => (
                <div key={i} className="bg-gray-300 rounded-3xl h-40 animate-pulse"></div>
              ))}
            </div>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              {Array.from({ length: 6 }).map((_, i) => (
                <div key={i} className="bg-gray-300 rounded-3xl h-64 animate-pulse"></div>
              ))}
            </div>
          </div>
        </PancakeCard>
      </div>
    )
  }

  if (!state.analytics) {
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
            <div className="flex gap-4 justify-center">
              <button
                onClick={() => loadAnalyticsData()}
                className="px-6 py-3 bg-gradient-to-r from-purple-500 to-indigo-500 text-white rounded-xl font-semibold hover:from-purple-600 hover:to-indigo-600"
              >
                Retry
              </button>
              <button
                onClick={onClose}
                className="px-6 py-3 border border-gray-200 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-xl font-semibold hover:bg-gray-100 dark:hover:bg-gray-700"
              >
                Close
              </button>
            </div>
          </div>
        </PancakeCard>
      </div>
    )
  }

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center p-4 z-50">
      <PancakeCard className="bg-white dark:bg-gray-800 max-w-7xl w-full max-h-[95vh] overflow-y-auto">
        <div className="p-8">
          {/* Enhanced Header */}
          <div className="flex items-center justify-between mb-8">
            <div>
              <h2 className="text-3xl font-bold bg-gradient-to-r from-purple-600 via-blue-600 to-indigo-600 bg-clip-text text-transparent">
                📊 {state.analytics.plan_name} Analytics
              </h2>
              <div className="flex items-center gap-4 mt-2">
                <span className="text-gray-600 dark:text-gray-400">
                  Period: {state.analytics.analytics_period}
                </span>
                <div className="flex items-center gap-2">
                  <span className={`px-3 py-1 rounded-full text-sm font-semibold ${
                    state.analytics.health_score >= 80 ? 'bg-green-100 text-green-700 dark:bg-green-900/20 dark:text-green-300' :
                    state.analytics.health_score >= 60 ? 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/20 dark:text-yellow-300' :
                    'bg-red-100 text-red-700 dark:bg-red-900/20 dark:text-red-300'
                  }`}>
                    Health Score: {state.analytics.health_score}%
                  </span>
                  {enableRealtimeUpdates && (
                    <button
                      onClick={() => setState(prev => ({ ...prev, realtimeData: !prev.realtimeData }))}
                      className={`px-3 py-1 rounded-full text-sm font-semibold ${
                        state.realtimeData
                          ? 'bg-green-100 text-green-700 dark:bg-green-900/20 dark:text-green-300'
                          : 'bg-gray-100 text-gray-700 dark:bg-gray-900/20 dark:text-gray-300'
                      }`}
                    >
                      {state.realtimeData ? '🔴 Live' : '⚫ Static'}
                    </button>
                  )}
                </div>
              </div>
            </div>
            <div className="flex items-center gap-4">
              <select
                value={state.selectedPeriod}
                onChange={(e) => setState(prev => ({ 
                  ...prev, 
                  selectedPeriod: e.target.value as typeof state.selectedPeriod 
                }))}
                className="px-4 py-2 rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-purple-500"
              >
                <option value="7d">Last 7 days</option>
                <option value="30d">Last 30 days</option>
                <option value="90d">Last 90 days</option>
                <option value="1y">Last year</option>
              </select>
              
              <div className="relative">
                <button
                  onClick={() => {
                    const dropdown = document.getElementById('export-dropdown')
                    dropdown?.classList.toggle('hidden')
                  }}
                  disabled={state.exportInProgress}
                  className="px-4 py-2 rounded-xl bg-gradient-to-r from-green-500 to-emerald-500 text-white font-semibold hover:from-green-600 hover:to-emerald-600 disabled:opacity-50"
                >
                  {state.exportInProgress ? 'Exporting...' : 'Export'}
                </button>
                <div id="export-dropdown" className="hidden absolute right-0 mt-2 w-48 bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 z-10">
                  <button
                    onClick={() => exportAnalyticsData('pdf')}
                    className="w-full px-4 py-2 text-left hover:bg-gray-100 dark:hover:bg-gray-700 rounded-t-xl"
                  >
                    PDF Report
                  </button>
                  <button
                    onClick={() => exportAnalyticsData('xlsx')}
                    className="w-full px-4 py-2 text-left hover:bg-gray-100 dark:hover:bg-gray-700"
                  >
                    Excel Spreadsheet
                  </button>
                  <button
                    onClick={() => exportAnalyticsData('csv')}
                    className="w-full px-4 py-2 text-left hover:bg-gray-100 dark:hover:bg-gray-700 rounded-b-xl"
                  >
                    CSV Data
                  </button>
                </div>
              </div>

              <button
                onClick={onClose}
                className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 text-2xl font-bold"
              >
                ✕
              </button>
            </div>
          </div>

          {/* Tab Navigation */}
          <div className="flex gap-2 mb-8 p-1 bg-gray-100 dark:bg-gray-700 rounded-2xl">
            {[
              { id: 'overview', label: 'Overview', icon: '📊' },
              { id: 'usage', label: 'Usage', icon: '📈' },
              { id: 'performance', label: 'Performance', icon: '⚡' },
              { id: 'insights', label: 'AI Insights', icon: '🤖' },
              { id: 'recommendations', label: 'Recommendations', icon: '💡' }
            ].map(tab => (
              <button
                key={tab.id}
                onClick={() => setState(prev => ({ ...prev, selectedTab: tab.id as any }))}
                className={`flex-1 flex items-center justify-center gap-2 px-4 py-3 rounded-xl font-semibold text-sm ${
                  state.selectedTab === tab.id
                    ? 'bg-white dark:bg-gray-600 text-purple-600 dark:text-purple-400 shadow-sm'
                    : 'text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200'
                }`}
              >
                <span>{tab.icon}</span>
                <span>{tab.label}</span>
              </button>
            ))}
          </div>

          {/* Key Metrics Overview */}
          {state.selectedTab === 'overview' && (
            <div className="space-y-8">
              <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
                <PancakeCard className="bg-gradient-to-br from-blue-50 to-blue-100 dark:from-blue-900/20 dark:to-blue-800/20 border border-blue-200 dark:border-blue-800">
                  <div className="p-6">
                    <div className="flex items-center justify-between mb-4">
                      <span className="text-2xl">👥</span>
                      <span className="text-sm font-medium text-blue-600 dark:text-blue-400">Subscribers</span>
                    </div>
                    <div className="space-y-1">
                      <div className="text-3xl font-bold text-blue-900 dark:text-blue-100">
                        {state.analytics.subscriber_metrics.total_subscribers}
                      </div>
                      <div className="text-sm text-blue-600 dark:text-blue-400">
                        +{state.analytics.subscriber_metrics.new_subscribers_this_period} new
                      </div>
                      <div className="text-xs text-blue-500">
                        {(state.analytics.subscriber_metrics.subscriber_retention_rate * 100).toFixed(1)}% retention
                      </div>
                    </div>
                  </div>
                </PancakeCard>

                <PancakeCard className="bg-gradient-to-br from-green-50 to-green-100 dark:from-green-900/20 dark:to-green-800/20 border border-green-200 dark:border-green-800">
                  <div className="p-6">
                    <div className="flex items-center justify-between mb-4">
                      <span className="text-2xl">💰</span>
                      <span className="text-sm font-medium text-green-600 dark:text-green-400">Revenue</span>
                    </div>
                    <div className="space-y-1">
                      <div className="text-3xl font-bold text-green-900 dark:text-green-100">
                        ${state.analytics.revenue_metrics.total_revenue.toFixed(0)}
                      </div>
                      <div className="text-sm text-green-600 dark:text-green-400">
                        {state.analytics.revenue_metrics.revenue_growth_rate > 0 ? '+' : ''}
                        {(state.analytics.revenue_metrics.revenue_growth_rate * 100).toFixed(1)}% growth
                      </div>
                      <div className="text-xs text-green-500">
                        ${state.analytics.revenue_metrics.average_revenue_per_user.toFixed(2)} ARPU
                      </div>
                    </div>
                  </div>
                </PancakeCard>

                <PancakeCard className="bg-gradient-to-br from-purple-50 to-purple-100 dark:from-purple-900/20 dark:to-purple-800/20 border border-purple-200 dark:border-purple-800">
                  <div className="p-6">
                    <div className="flex items-center justify-between mb-4">
                      <span className="text-2xl">📊</span>
                      <span className="text-sm font-medium text-purple-600 dark:text-purple-400">Usage</span>
                    </div>
                    <div className="space-y-1">
                      <div className="text-3xl font-bold text-purple-900 dark:text-purple-100">
                        {(state.analytics.usage_metrics.total_api_calls / 1000).toFixed(0)}K
                      </div>
                      <div className="text-sm text-purple-600 dark:text-purple-400">API calls</div>
                      <div className="text-xs text-purple-500">
                        {state.analytics.usage_metrics.total_data_transfer_gb.toFixed(1)} GB transferred
                      </div>
                    </div>
                  </div>
                </PancakeCard>

                <PancakeCard className="bg-gradient-to-br from-indigo-50 to-indigo-100 dark:from-indigo-900/20 dark:to-indigo-800/20 border border-indigo-200 dark:border-indigo-800">
                  <div className="p-6">
                    <div className="flex items-center justify-between mb-4">
                      <span className="text-2xl">⚡</span>
                      <span className="text-sm font-medium text-indigo-600 dark:text-indigo-400">Performance</span>
                    </div>
                    <div className="space-y-1">
                      <div className="text-3xl font-bold text-indigo-900 dark:text-indigo-100">
                        {state.analytics.performance_metrics.plan_efficiency_score.toFixed(0)}%
                      </div>
                      <div className="text-sm text-indigo-600 dark:text-indigo-400">Efficiency</div>
                      <div className="text-xs text-indigo-500">
                        {state.analytics.performance_metrics.average_response_time_ms}ms avg response
                      </div>
                    </div>
                  </div>
                </PancakeCard>
              </div>

              {/* Risk Factors */}
              {state.analytics.risk_factors.length > 0 && (
                <div className="bg-gradient-to-r from-red-50 via-orange-50 to-yellow-50 dark:from-red-900/20 dark:via-orange-900/20 dark:to-yellow-900/20 rounded-2xl p-6 border border-red-200 dark:border-red-700">
                  <h3 className="text-xl font-bold text-red-800 dark:text-red-300 mb-4 flex items-center gap-2">
                    ⚠️ Risk Factors
                  </h3>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    {state.analytics.risk_factors.map((risk, index) => (
                      <div key={index} className="bg-white/70 dark:bg-gray-800/70 rounded-xl p-4">
                        <div className="flex items-center justify-between mb-2">
                          <span className="font-semibold text-gray-800 dark:text-gray-200 capitalize">
                            {risk.type.replace('_', ' ')}
                          </span>
                          <span className={`px-2 py-1 rounded-full text-xs font-bold ${
                            risk.severity === 'critical' ? 'bg-red-100 text-red-700 dark:bg-red-900/20 dark:text-red-300' :
                            risk.severity === 'high' ? 'bg-orange-100 text-orange-700 dark:bg-orange-900/20 dark:text-orange-300' :
                            risk.severity === 'medium' ? 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/20 dark:text-yellow-300' :
                            'bg-blue-100 text-blue-700 dark:bg-blue-900/20 dark:text-blue-300'
                          }`}>
                            {risk.severity.toUpperCase()}
                          </span>
                        </div>
                        <p className="text-sm text-gray-600 dark:text-gray-400 mb-2">{risk.description}</p>
                        <p className="text-xs text-gray-500 dark:text-gray-500"><strong>Mitigation:</strong> {risk.mitigation_strategy}</p>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}

          {/* AI Insights Tab */}
          {state.selectedTab === 'insights' && state.analytics.ai_insights && (
            <div className="space-y-8">
              {/* Optimization Opportunities */}
              <div>
                <h3 className="text-2xl font-bold text-gray-900 dark:text-white mb-6 flex items-center gap-2">
                  🎯 Optimization Opportunities
                </h3>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  {state.analytics.ai_insights.optimization_opportunities.map((insight, index) => (
                    <PancakeCard key={index} className="bg-gradient-to-br from-amber-50 to-yellow-100 dark:from-amber-900/20 dark:to-yellow-800/20 border border-amber-200 dark:border-amber-700">
                      <div className="p-6">
                        <div className="flex items-center justify-between mb-4">
                          <h4 className="font-bold text-amber-800 dark:text-amber-300">{insight.title}</h4>
                          <div className="flex items-center gap-2">
                            <span className="text-xs bg-blue-100 text-blue-700 dark:bg-blue-900/20 dark:text-blue-300 px-2 py-1 rounded-full">
                              {Math.round(insight.confidence * 100)}% confident
                            </span>
                            <span className="text-xs bg-green-100 text-green-700 dark:bg-green-900/20 dark:text-green-300 px-2 py-1 rounded-full">
                              Impact: {insight.impact_score}/10
                            </span>
                          </div>
                        </div>
                        <p className="text-sm text-gray-700 dark:text-gray-300 mb-3">{insight.description}</p>
                        <p className="text-sm font-medium text-amber-800 dark:text-amber-300 mb-2">
                          Recommended Action:
                        </p>
                        <p className="text-sm text-gray-600 dark:text-gray-400 mb-3">{insight.recommended_action}</p>
                        {insight.potential_savings > 0 && (
                          <div className="flex justify-between text-sm">
                            <span className="text-green-600 dark:text-green-400 font-medium">
                              Potential Savings: ${insight.potential_savings.toFixed(2)}
                            </span>
                            <span className="text-gray-500">
                              Effort: {insight.implementation_effort}
                            </span>
                          </div>
                        )}
                      </div>
                    </PancakeCard>
                  ))}
                </div>
              </div>

              {/* Performance Predictions */}
              <div>
                <h3 className="text-2xl font-bold text-gray-900 dark:text-white mb-6 flex items-center gap-2">
                  📈 Performance Predictions
                </h3>
                <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                  {state.analytics.ai_insights.performance_predictions.map((prediction, index) => (
                    <PancakeCard key={index} className="bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm border border-gray-200 dark:border-gray-700">
                      <div className="p-6">
                        <h4 className="font-bold text-gray-900 dark:text-white mb-4 capitalize">
                          {prediction.metric.replace('_', ' ')}
                        </h4>
                        <div className="space-y-3">
                          <div className="flex justify-between">
                            <span className="text-sm text-gray-600 dark:text-gray-400">Current</span>
                            <span className="font-semibold">{prediction.current_value}</span>
                          </div>
                          <div className="flex justify-between">
                            <span className="text-sm text-gray-600 dark:text-gray-400">30d Prediction</span>
                            <span className={`font-semibold ${
                              prediction.trend === 'increasing' ? 'text-green-600' :
                              prediction.trend === 'decreasing' ? 'text-red-600' : 'text-gray-600'
                            }`}>
                              {prediction.predicted_30d}
                              {prediction.trend === 'increasing' ? ' ↗️' :
                               prediction.trend === 'decreasing' ? ' ↘️' : ' ➡️'}
                            </span>
                          </div>
                          <div className="flex justify-between">
                            <span className="text-sm text-gray-600 dark:text-gray-400">90d Prediction</span>
                            <span className="font-semibold">{prediction.predicted_90d}</span>
                          </div>
                          <div className="text-xs text-center text-gray-500 pt-2 border-t">
                            {Math.round(prediction.confidence * 100)}% confidence
                          </div>
                        </div>
                      </div>
                    </PancakeCard>
                  ))}
                </div>
              </div>
            </div>
          )}

          {/* Close Button */}
          <div className="flex justify-end pt-8 border-t border-gray-200 dark:border-gray-700">
            <button
              onClick={onClose}
              className="px-6 py-3 bg-gradient-to-r from-purple-500 to-indigo-500 text-white rounded-xl font-semibold hover:from-purple-600 hover:to-indigo-600"
            >
              Close Analytics
            </button>
          </div>
        </div>
      </PancakeCard>
    </div>
  )
}

export function EnhancedPlanAnalyticsModal(props: EnhancedPlanAnalyticsModalProps) {
  return (
    <PermissionErrorBoundary>
      <EnhancedPlanAnalyticsModalCore {...props} />
    </PermissionErrorBoundary>
  )
}

export default EnhancedPlanAnalyticsModal