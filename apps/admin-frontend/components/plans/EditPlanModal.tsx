'use client'

import { useState, useEffect } from 'react'

import { PancakeCard } from '@/components/ui/PancakeCard'
import { toast } from '@/hooks/use-toast'
import { createPlansClient, type PlanResponse, isApiSuccess } from '@/shared/api/plans'
import { createAdminApiClient } from '@/shared/utils/api-client'

interface EditPlanModalProps {
  plan: PlanResponse
  onClose: () => void
  onSuccess: () => void
}

/**
 *
 * @param root0
 * @param root0.plan
 * @param root0.onClose
 * @param root0.onSuccess
 */
export function EditPlanModal({ plan, onClose, onSuccess }: EditPlanModalProps) {
  const [formData, setFormData] = useState({
    name: plan.name,
    description: plan.description || '',
    current_price: plan.current_price,
    is_active: plan.is_active,
    api_calls_limit: 100,
    rankings_limit: 3,
    analytics_queries: 0,
    premium_features: false,
    export_limit: 10
  })
  const [saving, setSaving] = useState(false)

  // Extract API limits from permissions on mount
  useEffect(() => {
    const apiCallsPermission = plan.permissions?.find(p => p.startsWith('epsx:api:calls:'))
    const rankingsPermission = plan.permissions?.find(p => p.startsWith('epsx:rankings:view:'))
    const analyticsPermission = plan.permissions?.find(p => p.startsWith('epsx:analytics:queries:'))

    if (apiCallsPermission) {
      const limit = apiCallsPermission.split(':').pop()
      if (limit && limit !== 'unlimited') {
        setFormData(prev => ({ ...prev, api_calls_limit: parseInt(limit) }))
      }
    }

    if (rankingsPermission) {
      const limit = rankingsPermission.split(':').pop()
      if (limit && limit !== 'unlimited') {
        setFormData(prev => ({ ...prev, rankings_limit: parseInt(limit) }))
      }
    }

    if (analyticsPermission) {
      const limit = analyticsPermission.split(':').pop()
      if (limit && limit !== 'unlimited') {
        setFormData(prev => ({ ...prev, analytics_queries: parseInt(limit) }))
      }
    }

    setFormData(prev => ({
      ...prev,
      premium_features: plan.permissions?.some(p => p.includes('premium')) || false
    }))
  }, [plan])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    const adminClient = createPlansClient(createAdminApiClient())

    try {
      setSaving(true)

      // Build permissions array from limits
      const permissions = []

      // API calls permission
      if (formData.api_calls_limit > 0) {
        permissions.push(`epsx:api:calls:${formData.api_calls_limit}`)
      } else {
        permissions.push('epsx:api:calls:unlimited')
      }

      // Rankings permission
      if (formData.rankings_limit > 0) {
        permissions.push(`epsx:rankings:view:${formData.rankings_limit}`)
      } else {
        permissions.push('epsx:rankings:view:unlimited')
      }

      // Analytics permission
      if (formData.analytics_queries > 0) {
        permissions.push(`epsx:analytics:queries:${formData.analytics_queries}`)
      }

      // Premium features
      if (formData.premium_features) {
        permissions.push('epsx:trading:premium')
        permissions.push('epsx:analytics:premium')
      }

      // Export limit
      if (formData.export_limit > 0) {
        permissions.push(`epsx:export:limit:${formData.export_limit}`)
      }

      const response = await adminClient.updatePlan(plan.id, {
        name: formData.name,
        description: formData.description,
        current_price: formData.current_price,
        is_active: formData.is_active,
        permissions,
        metadata: {
          api_limits: {
            api_calls: formData.api_calls_limit,
            rankings: formData.rankings_limit,
            analytics_queries: formData.analytics_queries,
            export_limit: formData.export_limit,
            premium_features: formData.premium_features
          }
        }
      })

      if (isApiSuccess(response)) {
        toast({
          title: "Success",
          description: "Plan updated successfully",
        })
        onSuccess()
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

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4">
      <div className="bg-white dark:bg-gray-900 rounded-3xl max-w-4xl w-full max-h-[90vh] overflow-y-auto shadow-2xl">
        {/* Header */}
        <div className="sticky top-0 bg-gradient-to-r from-emerald-400 via-green-500 to-teal-500 p-6 rounded-t-3xl">
          <div className="flex items-center justify-between">
            <h2 className="text-2xl font-bold text-white">✏️ Edit Plan: {plan.name}</h2>
            <button
              onClick={onClose}
              className="text-white hover:bg-white/20 rounded-xl p-2 min-h-[44px] min-w-[44px]"
            >
              ✖
            </button>
          </div>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="p-6 space-y-6">
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
                  onChange={(e) => setFormData({ ...formData, current_price: parseFloat(e.target.value) })}
                  className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-green-500 focus:ring-2 focus:ring-green-500 focus:outline-none"
                  required
                />
              </div>

              <div>
                <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                  Status
                </label>
                <select
                  value={formData.is_active ? 'active' : 'inactive'}
                  onChange={(e) => setFormData({ ...formData, is_active: e.target.value === 'active' })}
                  className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-green-500 focus:ring-2 focus:ring-green-500 focus:outline-none"
                >
                  <option value="active">Active</option>
                  <option value="inactive">Inactive</option>
                </select>
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
                  min="0"
                  value={formData.api_calls_limit}
                  onChange={(e) => setFormData({ ...formData, api_calls_limit: parseInt(e.target.value) })}
                  className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-purple-500 focus:ring-2 focus:ring-purple-500 focus:outline-none"
                  placeholder="0 = unlimited"
                />
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">Set to 0 for unlimited</p>
              </div>

              <div>
                <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                  Rankings View Limit
                </label>
                <input
                  type="number"
                  min="0"
                  value={formData.rankings_limit}
                  onChange={(e) => setFormData({ ...formData, rankings_limit: parseInt(e.target.value) })}
                  className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-purple-500 focus:ring-2 focus:ring-purple-500 focus:outline-none"
                  placeholder="0 = unlimited"
                />
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">Max stocks to view</p>
              </div>

              <div>
                <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                  Analytics Queries (per month)
                </label>
                <input
                  type="number"
                  min="0"
                  value={formData.analytics_queries}
                  onChange={(e) => setFormData({ ...formData, analytics_queries: parseInt(e.target.value) })}
                  className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-purple-500 focus:ring-2 focus:ring-purple-500 focus:outline-none"
                  placeholder="0 = none"
                />
              </div>

              <div>
                <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                  Export Limit (per day)
                </label>
                <input
                  type="number"
                  min="0"
                  value={formData.export_limit}
                  onChange={(e) => setFormData({ ...formData, export_limit: parseInt(e.target.value) })}
                  className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-purple-500 focus:ring-2 focus:ring-purple-500 focus:outline-none"
                />
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

          {/* Actions */}
          <div className="flex gap-4 pt-4">
            <button
              type="button"
              onClick={onClose}
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
