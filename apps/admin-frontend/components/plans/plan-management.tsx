'use client'

import { GripVertical, Loader2 } from 'lucide-react'
import { useCallback, useEffect, useState } from 'react'
import { usePathname, useRouter } from 'next/navigation'

import { toast } from '@/hooks/use-toast'
import { createPlansClient, isApiSuccess, type Plan } from '@/shared/api/plans'
import { useSharedAuth } from '@/shared/components/auth'
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

// Extended plan type with admin-only fields from backend
interface AdminPlan extends Plan {
  tier_level: number;
  subscriber_count: number;
  revenue_last_30_days: string | number;
  plan_category?: string;
  currency?: string;
}

interface PlanManagementProps {
  currentUser?: Record<string, unknown>
}

function LoadingState() {
  return (
    <div className="max-w-7xl mx-auto space-y-8 animate-pulse">
      <div className="text-center mb-12">
        <div className="h-16 bg-primary/20 rounded-2xl w-96 mx-auto mb-6" />
        <div className="h-6 bg-muted rounded-full w-64 mx-auto" />
      </div>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
        {(['a', 'b', 'c'] as const).map(k => (
          <div key={k} className="bg-card border border-border rounded-3xl h-64" />
        ))}
      </div>
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
        {(['a', 'b', 'c', 'd'] as const).map(k => (
          <div key={k} className="bg-card border border-border rounded-3xl h-32" />
        ))}
      </div>
    </div>
  );
}

function StatsGrid({ plans, avgRevenue }: { plans: AdminPlan[]; avgRevenue: number }) {
  const activePlans = plans.filter(p => p.is_active);
  const enterprisePlans = plans.filter(p => p.plan_category === 'enterprise' && p.is_active);
  return (
    <div className="grid grid-cols-2 sm:grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-6 mb-6 sm:mb-8">
      <div className="bg-card rounded-2xl p-4 sm:p-6 shadow-xl border border-primary/20">
        <div className="flex items-center justify-between mb-3 sm:mb-4"><div className="text-xl sm:text-2xl">💳</div><span className="text-xs sm:text-sm font-medium text-muted-foreground">Total</span></div>
        <div className="space-y-1"><div className="text-2xl sm:text-3xl font-bold text-primary">{plans.length}</div><div className="text-xs sm:text-sm text-foreground/80">Plans</div><div className="text-xs text-muted-foreground">All time</div></div>
      </div>
      <div className="bg-card/80 rounded-2xl p-4 sm:p-6 shadow-xl border border-secondary/20">
        <div className="flex items-center justify-between mb-3 sm:mb-4"><div className="text-xl sm:text-2xl">✅</div><span className="text-xs sm:text-sm font-medium text-muted-foreground">Active</span></div>
        <div className="space-y-1"><div className="text-2xl sm:text-3xl font-bold text-secondary">{activePlans.length}</div><div className="text-xs sm:text-sm text-foreground/80">Active</div><div className="text-xs text-muted-foreground">Available</div></div>
      </div>
      <div className="bg-card/80 rounded-2xl p-4 sm:p-6 shadow-xl border border-secondary/20">
        <div className="flex items-center justify-between mb-3 sm:mb-4"><div className="text-xl sm:text-2xl">🏢</div><span className="text-xs sm:text-sm font-medium text-muted-foreground">Enterprise</span></div>
        <div className="space-y-1"><div className="text-2xl sm:text-3xl font-bold text-secondary">{enterprisePlans.length}</div><div className="text-xs sm:text-sm text-foreground/80">Enterprise</div><div className="text-xs text-muted-foreground">Premium</div></div>
      </div>
      <div className="bg-card/80 rounded-2xl p-4 sm:p-6 shadow-xl border border-success/20">
        <div className="flex items-center justify-between mb-3 sm:mb-4"><div className="text-xl sm:text-2xl">💵</div><span className="text-xs sm:text-sm font-medium text-muted-foreground">Price</span></div>
        <div className="space-y-1"><div className="text-xl sm:text-3xl font-bold text-success truncate">${avgRevenue.toFixed(2)}</div><div className="text-xs sm:text-sm text-foreground/80">Average</div><div className="text-xs text-muted-foreground">USD</div></div>
      </div>
    </div>
  );
}

async function fetchPlans(router: ReturnType<typeof useRouter>, pathname: string): Promise<AdminPlan[] | null> {
  const apiClient = createAdminApiClient()
  const plansClient = createPlansClient(apiClient)
  const response = await plansClient.listPlans({ limit: 100 })
  if (isApiSuccess(response)) {
    const raw = response.data.data as (Plan & { tier_level?: number; subscriber_count?: number; revenue_last_30_days?: string | number; plan_category?: string; currency?: string })[]
    return raw
      .map(p => ({ ...p, tier_level: p.tier_level ?? 0, subscriber_count: p.subscriber_count ?? 0, revenue_last_30_days: p.revenue_last_30_days ?? 0 }) as AdminPlan)
      .sort((a, b) => a.tier_level - b.tier_level)
  }
  const errMsg = response.error?.message
  if (errMsg !== undefined && (errMsg.includes('Unauthorized') || errMsg.includes('log in'))) {
    toast({ title: "Session Expired", description: "Please log in again to continue.", variant: "destructive" })
    return null
  }
  toast({ title: "Error", description: response.error?.message ?? "Failed to load plans", variant: "destructive" })
  return null
}

function arraySwap<T>(arr: T[], i1: number, i2: number): T[] {
  const out = [...arr];
  const a = out[i1];
  const b = out[i2];
  if (a !== undefined && b !== undefined) { out[i1] = b; out[i2] = a; }
  return out;
}

export function PlanManagement({ currentUser }: PlanManagementProps) {
  const { user: authUser } = useSharedAuth()
  const _user = currentUser ?? authUser
  const router = useRouter()
  const pathname = usePathname()
  const [plans, setPlans] = useState<AdminPlan[]>([])
  const [loading, setLoading] = useState(true)
  const [hasChanges, setHasChanges] = useState(false)
  const [originalOrder, setOriginalOrder] = useState<AdminPlan[]>([])
  const [activeId, setActiveId] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 8 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates })
  );

  const loadPlans = useCallback(async () => {
    try {
      setLoading(true)
      const data = await fetchPlans(router, pathname)
      if (data !== null) { setPlans(data); setOriginalOrder(data); setHasChanges(false); }
    } catch (error) {
      toast({ title: "Error", description: error instanceof Error ? error.message : "Failed to load plans", variant: "destructive" })
    } finally {
      setLoading(false)
    }
  }, [router, pathname]);

  useEffect(() => { void loadPlans(); }, [loadPlans])

  const checkForChanges = (newPlans: AdminPlan[]) => {
    setHasChanges(newPlans.some((plan, index) => plan.id !== originalOrder[index]?.id))
  }

  const handleDragStart = (event: DragStartEvent) => { setActiveId(event.active.id as string); };

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    if (over === null) { setActiveId(null); return; }
    if (active.id !== over.id) {
      setPlans((items) => {
        const newPlans = arraySwap(items, items.findIndex(i => i.id === active.id), items.findIndex(i => i.id === over.id));
        checkForChanges(newPlans);
        return newPlans;
      });
    }
    setActiveId(null);
  };

  const handleSaveOrder = async () => {
    if (!hasChanges) { return; }
    setSaving(true)
    try {
      const apiClient = createAdminApiClient()
      const plansClient = createPlansClient(apiClient)
      await Promise.all(plans.map((plan, index) => plansClient.updatePlan(plan.id, { tier_level: index })))
      toast({ title: 'Tier order saved successfully', description: 'Plan hierarchy has been updated.' })
      setOriginalOrder([...plans])
      setHasChanges(false)
    } catch (_error) {
      toast({ title: 'Failed to save tier order', variant: 'destructive' })
    } finally {
      setSaving(false)
    }
  }

  const totalRevenue = plans.reduce((sum, plan) => {
    const r = parseFloat(String(plan.revenue_last_30_days))
    return sum + (isNaN(r) ? 0 : r)
  }, 0)
  const avgRevenue = plans.length > 0 ? totalRevenue / plans.length : 0
  const standardPlans = plans.filter(p => p.plan_category === 'standard' && p.is_active)
  const apiPlans = plans.filter(p => p.plan_category === 'api' && p.is_active)
  const enterprisePlans = plans.filter(p => p.plan_category === 'enterprise' && p.is_active)

  if (loading) { return <LoadingState />; }

  return (
    <div>
      <div className="space-y-6 sm:space-y-8">
        <div className="relative max-w-7xl mx-auto">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 sm:gap-6 mb-8 sm:mb-12">
            <ActionCard title="Create Dynamic Plan" desc="Create unlimited plans with context-specific features" label="New Plan" color="success" onClick={() => router.push('/wallet-management/access/plans')} icon="➕" />
            <ActionCard title="Refresh Data" desc="Reload plan data and analytics from server" label="Refresh" color="secondary" onClick={() => void loadPlans()} icon="🔄" />
          </div>
          <StatsGrid plans={plans} avgRevenue={avgRevenue} />
          <div className="space-y-8">
            <DndContext sensors={sensors} collisionDetection={closestCenter} onDragStart={handleDragStart} onDragEnd={handleDragEnd}>
              <PlanGroupSection title="Standard Plans" category="standard" plans={standardPlans} router={router} />
              <PlanGroupSection title="API Plans" category="api" plans={apiPlans} router={router} />
              <PlanGroupSection title="Enterprise Plans" category="enterprise" plans={enterprisePlans} router={router} />
              <DragOverlay>
                {activeId !== null ? (() => {
                  const overlayPlan = plans.find(p => p.id === activeId);
                  return overlayPlan !== undefined ? <div className="opacity-80 scale-105 shadow-2xl cursor-grabbing"><PlanCard plan={overlayPlan} router={router} isOverlay /></div> : null;
                })() : null}
              </DragOverlay>
            </DndContext>
          </div>
        </div>
      </div>
      {hasChanges && (
        <div className="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 bg-card text-card-foreground px-6 py-4 rounded-full shadow-2xl border border-border flex items-center gap-4 animate-in slide-in-from-bottom-10 fade-in duration-300">
          <div className="flex items-center gap-2">
            <span className="bg-warning/10 text-warning p-1.5 rounded-full"><span className="text-lg">⚠️</span></span>
            <span className="font-semibold">Unsaved Tier Order</span>
          </div>
          <div className="h-6 w-px bg-border" />
          <div className="flex items-center gap-2">
            <button onClick={() => { setPlans([...originalOrder]); setHasChanges(false); }} className="px-4 py-2 rounded-xl text-sm font-medium hover:bg-muted transition-colors" disabled={saving}>Discard</button>
            <button onClick={() => void handleSaveOrder()} disabled={saving} className="px-4 py-2 rounded-xl text-sm font-bold bg-primary text-primary-foreground hover:opacity-90 transition-colors shadow-lg shadow-primary/10 flex items-center gap-2">
              {saving ? <><Loader2 className="h-4 w-4 animate-spin" />Saving...</> : 'Save Changes'}
            </button>
          </div>
        </div>
      )}
    </div>
  )
}

export default PlanManagement;

function ActionCard({ title, desc, label, color, onClick, icon }: { title: string; desc: string; label: string; color: 'success' | 'secondary'; onClick: () => void; icon: string }) {
  return (
    <div className={`relative group overflow-hidden rounded-2xl border border-${color}/20 bg-${color}/5 p-6 sm:p-8 cursor-pointer hover:bg-${color}/10 transition-all duration-300 active:scale-[0.98]`} onClick={onClick}>
      <div className={`bg-${color}/20 text-${color} rounded-2xl w-12 h-12 flex items-center justify-center mb-4 sm:mb-6 group-hover:scale-110 transition-transform`}><span className="text-xl sm:text-2xl">{icon}</span></div>
      <h3 className="text-xl sm:text-2xl font-bold mb-3 sm:mb-4 text-foreground">{title}</h3>
      <p className="text-muted-foreground mb-4 sm:mb-6 text-sm sm:text-base">{desc}</p>
      <div className={`bg-${color} text-${color}-foreground rounded-2xl px-4 sm:px-6 py-2 sm:py-3 text-center font-bold text-sm sm:text-base min-h-[44px] flex items-center justify-center shadow-lg shadow-${color}/20`}>{label}</div>
    </div>
  );
}

function SortablePlanItem({ plan, router }: { plan: AdminPlan, router: ReturnType<typeof useRouter> }) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({ id: plan.id });
  return (
    <div ref={setNodeRef} style={{ transform: CSS.Transform.toString(transform), transition, opacity: isDragging ? 0.3 : 1, zIndex: isDragging ? 50 : 'auto' }} {...attributes} {...listeners} className="touch-none">
      <PlanCard plan={plan} router={router} />
    </div>
  )
}

function PlanCard({ plan, router, isOverlay }: { plan: AdminPlan, router: ReturnType<typeof useRouter>, isOverlay?: boolean }) {
  const isPopular = plan.subscriber_count > 10
  const featureTags = plan.permissions.slice(0, 3).map(p => p.split(':').pop()?.replace(/_/g, ' ') ?? p).map(s => s.charAt(0).toUpperCase() + s.slice(1))
  return (
    <div className={`group relative flex items-stretch bg-card text-card-foreground rounded-2xl border transition-all duration-300 cursor-pointer overflow-hidden ${isOverlay === true ? 'border-primary shadow-2xl scale-105 opacity-80' : 'border-border hover:border-primary/50 hover:shadow-lg'}`} onClick={() => router.push(`/wallet-management/access/plans/${plan.id}`)}>
      {isPopular && <div className="absolute top-0 right-0"><div className="bg-primary text-primary-foreground text-[10px] font-bold px-3 py-1 rounded-bl-xl shadow-sm z-10 transition-transform group-hover:scale-110">🔥 POPULAR</div></div>}
      <div className="flex items-center justify-center w-14 border-r border-border bg-muted/30 cursor-grab active:cursor-grabbing text-muted-foreground group-hover:text-foreground transition-colors" onClick={(e) => e.stopPropagation()}>
        <GripVertical className="w-6 h-6" />
      </div>
      <div className="flex-1 p-5 sm:p-6 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div className="flex flex-col gap-1 min-w-0 flex-1">
          <div className="flex items-center gap-3">
            <h3 className="text-lg font-bold text-foreground truncate group-hover:text-primary transition-colors">{plan.name}</h3>
            <span className="text-xs font-mono text-muted-foreground bg-muted px-1.5 py-0.5 rounded">T{plan.tier_level}</span>
          </div>
          <div className="flex items-baseline gap-2">
            <span className="text-2xl font-bold text-foreground">{Number(plan.current_price) === 0 ? 'Free' : `$${plan.current_price}`}</span>
            {Number(plan.current_price) > 0 && <span className="text-xs text-muted-foreground uppercase">{plan.currency}</span>}
          </div>
          <div className="flex items-center gap-2 mt-2">
            {featureTags.map(tag => <span key={tag} className="text-[10px] font-medium px-2 py-0.5 rounded-md bg-muted text-muted-foreground border border-border">{tag}</span>)}
            {plan.permissions.length > 3 && <span className="text-[10px] font-medium text-muted-foreground">+{plan.permissions.length - 3} more</span>}
          </div>
        </div>
        <div className="flex items-center gap-6 sm:pl-6 sm:border-l border-border">
          <div className="flex flex-col gap-3 min-w-[120px]">
            <div className="flex items-center justify-between"><span className="text-xs text-muted-foreground uppercase font-bold tracking-wider">Users</span><span className="text-sm font-bold text-foreground">{plan.subscriber_count}</span></div>
            <div className="flex items-center justify-between"><span className="text-xs text-muted-foreground uppercase font-bold tracking-wider">Rev (30d)</span><span className="text-sm font-bold text-success">${Number(plan.revenue_last_30_days).toFixed(2)}</span></div>
          </div>
          <button className="hidden sm:block p-2 rounded-xl hover:bg-muted text-muted-foreground hover:text-primary transition-colors" onClick={(e) => { e.stopPropagation(); router.push(`/wallet-management/access/plans/${plan.id}`); }}><span className="text-xl">✎</span></button>
        </div>
      </div>
    </div>
  )
}

interface PlanGroupSectionProps {
  title: string; category: string; plans: AdminPlan[];
  router: ReturnType<typeof useRouter>;
}

function PlanGroupSection({ title, category, plans, router }: PlanGroupSectionProps) {
  if (plans.length === 0) { return null; }
  const groupRevenue = plans.reduce((sum, p) => sum + (Number(p.revenue_last_30_days) || 0), 0)
  const groupSubscribers = plans.reduce((sum, p) => sum + p.subscriber_count, 0)
  return (
    <SortableContext items={plans.map(p => p.id)} strategy={rectSwappingStrategy}>
      <div className="relative overflow-hidden rounded-2xl bg-border/20 p-0.5">
        <div className="relative bg-card rounded-2xl p-4 sm:p-6 lg:p-8">
          <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between mb-6 gap-4">
            <div className="flex flex-col gap-2">
              <div className="flex items-center gap-3">
                <div className={`h-10 w-10 rounded-xl flex items-center justify-center text-xl shadow-sm ${category === 'standard' ? 'bg-primary text-primary-foreground' : 'bg-secondary text-secondary-foreground'}`}>
                  {category === 'standard' ? '👤' : category === 'api' ? '🔧' : '🏢'}
                </div>
                <h2 className="text-xl sm:text-2xl font-bold text-foreground">{title}</h2>
              </div>
              <div className="flex items-center gap-3 pl-[52px]">
                <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-success/10 border border-success/20"><span className="text-xs font-bold text-success">${groupRevenue.toFixed(2)}</span><span className="text-[10px] uppercase font-semibold text-success/70">MRR</span></div>
                <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-primary/10 border border-primary/20"><span className="text-xs font-bold text-primary">{groupSubscribers}</span><span className="text-[10px] uppercase font-semibold text-primary/70">Subs</span></div>
              </div>
            </div>
            <button className="hidden sm:flex items-center gap-2 px-4 py-2 rounded-xl text-sm font-semibold bg-card text-card-foreground shadow-sm border border-border hover:bg-muted transition" onClick={() => router.push('/wallet-management/access/plans')}>
              <span>➕</span> New {title}
            </button>
          </div>
          <div className="space-y-4">{plans.map(plan => <SortablePlanItem key={plan.id} plan={plan} router={router} />)}</div>
        </div>
      </div>
    </SortableContext>
  )
}
