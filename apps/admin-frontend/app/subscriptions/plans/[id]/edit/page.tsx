'use client'

import { useParams, useRouter } from 'next/navigation'
import { useEffect } from 'react'

import { PageLoadingSpinner } from '@/components/ui/loading-spinner'
import { useAvailablePermissions } from '@/hooks/use-plan-permissions'
import { useSharedAuth } from '@/shared/components/auth/Provider'

import { PlanAdvancedPermissions } from '@/components/plans/edit/plan-advanced-permissions'
import { PlanApiLimits } from '@/components/plans/edit/plan-api-limits'
import { PlanBasicInfo } from '@/components/plans/edit/plan-basic-info'
import { PlanDeleteSection } from '@/components/plans/edit/plan-delete-section'
import { PlanPricing } from '@/components/plans/edit/plan-pricing'
import { PlanPromotions } from '@/components/plans/edit/plan-promotions'
import { usePlanActions } from './use-plan-actions'
import { usePlanData } from './use-plan-data'

/**
 * Edit Plan Page
 * Part of the Subscription & Access hub
 */
export default function EditPlanPage() {
  const router = useRouter()
  const params = useParams()
  const { user, isLoading: authLoading, isAuthenticated } = useSharedAuth()
  const {
    plan,
    loading: planLoading,
    formData,
    setFormData,
    customPermissions,
    setCustomPermissions,
  } = usePlanData(params.id as string)

  const { saving, handleSubmit, handleDelete } = usePlanActions(
    params.id as string
  )

  const { permissions: availablePermissions, isLoading: loadingPermissions } =
    useAvailablePermissions()

  useEffect(() => {
    if (!authLoading && (!isAuthenticated ?? !user)) {
      router.push('/subscriptions/plans')
    }
  }, [authLoading, isAuthenticated, user, router])

  if (authLoading ?? planLoading) {
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
          <h2 className="text-2xl font-bold text-white">
            Edit Plan: {plan.name}
          </h2>
          <button
            onClick={() => router.push('/subscriptions/plans')}
            className="text-white hover:bg-white/20 rounded-xl p-2 min-h-[44px] min-w-[44px]"
            aria-label="Close"
          >
            ✖
          </button>
        </div>
      </div>

      {/* Form */}
      <div className="max-w-7xl mx-auto p-6">
        <form
          onSubmit={(e) => {
            void handleSubmit(e, formData, customPermissions)
          }}
          className="space-y-6"
        >
          <PlanBasicInfo formData={formData} setFormData={setFormData} />
          <PlanPricing formData={formData} setFormData={setFormData} />
          <PlanPromotions formData={formData} setFormData={setFormData} />
          <PlanApiLimits formData={formData} setFormData={setFormData} />
          <PlanAdvancedPermissions
            availablePermissions={availablePermissions}
            customPermissions={customPermissions}
            setCustomPermissions={setCustomPermissions}
            loadingPermissions={loadingPermissions}
          />

          {/* Actions */}
          <div className="flex gap-4 pt-4">
            <button
              type="button"
              onClick={() => router.push('/subscriptions/plans')}
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

        <PlanDeleteSection planName={plan.name} onDelete={handleDelete} />
      </div>
    </div>
  )
}
