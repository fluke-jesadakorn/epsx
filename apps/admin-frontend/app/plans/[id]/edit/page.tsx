'use client'

import { toast } from '@/hooks/use-toast'
import { createPlansClient, isApiSuccess, type PlanResponse } from '@/shared/api/plans'
import { useSharedAuth } from '@/shared/components/auth/SharedOpenIDWeb3Provider'
import { createAdminApiClient } from '@/shared/utils/api-client'
import { useParams, useRouter } from 'next/navigation'
import { useEffect, useState } from 'react'

export default function EditPlanPage() {
  const router = useRouter()
  const params = useParams()
  const { user, isLoading: authLoading, isAuthenticated } = useSharedAuth()
  const [plan, setPlan] = useState<PlanResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [formData, setFormData] = useState({
    name: '',
    description: '',
    current_price: 0,
    is_active: true,
    api_calls_limit: 100,
    rankings_limit: 3,
    analytics_queries: 0,
    premium_features: false,
    export_limit: 10,
    feature_list: [] as string[]
  })
  const [newFeature, setNewFeature] = useState('')
  const [saving, setSaving] = useState(false)

  useEffect(() => {
    if (!authLoading && (!isAuthenticated || !user)) {
      router.push('/plans')
    }
  }, [authLoading, isAuthenticated, user, router])

  useEffect(() => {
    const loadPlan = async () => {
      if (!params.id) return

      try {
        setLoading(true)
        const apiClient = createAdminApiClient()
        const plansClient = createPlansClient(apiClient)
        const response = await plansClient.getPlan(params.id as string)

        if (isApiSuccess(response)) {
          const backendResponse = response.data as any
          const planData = backendResponse?.data || backendResponse
          setPlan(planData)

          // Extract API limits from permissions
          const apiCallsPermission = planData.permissions?.find((p: string) => p.startsWith('epsx:api:calls:'))
          const rankingsPermission = planData.permissions?.find((p: string) => p.startsWith('epsx:rankings:view:'))
          const analyticsPermission = planData.permissions?.find((p: string) => p.startsWith('epsx:analytics:queries:'))

          // Extract feature list from metadata
          let featureList: string[] = []

          // Try metadata.features (backend structure)
          if (planData.metadata?.features && Array.isArray(planData.metadata.features)) {
            featureList = planData.metadata.features.filter((f: any) => typeof f === 'string')
          }
          // Fallback to metadata.feature_list (our custom field)
          else if (planData.metadata?.feature_list && Array.isArray(planData.metadata.feature_list)) {
            featureList = planData.metadata.feature_list
          }
          // Fallback to top-level features array
          else if (planData.features && Array.isArray(planData.features) && planData.features.length > 0) {
            featureList = planData.features
              .map((f: any) => typeof f === 'string' ? f : f.name || f.feature_key || '')
              .filter((f: string) => f.length > 0)
          }

          console.log('Loaded plan data:', planData)
          console.log('Extracted feature list:', featureList)

          // Parse limits with proper fallbacks
          // Returns: -1 for unlimited, 0 for not granted, >0 for specific limit
          const parseLimit = (permission: string | undefined, defaultValue: number): number => {
            if (!permission) return defaultValue
            const parts = permission.split(':')
            const lastPart = parts[parts.length - 1]

            // Check if it's "unlimited"
            if (lastPart === 'unlimited') return -1

            const parsed = parseInt(lastPart)
            return isNaN(parsed) ? defaultValue : parsed
          }

          setFormData({
            name: planData.name,
            description: planData.description || '',
            current_price: typeof planData.current_price === 'string' ? parseFloat(planData.current_price) : (planData.current_price || 0),
            is_active: planData.is_active,
            api_calls_limit: parseLimit(apiCallsPermission, 100),
            rankings_limit: parseLimit(rankingsPermission, 3),
            analytics_queries: parseLimit(analyticsPermission, 0),
            premium_features: planData.permissions?.some((p: string) => p.includes('premium')) || false,
            export_limit: 10,
            feature_list: featureList
          })
        } else {
          toast({
            title: "Error",
            description: "Failed to load plan",
            variant: "destructive"
          })
          router.push('/plans')
        }
      } catch (error) {
        toast({
          title: "Error",
          description: "Failed to load plan",
          variant: "destructive"
        })
        router.push('/plans')
      } finally {
        setLoading(false)
      }
    }

    loadPlan()
  }, [params.id, router])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    const adminClient = createPlansClient(createAdminApiClient())

    try {
      setSaving(true)

      // Build permissions array from limits
      // -1 = unlimited, 0 = not granted/disabled, >0 = specific limit
      const permissions = []

      // API calls permission
      if (formData.api_calls_limit === -1) {
        permissions.push('epsx:api:calls:unlimited')
      } else if (formData.api_calls_limit > 0) {
        permissions.push(`epsx:api:calls:${formData.api_calls_limit}`)
      }
      // If 0, don't add permission (not granted)

      // Rankings permission
      if (formData.rankings_limit === -1) {
        permissions.push('epsx:rankings:view:unlimited')
      } else if (formData.rankings_limit > 0) {
        permissions.push(`epsx:rankings:view:${formData.rankings_limit}`)
      }
      // If 0, don't add permission (not granted)

      // Analytics permission
      if (formData.analytics_queries === -1) {
        permissions.push('epsx:analytics:queries:unlimited')
      } else if (formData.analytics_queries > 0) {
        permissions.push(`epsx:analytics:queries:${formData.analytics_queries}`)
      }
      // If 0, don't add permission (not granted)

      // Premium features
      if (formData.premium_features) {
        permissions.push('epsx:trading:premium')
        permissions.push('epsx:analytics:premium')
      }

      // Export limit
      if (formData.export_limit === -1) {
        permissions.push('epsx:export:limit:unlimited')
      } else if (formData.export_limit > 0) {
        permissions.push(`epsx:export:limit:${formData.export_limit}`)
      }
      // If 0, don't add permission (not granted)

      const response = await adminClient.updatePlan(params.id as string, {
        name: formData.name,
        description: formData.description,
        current_price: formData.current_price,
        is_active: formData.is_active,
        permissions,
        metadata: {
          features: formData.feature_list, // Backend expects 'features' not 'feature_list'
          api_limits: {
            api_calls: formData.api_calls_limit,
            rankings: formData.rankings_limit,
            analytics_queries: formData.analytics_queries,
            export_limit: formData.export_limit,
            premium_features: formData.premium_features
          },
          feature_list: formData.feature_list // Keep both for compatibility
        }
      })

      if (isApiSuccess(response)) {
        toast({
          title: "Success",
          description: "Plan updated successfully",
        })
        router.push('/plans')
      } else {
        toast({
          title: "Error",
          description: response.error || "Failed to update plan",
          variant: "destructive"
        })
      }
    } catch (_error) {
      toast({
        title: "Error",
        description: "Failed to update plan",
        variant: "destructive"
      })
    } finally {
      setSaving(false)
    }
  }

  if (authLoading || loading) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 flex items-center justify-center">
        <div className="text-center">
          <div className="text-2xl mb-4">Loading...</div>
        </div>
      </div>
    )
  }

  if (!plan) {
    return null
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900">
      {/* Header */}
      <div className="bg-gradient-to-r from-emerald-400 via-green-500 to-teal-500 p-6">
        <div className="max-w-7xl mx-auto flex items-center justify-between">
          <h2 className="text-2xl font-bold text-white">✏️ Edit Plan: {plan.name}</h2>
          <button
            onClick={() => router.push('/plans')}
            className="text-white hover:bg-white/20 rounded-xl p-2 min-h-[44px] min-w-[44px]"
          >
            ✖
          </button>
        </div>
      </div>

      {/* Form */}
      <div className="max-w-7xl mx-auto p-6">
        <form onSubmit={handleSubmit} className="space-y-6">
            {/* Basic Info */}
            <div className="bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 rounded-2xl p-6">
              <h3 className="text-lg font-bold text-gray-900 dark:text-white mb-4">📋 Basic Information</h3>

              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                    Plan Name
                  </label>
                  <input
                    type="text"
                    value={formData.name}
                    onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                    className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-emerald-500 focus:ring-2 focus:ring-emerald-500 focus:outline-none"
                    required
                  />
                </div>

                <div>
                  <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                    Description
                  </label>
                  <textarea
                    value={formData.description}
                    onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                    rows={3}
                    className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-emerald-500 focus:ring-2 focus:ring-emerald-500 focus:outline-none"
                  />
                </div>
              </div>
            </div>

            {/* Pricing */}
            <div className="bg-gradient-to-r from-green-50 to-emerald-50 dark:from-green-900/20 dark:to-emerald-900/20 rounded-2xl p-6">
              <h3 className="text-lg font-bold text-gray-900 dark:text-white mb-4">💰 Pricing</h3>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                    Price (USD)
                  </label>
                  <input
                    type="number"
                    step="0.01"
                    min="0"
                    value={formData.current_price}
                    onChange={(e) => setFormData({ ...formData, current_price: parseFloat(e.target.value) || 0 })}
                    className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-green-500 focus:ring-2 focus:ring-green-500 focus:outline-none"
                    required
                  />
                </div>

                <div>
                  <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                    Status
                  </label>
                  <div className="flex items-center gap-3 p-4 bg-white dark:bg-gray-800 rounded-xl border-2 border-gray-300 dark:border-gray-600">
                    <input
                      type="checkbox"
                      id="is_active"
                      checked={formData.is_active}
                      onChange={(e) => setFormData({ ...formData, is_active: e.target.checked })}
                      className="w-6 h-6 rounded border-2 border-gray-300 dark:border-gray-600 text-green-500 focus:ring-2 focus:ring-green-500"
                    />
                    <label htmlFor="is_active" className="text-sm font-semibold text-gray-700 dark:text-gray-300 cursor-pointer">
                      {formData.is_active ? 'Active' : 'Inactive'}
                    </label>
                  </div>
                </div>
              </div>
            </div>

            {/* API Limitations */}
            <div className="bg-gradient-to-r from-purple-50 to-pink-50 dark:from-purple-900/20 dark:to-pink-900/20 rounded-2xl p-6">
              <h3 className="text-lg font-bold text-gray-900 dark:text-white mb-4">🔧 API Limitations</h3>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                    API Calls Limit (per month)
                  </label>
                  <input
                    type="number"
                    min="-1"
                    value={formData.api_calls_limit}
                    onChange={(e) => setFormData({ ...formData, api_calls_limit: parseInt(e.target.value) || 0 })}
                    className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-purple-500 focus:ring-2 focus:ring-purple-500 focus:outline-none"
                    placeholder="-1 = unlimited, 0 = not granted"
                  />
                  <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">-1 = unlimited, 0 = not granted, &gt;0 = specific limit</p>
                </div>

                <div>
                  <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                    Rankings View Limit
                  </label>
                  <input
                    type="number"
                    min="-1"
                    value={formData.rankings_limit}
                    onChange={(e) => setFormData({ ...formData, rankings_limit: parseInt(e.target.value) || 0 })}
                    className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-purple-500 focus:ring-2 focus:ring-purple-500 focus:outline-none"
                    placeholder="-1 = unlimited, 0 = not granted"
                  />
                  <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">-1 = unlimited, 0 = not granted</p>
                </div>

                <div>
                  <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                    Analytics Queries (per month)
                  </label>
                  <input
                    type="number"
                    min="-1"
                    value={formData.analytics_queries}
                    onChange={(e) => setFormData({ ...formData, analytics_queries: parseInt(e.target.value) || 0 })}
                    className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-purple-500 focus:ring-2 focus:ring-purple-500 focus:outline-none"
                    placeholder="-1 = unlimited, 0 = not granted"
                  />
                  <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">-1 = unlimited, 0 = not granted</p>
                </div>

                <div>
                  <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                    Export Limit (per day)
                  </label>
                  <input
                    type="number"
                    min="-1"
                    value={formData.export_limit}
                    onChange={(e) => setFormData({ ...formData, export_limit: parseInt(e.target.value) || 0 })}
                    className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-purple-500 focus:ring-2 focus:ring-purple-500 focus:outline-none"
                    placeholder="-1 = unlimited, 0 = not granted"
                  />
                  <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">-1 = unlimited, 0 = not granted</p>
                </div>
              </div>

              {/* Premium Features Toggle */}
              <div className="mt-4">
                <label className="flex items-center gap-3 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={formData.premium_features}
                    onChange={(e) => setFormData({ ...formData, premium_features: e.target.checked })}
                    className="w-6 h-6 rounded border-2 border-gray-300 dark:border-gray-600 text-purple-500 focus:ring-2 focus:ring-purple-500"
                  />
                  <span className="text-sm font-semibold text-gray-700 dark:text-gray-300">
                    Enable Premium Features (Advanced Trading, Premium Analytics)
                  </span>
                </label>
              </div>
            </div>

            {/* Feature List */}
            <div className="bg-gradient-to-r from-amber-50 to-yellow-50 dark:from-amber-900/20 dark:to-yellow-900/20 rounded-2xl p-6">
              <h3 className="text-lg font-bold text-gray-900 dark:text-white mb-4">✨ Plan Features</h3>

              {/* Add Feature Input */}
              <div className="mb-4">
                <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                  Add Feature
                </label>
                <div className="flex gap-3">
                  <input
                    type="text"
                    value={newFeature}
                    onChange={(e) => setNewFeature(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter') {
                        e.preventDefault()
                        if (newFeature.trim() && !formData.feature_list.includes(newFeature.trim())) {
                          setFormData({ ...formData, feature_list: [...formData.feature_list, newFeature.trim()] })
                          setNewFeature('')
                        }
                      }
                    }}
                    placeholder="e.g., 50 stock rankings"
                    className="flex-1 px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-amber-500 focus:ring-2 focus:ring-amber-500 focus:outline-none"
                  />
                  <button
                    type="button"
                    onClick={() => {
                      if (newFeature.trim() && !formData.feature_list.includes(newFeature.trim())) {
                        setFormData({ ...formData, feature_list: [...formData.feature_list, newFeature.trim()] })
                        setNewFeature('')
                      }
                    }}
                    className="px-6 py-3 rounded-xl font-semibold bg-gradient-to-r from-amber-400 to-yellow-500 text-white hover:from-amber-500 hover:to-yellow-600 min-h-[44px] min-w-[44px]"
                  >
                    ➕ Add
                  </button>
                </div>
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">Press Enter or click Add to add a feature</p>
              </div>

              {/* Feature List */}
              {formData.feature_list.length > 0 ? (
                <div className="space-y-2">
                  <p className="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                    Current Features ({formData.feature_list.length})
                  </p>
                  <div className="max-h-64 overflow-y-auto space-y-2 bg-white dark:bg-gray-800 rounded-xl p-4 border-2 border-gray-200 dark:border-gray-700">
                    {formData.feature_list.map((feature, index) => (
                      <div
                        key={index}
                        className="flex items-center justify-between p-3 bg-gradient-to-r from-amber-50 to-yellow-50 dark:from-amber-900/20 dark:to-yellow-900/20 rounded-xl border border-amber-200 dark:border-amber-700"
                      >
                        <div className="flex items-center gap-3">
                          <span className="text-amber-600 dark:text-amber-400 font-bold">✓</span>
                          <span className="text-gray-900 dark:text-white font-medium">{feature}</span>
                        </div>
                        <button
                          type="button"
                          onClick={() => {
                            setFormData({
                              ...formData,
                              feature_list: formData.feature_list.filter((_, i) => i !== index)
                            })
                          }}
                          className="text-red-500 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300 font-bold p-2 min-h-[44px] min-w-[44px] hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg"
                          title="Remove feature"
                        >
                          ✖
                        </button>
                      </div>
                    ))}
                  </div>
                </div>
              ) : (
                <div className="bg-gray-50 dark:bg-gray-800 rounded-xl p-6 text-center border-2 border-dashed border-gray-300 dark:border-gray-600">
                  <p className="text-gray-500 dark:text-gray-400">No features added yet. Add features to describe what's included in this plan.</p>
                  <p className="text-sm text-gray-400 dark:text-gray-500 mt-2">Examples: "50 stock rankings", "Advanced analytics", "Export capabilities"</p>
                </div>
              )}
            </div>

            {/* Actions */}
            <div className="flex gap-4 pt-4">
              <button
                type="button"
                onClick={() => router.push('/plans')}
                className="flex-1 px-6 py-3 rounded-xl font-semibold bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600 min-h-[44px]"
              >
                Cancel
              </button>
              <button
                type="submit"
                disabled={saving}
                className="flex-1 px-6 py-3 rounded-xl font-semibold bg-gradient-to-r from-emerald-400 to-green-500 text-white hover:from-emerald-500 hover:to-green-600 disabled:opacity-50 disabled:cursor-not-allowed min-h-[44px]"
              >
                {saving ? 'Saving...' : 'Save Changes'}
              </button>
            </div>
          </form>
        </div>
    </div>
  )
}
