'use client'

import { useState, useEffect, useCallback, useRef } from 'react'
import { PancakeCard } from '@/components/ui/PancakeCard'
import { toast } from '@/hooks/use-toast'
import { PermissionErrorBoundary } from '@/components/error-boundaries/PermissionErrorBoundary'
import { PermissionErrorUI } from '@/components/ui/PermissionErrorUI'
import { enhancedPermissionAuthority } from '@/lib/api/enhanced-backend-permission-authority'
import { permissionErrorAnalytics } from '@/lib/analytics/permission-error-analytics'
import { EnhancedCreatePlanForm } from './EnhancedCreatePlanForm'
import { PlanAnalyticsModal } from './PlanAnalyticsModal'
import type { ApiError, PermissionValidationResult } from '@/types/api'

interface PlanData {
  id: number
  name: string
  description: string
  plan_category: 'standard' | 'api' | 'enterprise' | 'custom'
  current_price: number
  currency: string
  target_audience: string
  billing_model: string
  is_active: boolean
  subscriber_count: number
  revenue_last_30_days: number
  features: any[]
  permissions?: string[]
  created_at: string
  updated_at: string
  health_score?: number
  optimization_score?: number
  recommendation_count?: number
}

interface PlanRecommendation {
  id: string
  type: 'pricing' | 'feature' | 'permission' | 'audience' | 'performance'
  priority: 'low' | 'medium' | 'high' | 'critical'
  title: string
  description: string
  confidence: number
  impact_score: number
  implementation_effort: 'low' | 'medium' | 'high'
  revenue_impact?: number
  user_impact?: number
  technical_requirements?: string[]
  business_value: number
  recommended_action: string
}

interface PlanAnalytics {
  total_plans: number
  active_plans: number
  total_revenue: number
  avg_revenue_per_plan: number
  subscriber_growth_rate: number
  churn_rate: number
  most_popular_plan: string
  highest_revenue_plan: string
  plan_utilization_rate: number
  recommendation_count: number
}

interface EnhancedUnifiedPlanManagementState {
  plans: PlanData[]
  loading: boolean
  validating: boolean
  analytics: PlanAnalytics | null
  recommendations: PlanRecommendation[]
  validationResult: PermissionValidationResult | null
  filterCategory: 'all' | 'standard' | 'api' | 'enterprise' | 'custom'
  filterActive: boolean | null
  selectedPlan: PlanData | null
  showAnalytics: number | null
  isCreating: boolean
  searchQuery: string
  sortField: 'name' | 'price' | 'subscribers' | 'revenue' | 'created_at'
  sortDirection: 'asc' | 'desc'
}

interface EnhancedUnifiedPlanManagementProps {
  currentUserId: string
  initialUser?: any
  component?: string
  enableAnalytics?: boolean
  enableRecommendations?: boolean
}

function EnhancedUnifiedPlanManagementCore({
  currentUserId,
  initialUser,
  component = 'EnhancedUnifiedPlanManagement',
  enableAnalytics = true,
  enableRecommendations = true,
}: EnhancedUnifiedPlanManagementProps) {
  const [state, setState] = useState<EnhancedUnifiedPlanManagementState>({
    plans: [],
    loading: true,
    validating: false,
    analytics: null,
    recommendations: [],
    validationResult: null,
    filterCategory: 'all',
    filterActive: null,
    selectedPlan: null,
    showAnalytics: null,
    isCreating: false,
    searchQuery: '',
    sortField: 'created_at',
    sortDirection: 'desc'
  })

  const analytics = permissionErrorAnalytics
  const abortControllerRef = useRef<AbortController>()

  // Enhanced permission validation with comprehensive context
  const validatePlanPermissions = useCallback(async () => {
    setState(prev => ({ ...prev, validating: true }))
    
    try {
      const permissionsToValidate = [
        'admin:plans:view',
        'admin:plans:manage',
        'admin:analytics:view',
        'admin:recommendations:view'
      ]

      const results = await Promise.allSettled(
        permissionsToValidate.map(permission =>
          enhancedPermissionAuthority.validatePermission(
            currentUserId,
            permission,
            {
              component,
              context: { 
                action: 'plan_management',
                resource_type: 'admin_plans',
                security_level: 'elevated',
                requires_elevated_privileges: true
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
          security_level: 'elevated'
        }
      }

      setState(prev => ({ ...prev, validationResult, validating: false }))

      if (!validationResult.hasPermission) {
        const deniedPermissions = permissionsToValidate.filter(p => !validationResult.permissions[p])
        analytics.trackPermissionDenied(
          currentUserId,
          deniedPermissions[0] || 'admin:plans:view',
          { component, context: 'plan_management_validation' }
        )
      }

      return validationResult

    } catch (error) {
      console.error('Permission validation failed:', error)
      setState(prev => ({ ...prev, validating: false }))
      
      const apiError: ApiError = {
        success: false,
        error: {
          type: 'VALIDATION_ERROR',
          code: 'PLAN_PERMISSION_CHECK_FAILED',
          message: 'Failed to validate plan management permissions',
          user_message: 'Unable to verify your permissions. Please refresh and try again.'
        }
      }
      
      if (enableAnalytics) {
        analytics.trackError(apiError, { 
          component, 
          context: { user_id: currentUserId, action: 'permission_validation' }
        })
      }
      
      throw apiError
    }
  }, [currentUserId, component, enableAnalytics, analytics])

  // Load plan data with comprehensive error handling
  const loadPlanData = useCallback(async () => {
    abortControllerRef.current?.abort()
    abortControllerRef.current = new AbortController()
    
    try {
      setState(prev => ({ ...prev, loading: true }))

      // Validate permissions first
      const validationResult = await validatePlanPermissions()
      if (!validationResult.hasPermission) {
        setState(prev => ({ ...prev, loading: false }))
        return
      }

      const [plansResponse, analyticsResponse] = await Promise.all([
        fetch('/api/admin/plans', {
          credentials: 'include',
          signal: abortControllerRef.current.signal,
          headers: {
            'Content-Type': 'application/json',
            'x-component': component
          }
        }),
        enableAnalytics ? fetch('/api/admin/analytics/plans', {
          credentials: 'include',
          signal: abortControllerRef.current.signal,
          headers: {
            'Content-Type': 'application/json',
            'x-component': component
          }
        }) : Promise.resolve({ ok: false })
      ])

      if (!plansResponse.ok) {
        const errorData = await plansResponse.json().catch(() => ({}))
        const apiError: ApiError = {
          success: false,
          error: {
            type: plansResponse.status === 403 ? 'PERMISSION_DENIED' : 'NETWORK_ERROR',
            code: 'PLAN_DATA_FETCH_FAILED',
            message: errorData.message || 'Failed to fetch plan data',
            user_message: 'Unable to load plan data. Please check your permissions and try again.'
          }
        }
        
        if (enableAnalytics) {
          analytics.trackError(apiError, { 
            component, 
            context: { user_id: currentUserId, http_status: plansResponse.status }
          })
        }
        throw apiError
      }

      const plansData = await plansResponse.json()
      let analyticsData = null

      if (analyticsResponse.ok) {
        analyticsData = await analyticsResponse.json()
      }

      // Generate AI-powered recommendations if enabled
      const recommendations = enableRecommendations ? 
        await generatePlanRecommendations(plansData.plans || []) : []

      setState(prev => ({
        ...prev,
        plans: plansData.plans || [],
        analytics: analyticsData?.analytics || null,
        recommendations,
        loading: false
      }))

      if (enableAnalytics) {
        analytics.trackEvent('plan_data_loaded', {
          component,
          user_id: currentUserId,
          plan_count: plansData.plans?.length || 0,
          has_analytics: !!analyticsData,
          recommendation_count: recommendations.length
        })
      }

    } catch (error) {
      console.error('Failed to load plan data:', error)
      setState(prev => ({ ...prev, loading: false }))
      
      if (error.name !== 'AbortError') {
        if (enableAnalytics) {
          analytics.trackError(error as ApiError, { 
            component, 
            context: { user_id: currentUserId, action: 'load_plan_data' }
          })
        }
        throw error
      }
    }
  }, [currentUserId, component, enableAnalytics, enableRecommendations, validatePlanPermissions, analytics])

  // AI-powered plan recommendation engine
  const generatePlanRecommendations = useCallback(async (plans: PlanData[]): Promise<PlanRecommendation[]> => {
    try {
      const recommendations: PlanRecommendation[] = []
      
      // Performance-based recommendations
      plans.forEach(plan => {
        const revenuePerSubscriber = plan.subscriber_count > 0 ? 
          plan.revenue_last_30_days / plan.subscriber_count : 0
        
        if (revenuePerSubscriber < 10 && plan.current_price > 20) {
          recommendations.push({
            id: `pricing_opt_${plan.id}_${Date.now()}`,
            type: 'pricing',
            priority: 'medium',
            title: `Optimize ${plan.name} Pricing`,
            description: 'Low revenue per subscriber suggests pricing may be too high for current value proposition',
            confidence: 0.75,
            impact_score: 7,
            implementation_effort: 'low',
            revenue_impact: plan.current_price * 0.2,
            user_impact: plan.subscriber_count * 1.5,
            business_value: 8,
            recommended_action: 'Consider reducing price by 15-20% or adding more value features'
          })
        }

        if (plan.subscriber_count > 100 && !plan.features.includes('premium_support')) {
          recommendations.push({
            id: `feature_add_${plan.id}_${Date.now()}`,
            type: 'feature',
            priority: 'medium',
            title: `Add Premium Support to ${plan.name}`,
            description: 'High subscriber count indicates opportunity for premium support upselling',
            confidence: 0.85,
            impact_score: 8,
            implementation_effort: 'medium',
            business_value: 9,
            recommended_action: 'Add premium support feature and increase price by $5-10'
          })
        }
      })

      // Portfolio-level recommendations
      const totalRevenue = plans.reduce((sum, p) => sum + p.revenue_last_30_days, 0)
      const enterprisePlans = plans.filter(p => p.plan_category === 'enterprise')
      
      if (enterprisePlans.length === 0 && totalRevenue > 10000) {
        recommendations.push({
          id: `enterprise_plan_${Date.now()}`,
          type: 'audience',
          priority: 'high',
          title: 'Create Enterprise Plan',
          description: 'High revenue indicates market readiness for enterprise offerings',
          confidence: 0.9,
          impact_score: 9,
          implementation_effort: 'high',
          revenue_impact: totalRevenue * 0.3,
          business_value: 10,
          recommended_action: 'Develop enterprise plan with advanced features and dedicated support'
        })
      }

      // Security and compliance recommendations
      plans.forEach(plan => {
        if (plan.plan_category === 'enterprise' && !plan.permissions?.some(p => p.includes('audit'))) {
          recommendations.push({
            id: `compliance_${plan.id}_${Date.now()}`,
            type: 'permission',
            priority: 'high',
            title: `Add Compliance Features to ${plan.name}`,
            description: 'Enterprise plans should include audit and compliance capabilities',
            confidence: 0.95,
            impact_score: 8,
            implementation_effort: 'medium',
            technical_requirements: ['audit log access', 'compliance reporting', 'data retention controls'],
            business_value: 9,
            recommended_action: 'Add audit and compliance permissions to meet enterprise requirements'
          })
        }
      })

      return recommendations.sort((a, b) => {
        if (a.priority !== b.priority) {
          const priorityOrder = { critical: 4, high: 3, medium: 2, low: 1 }
          return priorityOrder[b.priority] - priorityOrder[a.priority]
        }
        return b.business_value - a.business_value
      }).slice(0, 10) // Top 10 recommendations

    } catch (error) {
      console.error('Failed to generate plan recommendations:', error)
      return []
    }
  }, [])

  // Enhanced plan status toggle with optimistic updates
  const handleTogglePlanStatus = useCallback(async (planId: number, currentStatus: boolean) => {
    try {
      // Optimistic update
      setState(prev => ({
        ...prev,
        plans: prev.plans.map(plan => 
          plan.id === planId ? { ...plan, is_active: !currentStatus } : plan
        )
      }))

      const response = await fetch(`/api/admin/plans/${planId}`, {
        method: 'PATCH',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
          'x-component': component
        },
        body: JSON.stringify({ is_active: !currentStatus })
      })

      if (!response.ok) {
        // Revert optimistic update on failure
        setState(prev => ({
          ...prev,
          plans: prev.plans.map(plan => 
            plan.id === planId ? { ...plan, is_active: currentStatus } : plan
          )
        }))

        const errorData = await response.json().catch(() => ({}))
        throw new Error(errorData.message || 'Failed to update plan status')
      }

      toast({
        title: "Success",
        description: `Plan ${!currentStatus ? 'activated' : 'deactivated'} successfully`,
      })

      if (enableAnalytics) {
        analytics.trackEvent('plan_status_changed', {
          component,
          user_id: currentUserId,
          plan_id: planId,
          new_status: !currentStatus
        })
      }

    } catch (error) {
      console.error('Failed to toggle plan status:', error)
      toast({
        title: "Error",
        description: error.message || "Failed to update plan status",
        variant: "destructive"
      })

      if (enableAnalytics) {
        analytics.trackError(error as ApiError, { 
          component, 
          context: { user_id: currentUserId, plan_id: planId, action: 'toggle_status' }
        })
      }
    }
  }, [component, currentUserId, enableAnalytics, analytics])

  // Initialize component
  useEffect(() => {
    loadPlanData()

    return () => {
      abortControllerRef.current?.abort()
    }
  }, [loadPlanData])

  // Filter and sort plans
  const filteredAndSortedPlans = state.plans
    .filter(plan => {
      const categoryMatch = state.filterCategory === 'all' || plan.plan_category === state.filterCategory
      const activeMatch = state.filterActive === null || plan.is_active === state.filterActive
      const searchMatch = !state.searchQuery || 
        plan.name.toLowerCase().includes(state.searchQuery.toLowerCase()) ||
        plan.description?.toLowerCase().includes(state.searchQuery.toLowerCase())
      
      return categoryMatch && activeMatch && searchMatch
    })
    .sort((a, b) => {
      const aValue = a[state.sortField]
      const bValue = b[state.sortField]
      const direction = state.sortDirection === 'asc' ? 1 : -1
      
      if (typeof aValue === 'string' && typeof bValue === 'string') {
        return aValue.localeCompare(bValue) * direction
      }
      
      return ((aValue as number) - (bValue as number)) * direction
    })

  // Permission denial UI
  if (state.validationResult && !state.validationResult.hasPermission) {
    return (
      <PermissionErrorUI
        title="Plan Management Access Required"
        message="You need elevated admin permissions to manage plans"
        missingPermissions={Object.keys(state.validationResult.permissions).filter(
          p => !state.validationResult!.permissions[p]
        )}
        requiredLevel="admin"
        onRetry={validatePlanPermissions}
        showEscalation={true}
        context={{
          component,
          user_id: currentUserId,
          required_permissions: ['admin:plans:view', 'admin:plans:manage']
        }}
      />
    )
  }

  if (state.loading) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-purple-50 via-blue-50 to-indigo-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
        <div className="max-w-7xl mx-auto space-y-8">
          <div className="text-center">
            <div className="h-16 bg-gradient-to-r from-purple-400 to-indigo-500 rounded-2xl w-96 mx-auto mb-6 animate-pulse"></div>
            <div className="h-6 bg-gray-300 rounded-full w-64 mx-auto animate-pulse"></div>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
            {Array.from({ length: 4 }).map((_, i) => (
              <div key={i} className="bg-gray-300 rounded-3xl h-32 animate-pulse"></div>
            ))}
          </div>
          <div className="bg-gray-300 rounded-3xl h-96 animate-pulse"></div>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-purple-50 via-blue-50 to-indigo-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900">
      {/* Enhanced Create Plan Form Modal */}
      {state.isCreating && (
        <EnhancedCreatePlanForm
          currentUserId={currentUserId}
          onClose={() => setState(prev => ({ ...prev, isCreating: false }))}
          onSuccess={() => {
            setState(prev => ({ ...prev, isCreating: false }))
            loadPlanData()
          }}
          component={component}
          enableAnalytics={enableAnalytics}
        />
      )}

      {/* Plan Analytics Modal */}
      {state.showAnalytics && (
        <PlanAnalyticsModal
          planId={state.showAnalytics}
          onClose={() => setState(prev => ({ ...prev, showAnalytics: null }))}
        />
      )}

      <div className="p-6">
        <div className="max-w-7xl mx-auto space-y-8">
          {/* Enhanced Header */}
          <div className="text-center">
            <h1 className="text-4xl lg:text-5xl font-bold bg-gradient-to-r from-purple-600 via-blue-600 to-indigo-600 bg-clip-text text-transparent mb-4">
              🎯 Unified Plan Management
            </h1>
            <p className="text-lg text-gray-600 dark:text-gray-400 max-w-3xl mx-auto">
              AI-powered plan optimization with real-time analytics and intelligent recommendations
            </p>
          </div>

          {/* AI Recommendations Panel */}
          {enableRecommendations && state.recommendations.length > 0 && (
            <div className="bg-gradient-to-r from-amber-100 via-yellow-100 to-orange-100 dark:from-amber-900/20 dark:via-yellow-900/20 dark:to-orange-900/20 rounded-2xl p-6 border border-amber-200 dark:border-amber-700">
              <div className="flex items-center gap-3 mb-4">
                <span className="text-2xl">🎯</span>
                <h3 className="text-xl font-bold text-amber-800 dark:text-amber-300">
                  AI Recommendations ({state.recommendations.length})
                </h3>
              </div>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {state.recommendations.slice(0, 6).map(rec => (
                  <div key={rec.id} className="bg-white/70 dark:bg-gray-800/70 rounded-xl p-4 border border-amber-200/50 dark:border-amber-700/50">
                    <div className="flex items-center justify-between mb-2">
                      <span className={`px-2 py-1 rounded-full text-xs font-bold ${
                        rec.priority === 'critical' ? 'bg-red-100 text-red-700 dark:bg-red-900/20 dark:text-red-300' :
                        rec.priority === 'high' ? 'bg-orange-100 text-orange-700 dark:bg-orange-900/20 dark:text-orange-300' :
                        'bg-blue-100 text-blue-700 dark:bg-blue-900/20 dark:text-blue-300'
                      }`}>
                        {rec.priority.toUpperCase()}
                      </span>
                      <span className="text-xs text-gray-500">
                        {Math.round(rec.confidence * 100)}% confident
                      </span>
                    </div>
                    <h4 className="font-semibold text-gray-900 dark:text-white mb-1 text-sm">
                      {rec.title}
                    </h4>
                    <p className="text-xs text-gray-600 dark:text-gray-400 mb-2">
                      {rec.description}
                    </p>
                    <div className="text-xs text-green-600 dark:text-green-400 font-medium">
                      Impact: {rec.impact_score}/10 | Value: {rec.business_value}/10
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Enhanced Analytics Dashboard */}
          {state.analytics && (
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
              <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl p-6 shadow-xl border border-purple-200 dark:border-purple-700">
                <div className="flex items-center justify-between mb-4">
                  <span className="text-2xl">📊</span>
                  <span className="text-sm font-medium text-purple-600 dark:text-purple-400">Total</span>
                </div>
                <div className="space-y-1">
                  <div className="text-3xl font-bold text-purple-600 dark:text-purple-400">
                    {state.analytics.total_plans}
                  </div>
                  <div className="text-sm text-gray-600 dark:text-gray-300">Plans</div>
                  <div className="text-xs text-gray-500">Active: {state.analytics.active_plans}</div>
                </div>
              </div>

              <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl p-6 shadow-xl border border-green-200 dark:border-green-700">
                <div className="flex items-center justify-between mb-4">
                  <span className="text-2xl">💰</span>
                  <span className="text-sm font-medium text-green-600 dark:text-green-400">Revenue</span>
                </div>
                <div className="space-y-1">
                  <div className="text-3xl font-bold text-green-600 dark:text-green-400">
                    ${state.analytics.total_revenue.toFixed(0)}
                  </div>
                  <div className="text-sm text-gray-600 dark:text-gray-300">Total</div>
                  <div className="text-xs text-gray-500">
                    Avg: ${state.analytics.avg_revenue_per_plan.toFixed(0)}
                  </div>
                </div>
              </div>

              <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl p-6 shadow-xl border border-blue-200 dark:border-blue-700">
                <div className="flex items-center justify-between mb-4">
                  <span className="text-2xl">📈</span>
                  <span className="text-sm font-medium text-blue-600 dark:text-blue-400">Growth</span>
                </div>
                <div className="space-y-1">
                  <div className="text-3xl font-bold text-blue-600 dark:text-blue-400">
                    +{(state.analytics.subscriber_growth_rate * 100).toFixed(1)}%
                  </div>
                  <div className="text-sm text-gray-600 dark:text-gray-300">Subscribers</div>
                  <div className="text-xs text-gray-500">
                    Churn: {(state.analytics.churn_rate * 100).toFixed(1)}%
                  </div>
                </div>
              </div>

              <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl p-6 shadow-xl border border-indigo-200 dark:border-indigo-700">
                <div className="flex items-center justify-between mb-4">
                  <span className="text-2xl">🎯</span>
                  <span className="text-sm font-medium text-indigo-600 dark:text-indigo-400">AI Insights</span>
                </div>
                <div className="space-y-1">
                  <div className="text-3xl font-bold text-indigo-600 dark:text-indigo-400">
                    {state.recommendations.length}
                  </div>
                  <div className="text-sm text-gray-600 dark:text-gray-300">Recommendations</div>
                  <div className="text-xs text-gray-500">
                    Utilization: {(state.analytics.plan_utilization_rate * 100).toFixed(0)}%
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Enhanced Control Panel */}
          <div className="bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-2xl p-6 shadow-xl border border-gray-200 dark:border-gray-700">
            <div className="flex flex-col lg:flex-row gap-4 mb-6">
              <div className="flex-1">
                <input
                  type="text"
                  placeholder="Search plans..."
                  value={state.searchQuery}
                  onChange={(e) => setState(prev => ({ ...prev, searchQuery: e.target.value }))}
                  className="w-full px-4 py-3 rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-purple-500"
                />
              </div>
              <div className="flex gap-4">
                <select
                  value={state.filterCategory}
                  onChange={(e) => setState(prev => ({ 
                    ...prev, 
                    filterCategory: e.target.value as typeof state.filterCategory 
                  }))}
                  className="px-4 py-3 rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-purple-500"
                >
                  <option value="all">All Categories</option>
                  <option value="standard">Standard</option>
                  <option value="api">API Plans</option>
                  <option value="enterprise">Enterprise</option>
                  <option value="custom">Custom</option>
                </select>
                <button
                  onClick={() => setState(prev => ({ ...prev, isCreating: true }))}
                  className="px-6 py-3 rounded-xl bg-gradient-to-r from-purple-500 to-indigo-500 text-white font-semibold hover:from-purple-600 hover:to-indigo-600"
                >
                  Create Plan
                </button>
                <button
                  onClick={loadPlanData}
                  className="px-6 py-3 rounded-xl border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 font-semibold hover:bg-gray-100 dark:hover:bg-gray-700"
                >
                  Refresh
                </button>
              </div>
            </div>

            {/* Enhanced Plans List */}
            <div className="space-y-4">
              {filteredAndSortedPlans.map(plan => (
                <div
                  key={plan.id}
                  className="flex items-center justify-between p-6 bg-gradient-to-r from-gray-50 to-gray-100 dark:from-gray-700 dark:to-gray-800 rounded-2xl hover:from-purple-50 hover:to-indigo-50 dark:hover:from-gray-600 dark:hover:to-gray-700 cursor-pointer"
                  onClick={() => setState(prev => ({ ...prev, selectedPlan: plan }))}
                >
                  <div className="flex items-center gap-6 flex-1">
                    <div className={`h-12 w-12 rounded-2xl flex items-center justify-center text-2xl ${
                      plan.plan_category === 'standard' 
                        ? 'bg-gradient-to-br from-blue-400 to-purple-500'
                        : plan.plan_category === 'api'
                        ? 'bg-gradient-to-br from-green-400 to-teal-500'
                        : plan.plan_category === 'enterprise'
                        ? 'bg-gradient-to-br from-purple-400 to-pink-500'
                        : 'bg-gradient-to-br from-gray-400 to-gray-500'
                    }`}>
                      {plan.plan_category === 'standard' ? '👤' : 
                       plan.plan_category === 'api' ? '🔧' : 
                       plan.plan_category === 'enterprise' ? '🏢' : '⚙️'}
                    </div>
                    
                    <div className="flex-1">
                      <div className="flex items-center gap-3 mb-2">
                        <h3 className="text-lg font-bold text-gray-900 dark:text-white">
                          {plan.name}
                        </h3>
                        {!plan.is_active && (
                          <span className="bg-gray-400 text-white text-xs px-3 py-1 rounded-full font-semibold">
                            INACTIVE
                          </span>
                        )}
                        {plan.health_score && (
                          <span className={`text-xs px-2 py-1 rounded-full font-semibold ${
                            plan.health_score >= 80 ? 'bg-green-100 text-green-700 dark:bg-green-900/20 dark:text-green-300' :
                            plan.health_score >= 60 ? 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/20 dark:text-yellow-300' :
                            'bg-red-100 text-red-700 dark:bg-red-900/20 dark:text-red-300'
                          }`}>
                            Health: {plan.health_score}%
                          </span>
                        )}
                      </div>
                      <div className="flex items-center gap-4 text-sm text-gray-600 dark:text-gray-400">
                        <span className="font-semibold">
                          ${plan.current_price} {plan.currency}
                        </span>
                        <span>•</span>
                        <span>{plan.subscriber_count} subscribers</span>
                        <span>•</span>
                        <span>${plan.revenue_last_30_days} revenue</span>
                        {plan.recommendation_count && plan.recommendation_count > 0 && (
                          <>
                            <span>•</span>
                            <span className="text-amber-600 dark:text-amber-400 font-medium">
                              {plan.recommendation_count} AI insights
                            </span>
                          </>
                        )}
                      </div>
                    </div>
                  </div>

                  <div className="flex items-center gap-4">
                    <button
                      onClick={(e) => {
                        e.stopPropagation()
                        handleTogglePlanStatus(plan.id, plan.is_active)
                      }}
                      className={`px-4 py-2 rounded-xl font-semibold ${
                        plan.is_active
                          ? 'bg-gradient-to-r from-green-400 to-green-500 text-white'
                          : 'bg-gray-200 dark:bg-gray-600 text-gray-600 dark:text-gray-300'
                      }`}
                    >
                      {plan.is_active ? 'Active' : 'Inactive'}
                    </button>
                    
                    <button
                      onClick={(e) => {
                        e.stopPropagation()
                        setState(prev => ({ ...prev, showAnalytics: plan.id }))
                      }}
                      className="px-4 py-2 rounded-xl font-semibold bg-gradient-to-r from-purple-400 to-indigo-500 text-white hover:from-purple-500 hover:to-indigo-600"
                    >
                      Analytics
                    </button>
                  </div>
                </div>
              ))}
            </div>

            {filteredAndSortedPlans.length === 0 && (
              <div className="text-center py-16">
                <div className="h-20 w-20 bg-gradient-to-br from-purple-200 to-indigo-200 dark:from-purple-800 dark:to-indigo-800 rounded-2xl flex items-center justify-center mx-auto mb-4">
                  <span className="text-4xl">📋</span>
                </div>
                <h3 className="text-xl font-semibold text-gray-600 dark:text-gray-400 mb-2">
                  No plans found
                </h3>
                <p className="text-gray-500 dark:text-gray-500 mb-4">
                  {state.searchQuery 
                    ? `No plans match "${state.searchQuery}". Try adjusting your search or filters.`
                    : 'Start by creating your first plan to begin managing subscriptions and permissions.'
                  }
                </p>
                <button
                  onClick={() => setState(prev => ({ ...prev, isCreating: true }))}
                  className="px-6 py-3 rounded-xl bg-gradient-to-r from-purple-500 to-indigo-500 text-white font-semibold hover:from-purple-600 hover:to-indigo-600"
                >
                  Create Your First Plan
                </button>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}

export function EnhancedUnifiedPlanManagement(props: EnhancedUnifiedPlanManagementProps) {
  return (
    <PermissionErrorBoundary>
      <EnhancedUnifiedPlanManagementCore {...props} />
    </PermissionErrorBoundary>
  )
}

export default EnhancedUnifiedPlanManagement