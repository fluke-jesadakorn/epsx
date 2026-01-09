'use client'

import { GripVertical, Loader2 } from 'lucide-react'
import { usePathname, useRouter } from 'next/navigation'
import { useEffect, useState } from 'react'

import { toast } from '@/hooks/use-toast'
import { createPlansClient, isApiSuccess, type PlanResponse } from '@/shared/api/plans'
import { useSharedAuth } from '@/shared/components/auth/Provider'
import { createAdminApiClient } from '@/shared/utils/api-client'

interface PlanManagementProps {
  currentUser?: any
}

/**
 *
 * @param root0
 * @param root0.currentUser
 */
export function PlanManagement({ currentUser }: PlanManagementProps) {
  const { user: authUser } = useSharedAuth()
  // Note: currentUser and authUser available for future permission checks
  const _user = currentUser || authUser
  const router = useRouter()
  const pathname = usePathname()
  const [plans, setPlans] = useState<PlanResponse[]>([])
  const [loading, setLoading] = useState(true)
  const [_selectedPlan, setSelectedPlan] = useState<PlanResponse | null>(null)

  // Drag and Drop State
  const [hasChanges, setHasChanges] = useState(false)
  const [originalOrder, setOriginalOrder] = useState<PlanResponse[]>([])
  const [draggedItem, setDraggedItem] = useState<{ plan: PlanResponse, index: number } | null>(null)
  const [dragOverIndex, setDragOverIndex] = useState<number | null>(null)
  const [saving, setSaving] = useState(false)
  const [dragNodeRef, setDragNodeRef] = useState<HTMLDivElement | null>(null)

  // Load plans on component mount
  useEffect(() => {
    loadPlans()
  }, [])

  const loadPlans = async () => {
    try {
      setLoading(true)
      const apiClient = createAdminApiClient()
      const plansClient = createPlansClient(apiClient)

      const response = await plansClient.getPlans({
        limit: 100
      })

      if (isApiSuccess(response)) {
        // Backend returns: { success, data: { plans: [...], has_more, total_count }, message }
        // API client wraps backend response as-is in its own data field
        // So response.data is the entire backend JSON response
        const backendResponse = response.data as any

        // Sort plans by tier_level initially
        const plansData = (backendResponse?.data?.plans || backendResponse?.plans || []).map((p: PlanResponse) => ({
          ...p,
          tier_level: p.tier_level ?? 0
        })).sort((a: PlanResponse, b: PlanResponse) => (a.tier_level ?? 0) - (b.tier_level ?? 0))

        setPlans(plansData)
        setOriginalOrder(plansData)
        setHasChanges(false)
      } else {
        // Handle unauthorized access first
        if (response.error?.includes('Unauthorized') || response.error?.includes('log in')) {
          console.warn('[PlanManagement] Session expired, redirecting...')
          toast({
            title: "Session Expired",
            description: "Please log in again to continue.",
            variant: "destructive"
          })

          router.push(`/auth?returnUrl=${encodeURIComponent(pathname)}`)
          return
        }

        console.error('[PlanManagement] API error:', response.error)
        toast({
          title: "Error",
          description: response.error || "Failed to load plans",
          variant: "destructive"
        })
      }
    } catch (error) {
      console.error('[PlanManagement] Exception:', error)
      toast({
        title: "Error",
        description: error instanceof Error ? error.message : "Failed to load plans",
        variant: "destructive"
      })
    } finally {
      setLoading(false)
    }
  }

  // Drag Handlers
  const checkForChanges = (newPlans: PlanResponse[]) => {
    const hasChanged = newPlans.some((plan, index) => plan.id !== originalOrder[index]?.id)
    setHasChanges(hasChanged)
  }

  // Helper to reorder within a specific group while preserving global tier spacing
  const updateGroupOrder = (groupPlans: PlanResponse[], draggedPlan: PlanResponse, dropIndex: number) => {
    const sortedTiers = groupPlans.map(p => p.tier_level ?? 0).sort((a, b) => a - b)

    // Create new list order
    const currentGroupIndex = groupPlans.findIndex(p => p.id === draggedPlan.id)
    const newGroupList = [...groupPlans]
    newGroupList.splice(currentGroupIndex, 1)
    newGroupList.splice(dropIndex, 0, draggedPlan)

    // Assign sorted tiers back to the new order
    const updatedGroup = newGroupList.map((p, idx) => ({
      ...p,
      tier_level: sortedTiers[idx]
    }))

    // Merge back into main plans list
    const otherPlans = plans.filter(p => !updatedGroup.find(up => up.id === p.id))
    const finalPlans = [...otherPlans, ...updatedGroup].sort((a, b) => (a.tier_level ?? 0) - (b.tier_level ?? 0))

    setPlans(finalPlans)
    checkForChanges(finalPlans)
  }

  const handleDragStart = (e: React.DragEvent, plan: PlanResponse, category: string) => {
    setDraggedItem({ plan, index: -1 }) // Index relative to group is calculated safely later
    e.dataTransfer.effectAllowed = 'move'
    e.dataTransfer.setData('text/plain', JSON.stringify({ id: plan.id, category })) // Pass category to restrict drop

    if (e.currentTarget instanceof HTMLElement) {
      const node = e.currentTarget as HTMLDivElement
      setDragNodeRef(node)
      setTimeout(() => node.classList.add('opacity-50'), 0)
    }
  }

  const handleDragEnd = () => {
    if (dragNodeRef) {
      dragNodeRef.classList.remove('opacity-50')
      setDragNodeRef(null)
    }
    setDraggedItem(null)
    setDragOverIndex(null)
  }

  const handleDragOver = (e: React.DragEvent, category: string) => {
    e.preventDefault()
    // Verify we are dragging within same category
    if (!draggedItem || draggedItem.plan.plan_category !== category) {
      e.dataTransfer.dropEffect = 'none'
      return
    }
    e.dataTransfer.dropEffect = 'move'
  }

  const handleDrop = (e: React.DragEvent, dropIndex: number, category: string, groupPlans: PlanResponse[]) => {
    e.preventDefault()

    if (!draggedItem || draggedItem.plan.plan_category !== category) return

    // Find the index in the current group list (not global list)
    updateGroupOrder(groupPlans, draggedItem.plan, dropIndex)

    // Cleanup
    if (dragNodeRef) {
      dragNodeRef.classList.remove('opacity-50')
      setDragNodeRef(null)
    }
    setDraggedItem(null)
    setDragOverIndex(null)
  }

  const handleSaveOrder = async () => {
    if (!hasChanges) return

    setSaving(true)
    try {
      const apiClient = createAdminApiClient()
      const plansClient = createPlansClient(apiClient)

      // Update each plan's tier_level based on its new position
      // Using index as tier_level
      const updates = plans.map((plan, index) => ({
        plan_id: plan.id,
        tier_level: index
      }))

      await Promise.all(
        updates.map(update =>
          plansClient.updatePlan(update.plan_id, { tier_level: update.tier_level })
        )
      )

      toast({ title: 'Tier order saved successfully', description: 'Plan hierarchy has been updated.' })
      setOriginalOrder([...plans])
      setHasChanges(false)
    } catch (error) {
      console.error('[PlanManagement] Error saving order:', error)
      toast({ title: 'Failed to save tier order', variant: 'destructive' })
    } finally {
      setSaving(false)
    }
  }

  const handleDiscardChanges = () => {
    setPlans([...originalOrder])
    setHasChanges(false)
  }

  // Group plans for display
  const standardPlans = plans.filter(p => p.plan_category === 'standard')
  const apiPlans = plans.filter(p => p.plan_category === 'api')
  const enterprisePlans = plans.filter(p => p.plan_category === 'enterprise')
  // We can treat custom plans as enterprise or separate, grouping with enterprise for now
  const activePlans = plans.filter(p => p.is_active)

  const totalRevenue = plans.reduce((sum, plan) => {
    const revenue = typeof plan.revenue_last_30_days === 'string'
      ? parseFloat(plan.revenue_last_30_days)
      : plan.revenue_last_30_days
    return sum + (isNaN(revenue) ? 0 : revenue)
  }, 0)
  const avgRevenue = plans.length > 0 ? totalRevenue / plans.length : 0

  if (loading) {
    return (
      <div className="max-w-7xl mx-auto space-y-8 animate-pulse">
        <div className="text-center mb-12">
          <div className="h-16 bg-gradient-to-r from-emerald-400 to-green-500 rounded-2xl w-96 mx-auto mb-6"></div>
          <div className="h-6 bg-gray-300 rounded-full w-64 mx-auto"></div>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
          {Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="bg-gradient-to-br from-gray-300 to-gray-400 rounded-3xl h-64"></div>
          ))}
        </div>
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          {Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className="bg-gray-300 rounded-3xl h-32"></div>
          ))}
        </div>
      </div>
    )
  }

  return (
    <div>
      <div className="space-y-6 sm:space-y-8">
        {/* Background Decorations */}
        <div className="fixed inset-0 overflow-hidden pointer-events-none">
          <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-yellow-400/20 to-orange-500/20 rounded-full blur-xl"></div>
          <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-pink-400/20 to-purple-500/20 rounded-full blur-lg"></div>
          <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-orange-400/15 to-yellow-500/15 rounded-full blur-xl"></div>
        </div>

        <div className="relative max-w-7xl mx-auto">
          {/* Hero Section */}
          <div className="text-center mb-8 sm:mb-12">
            <div className="relative inline-block">
              <h1 className="text-3xl sm:text-4xl lg:text-5xl font-bold bg-gradient-to-r from-yellow-600 via-orange-600 via-pink-600 to-purple-600 bg-clip-text text-transparent mb-4">
                💳 Dynamic Plans
              </h1>
              <div className="absolute -top-2 -right-2 w-4 h-4 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-full"></div>
            </div>
            <p className="text-base sm:text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
              Create and manage unlimited plans with context-specific features for web app, API, and admin access
            </p>
          </div>

          {/* Action Cards */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 sm:gap-6 mb-8 sm:mb-12">
            <div
              className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-emerald-400/20 via-green-500/20 to-teal-500/20 p-0.5 cursor-pointer"
              onClick={() => router.push('/plans/new')}
            >
              <div className="relative bg-gradient-to-br from-emerald-400 via-green-500 to-teal-500 text-white rounded-2xl sm:rounded-3xl">
                <div className="p-6 sm:p-8">
                  <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6">
                    <span className="text-xl sm:text-2xl">➕</span>
                  </div>
                  <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4">Create Dynamic Plan</h3>
                  <p className="text-white/80 mb-4 sm:mb-6 text-sm sm:text-base">Create unlimited plans with context-specific features</p>
                  <div className="bg-white/20 rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-semibold text-sm sm:text-base min-h-[44px] flex items-center justify-center">
                    New Plan
                  </div>
                </div>
              </div>
            </div>

            <div
              className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-purple-400/20 via-pink-500/20 to-rose-500/20 p-0.5 cursor-pointer"
              onClick={() => loadPlans()}
            >
              <div className="relative bg-gradient-to-br from-purple-400 via-pink-500 to-rose-500 text-white rounded-2xl sm:rounded-3xl">
                <div className="p-6 sm:p-8">
                  <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6">
                    <span className="text-xl sm:text-2xl">🔄</span>
                  </div>
                  <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4">Refresh Data</h3>
                  <p className="text-white/80 mb-4 sm:mb-6 text-sm sm:text-base">Reload plan data and analytics from server</p>
                  <div className="bg-white/20 rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-semibold text-sm sm:text-base min-h-[44px] flex items-center justify-center">
                    Refresh
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* Stats Grid */}
          <div className="grid grid-cols-2 sm:grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-6 mb-6 sm:mb-8">
            <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-emerald-300/50 dark:border-emerald-700/50">
              <div className="flex items-center justify-between mb-3 sm:mb-4">
                <div className="text-xl sm:text-2xl">💳</div>
                <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Total</span>
              </div>
              <div className="space-y-1">
                <div className="text-2xl sm:text-3xl font-bold text-emerald-600 dark:text-emerald-400">{plans.length}</div>
                <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Plans</div>
                <div className="text-xs text-gray-500 dark:text-gray-400">All types</div>
              </div>
            </div>

            <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-blue-300/50 dark:border-blue-700/50">
              <div className="flex items-center justify-between mb-3 sm:mb-4">
                <div className="text-xl sm:text-2xl">✅</div>
                <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Active</span>
              </div>
              <div className="space-y-1">
                <div className="text-2xl sm:text-3xl font-bold text-blue-600 dark:text-blue-400">{activePlans.length}</div>
                <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Active</div>
                <div className="text-xs text-gray-500 dark:text-gray-400">Available</div>
              </div>
            </div>

            <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-purple-300/50 dark:border-purple-700/50">
              <div className="flex items-center justify-between mb-3 sm:mb-4">
                <div className="text-xl sm:text-2xl">🏢</div>
                <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Enterprise</span>
              </div>
              <div className="space-y-1">
                <div className="text-2xl sm:text-3xl font-bold text-purple-600 dark:text-purple-400">{enterprisePlans.length}</div>
                <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Enterprise</div>
                <div className="text-xs text-gray-500 dark:text-gray-400">Premium</div>
              </div>
            </div>

            <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border-2 border-green-300/50 dark:border-green-700/50">
              <div className="flex items-center justify-between mb-3 sm:mb-4">
                <div className="text-xl sm:text-2xl">💵</div>
                <span className="text-xs sm:text-sm font-medium text-gray-500 dark:text-gray-400">Price</span>
              </div>
              <div className="space-y-1">
                <div className="text-xl sm:text-3xl font-bold text-green-600 dark:text-green-400 truncate">
                  ${avgRevenue.toFixed(0)}
                </div>
                <div className="text-xs sm:text-sm text-gray-600 dark:text-gray-300">Average</div>
                <div className="text-xs text-gray-500 dark:text-gray-400">USD</div>
              </div>
            </div>
          </div>
          {/* Plans Lists by Group */}
          <div className="space-y-8">

            {/* 1. Standard Plans Group */}
            <PlanGroupSection
              title="Standard Plans"
              category="standard"
              plans={standardPlans}
              onDragStart={handleDragStart}
              onDragOver={handleDragOver}
              onDrop={handleDrop}
              onDragEnd={() => { setDraggedItem(null); setDragOverIndex(null); }}
              draggedItem={draggedItem}
              dragOverIndex={dragOverIndex}
              onSelect={setSelectedPlan}
              router={router}
            />

            {/* 2. API Plans Group */}
            <PlanGroupSection
              title="API Plans"
              category="api"
              plans={apiPlans}
              onDragStart={handleDragStart}
              onDragOver={handleDragOver}
              onDrop={handleDrop}
              onDragEnd={() => { setDraggedItem(null); setDragOverIndex(null); }}
              draggedItem={draggedItem}
              dragOverIndex={dragOverIndex}
              onSelect={setSelectedPlan}
              router={router}
            />

            {/* 3. Enterprise Plans Group */}
            <PlanGroupSection
              title="Enterprise Plans"
              category="enterprise"
              plans={enterprisePlans}
              onDragStart={handleDragStart}
              onDragOver={handleDragOver}
              onDrop={handleDrop}
              onDragEnd={() => { setDraggedItem(null); setDragOverIndex(null); }}
              draggedItem={draggedItem}
              dragOverIndex={dragOverIndex}
              onSelect={setSelectedPlan}
              router={router}
            />

          </div>
        </div>
      </div>

      {/* Unsaved Changes Bar */}
      {hasChanges && (
        <div className="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 px-6 py-4 rounded-full shadow-2xl border border-gray-200 dark:border-gray-700 flex items-center gap-4 animate-in slide-in-from-bottom-10 fade-in duration-300">
          <div className="flex items-center gap-2">
            <span className="bg-amber-100 dark:bg-amber-900/30 text-amber-600 dark:text-amber-400 p-1.5 rounded-full">
              <span className="text-lg">⚠️</span>
            </span>
            <span className="font-semibold">Unsaved Tier Order</span>
          </div>
          <div className="h-6 w-px bg-gray-200 dark:bg-gray-700" />
          <div className="flex items-center gap-2">
            <button
              onClick={handleDiscardChanges}
              className="px-4 py-2 rounded-xl text-sm font-medium hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
              disabled={saving}
            >
              Discard
            </button>
            <button
              onClick={handleSaveOrder}
              disabled={saving}
              className="px-4 py-2 rounded-xl text-sm font-bold bg-gray-900 dark:bg-white text-white dark:text-gray-900 hover:bg-gray-800 dark:hover:bg-gray-100 transition-colors shadow-lg shadow-gray-900/10 flex items-center gap-2"
            >
              {saving ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Saving...
                </>
              ) : (
                <>
                  Save Changes
                </>
              )}
            </button>
          </div>
        </div>
      )}
    </div>
  )
}

export default PlanManagement;

// Helper Component for Group Sections
function PlanGroupSection({
  title, category, plans, onDragStart, onDragOver, onDrop, onDragEnd, draggedItem, dragOverIndex, onSelect, router
}: any) {
  if (plans.length === 0) return null

  // Calculate Group Stats
  const groupRevenue = plans.reduce((sum: number, p: any) => sum + (Number(p.revenue_last_30_days) || 0), 0)
  const groupSubscribers = plans.reduce((sum: number, p: any) => sum + (p.subscriber_count || 0), 0)

  return (
    <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-gray-100/50 to-gray-200/50 dark:from-gray-800/50 dark:to-gray-900/50 p-0.5">
      <div className="relative bg-white/60 dark:bg-gray-900/60 backdrop-blur-xl rounded-2xl sm:rounded-3xl p-4 sm:p-6 lg:p-8">
        {/* Header Section */}
        <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between mb-6 gap-4">
          <div className="flex flex-col gap-2">
            <div className="flex items-center gap-3">
              <div className={`h-10 w-10 rounded-xl flex items-center justify-center text-xl shadow-sm ${category === 'standard' ? 'bg-gradient-to-br from-blue-400 to-purple-500 text-white' :
                category === 'api' ? 'bg-gradient-to-br from-orange-400 to-red-500 text-white' :
                  'bg-gradient-to-br from-purple-400 to-pink-500 text-white'
                }`}>
                {category === 'standard' ? '👤' : category === 'api' ? '🔧' : '🏢'}
              </div>
              <h2 className="text-xl sm:text-2xl font-bold text-gray-900 dark:text-white">{title}</h2>
            </div>
            {/* Aggregate Stats */}
            <div className="flex items-center gap-3 pl-[52px]">
              <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-emerald-100/50 dark:bg-emerald-900/20 border border-emerald-200 dark:border-emerald-800">
                <span className="text-xs font-bold text-emerald-600 dark:text-emerald-400">${groupRevenue.toFixed(0)}</span>
                <span className="text-[10px] uppercase font-semibold text-emerald-600/70 dark:text-emerald-400/70">MRR</span>
              </div>
              <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-blue-100/50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800">
                <span className="text-xs font-bold text-blue-600 dark:text-blue-400">{groupSubscribers}</span>
                <span className="text-[10px] uppercase font-semibold text-blue-600/70 dark:text-blue-400/70">Subs</span>
              </div>
            </div>
          </div>

          <button
            className="hidden sm:flex items-center gap-2 px-4 py-2 rounded-xl text-sm font-semibold bg-white dark:bg-gray-800 shadow-sm border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 transition"
            onClick={() => router.push('/plans/new')}
          >
            <span>➕</span> New {title}
          </button>
        </div>

        <div className="space-y-4">
          {plans.map((plan: PlanResponse, index: number) => {
            // Popular Badge Logic
            const isPopular = plan.subscriber_count > 10

            // Extract simple features from permissions
            const featureTags = (plan.permissions || [])
              .slice(0, 3)
              .map(p => p.split(':').pop()?.replace(/_/g, ' ') || p)
              .map(s => s.charAt(0).toUpperCase() + s.slice(1))

            return (
              <div
                key={plan.id}
                draggable={true}
                onDragStart={(e) => onDragStart(e, plan, category)}
                onDragOver={(e) => onDragOver(e, category)}
                onDrop={(e) => onDrop(e, index, category, plans)}
                onDragEnd={onDragEnd}
                className={`group relative flex items-stretch bg-white dark:bg-gray-800 rounded-2xl border transition-all duration-300 cursor-pointer overflow-hidden ${draggedItem?.plan.id === plan.id
                  ? 'opacity-40 border-dashed border-gray-400 scale-[0.98]'
                  : 'border-gray-200 dark:border-gray-700 hover:border-emerald-300 dark:hover:border-emerald-600 hover:shadow-lg'
                  }`}
                onClick={() => router.push(`/plans/${plan.id}/edit`)}
              >
                {/* Popular Ribbon */}
                {isPopular && (
                  <div className="absolute top-0 right-0">
                    <div className="bg-gradient-to-r from-orange-500 to-rose-500 text-white text-[10px] font-bold px-3 py-1 rounded-bl-xl shadow-sm z-10">
                      🔥 POPULAR
                    </div>
                  </div>
                )}

                {/* Enhanced Grip Handle */}
                <div
                  className="flex items-center justify-center w-14 border-r border-gray-100 dark:border-gray-700 bg-gray-50/50 dark:bg-gray-800/50 cursor-grab active:cursor-grabbing text-gray-400 group-hover:text-gray-600 dark:group-hover:text-gray-200 transition-colors"
                  onClick={(e) => e.stopPropagation()}
                >
                  <GripVertical className="w-6 h-6" />
                </div>

                {/* Content */}
                <div className="flex-1 p-5 sm:p-6 flex flex-col sm:flex-row sm:items-center justify-between gap-4">

                  {/* Main Info */}
                  <div className="flex flex-col gap-1 min-w-0 flex-1">
                    <div className="flex items-center gap-3">
                      <h3 className="text-lg font-bold text-gray-900 dark:text-white truncate group-hover:text-emerald-600 dark:group-hover:text-emerald-400 transition-colors">{plan.name}</h3>
                      <span className="text-xs font-mono text-gray-400 bg-gray-100 dark:bg-gray-700/50 px-1.5 py-0.5 rounded">T{plan.tier_level}</span>
                    </div>

                    <div className="flex items-baseline gap-2">
                      <span className="text-2xl font-bold bg-gradient-to-br from-gray-900 to-gray-600 dark:from-white dark:to-gray-400 bg-clip-text text-transparent">
                        {Number(plan.current_price) === 0 ? 'Free' : `$${plan.current_price}`}
                      </span>
                      {Number(plan.current_price) > 0 && <span className="text-xs text-gray-500 uppercase">{plan.currency}</span>}
                    </div>

                    {/* Feature Tags */}
                    <div className="flex items-center gap-2 mt-2">
                      {featureTags.map((tag, i) => (
                        <span key={i} className="text-[10px] font-medium px-2 py-0.5 rounded-md bg-gray-100 dark:bg-gray-700/50 text-gray-600 dark:text-gray-300 border border-gray-200 dark:border-gray-600/50">
                          {tag}
                        </span>
                      ))}
                      {(plan.permissions?.length || 0) > 3 && (
                        <span className="text-[10px] font-medium text-gray-400">+{plan.permissions!.length - 3} more</span>
                      )}
                    </div>
                  </div>

                  {/* Stats & Actions */}
                  <div className="flex items-center gap-6 sm:pl-6 sm:border-l border-gray-100 dark:border-gray-700">
                    <div className="flex flex-col gap-3 min-w-[120px]">
                      {/* Subscriber Stat */}
                      <div className="flex items-center justify-between">
                        <span className="text-xs text-gray-500 uppercase font-bold tracking-wider">Users</span>
                        <span className="text-sm font-bold text-gray-700 dark:text-gray-200">{plan.subscriber_count}</span>
                      </div>
                      {/* Revenue Stat */}
                      <div className="flex items-center justify-between">
                        <span className="text-xs text-gray-500 uppercase font-bold tracking-wider">Rev (30d)</span>
                        <span className="text-sm font-bold text-emerald-600 dark:text-emerald-400">${plan.revenue_last_30_days}</span>
                      </div>
                    </div>

                    <button
                      className="hidden sm:block p-2 rounded-xl hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-400 hover:text-purple-500 transition-colors"
                      onClick={(e) => {
                        e.stopPropagation();
                        router.push(`/plans/${plan.id}/edit`);
                      }}
                    >
                      <span className="text-xl">✎</span>
                    </button>
                  </div>
                </div>
              </div>
            )
          })}
        </div>
      </div>
    </div>
  )
}