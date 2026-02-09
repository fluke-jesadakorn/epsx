/* eslint-disable @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-return, @typescript-eslint/no-explicit-any */
'use client'

import { GripVertical, Loader2 } from 'lucide-react'
import { usePathname, useRouter } from 'next/navigation'
import { useEffect, useState } from 'react'

import { toast } from '@/hooks/use-toast'
import { createPlansClient, isApiSuccess, type PlanResponse } from '@/shared/api/plans'
import { useSharedAuth } from '@/shared/components/auth/Provider'
import { createAdminApiClient } from '@/shared/utils/api-client'
import type {
  DragEndEvent,
  DragStartEvent} from '@dnd-kit/core';
import {
  closestCenter,
  DndContext,
  DragOverlay,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors
} from '@dnd-kit/core'
import {
  rectSwappingStrategy,
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable
} from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'

interface PlanManagementProps {
  currentUser?: any
}

/**
 *
 * @param root0
 * @param root0.currentUser
 */
function LoadingState() {
  return (
    <div className="max-w-7xl mx-auto space-y-8 animate-pulse">
      <div className="text-center mb-12">
        <div className="h-16 bg-primary/20 rounded-2xl w-96 mx-auto mb-6" />
        <div className="h-6 bg-muted rounded-full w-64 mx-auto" />
      </div>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
        {Array.from({ length: 3 }).map((_, i) => (
          <div key={i} className="bg-card border border-border rounded-3xl h-64" />
        ))}
      </div>
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
        {Array.from({ length: 4 }).map((_, i) => (
          <div key={i} className="bg-card border border-border rounded-3xl h-32" />
        ))}
      </div>
    </div>
  );
}

function HeroSection() {
  return (
    <div className="text-center mb-8 sm:mb-12">
      <div className="relative inline-block">
        <h1 className="text-3xl sm:text-4xl lg:text-5xl font-bold text-foreground mb-4">
          💳 <span className="bg-gradient-to-r from-primary to-secondary bg-clip-text text-transparent">Dynamic Plans</span>
        </h1>
        <div className="absolute -top-2 -right-2 w-4 h-4 bg-primary rounded-full animate-ping" />
      </div>
      <p className="text-base sm:text-lg text-muted-foreground max-w-2xl mx-auto">
        Create and manage unlimited plans with context-specific features for web app, API, and admin access
      </p>
    </div>
  );
}

function StatsGrid({ plans, avgRevenue }: { plans: PlanResponse[]; avgRevenue: number }) {
  const activePlans = plans.filter(p => p.is_active);
  const enterprisePlans = plans.filter(p => p.plan_category === 'enterprise' && p.is_active);

  return (
    <div className="grid grid-cols-2 sm:grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-6 mb-6 sm:mb-8">
      <div className="bg-card rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border border-primary/20">
        <div className="flex items-center justify-between mb-3 sm:mb-4">
          <div className="text-xl sm:text-2xl">💳</div>
          <span className="text-xs sm:text-sm font-medium text-muted-foreground">Total</span>
        </div>
        <div className="space-y-1">
          <div className="text-2xl sm:text-3xl font-bold text-primary">{plans.length}</div>
          <div className="text-xs sm:text-sm text-foreground/80">Plans</div>
          <div className="text-xs text-muted-foreground">All time</div>
        </div>
      </div>

      <div className="bg-card/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border border-secondary/20">
        <div className="flex items-center justify-between mb-3 sm:mb-4">
          <div className="text-xl sm:text-2xl">✅</div>
          <span className="text-xs sm:text-sm font-medium text-muted-foreground">Active</span>
        </div>
        <div className="space-y-1">
          <div className="text-2xl sm:text-3xl font-bold text-secondary">{activePlans.length}</div>
          <div className="text-xs sm:text-sm text-foreground/80">Active</div>
          <div className="text-xs text-muted-foreground">Available</div>
        </div>
      </div>

      <div className="bg-card/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border border-secondary/20">
        <div className="flex items-center justify-between mb-3 sm:mb-4">
          <div className="text-xl sm:text-2xl">🏢</div>
          <span className="text-xs sm:text-sm font-medium text-muted-foreground">Enterprise</span>
        </div>
        <div className="space-y-1">
          <div className="text-2xl sm:text-3xl font-bold text-secondary">{enterprisePlans.length}</div>
          <div className="text-xs sm:text-sm text-foreground/80">Enterprise</div>
          <div className="text-xs text-muted-foreground">Premium</div>
        </div>
      </div>

      <div className="bg-card/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-4 sm:p-6 shadow-xl border border-success/20">
        <div className="flex items-center justify-between mb-3 sm:mb-4">
          <div className="text-xl sm:text-2xl">💵</div>
          <span className="text-xs sm:text-sm font-medium text-muted-foreground">Price</span>
        </div>
        <div className="space-y-1">
          <div className="text-xl sm:text-3xl font-bold text-success truncate">
            ${avgRevenue.toFixed(2)}
          </div>
          <div className="text-xs sm:text-sm text-foreground/80">Average</div>
          <div className="text-xs text-muted-foreground">USD</div>
        </div>
      </div>
    </div>
  );
}

export function PlanManagement({ currentUser }: PlanManagementProps) {
  const { user: authUser } = useSharedAuth()
  const _user = currentUser ?? authUser
  const router = useRouter()
  const pathname = usePathname()
  const [plans, setPlans] = useState<PlanResponse[]>([])
  const [loading, setLoading] = useState(true)
  const [_selectedPlan, setSelectedPlan] = useState<PlanResponse | null>(null)
  const [hasChanges, setHasChanges] = useState(false)
  const [originalOrder, setOriginalOrder] = useState<PlanResponse[]>([])
  const [activeId, setActiveId] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 8 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates })
  );

  useEffect(() => {
    loadPlans()
  }, [])

  const loadPlans = async () => {
    try {
      setLoading(true)
      const apiClient = createAdminApiClient()
      const plansClient = createPlansClient(apiClient)
      const response = await plansClient.getPlans({ limit: 100 })

      if (isApiSuccess(response)) {
        const backendResponse = response.data as any
        const plansData = (backendResponse?.data?.plans ?? backendResponse?.plans ?? [])
          .map((p: PlanResponse) => ({ ...p, tier_level: p.tier_level ?? 0 }))
          .sort((a: PlanResponse, b: PlanResponse) => (a.tier_level ?? 0) - (b.tier_level ?? 0))

        setPlans(plansData)
        setOriginalOrder(plansData)
        setHasChanges(false)
      } else if (response.error?.includes('Unauthorized') ?? response.error?.includes('log in')) {
        console.warn('[PlanManagement] Session expired, redirecting...')
        toast({ title: "Session Expired", description: "Please log in again to continue.", variant: "destructive" })
        router.push(`/auth?returnUrl=${encodeURIComponent(pathname)}`)
      } else {
        console.error('[PlanManagement] API error:', response.error)
        toast({ title: "Error", description: response.error ?? "Failed to load plans", variant: "destructive" })
      }
    } catch (error) {
      console.error('[PlanManagement] Exception:', error)
      toast({ title: "Error", description: error instanceof Error ? error.message : "Failed to load plans", variant: "destructive" })
    } finally {
      setLoading(false)
    }
  }

  const checkForChanges = (newPlans: PlanResponse[]) => {
    const hasChanged = newPlans.some((plan, index) => plan.id !== originalOrder[index]?.id)
    setHasChanges(hasChanged)
  }

  const arraySwap = <T,>(arr: T[], index1: number, index2: number): T[] => {
    const newArr = [...arr];
    const item1 = newArr[index1];
    const item2 = newArr[index2];
    if (item1 !== undefined && item2 !== undefined) {
      newArr[index1] = item2;
      newArr[index2] = item1;
    }
    return newArr;
  };

  const handleDragStart = (event: DragStartEvent) => {
    setActiveId(event.active.id as string);
  };

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    if (!over) {
      setActiveId(null);
      return;
    }
    if (active.id !== over.id) {
      setPlans((items) => {
        const oldIndex = items.findIndex((item) => item.id === active.id);
        const newIndex = items.findIndex((item) => item.id === over.id);
        const newPlans = arraySwap(items, oldIndex, newIndex);
        checkForChanges(newPlans);
        return newPlans;
      });
    }
    setActiveId(null);
  };

  const handleSaveOrder = async () => {
    if (!hasChanges) {return;}
    setSaving(true)
    try {
      const apiClient = createAdminApiClient()
      const plansClient = createPlansClient(apiClient)
      const updates = plans.map((plan, index) => ({ plan_id: plan.id, tier_level: index }))
      await Promise.all(updates.map(u => plansClient.updatePlan(u.plan_id, { tier_level: u.tier_level })))
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

  const standardPlans = plans.filter(p => p.plan_category === 'standard' && p.is_active)
  const apiPlans = plans.filter(p => p.plan_category === 'api' && p.is_active)
  const enterprisePlans = plans.filter(p => p.plan_category === 'enterprise' && p.is_active)
  const activePlans = plans.filter(p => p.is_active)
  const totalRevenue = plans.reduce((sum, plan) => {
    const revenue = typeof plan.revenue_last_30_days === 'string' ? parseFloat(plan.revenue_last_30_days) : plan.revenue_last_30_days
    return sum + (isNaN(revenue) ? 0 : revenue)
  }, 0)
  const avgRevenue = plans.length > 0 ? totalRevenue / plans.length : 0

  if (loading) {return <LoadingState />}

  return (
    <div>
      <div className="space-y-6 sm:space-y-8">
        <div className="fixed inset-0 overflow-hidden pointer-events-none">
          <div className="absolute top-20 left-20 w-32 h-32 bg-primary/10 rounded-full blur-3xl animate-pulse" />
          <div className="absolute top-40 right-32 w-24 h-24 bg-secondary/10 rounded-full blur-2xl" />
          <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-primary/5 rounded-full blur-3xl" />
        </div>

        <div className="relative max-w-7xl mx-auto">
          <HeroSection />

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 sm:gap-6 mb-8 sm:mb-12">
            <div
              className="relative group overflow-hidden rounded-2xl sm:rounded-3xl border border-success/20 bg-success/5 p-6 sm:p-8 cursor-pointer hover:bg-success/10 transition-all duration-300 active:scale-[0.98]"
              onClick={() => router.push('/subscriptions/plans/new')}
            >
              <div className="bg-success/20 text-success rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6 group-hover:scale-110 transition-transform">
                <span className="text-xl sm:text-2xl">➕</span>
              </div>
              <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4 text-foreground">Create Dynamic Plan</h3>
              <p className="text-muted-foreground mb-4 sm:mb-6 text-sm sm:text-base">Create unlimited plans with context-specific features</p>
              <div className="bg-success text-success-foreground rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-bold text-sm sm:text-base min-h-[44px] flex items-center justify-center shadow-lg shadow-success/20">
                New Plan
              </div>
            </div>

            <div
              className="relative group overflow-hidden rounded-2xl sm:rounded-3xl border border-secondary/20 bg-secondary/5 p-6 sm:p-8 cursor-pointer hover:bg-secondary/10 transition-all duration-300 active:scale-[0.98]"
              onClick={() => loadPlans()}
            >
              <div className="bg-secondary/20 text-secondary rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6 group-hover:scale-110 transition-transform">
                <span className="text-xl sm:text-2xl">🔄</span>
              </div>
              <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4 text-foreground">Refresh Data</h3>
              <p className="text-muted-foreground mb-4 sm:mb-6 text-sm sm:text-base">Reload plan data and analytics from server</p>
              <div className="bg-secondary text-secondary-foreground rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-bold text-sm sm:text-base min-h-[44px] flex items-center justify-center shadow-lg shadow-secondary/20">
                Refresh
              </div>
            </div>
          </div>

          <StatsGrid plans={plans} avgRevenue={avgRevenue} />
          {/* Plans Lists by Group */}
          <div className="space-y-8">
            <DndContext
              sensors={sensors}
              collisionDetection={closestCenter}
              onDragStart={handleDragStart}
              onDragEnd={handleDragEnd}
            >
              {/* 1. Standard Plans Group */}
              <PlanGroupSection
                title="Standard Plans"
                category="standard"
                plans={standardPlans}
                onSelect={setSelectedPlan}
                router={router}
              />

              {/* 2. API Plans Group */}
              <PlanGroupSection
                title="API Plans"
                category="api"
                plans={apiPlans}
                onSelect={setSelectedPlan}
                router={router}
              />

              {/* 3. Enterprise Plans Group */}
              <PlanGroupSection
                title="Enterprise Plans"
                category="enterprise"
                plans={enterprisePlans}
                onSelect={setSelectedPlan}
                router={router}
              />

              <DragOverlay>
                {activeId ? (
                  <div className="opacity-80 scale-105 shadow-2xl cursor-grabbing">
                    {/* Render a static version or clone of the card for overlay */}
                    <PlanCard plan={plans.find(p => p.id === activeId)} router={router} isOverlay />
                  </div>
                ) : null}
              </DragOverlay>
            </DndContext>
          </div>
        </div>
      </div>

      {hasChanges && (
        <div className="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 bg-card text-card-foreground px-6 py-4 rounded-full shadow-2xl border border-border flex items-center gap-4 animate-in slide-in-from-bottom-10 fade-in duration-300">
          <div className="flex items-center gap-2">
            <span className="bg-warning/10 text-warning p-1.5 rounded-full">
              <span className="text-lg">⚠️</span>
            </span>
            <span className="font-semibold">Unsaved Tier Order</span>
          </div>
          <div className="h-6 w-px bg-border" />
          <div className="flex items-center gap-2">
            <button
              onClick={handleDiscardChanges}
              className="px-4 py-2 rounded-xl text-sm font-medium hover:bg-muted transition-colors"
              disabled={saving}
            >
              Discard
            </button>
            <button
              onClick={handleSaveOrder}
              disabled={saving}
              className="px-4 py-2 rounded-xl text-sm font-bold bg-primary text-primary-foreground hover:opacity-90 transition-colors shadow-lg shadow-primary/10 flex items-center gap-2"
            >
              {saving ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Saving...
                </>
              ) : (
                'Save Changes'
              )}
            </button>
          </div>
        </div>
      )}
    </div>
  )
}

export default PlanManagement;

// Sortable Item Wrapper
function SortablePlanItem({ plan, router }: { plan: PlanResponse, router: any }) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging
  } = useSortable({ id: plan.id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.3 : 1,
    zIndex: isDragging ? 50 : 'auto',
  };

  return (
    <div ref={setNodeRef} style={style} {...attributes} {...listeners} className="touch-none">
      <PlanCard plan={plan} router={router} />
    </div>
  )
}

// Extracted Plan Card Component for reuse
function PlanCard({ plan, router, isOverlay }: { plan: PlanResponse, router: any, isOverlay?: boolean }) {
  // Popular Badge Logic
  const isPopular = plan.subscriber_count > 10

  // Extract simple features from permissions
  const featureTags = (plan.permissions ?? [])
    .slice(0, 3)
    .map(p => p.split(':').pop()?.replace(/_/g, ' ') ?? p)
    .map(s => s.charAt(0).toUpperCase() + s.slice(1))

  return (
    <div
      className={`group relative flex items-stretch bg-card text-card-foreground rounded-2xl border transition-all duration-300 cursor-pointer overflow-hidden ${isOverlay ? 'border-primary shadow-2xl scale-105 opacity-80' : 'border-border hover:border-primary/50 hover:shadow-lg'
        }`}
      onClick={() => router.push(`/subscriptions/plans/${plan.id}/edit`)}
    >
      {/* Popular Ribbon */}
      {isPopular && (
        <div className="absolute top-0 right-0">
          <div className="bg-primary text-primary-foreground text-[10px] font-bold px-3 py-1 rounded-bl-xl shadow-sm z-10 transition-transform group-hover:scale-110">
            🔥 POPULAR
          </div>
        </div>
      )}

      {/* Enhanced Grip Handle */}
      <div
        className="flex items-center justify-center w-14 border-r border-border bg-muted/30 cursor-grab active:cursor-grabbing text-muted-foreground group-hover:text-foreground transition-colors"
        onClick={(e) => e.stopPropagation()}
      >
        <GripVertical className="w-6 h-6" />
      </div>

      {/* Content */}
      <div className="flex-1 p-5 sm:p-6 flex flex-col sm:flex-row sm:items-center justify-between gap-4">

        {/* Main Info */}
        <div className="flex flex-col gap-1 min-w-0 flex-1">
          <div className="flex items-center gap-3">
            <h3 className="text-lg font-bold text-foreground truncate group-hover:text-primary transition-colors">{plan.name}</h3>
            <span className="text-xs font-mono text-muted-foreground bg-muted px-1.5 py-0.5 rounded">T{plan.tier_level}</span>
          </div>

          <div className="flex items-baseline gap-2">
            <span className="text-2xl font-bold text-foreground">
              {Number(plan.current_price) === 0 ? 'Free' : `$${plan.current_price}`}
            </span>
            {Number(plan.current_price) > 0 && <span className="text-xs text-muted-foreground uppercase">{plan.currency}</span>}
          </div>

          {/* Feature Tags */}
          <div className="flex items-center gap-2 mt-2">
            {featureTags.map((tag, i) => (
              <span key={i} className="text-[10px] font-medium px-2 py-0.5 rounded-md bg-muted text-muted-foreground border border-border">
                {tag}
              </span>
            ))}
            {(plan.permissions?.length ?? 0) > 3 && (
              <span className="text-[10px] font-medium text-muted-foreground">+{plan.permissions!.length - 3} more</span>
            )}
          </div>
        </div>

        {/* Stats & Actions */}
        <div className="flex items-center gap-6 sm:pl-6 sm:border-l border-border">
          <div className="flex flex-col gap-3 min-w-[120px]">
            {/* Subscriber Stat */}
            <div className="flex items-center justify-between">
              <span className="text-xs text-muted-foreground uppercase font-bold tracking-wider">Users</span>
              <span className="text-sm font-bold text-foreground">{plan.subscriber_count}</span>
            </div>
            {/* Revenue Stat */}
            <div className="flex items-center justify-between">
              <span className="text-xs text-muted-foreground uppercase font-bold tracking-wider">Rev (30d)</span>
              <span className="text-sm font-bold text-success">${Number(plan.revenue_last_30_days).toFixed(2)}</span>
            </div>
          </div>

          <button
            className="hidden sm:block p-2 rounded-xl hover:bg-muted text-muted-foreground hover:text-primary transition-colors"
            onClick={(e) => {
              e.stopPropagation();
              router.push(`/subscriptions/plans/${plan.id}/edit`);
            }}
          >
            <span className="text-xl">✎</span>
          </button>
        </div>
      </div>
    </div>
  )
}

// Helper Component for Group Sections
function PlanGroupSection({
  title, category, plans, onSelect, router
}: any) {
  if (plans.length === 0) {return null}

  // Calculate Group Stats
  const groupRevenue = plans.reduce((sum: number, p: any) => sum + (Number(p.revenue_last_30_days) ?? 0), 0)
  const groupSubscribers = plans.reduce((sum: number, p: any) => sum + (p.subscriber_count ?? 0), 0)

  return (
    <SortableContext
      items={plans.map((p: any) => p.id)}
      strategy={rectSwappingStrategy} // or verticalListSortingStrategy
    >
      <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-border/20 p-0.5">
        <div className="relative bg-card rounded-2xl sm:rounded-3xl p-4 sm:p-6 lg:p-8">
          {/* Header Section */}
          <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between mb-6 gap-4">
            <div className="flex flex-col gap-2">
              <div className="flex items-center gap-3">
                <div className={`h-10 w-10 rounded-xl flex items-center justify-center text-xl shadow-sm ${category === 'standard' ? 'bg-primary text-primary-foreground' :
                  category === 'api' ? 'bg-secondary text-secondary-foreground' :
                    'bg-secondary text-secondary-foreground'
                  }`}>
                  {category === 'standard' ? '👤' : category === 'api' ? '🔧' : '🏢'}
                </div>
                <h2 className="text-xl sm:text-2xl font-bold text-foreground">{title}</h2>
              </div>
              {/* Aggregate Stats */}
              <div className="flex items-center gap-3 pl-[52px]">
                <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-success/10 border border-success/20">
                  <span className="text-xs font-bold text-success">${groupRevenue.toFixed(2)}</span>
                  <span className="text-[10px] uppercase font-semibold text-success/70">MRR</span>
                </div>
                <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-primary/10 border border-primary/20">
                  <span className="text-xs font-bold text-primary">{groupSubscribers}</span>
                  <span className="text-[10px] uppercase font-semibold text-primary/70">Subs</span>
                </div>
              </div>
            </div>

            <button
              className="hidden sm:flex items-center gap-2 px-4 py-2 rounded-xl text-sm font-semibold bg-card text-card-foreground shadow-sm border border-border hover:bg-muted transition"
              onClick={() => router.push('/subscriptions/plans/new')}
            >
              <span>➕</span> New {title}
            </button>
          </div>

          <div className="space-y-4">
            {plans.map((plan: PlanResponse, index: number) => (
              <SortablePlanItem key={plan.id} plan={plan} router={router} />
            ))}
          </div>
        </div>
      </div>
    </SortableContext>
  )
}