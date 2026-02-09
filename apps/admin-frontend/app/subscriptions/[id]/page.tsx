'use client'

import {
    ArrowLeft,
    Calendar,
    CheckCircle2,
    Clock,
    Edit3,
    Hash,
    RefreshCw,
    Shield,
    ShieldCheck,
    Trash2,
    User,
} from 'lucide-react'
import Link from 'next/link'
import { useParams, useRouter } from 'next/navigation'
import { useCallback, useEffect, useMemo, useState } from 'react'

import { useToast } from '@/hooks/use-toast'
import { logger } from '@/lib/logger'
import {
    createPlansClient,
    isApiSuccess,
    type Plan,
    type SubscriptionResponse,
    type UpdateSubscriptionRequest,
} from '@/shared/api/plans'
import { Badge, Button as BaseButton, Card as BaseCard } from '@/shared/components/ui'
import { createAdminApiClient } from '@/shared/utils/api-client'

const SUBSCRIPTIONS_ROUTE = '/subscriptions' as const;

interface TimelineItemProps {
    label: string
    value: string
}

interface InfoItemProps {
    icon: React.ReactNode
    label: string
    value: string
    mono?: boolean
}

/**
 * Custom hook for subscription data fetching
 */
function useSubscriptionFetchers(subscriptionId: string) {
    const { toast } = useToast()
    const router = useRouter()
    const [subscription, setSubscription] = useState<SubscriptionResponse | null>(null)
    const [loading, setLoading] = useState(true)
    const [plans, setPlans] = useState<Plan[]>([])

    const loadSubscription = useCallback(async () => {
        if (!subscriptionId) {
            return
        }
        const adminClient = createPlansClient(createAdminApiClient())
        try {
            setLoading(true)
            const response = await adminClient.getSubscription(subscriptionId)
            if (isApiSuccess(response)) {
                setSubscription(response.data)
            } else {
                toast({
                    title: 'Error',
                    description: response.error?.message ?? 'Subscription not found',
                    variant: 'destructive',
                })
                router.push(SUBSCRIPTIONS_ROUTE)
            }
        } catch (error) {
            logger.error('Failed to load subscription', { error, subscriptionId })
            toast({
                title: 'Error',
                description: 'Failed to load subscription data',
                variant: 'destructive',
            })
            router.push(SUBSCRIPTIONS_ROUTE)
        } finally {
            setLoading(false)
        }
    }, [subscriptionId, router, toast])

    const loadPlans = useCallback(async () => {
        const adminClient = createPlansClient(createAdminApiClient())
        try {
            const response = await adminClient.listPlans({ is_active: true })
            if (isApiSuccess(response)) {
                setPlans(response.data.data)
            }
        } catch (error) {
            logger.error('Failed to load plans', { error })
        }
    }, [])

    useEffect(() => {
        void loadSubscription()
        void loadPlans()
    }, [loadSubscription, loadPlans])

    return { subscription, loading, plans, loadSubscription }
}

/**
 * Custom hook for subscription detail logic
 */
function useSubscriptionDetails(subscriptionId: string) {
    const { toast } = useToast()
    const router = useRouter()
    const { subscription, loading, plans, loadSubscription } = useSubscriptionFetchers(subscriptionId)

    const [isEditing, setIsEditing] = useState(false)
    const [saving, setSaving] = useState(false)
    const [editData, setEditData] = useState<UpdateSubscriptionRequest>({
        status: 'active',
        plan_id: '',
        expires_at: '',
        auto_renew: false,
    })

    useEffect(() => {
        if (subscription) {
            setEditData({
                status: subscription.status,
                plan_id: subscription.plan_id,
                expires_at: subscription.expires_at?.split('T')[0] ?? '',
                auto_renew: subscription.auto_renew,
            })
        }
    }, [subscription])

    const handleUpdate = async () => {
        const adminClient = createPlansClient(createAdminApiClient())
        try {
            setSaving(true)
            const response = await adminClient.updateSubscription(subscriptionId, {
                ...editData,
                expires_at: editData.expires_at ?? undefined,
            })

            if (isApiSuccess(response)) {
                setIsEditing(false)
                void loadSubscription()
                toast({ title: 'Success', description: 'Subscription updated successfully' })
            } else {
                toast({
                    title: 'Error',
                    description: response.error?.message ?? 'Failed to update subscription',
                    variant: 'destructive',
                })
            }
        } catch (error) {
            logger.error('Error updating subscription', { error, subscriptionId })
            toast({
                title: 'Error',
                description: 'Failed to update subscription',
                variant: 'destructive',
            })
        } finally {
            setSaving(false)
        }
    }

    const handleCancel = async () => {
        // eslint-disable-next-line no-alert
        if (!window.confirm('Are you sure you want to cancel this subscription?')) {
            return
        }
        const adminClient = createPlansClient(createAdminApiClient())
        try {
            setSaving(true)
            const response = await adminClient.cancelSubscription(subscriptionId)
            if (isApiSuccess(response)) {
                toast({ title: 'Success', description: 'Subscription cancelled successfully' })
                router.push(SUBSCRIPTIONS_ROUTE)
            } else {
                toast({
                    title: 'Error',
                    description: response.error?.message ?? 'Failed to cancel subscription',
                    variant: 'destructive',
                })
            }
        } catch (error) {
            logger.error('Error cancelling subscription', { error, subscriptionId })
            toast({
                title: 'Error',
                description: 'An unexpected error occurred during cancellation',
                variant: 'destructive',
            })
        } finally {
            setSaving(false)
        }
    }

    const statusBadge = useMemo(() => {
        if (!subscription) {
            return null
        }
        const status = subscription.status.toLowerCase()
        const variants: Record<string, 'secondary' | 'outline' | 'destructive'> = {
            active: 'secondary',
            expired: 'outline',
            cancelled: 'destructive',
        }
        return <Badge variant={variants[status] ?? 'outline'}>{status.toUpperCase()}</Badge>
    }, [subscription])

    return {
        subscription,
        loading,
        isEditing,
        setIsEditing,
        saving,
        plans,
        editData,
        setEditData,
        handleUpdate,
        handleCancel,
        statusBadge,
    }
}

/**
 * Subscription Detail Page Component
 */
// eslint-disable-next-line max-lines-per-function
export default function SubscriptionDetailPage() {
    const params = useParams()
    const router = useRouter()
    const subscriptionId = params['id'] as string

    const {
        subscription,
        loading,
        isEditing,
        setIsEditing,
        saving,
        plans,
        editData,
        setEditData,
        handleUpdate,
        handleCancel,
        statusBadge,
    } = useSubscriptionDetails(subscriptionId)

    const formatDate = (dateString?: string) => {
        if (dateString === undefined || dateString === '') {
            return 'Never'
        }
        try {
            return new Date(dateString).toLocaleString()
        } catch (error) {
            logger.warn('Failed to format date', { dateString, error })
            return 'Invalid Date'
        }
    }

    const formatUsage = (usage?: Record<string, unknown>) => {
        if (usage === undefined || Object.keys(usage).length === 0) {
            return 'No usage data'
        }
        return Object.entries(usage)
            .map(([key, value]) => `${key}: ${String(value)}`)
            .join(', ')
    }

    if (loading) {
        return (
            <div className="min-h-screen p-6">
                <div className="max-w-5xl mx-auto">
                    <div className="animate-pulse space-y-6">
                        <div className="h-8 bg-primary/20 rounded-2xl w-1/3" />
                        <div className="h-64 bg-card rounded-3xl border border-border/50" />
                    </div>
                </div>
            </div>
        )
    }

    if (!subscription) {
        return (
            <div className="min-h-screen p-6 flex items-center justify-center">
                <div className="text-center space-y-4">
                    <h2 className="text-xl font-semibold">Subscription not found</h2>
                    <BaseButton onClick={() => { router.push(SUBSCRIPTIONS_ROUTE) }}>
                        Back to Subscriptions
                    </BaseButton>
                </div>
            </div>
        )
    }

    return (
        <div className="min-h-screen p-6 bg-background">
            <div className="max-w-5xl mx-auto space-y-6">
                {/* Header */}
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-4">
                        <Link
                            href={SUBSCRIPTIONS_ROUTE}
                            className="p-2 rounded-xl bg-card border border-border/50 hover:border-primary/50 transition-colors"
                        >
                            <ArrowLeft className="h-5 w-5" />
                        </Link>
                        <div>
                            <h1 className="text-2xl font-bold bg-gradient-to-r from-primary to-secondary bg-clip-text text-transparent">
                                Subscription Management
                            </h1>
                            <div className="flex items-center gap-3 mt-1">
                                <span className="text-lg font-semibold text-foreground">
                                    {subscription.plan_name}
                                </span>
                                {statusBadge}
                            </div>
                        </div>
                    </div>
                    <div className="flex gap-3">
                        {!isEditing && subscription.status !== 'cancelled' && (
                            <BaseButton onClick={() => { setIsEditing(true) }} variant="secondary">
                                <Edit3 className="h-4 w-4 mr-2" />
                                Edit
                            </BaseButton>
                        )}
                    </div>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                    <IdentityInfoCard subscription={subscription} />
                    <TimelineCard
                        subscription={subscription}
                        formatDate={formatDate}
                    />
                </div>

                <UsageCard
                    subscription={subscription}
                    formatUsage={formatUsage}
                />

                {isEditing && (
                    <AdminOverridesCard
                        editData={editData}
                        setEditData={setEditData}
                        plans={plans}
                        saving={saving}
                        onSave={handleUpdate}
                        onCancel={() => { setIsEditing(false) }}
                    />
                )}

                {subscription.metadata && Object.keys(subscription.metadata).length > 0 && (
                    <MetadataCard metadata={subscription.metadata} />
                )}

                <div className="flex gap-4 pt-4">
                    <Link href={SUBSCRIPTIONS_ROUTE} className="flex-1">
                        <BaseButton variant="outline" className="w-full">
                            Return to Registry
                        </BaseButton>
                    </Link>

                    {subscription.status === 'active' && !isEditing && (
                        <BaseButton
                            variant="destructive"
                            className="flex-1"
                            onClick={() => { void handleCancel() }}
                            disabled={saving}
                        >
                            <Trash2 className="h-4 w-4 mr-2" />
                            {saving ? 'Processing...' : 'Terminate Subscription'}
                        </BaseButton>
                    )}
                </div>
            </div>
        </div>
    )
}

function IdentityInfoCard({ subscription }: { subscription: SubscriptionResponse }) {
    return (
        <BaseCard className="md:col-span-2">
            <div className="flex items-center gap-2 mb-6">
                <Shield className="h-5 w-5 text-primary" />
                <h3 className="font-bold text-lg">Identity & Context</h3>
            </div>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-y-6 gap-x-8">
                <InfoItem icon={<Hash />} label="Subscription ID" value={subscription.id} mono />
                <InfoItem icon={<User />} label="User ID" value={subscription.user_id} mono />
                <InfoItem icon={<ShieldCheck />} label="Access Context" value={subscription.access_context.toUpperCase()} />
                {subscription.api_key_name !== undefined && subscription.api_key_name !== '' && (
                    <InfoItem icon={<Shield />} label="API Key" value={subscription.api_key_name} />
                )}
            </div>
        </BaseCard>
    )
}

function TimelineCard({ subscription, formatDate }: {
    subscription: SubscriptionResponse,
    formatDate: (d?: string) => string
}) {
    return (
        <BaseCard>
            <div className="flex items-center gap-2 mb-6">
                <Calendar className="h-5 w-5 text-primary" />
                <h3 className="font-bold text-lg">Timeline</h3>
            </div>
            <div className="space-y-4">
                <TimelineItem label="Started" value={formatDate(subscription.started_at)} />
                <TimelineItem label="Expires" value={formatDate(subscription.expires_at)} />
                <div className="flex justify-between items-center py-2 border-t border-border/50 mt-2">
                    <span className="text-sm text-muted-foreground flex items-center gap-2">
                        <RefreshCw className="h-4 w-4" />
                        Auto-renew
                    </span>
                    <span className={`font-bold ${subscription.auto_renew ? 'text-success' : 'text-destructive'}`}>
                        {subscription.auto_renew ? 'Enabled' : 'Disabled'}
                    </span>
                </div>
            </div>
        </BaseCard>
    )
}

function UsageCard({ subscription, formatUsage }: {
    subscription: SubscriptionResponse,
    formatUsage: (u?: Record<string, unknown>) => string
}) {
    return (
        <BaseCard>
            <div className="flex items-center justify-between mb-6">
                <div className="flex items-center gap-2">
                    <Clock className="h-5 w-5 text-primary" />
                    <h3 className="font-bold text-lg">Current Utilization</h3>
                </div>
                <Badge variant="outline" className="font-mono">Real-time Data</Badge>
            </div>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                <div>
                    <h4 className="text-sm font-medium text-muted-foreground mb-3">Resource Consumption</h4>
                    <div className="bg-muted/30 p-4 rounded-2xl border border-border/50 font-mono text-sm">
                        {formatUsage(subscription.current_usage ?? {})}
                    </div>
                </div>
                <div>
                    <h4 className="text-sm font-medium text-muted-foreground mb-3">Hard Quota Limits</h4>
                    <div className="bg-muted/30 p-4 rounded-2xl border border-border/50 font-mono text-sm">
                        {formatUsage(subscription.quota_limits ?? {})}
                    </div>
                </div>
            </div>
        </BaseCard>
    )
}

function AdminOverridesCard({ editData, setEditData, plans, saving, onSave, onCancel }: {
    editData: UpdateSubscriptionRequest,
    setEditData: (d: UpdateSubscriptionRequest) => void,
    plans: Plan[],
    saving: boolean,
    onSave: () => void | Promise<void>,
    onCancel: () => void
}) {
    return (
        <BaseCard className="border-primary/20 bg-primary/5">
            <h3 className="font-bold text-lg mb-6">Administrative Overrides</h3>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="space-y-2">
                    <label className="text-sm font-medium text-muted-foreground">Lifecycle Status</label>
                    <select
                        value={editData.status}
                        onChange={(e) => { setEditData({ ...editData, status: e.target.value as SubscriptionResponse['status'] }) }}
                        className="w-full px-4 py-3 rounded-xl border border-border bg-card text-foreground focus:ring-2 focus:ring-primary outline-none"
                    >
                        <option value="active">Active</option>
                        <option value="expired">Expired</option>
                        <option value="cancelled">Cancelled</option>
                        <option value="paused">Paused</option>
                    </select>
                </div>

                <div className="space-y-2">
                    <label className="text-sm font-medium text-muted-foreground">Assigned Plan</label>
                    <select
                        value={editData.plan_id}
                        onChange={(e) => { setEditData({ ...editData, plan_id: e.target.value }) }}
                        className="w-full px-4 py-3 rounded-xl border border-border bg-card text-foreground focus:ring-2 focus:ring-primary outline-none"
                    >
                        {plans.map(plan => (
                            <option key={plan.id} value={plan.id}>
                                {plan.name}
                            </option>
                        ))}
                    </select>
                </div>

                <div className="space-y-2">
                    <label className="text-sm font-medium text-muted-foreground">Expiration Date</label>
                    <input
                        type="date"
                        value={editData.expires_at}
                        onChange={(e) => { setEditData({ ...editData, expires_at: e.target.value }) }}
                        min={new Date().toISOString().split('T')[0]}
                        className="w-full px-4 py-3 rounded-xl border border-border bg-card text-foreground focus:ring-2 focus:ring-primary outline-none"
                    />
                </div>

                <div className="flex items-center pt-8">
                    <label className="flex items-center gap-3 cursor-pointer group">
                        <div className="relative">
                            <input
                                type="checkbox"
                                checked={editData.auto_renew === true}
                                onChange={(e) => { setEditData({ ...editData, auto_renew: e.target.checked }) }}
                                className="sr-only"
                            />
                            <div className={`w-12 h-6 rounded-full transition-colors ${editData.auto_renew === true ? 'bg-primary' : 'bg-muted'}`} />
                            <div className={`absolute top-1 left-1 w-4 h-4 bg-white rounded-full transition-transform ${editData.auto_renew === true ? 'translate-x-6' : ''}`} />
                        </div>
                        <span className="text-sm font-medium">Automatic Renewal</span>
                    </label>
                </div>
            </div>

            <div className="flex gap-4 mt-8">
                <BaseButton variant="outline" className="flex-1" onClick={onCancel}>
                    Abort Changes
                </BaseButton>
                <BaseButton
                    className="flex-1"
                    onClick={() => {
                        void onSave()
                    }}
                    disabled={saving}
                >
                    <CheckCircle2 className="h-4 w-4 mr-2" />
                    {saving ? 'Propagating...' : 'Commit Changes'}
                </BaseButton>
            </div>
        </BaseCard>
    )
}

function MetadataCard({ metadata }: { metadata: Record<string, unknown> }) {
    return (
        <BaseCard>
            <h3 className="font-bold text-lg mb-4">Extended Metadata</h3>
            <pre className="text-xs text-muted-foreground bg-muted/50 p-4 rounded-xl border border-border/50 font-mono overflow-x-auto">
                {JSON.stringify(metadata, null, 2)}
            </pre>
        </BaseCard>
    )
}

function InfoItem({ icon, label, value, mono = false }: InfoItemProps) {
    return (
        <div className="flex gap-4">
            <div className="p-2 h-fit rounded-lg bg-primary/10 text-primary">
                {icon}
            </div>
            <div className="space-y-1">
                <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">{label}</p>
                <p className={`text-sm font-bold ${mono ? 'font-mono break-all' : ''}`}>{value}</p>
            </div>
        </div>
    )
}

function TimelineItem({ label, value }: TimelineItemProps) {
    return (
        <div className="flex justify-between items-center py-2">
            <div className="flex items-center gap-2">
                <Clock className="h-4 w-4 text-muted-foreground" />
                <span className="text-sm text-muted-foreground">{label}</span>
            </div>
            <span className="text-sm font-bold">{value}</span>
        </div>
    )
}
