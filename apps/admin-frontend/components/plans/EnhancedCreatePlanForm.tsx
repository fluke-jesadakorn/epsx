'use client'

import { useState, useCallback, useEffect } from 'react'
import { PancakeCard } from '@/components/ui/PancakeCard'
import { toast } from '@/hooks/use-toast'
import { PermissionErrorBoundary } from '@/components/error-boundaries/PermissionErrorBoundary'
import { PermissionErrorUI } from '@/components/ui/PermissionErrorUI'
import { enhancedPermissionAuthority } from '@/lib/api/enhanced-backend-permission-authority'
import { permissionErrorAnalytics } from '@/lib/analytics/permission-error-analytics'
import type { ApiError, PermissionValidationResult } from '@/types/api'

interface PlanFormData {
  name: string
  description: string
  plan_category: 'standard' | 'api' | 'enterprise' | 'custom'
  current_price: number
  currency: string
  target_audience: 'web_users' | 'api_developers' | 'enterprises' | 'custom'
  billing_model: 'subscription' | 'pay_per_use' | 'hybrid' | 'enterprise'
  permissions: string[]
  features: PlanFeature[]
  metadata: {
    permission_template?: string
    ai_recommendations?: AIRecommendation[]
    compliance_requirements?: string[]
    security_level?: 'standard' | 'elevated' | 'enterprise'
    support_tier?: 'basic' | 'premium' | 'enterprise'
  }
  tier_group?: string
  max_concurrent_users?: number
  api_rate_limit?: number
  storage_quota_gb?: number
  support_priority?: 'low' | 'medium' | 'high' | 'critical'
}

interface PlanFeature {
  context_name: string
  feature_key: string
  feature_config: any
  resource_cost: number
  is_active: boolean
}

interface AIRecommendation {
  type: 'pricing' | 'feature' | 'permission' | 'audience' | 'compliance'
  confidence: number
  suggestion: string
  reasoning: string
  impact_score: number
}

interface PermissionTemplate {
  name: string
  description: string
  permissions: string[]
  features: string[]
  recommended_price: number
  target_audience: string
  compliance_level: 'basic' | 'elevated' | 'enterprise'
}

interface EnhancedCreatePlanFormState {
  formData: PlanFormData
  loading: boolean
  validating: boolean
  aiRecommendations: AIRecommendation[]
  permissionTemplates: PermissionTemplate[]
  validationResult: PermissionValidationResult | null
  errors: Record<string, string>
  showAIInsights: boolean
  duplicateCheckResult: { isDuplicate: boolean; similarPlans: string[] }
}

interface EnhancedCreatePlanFormProps {
  currentUserId: string
  onClose: () => void
  onSuccess: () => void
  component?: string
  enableAnalytics?: boolean
  initialData?: Partial<PlanFormData>
}

function EnhancedCreatePlanFormCore({
  currentUserId,
  onClose,
  onSuccess,
  component = 'EnhancedCreatePlanForm',
  enableAnalytics = true,
  initialData
}: EnhancedCreatePlanFormProps) {
  const [state, setState] = useState<EnhancedCreatePlanFormState>({
    formData: {
      name: '',
      description: '',
      plan_category: 'standard',
      current_price: 0,
      currency: 'USD',
      target_audience: 'web_users',
      billing_model: 'subscription',
      permissions: [],
      features: [],
      metadata: {
        security_level: 'standard',
        support_tier: 'basic'
      },
      ...initialData
    },
    loading: false,
    validating: false,
    aiRecommendations: [],
    permissionTemplates: [],
    validationResult: null,
    errors: {},
    showAIInsights: false,
    duplicateCheckResult: { isDuplicate: false, similarPlans: [] }
  })

  const analytics = permissionErrorAnalytics

  // Enhanced permission validation for plan creation
  const validateCreatePlanPermissions = useCallback(async () => {
    setState(prev => ({ ...prev, validating: true }))
    
    try {
      const permissionsToValidate = [
        'admin:plans:create',
        'admin:plans:manage',
        'admin:permissions:assign',
        'admin:features:manage'
      ]

      const results = await Promise.allSettled(
        permissionsToValidate.map(permission =>
          enhancedPermissionAuthority.validatePermission(
            currentUserId,
            permission,
            {
              component,
              context: { 
                action: 'create_plan',
                resource_type: 'admin_plans',
                security_level: 'elevated',
                requires_elevated_privileges: true,
                plan_category: state.formData.plan_category
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
          security_level: 'elevated',
          action_type: 'create_plan'
        }
      }

      setState(prev => ({ ...prev, validationResult, validating: false }))

      if (!validationResult.hasPermission) {
        const deniedPermissions = permissionsToValidate.filter(p => !validationResult.permissions[p])
        if (enableAnalytics) {
          analytics.trackPermissionDenied(
            currentUserId,
            deniedPermissions[0] || 'admin:plans:create',
            { component, context: 'create_plan_validation' }
          )
        }
      }

      return validationResult

    } catch (error) {
      console.error('Create plan permission validation failed:', error)
      setState(prev => ({ ...prev, validating: false }))
      
      const apiError: ApiError = {
        success: false,
        error: {
          type: 'VALIDATION_ERROR',
          code: 'CREATE_PLAN_PERMISSION_CHECK_FAILED',
          message: 'Failed to validate plan creation permissions',
          user_message: 'Unable to verify your permissions for plan creation. Please refresh and try again.'
        }
      }
      
      if (enableAnalytics) {
        analytics.trackError(apiError, { 
          component, 
          context: { user_id: currentUserId, action: 'create_plan_validation' }
        })
      }
      
      throw apiError
    }
  }, [currentUserId, component, enableAnalytics, state.formData.plan_category, analytics])

  // AI-powered plan recommendations
  const generateAIRecommendations = useCallback(async (formData: PlanFormData): Promise<AIRecommendation[]> => {
    try {
      const recommendations: AIRecommendation[] = []

      // Pricing recommendations based on category and audience
      if (formData.plan_category === 'enterprise' && formData.current_price < 100) {
        recommendations.push({
          type: 'pricing',
          confidence: 0.85,
          suggestion: 'Consider pricing between $200-500 for enterprise plans',
          reasoning: 'Enterprise customers expect premium pricing and comprehensive features',
          impact_score: 8
        })
      }

      // Feature recommendations based on audience
      if (formData.target_audience === 'api_developers' && !formData.features.some(f => f.feature_key.includes('api'))) {
        recommendations.push({
          type: 'feature',
          confidence: 0.9,
          suggestion: 'Add API-specific features like rate limiting, webhooks, and documentation access',
          reasoning: 'API developers require technical features for integration success',
          impact_score: 9
        })
      }

      // Permission recommendations based on category
      if (formData.plan_category === 'standard' && formData.permissions.length === 0) {
        recommendations.push({
          type: 'permission',
          confidence: 0.95,
          suggestion: 'Add basic permissions: epsx:rankings:view, epsx:data:basic, epsx:profile:manage',
          reasoning: 'Standard plans should include core platform access permissions',
          impact_score: 9
        })
      }

      // Compliance recommendations for enterprise
      if (formData.plan_category === 'enterprise') {
        recommendations.push({
          type: 'compliance',
          confidence: 0.88,
          suggestion: 'Include audit logging, data retention controls, and compliance reporting',
          reasoning: 'Enterprise customers require compliance features for regulatory requirements',
          impact_score: 8
        })
      }

      // Audience targeting recommendations
      if (formData.current_price > 50 && formData.target_audience === 'web_users') {
        recommendations.push({
          type: 'audience',
          confidence: 0.75,
          suggestion: 'Consider targeting "api_developers" or "enterprises" for higher-priced plans',
          reasoning: 'Web users typically prefer lower-cost options, while developers/enterprises accept premium pricing',
          impact_score: 7
        })
      }

      return recommendations.sort((a, b) => b.impact_score - a.impact_score).slice(0, 5)

    } catch (error) {
      console.error('Failed to generate AI recommendations:', error)
      return []
    }
  }, [])

  // Load permission templates and generate recommendations
  useEffect(() => {
    const loadTemplatesAndRecommendations = async () => {
      try {
        // Load permission templates
        const templatesResponse = await fetch('/api/admin/permission-templates', {
          credentials: 'include',
          headers: { 'x-component': component }
        })

        if (templatesResponse.ok) {
          const templates = await templatesResponse.json()
          setState(prev => ({ ...prev, permissionTemplates: templates.templates || [] }))
        }

        // Generate AI recommendations based on current form data
        const recommendations = await generateAIRecommendations(state.formData)
        setState(prev => ({ ...prev, aiRecommendations: recommendations }))

      } catch (error) {
        console.error('Failed to load templates and recommendations:', error)
      }
    }

    loadTemplatesAndRecommendations()
  }, [component, generateAIRecommendations, state.formData])

  // Duplicate plan name check
  const checkForDuplicates = useCallback(async (planName: string) => {
    if (!planName.trim()) return

    try {
      const response = await fetch(`/api/admin/plans/check-duplicate`, {
        method: 'POST',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
          'x-component': component
        },
        body: JSON.stringify({ name: planName })
      })

      if (response.ok) {
        const result = await response.json()
        setState(prev => ({
          ...prev,
          duplicateCheckResult: {
            isDuplicate: result.isDuplicate || false,
            similarPlans: result.similarPlans || []
          }
        }))
      }
    } catch (error) {
      console.error('Duplicate check failed:', error)
    }
  }, [component])

  // Form validation
  const validateForm = useCallback((): { isValid: boolean; errors: Record<string, string> } => {
    const errors: Record<string, string> = {}

    if (!state.formData.name.trim()) {
      errors.name = 'Plan name is required'
    } else if (state.duplicateCheckResult.isDuplicate) {
      errors.name = 'Plan name already exists'
    }

    if (state.formData.current_price < 0) {
      errors.current_price = 'Price cannot be negative'
    }

    if (state.formData.plan_category === 'enterprise' && state.formData.current_price < 50) {
      errors.current_price = 'Enterprise plans should be priced above $50'
    }

    if (state.formData.permissions.length === 0 && state.formData.plan_category !== 'custom') {
      errors.permissions = 'At least one permission is required for standard plans'
    }

    if (state.formData.features.length === 0) {
      errors.features = 'At least one feature is required'
    }

    return { isValid: Object.keys(errors).length === 0, errors }
  }, [state.formData, state.duplicateCheckResult])

  // Enhanced form submission with comprehensive error handling
  const handleSubmit = useCallback(async (e: React.FormEvent) => {
    e.preventDefault()

    // Validate permissions first
    const validationResult = await validateCreatePlanPermissions()
    if (!validationResult.hasPermission) {
      return
    }

    // Validate form
    const { isValid, errors } = validateForm()
    if (!isValid) {
      setState(prev => ({ ...prev, errors }))
      toast({
        title: "Validation Error",
        description: "Please fix the form errors and try again",
        variant: "destructive"
      })
      return
    }

    setState(prev => ({ ...prev, loading: true, errors: {} }))

    try {
      // Prepare plan data with AI recommendations
      const planData = {
        ...state.formData,
        metadata: {
          ...state.formData.metadata,
          ai_recommendations: state.aiRecommendations,
          created_by: currentUserId,
          creation_timestamp: Date.now(),
          validation_passed: true,
          duplicate_checked: true
        }
      }

      const response = await fetch('/api/admin/plans', {
        method: 'POST',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
          'x-component': component
        },
        body: JSON.stringify(planData)
      })

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}))
        const apiError: ApiError = {
          success: false,
          error: {
            type: response.status === 403 ? 'PERMISSION_DENIED' : 'CREATION_ERROR',
            code: 'PLAN_CREATION_FAILED',
            message: errorData.message || 'Failed to create plan',
            user_message: errorData.user_message || 'Unable to create plan. Please check your input and try again.'
          }
        }
        
        if (enableAnalytics) {
          analytics.trackError(apiError, { 
            component, 
            context: { user_id: currentUserId, plan_name: state.formData.name, http_status: response.status }
          })
        }
        throw apiError
      }

      const result = await response.json()

      // Success tracking
      if (enableAnalytics) {
        analytics.trackEvent('plan_created', {
          component,
          user_id: currentUserId,
          plan_id: result.plan?.id,
          plan_category: state.formData.plan_category,
          plan_price: state.formData.current_price,
          permissions_count: state.formData.permissions.length,
          features_count: state.formData.features.length,
          ai_recommendations_count: state.aiRecommendations.length
        })
      }

      toast({
        title: "Success",
        description: "Plan created successfully with AI-powered optimizations",
      })

      onSuccess()

    } catch (error) {
      console.error('Plan creation failed:', error)
      
      const errorMessage = error instanceof Error ? error.message : 'Failed to create plan'
      toast({
        title: "Error",
        description: errorMessage,
        variant: "destructive"
      })

      if (enableAnalytics && !(error as ApiError).error) {
        analytics.trackError(error as ApiError, { 
          component, 
          context: { user_id: currentUserId, action: 'create_plan' }
        })
      }
    } finally {
      setState(prev => ({ ...prev, loading: false }))
    }
  }, [
    validateCreatePlanPermissions, 
    validateForm, 
    state.formData, 
    state.aiRecommendations, 
    currentUserId, 
    component, 
    enableAnalytics, 
    analytics, 
    onSuccess
  ])

  // Apply permission template
  const applyTemplate = useCallback((template: PermissionTemplate) => {
    setState(prev => ({
      ...prev,
      formData: {
        ...prev.formData,
        permissions: [...template.permissions],
        features: template.features.map(feature => ({
          context_name: 'web_app',
          feature_key: feature,
          feature_config: {},
          resource_cost: 1.0,
          is_active: true
        })),
        current_price: template.recommended_price,
        target_audience: template.target_audience as any,
        metadata: {
          ...prev.formData.metadata,
          permission_template: template.name,
          compliance_level: template.compliance_level,
          security_level: template.compliance_level === 'enterprise' ? 'enterprise' : 'standard'
        }
      }
    }))

    toast({
      title: "Template Applied",
      description: `Applied ${template.name} with ${template.permissions.length} permissions`,
    })
  }, [])

  // Initialize validation
  useEffect(() => {
    validateCreatePlanPermissions()
  }, [validateCreatePlanPermissions])

  // Check for duplicates when name changes
  useEffect(() => {
    if (state.formData.name.trim()) {
      const timeoutId = setTimeout(() => {
        checkForDuplicates(state.formData.name)
      }, 500)
      return () => clearTimeout(timeoutId)
    }
  }, [state.formData.name, checkForDuplicates])

  // Permission denial UI
  if (state.validationResult && !state.validationResult.hasPermission) {
    return (
      <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center p-4 z-50">
        <div className="bg-white dark:bg-gray-800 rounded-2xl p-8 max-w-2xl w-full">
          <PermissionErrorUI
            title="Plan Creation Access Required"
            message="You need elevated admin permissions to create plans"
            missingPermissions={Object.keys(state.validationResult.permissions).filter(
              p => !state.validationResult!.permissions[p]
            )}
            requiredLevel="admin"
            onRetry={validateCreatePlanPermissions}
            showEscalation={true}
            context={{
              component,
              user_id: currentUserId,
              required_permissions: ['admin:plans:create', 'admin:plans:manage']
            }}
          />
        </div>
      </div>
    )
  }

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center p-4 z-50">
      <PancakeCard className="bg-white dark:bg-gray-800 max-w-6xl w-full max-h-[95vh] overflow-y-auto">
        <div className="p-8">
          <div className="flex items-center justify-between mb-8">
            <div>
              <h2 className="text-3xl font-bold bg-gradient-to-r from-purple-600 via-blue-600 to-indigo-600 bg-clip-text text-transparent">
                🎯 Create Enhanced Plan
              </h2>
              <p className="text-gray-600 dark:text-gray-400 mt-1">
                AI-powered plan creation with intelligent recommendations and validation
              </p>
            </div>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 text-2xl font-bold"
            >
              ✕
            </button>
          </div>

          {/* AI Insights Toggle */}
          <div className="flex items-center justify-between mb-6">
            <div className="flex items-center gap-3">
              <span className="text-2xl">🤖</span>
              <span className="font-semibold text-gray-800 dark:text-gray-200">
                AI Insights ({state.aiRecommendations.length})
              </span>
            </div>
            <button
              onClick={() => setState(prev => ({ ...prev, showAIInsights: !prev.showAIInsights }))}
              className="px-4 py-2 rounded-xl bg-gradient-to-r from-amber-400 to-orange-500 text-white font-semibold hover:from-amber-500 hover:to-orange-600"
            >
              {state.showAIInsights ? 'Hide' : 'Show'} AI Recommendations
            </button>
          </div>

          {/* AI Recommendations Panel */}
          {state.showAIInsights && state.aiRecommendations.length > 0 && (
            <div className="bg-gradient-to-r from-amber-50 via-yellow-50 to-orange-50 dark:from-amber-900/20 dark:via-yellow-900/20 dark:to-orange-900/20 rounded-2xl p-6 mb-8 border border-amber-200 dark:border-amber-700">
              <div className="space-y-4">
                {state.aiRecommendations.map((rec, index) => (
                  <div key={index} className="bg-white/70 dark:bg-gray-800/70 rounded-xl p-4">
                    <div className="flex items-center justify-between mb-2">
                      <div className="flex items-center gap-3">
                        <span className="text-lg">
                          {rec.type === 'pricing' ? '💰' : 
                           rec.type === 'feature' ? '⚡' : 
                           rec.type === 'permission' ? '🔐' : 
                           rec.type === 'audience' ? '👥' : '📋'}
                        </span>
                        <span className="font-semibold text-gray-800 dark:text-gray-200 capitalize">
                          {rec.type} Recommendation
                        </span>
                      </div>
                      <div className="flex items-center gap-2">
                        <span className="text-xs bg-blue-100 text-blue-700 dark:bg-blue-900/20 dark:text-blue-300 px-2 py-1 rounded-full">
                          {Math.round(rec.confidence * 100)}% confident
                        </span>
                        <span className="text-xs bg-green-100 text-green-700 dark:bg-green-900/20 dark:text-green-300 px-2 py-1 rounded-full">
                          Impact: {rec.impact_score}/10
                        </span>
                      </div>
                    </div>
                    <p className="text-sm text-gray-700 dark:text-gray-300 mb-2 font-medium">
                      {rec.suggestion}
                    </p>
                    <p className="text-xs text-gray-600 dark:text-gray-400">
                      {rec.reasoning}
                    </p>
                  </div>
                ))}
              </div>
            </div>
          )}

          <form onSubmit={handleSubmit} className="space-y-8">
            {/* Basic Information */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                  Plan Name *
                </label>
                <input
                  type="text"
                  value={state.formData.name}
                  onChange={(e) => setState(prev => ({
                    ...prev,
                    formData: { ...prev.formData, name: e.target.value }
                  }))}
                  className={`w-full px-4 py-3 rounded-xl border ${
                    state.errors.name ? 'border-red-300' : 'border-gray-200 dark:border-gray-600'
                  } bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-purple-500`}
                  placeholder="e.g., Professional Analytics Plan"
                  required
                />
                {state.errors.name && (
                  <p className="text-red-500 text-sm mt-1">{state.errors.name}</p>
                )}
                {state.duplicateCheckResult.isDuplicate && (
                  <p className="text-amber-600 dark:text-amber-400 text-sm mt-1">
                    Similar plans: {state.duplicateCheckResult.similarPlans.join(', ')}
                  </p>
                )}
              </div>

              <div>
                <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                  Plan Category *
                </label>
                <select
                  value={state.formData.plan_category}
                  onChange={(e) => setState(prev => ({
                    ...prev,
                    formData: { ...prev.formData, plan_category: e.target.value as any }
                  }))}
                  className="w-full px-4 py-3 rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-purple-500"
                  required
                >
                  <option value="standard">Standard - Web Users</option>
                  <option value="api">API - Developers</option>
                  <option value="enterprise">Enterprise - Organizations</option>
                  <option value="custom">Custom - Specialized</option>
                </select>
              </div>
            </div>

            {/* Pricing */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
              <div>
                <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                  Price *
                </label>
                <div className="flex">
                  <span className="inline-flex items-center px-3 rounded-l-xl border border-r-0 border-gray-200 dark:border-gray-600 bg-gray-50 dark:bg-gray-600 text-gray-500 dark:text-gray-400">
                    $
                  </span>
                  <input
                    type="number"
                    step="0.01"
                    value={state.formData.current_price}
                    onChange={(e) => setState(prev => ({
                      ...prev,
                      formData: { ...prev.formData, current_price: parseFloat(e.target.value) || 0 }
                    }))}
                    className={`flex-1 px-4 py-3 rounded-r-xl border ${
                      state.errors.current_price ? 'border-red-300' : 'border-gray-200 dark:border-gray-600'
                    } bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-purple-500`}
                    placeholder="29.99"
                    required
                  />
                </div>
                {state.errors.current_price && (
                  <p className="text-red-500 text-sm mt-1">{state.errors.current_price}</p>
                )}
              </div>

              <div>
                <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                  Target Audience
                </label>
                <select
                  value={state.formData.target_audience}
                  onChange={(e) => setState(prev => ({
                    ...prev,
                    formData: { ...prev.formData, target_audience: e.target.value as any }
                  }))}
                  className="w-full px-4 py-3 rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-purple-500"
                >
                  <option value="web_users">Web Users</option>
                  <option value="api_developers">API Developers</option>
                  <option value="enterprises">Enterprises</option>
                  <option value="custom">Custom Audience</option>
                </select>
              </div>

              <div>
                <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                  Billing Model
                </label>
                <select
                  value={state.formData.billing_model}
                  onChange={(e) => setState(prev => ({
                    ...prev,
                    formData: { ...prev.formData, billing_model: e.target.value as any }
                  }))}
                  className="w-full px-4 py-3 rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-purple-500"
                >
                  <option value="subscription">Monthly Subscription</option>
                  <option value="pay_per_use">Pay Per Use</option>
                  <option value="hybrid">Hybrid Model</option>
                  <option value="enterprise">Enterprise Contract</option>
                </select>
              </div>
            </div>

            {/* Description */}
            <div>
              <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                Description
              </label>
              <textarea
                value={state.formData.description}
                onChange={(e) => setState(prev => ({
                  ...prev,
                  formData: { ...prev.formData, description: e.target.value }
                }))}
                rows={3}
                className="w-full px-4 py-3 rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-purple-500"
                placeholder="Describe your plan features and benefits..."
              />
            </div>

            {/* Permission Templates */}
            {state.permissionTemplates.length > 0 && (
              <div>
                <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-4">
                  Quick Setup Templates
                </label>
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                  {state.permissionTemplates.map((template, index) => (
                    <div
                      key={index}
                      className="p-4 border border-gray-200 dark:border-gray-600 rounded-xl hover:bg-gray-50 dark:hover:bg-gray-700 cursor-pointer"
                      onClick={() => applyTemplate(template)}
                    >
                      <h4 className="font-semibold text-gray-800 dark:text-gray-200 mb-2">
                        {template.name}
                      </h4>
                      <p className="text-sm text-gray-600 dark:text-gray-400 mb-2">
                        {template.description}
                      </p>
                      <div className="text-xs text-gray-500 space-y-1">
                        <div>{template.permissions.length} permissions</div>
                        <div>{template.features.length} features</div>
                        <div>${template.recommended_price} recommended</div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Current Permissions */}
            {state.formData.permissions.length > 0 && (
              <div>
                <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-4">
                  Plan Permissions ({state.formData.permissions.length})
                </label>
                <div className="max-h-48 overflow-y-auto space-y-2 bg-gray-50 dark:bg-gray-700 rounded-xl p-4">
                  {state.formData.permissions.map((permission, index) => (
                    <div key={index} className="flex items-center justify-between p-2 bg-white dark:bg-gray-800 rounded-lg">
                      <code className="text-sm font-mono text-gray-800 dark:text-gray-200">
                        {permission}
                      </code>
                      <button
                        type="button"
                        onClick={() => setState(prev => ({
                          ...prev,
                          formData: {
                            ...prev.formData,
                            permissions: prev.formData.permissions.filter((_, i) => i !== index)
                          }
                        }))}
                        className="text-red-500 hover:text-red-700 font-bold"
                      >
                        ✕
                      </button>
                    </div>
                  ))}
                </div>
                {state.errors.permissions && (
                  <p className="text-red-500 text-sm mt-1">{state.errors.permissions}</p>
                )}
              </div>
            )}

            {/* Submit Actions */}
            <div className="flex gap-4 pt-6 border-t border-gray-200 dark:border-gray-700">
              <button
                type="button"
                onClick={onClose}
                disabled={state.loading}
                className="flex-1 px-6 py-3 rounded-xl border border-gray-200 dark:border-gray-600 text-gray-700 dark:text-gray-300 font-semibold hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50"
              >
                Cancel
              </button>
              <button
                type="submit"
                disabled={state.loading || state.validating}
                className="flex-1 px-6 py-3 rounded-xl bg-gradient-to-r from-purple-500 to-indigo-500 text-white font-semibold hover:from-purple-600 hover:to-indigo-600 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {state.loading ? (
                  <span className="flex items-center justify-center gap-2">
                    <div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                    Creating Plan...
                  </span>
                ) : (
                  'Create Enhanced Plan'
                )}
              </button>
            </div>
          </form>
        </div>
      </PancakeCard>
    </div>
  )
}

export function EnhancedCreatePlanForm(props: EnhancedCreatePlanFormProps) {
  return (
    <PermissionErrorBoundary>
      <EnhancedCreatePlanFormCore {...props} />
    </PermissionErrorBoundary>
  )
}

export default EnhancedCreatePlanForm