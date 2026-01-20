'use client'

import { useParams, useRouter } from 'next/navigation'
import { useEffect, useState } from 'react'

import { PermissionTransferList } from '@/components/groups/PermissionTransferList'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog"
import { PageLoadingSpinner } from '@/components/ui/LoadingSpinner'
import { toast } from '@/hooks/use-toast'
import { useAvailablePermissions } from '@/hooks/useGroupPermissions'
import { createPlansClient, isApiSuccess, type PlanResponse } from '@/shared/api/plans'
import { useSharedAuth } from '@/shared/components/auth/Provider'
import { createAdminApiClient } from '@/shared/utils/api-client'
import * as Promo from '@/shared/utils/promo'

/**
 *
 */
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
    ranking_offset: 100, // Number of top ranks locked (e.g. 24 = ranks 1-24 locked)
    analytics_queries: 0,
    premium_features: false,
    export_limit: 10,
    // Promotion fields
    promo_enabled: false,
    promo_type: 'percentage' as 'percentage' | 'fixed',
    promo_value: 0,
    promo_price: 0,
    promo_start: '',
    promo_end: ''
  })
  const [saving, setSaving] = useState(false)
  const { permissions: availablePermissions, isLoading: loadingPermissions } = useAvailablePermissions()
  const [customPermissions, setCustomPermissions] = useState<string[]>([])

  useEffect(() => {
    if (!authLoading && (!isAuthenticated || !user)) {
      router.push('/plans')
    }
  }, [authLoading, isAuthenticated, user, router])

  useEffect(() => {
    const loadPlan = async () => {
      if (!params.id) { return }

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
          const analyticsPermission = planData.permissions?.find((p: string) => p.startsWith('epsx:analytics:queries:'))
          
          // Extract ranking_offset from metadata (default to 100 for free tier)
          const rankingOffset = planData.metadata?.ranking_offset ?? 100

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

          // Parse limits with proper fallbacks
          // Returns: -1 for unlimited, 0 for not granted, >0 for specific limit
          const parseLimit = (permission: string | undefined, defaultValue: number): number => {
            if (!permission) { return defaultValue }
            const parts = permission.split(':')
            const lastPart = parts[parts.length - 1]

            // Check if it's "unlimited"
            if (lastPart === 'unlimited') { return -1 }

            const parsed = parseInt(lastPart || '0')
            return isNaN(parsed) ? defaultValue : parsed
          }

          // Extract promotion data from metadata
          const promo = planData.metadata?.promotion || {}

          setFormData({
            name: planData.name,
            description: planData.description || '',
            current_price: typeof planData.current_price === 'string' ? parseFloat(planData.current_price) : (planData.current_price || 0),
            is_active: planData.is_active,
            api_calls_limit: parseLimit(apiCallsPermission, 100),
            ranking_offset: rankingOffset,
            analytics_queries: parseLimit(analyticsPermission, 0),
            premium_features: planData.permissions?.some((p: string) => p.includes('premium')) || false,
            export_limit: 10,
            // Promotion fields
            promo_enabled: promo.enabled || false,
            promo_type: promo.type || 'percentage',
            promo_value: promo.value || 0,
            promo_price: promo.price || 0,
            promo_start: promo.start_date || '',
            promo_end: promo.end_date || ''
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

  // Sync custom permissions from plan data
  useEffect(() => {
    if (plan && plan.permissions) {
      // Filter out permission strings that are handled by the UI inputs (limits)
      const managedPrefixes = [
        'epsx:api:calls:',
        'epsx:rankings:offset:',
        'epsx:analytics:queries:',
        'epsx:export:limit:',
        'epsx:trading:premium',
        'epsx:analytics:premium'
      ]
      const others = plan.permissions.filter(p => !managedPrefixes.some(prefix => p.startsWith(prefix)))
      setCustomPermissions(others)
    }
  }, [plan])

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

      // Analytics permission
      if (formData.analytics_queries === -1) {
        permissions.push('epsx:analytics:queries:unlimited')
      } else if (formData.analytics_queries > 0) {
        permissions.push(`epsx:analytics:queries:${formData.analytics_queries}`)
      }

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

      // Add custom permissions from TransferList
      permissions.push(...customPermissions)

      const response = await adminClient.updatePlan(params.id as string, {
        name: formData.name,
        description: formData.description,
        current_price: formData.current_price,
        is_active: formData.is_active,
        permissions,
        metadata: {
          ranking_offset: formData.ranking_offset, // Top-level for easy access
          api_limits: {
            api_calls: formData.api_calls_limit,
            analytics_queries: formData.analytics_queries,
            export_limit: formData.export_limit,
            premium_features: formData.premium_features
          },
          promotion: formData.promo_enabled ? {
            enabled: true,
            type: formData.promo_type,
            value: formData.promo_value,
            price: formData.promo_price,
            start_date: formData.promo_start,
            end_date: formData.promo_end
          } : null
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
        <PageLoadingSpinner label="Loading plan..." />
      </div>
    )
  }

  if (!plan) {
    return null
  }

  return (
    <div className="min-h-screen bg-background">
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

          {/* Promotions */}
          <div className="bg-gradient-to-r from-rose-50 to-red-50 dark:from-rose-900/20 dark:to-red-900/20 rounded-2xl p-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-bold text-gray-900 dark:text-white">🎁 Promotion & Discounts</h3>
              {formData.promo_enabled && formData.promo_start && formData.promo_end && (
                <div className={`px-4 py-2 rounded-xl font-semibold ${Promo.getStatusColor(Promo.getStatus(formData.promo_enabled, formData.promo_start, formData.promo_end))
                  }`}>
                  {Promo.getStatusIcon(Promo.getStatus(formData.promo_enabled, formData.promo_start, formData.promo_end))}
                  {' '}
                  {Promo.getStatusText(Promo.getStatus(formData.promo_enabled, formData.promo_start, formData.promo_end))}
                  {Promo.getStatus(formData.promo_enabled, formData.promo_start, formData.promo_end) === 'active' && (
                    <span className="ml-2 text-xs">
                      ({Promo.getTimeRemaining(formData.promo_end)})
                    </span>
                  )}
                  {Promo.getStatus(formData.promo_enabled, formData.promo_start, formData.promo_end) === 'upcoming' && (
                    <span className="ml-2 text-xs">
                      ({Promo.getTimeUntilStart(formData.promo_start)})
                    </span>
                  )}
                </div>
              )}
            </div>

            {/* Enable Promotion Toggle */}
            <div className="mb-6">
              <label className="flex items-center gap-3 cursor-pointer">
                <input
                  type="checkbox"
                  checked={formData.promo_enabled}
                  onChange={(e) => setFormData({ ...formData, promo_enabled: e.target.checked })}
                  className="w-6 h-6 rounded border-2 border-gray-300 dark:border-gray-600 text-rose-500 focus:ring-2 focus:ring-rose-500"
                />
                <span className="text-sm font-semibold text-gray-700 dark:text-gray-300">
                  Enable Promotion
                </span>
              </label>
            </div>

            {formData.promo_enabled && (
              <div className="space-y-4">
                {/* Discount Type */}
                <div>
                  <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                    Discount Type
                  </label>
                  <div className="grid grid-cols-2 gap-4">
                    <button
                      type="button"
                      onClick={() => setFormData({ ...formData, promo_type: 'percentage' })}
                      className={`p-4 rounded-xl border-2 font-semibold ${formData.promo_type === 'percentage'
                        ? 'border-rose-500 bg-rose-50 dark:bg-rose-900/20 text-rose-700 dark:text-rose-300'
                        : 'border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300'
                        }`}
                    >
                      % Percentage
                    </button>
                    <button
                      type="button"
                      onClick={() => setFormData({ ...formData, promo_type: 'fixed' })}
                      className={`p-4 rounded-xl border-2 font-semibold ${formData.promo_type === 'fixed'
                        ? 'border-rose-500 bg-rose-50 dark:bg-rose-900/20 text-rose-700 dark:text-rose-300'
                        : 'border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300'
                        }`}
                    >
                      $ Fixed Amount
                    </button>
                  </div>
                </div>

                {/* Discount Value */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                      {formData.promo_type === 'percentage' ? 'Discount (%)' : 'Discount Amount ($)'}
                    </label>
                    <input
                      type="number"
                      step={formData.promo_type === 'percentage' ? '1' : '0.01'}
                      min="0"
                      max={formData.promo_type === 'percentage' ? '100' : undefined}
                      value={formData.promo_value}
                      onChange={(e) => {
                        const value = parseFloat(e.target.value) || 0
                        const newValue = formData.promo_type === 'percentage' ? Math.min(value, 100) : value
                        setFormData({ ...formData, promo_value: newValue })
                      }}
                      className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-rose-500 focus:ring-2 focus:ring-rose-500 focus:outline-none"
                      placeholder={formData.promo_type === 'percentage' ? '20' : '5.00'}
                    />
                  </div>

                  <div>
                    <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                      Final Promotional Price ($)
                    </label>
                    <input
                      type="number"
                      step="0.01"
                      min="0"
                      value={formData.promo_price}
                      onChange={(e) => setFormData({ ...formData, promo_price: parseFloat(e.target.value) || 0 })}
                      className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-rose-500 focus:ring-2 focus:ring-rose-500 focus:outline-none"
                      placeholder="Auto-calculated or custom"
                    />
                    <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                      Auto: ${Promo.calcPrice(
                        formData.current_price,
                        formData.promo_type,
                        formData.promo_value
                      ).toFixed(2)}
                    </p>
                  </div>
                </div>

                {/* Promotion Period */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                      Start Date
                    </label>
                    <input
                      type="datetime-local"
                      value={formData.promo_start}
                      onChange={(e) => setFormData({ ...formData, promo_start: e.target.value })}
                      className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-rose-500 focus:ring-2 focus:ring-rose-500 focus:outline-none"
                    />
                  </div>

                  <div>
                    <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                      End Date
                    </label>
                    <input
                      type="datetime-local"
                      value={formData.promo_end}
                      onChange={(e) => setFormData({ ...formData, promo_end: e.target.value })}
                      className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-rose-500 focus:ring-2 focus:ring-rose-500 focus:outline-none"
                    />
                  </div>
                </div>

                {/* Promotion Preview */}
                <div className="mt-4 p-4 bg-gradient-to-r from-rose-100 to-red-100 dark:from-rose-900/30 dark:to-red-900/30 rounded-xl border-2 border-rose-200 dark:border-rose-700">
                  <p className="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">Preview:</p>
                  <div className="space-y-1">
                    <p className="text-gray-900 dark:text-white">
                      <span className="line-through text-gray-500">${formData.current_price.toFixed(2)}</span>
                      {' → '}
                      <span className="font-bold text-rose-600 dark:text-rose-400">
                        ${Promo.calcPrice(
                          formData.current_price,
                          formData.promo_type,
                          formData.promo_value,
                          formData.promo_price > 0 ? formData.promo_price : undefined
                        ).toFixed(2)}
                      </span>
                      <span className="ml-2 text-sm text-rose-600 dark:text-rose-400 font-semibold">
                        {Promo.formatBadge(formData.promo_type, formData.promo_value, 'active')}
                      </span>
                    </p>
                    {formData.promo_start && formData.promo_end && (
                      <>
                        <p className="text-xs text-gray-600 dark:text-gray-400">
                          Valid: {new Date(formData.promo_start).toLocaleDateString()} - {new Date(formData.promo_end).toLocaleDateString()}
                        </p>
                        <p className="text-xs font-semibold text-gray-700 dark:text-gray-300">
                          Status: {Promo.getStatusText(Promo.getStatus(formData.promo_enabled, formData.promo_start, formData.promo_end))}
                        </p>
                      </>
                    )}
                  </div>
                </div>
              </div>
            )}
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
                  Ranking Offset (Premium Ranks)
                </label>
                <input
                  type="number"
                  min="0"
                  value={formData.ranking_offset}
                  onChange={(e) => setFormData({ ...formData, ranking_offset: parseInt(e.target.value) || 0 })}
                  className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-purple-500 focus:ring-2 focus:ring-purple-500 focus:outline-none"
                  placeholder="0 = full access, 24 = ranks 1-24 locked"
                />
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  Number of top ranks locked. e.g. 24 = ranks 1-24 locked, user sees 25+. 0 = full access.
                </p>
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

          {/* Advanced Permissions (Drag & Drop) */}
          <div className="bg-gradient-to-r from-gray-50 to-slate-50 dark:from-gray-900/30 dark:to-slate-900/30 rounded-2xl p-6">
            <h3 className="text-lg font-bold text-gray-900 dark:text-white mb-4">🔐 Advanced Permissions</h3>
            <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
              Manage specific permissions directly. Drag and drop permissions to assign them to this plan.
              Note: API limits configured above are automatically handled and shouldn't be added here manually.
            </p>
            <PermissionTransferList
              available={(availablePermissions || []).filter(p =>
                !p.startsWith('epsx:api:calls:') &&
                !p.startsWith('epsx:rankings:offset:') &&
                !p.startsWith('epsx:analytics:queries:') &&
                !p.startsWith('epsx:export:limit:')
              )}
              selected={customPermissions}
              onChange={setCustomPermissions}
              isLoading={loadingPermissions}
              systemPermissions={new Set(
                (availablePermissions || []).filter(p => p.startsWith('system:') || p.startsWith('admin:'))
              )}
            />
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

        {/* Delete Section */}
        <div className="mt-12 pt-8 border-t border-gray-200 dark:border-gray-700">
          <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 p-6 bg-red-50 dark:bg-red-900/10 rounded-2xl border border-red-100 dark:border-red-900/30">
            <div>
              <h3 className="text-lg font-bold text-red-700 dark:text-red-400">Danger Zone</h3>
              <p className="text-sm text-red-600/80 dark:text-red-400/70 mt-1">
                Deleting this plan will remove it from the system. This action cannot be undone.
              </p>
            </div>
            <div>
              <AlertDialog>
                <AlertDialogTrigger asChild>
                  <button
                    type="button"
                    className="px-6 py-3 rounded-xl font-semibold bg-white border-2 border-red-200 text-red-600 hover:bg-red-50 dark:bg-transparent dark:border-red-800 dark:text-red-400 dark:hover:bg-red-900/20 transition-colors"
                  >
                    Delete Plan
                  </button>
                </AlertDialogTrigger>
                <AlertDialogContent>
                  <AlertDialogHeader>
                    <AlertDialogTitle>Are you absolutely sure?</AlertDialogTitle>
                    <AlertDialogDescription>
                      This action cannot be undone. This will permanently delete the
                      <span className="font-bold text-foreground"> "{plan.name}" </span>
                      plan and remove it from our servers.
                    </AlertDialogDescription>
                  </AlertDialogHeader>
                  <AlertDialogFooter>
                    <AlertDialogCancel>Cancel</AlertDialogCancel>
                    <AlertDialogAction
                      className="bg-red-600 hover:bg-red-700 text-white"
                      onClick={async () => {
                        try {
                          setSaving(true)
                          const apiClient = createAdminApiClient()
                          const plansClient = createPlansClient(apiClient)
                          const response = await plansClient.deletePlan(params.id as string)

                          if (isApiSuccess(response)) {
                            toast({
                              title: "Deleted",
                              description: "Plan deleted successfully",
                            })
                            router.push('/plans')
                          } else {
                            toast({
                              title: "Error",
                              description: response.error || "Failed to delete plan",
                              variant: "destructive"
                            })
                          }
                        } catch (error) {
                          toast({
                            title: "Error",
                            description: "Failed to delete plan",
                            variant: "destructive"
                          })
                        } finally {
                          setSaving(false)
                        }
                      }}
                    >
                      Delete Plan
                    </AlertDialogAction>
                  </AlertDialogFooter>
                </AlertDialogContent>
              </AlertDialog>
            </div>
          </div>
        </div>
      </div >
    </div >
  )
}
