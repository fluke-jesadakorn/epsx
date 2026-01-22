'use client'

import { ArrowLeft } from 'lucide-react'
import Link from 'next/link'
import { useParams, useRouter } from 'next/navigation'
import { useEffect, useState } from 'react'

import { toast } from '@/hooks/use-toast'
import { logger } from '@/lib/logger'
import { createPlansClient, isApiSuccess, type SubscriptionResponse, type UpdateSubscriptionRequest } from '@/shared/api/plans'
import { createAdminApiClient } from '@/shared/utils/api-client'

/**
 *
 */
export default function SubscriptionDetailPage() {
    const router = useRouter()
    const params = useParams()
    const subscriptionId = params['id'] as string

    const [subscription, setSubscription] = useState<SubscriptionResponse | null>(null)
    const [loading, setLoading] = useState(true)
    const [isEditing, setIsEditing] = useState(false)
    const [saving, setSaving] = useState(false)
    const [plans, setPlans] = useState<any[]>([])
    const [editData, setEditData] = useState<UpdateSubscriptionRequest>({
        status: '',
        plan_id: 0,
        expires_at: '',
        auto_renew: false
    })

    useEffect(() => {
        loadSubscription()
        loadPlans()
    }, [subscriptionId])

    const loadSubscription = async () => {
        const adminClient = createPlansClient(createAdminApiClient())
        try {
            setLoading(true)
            const response = await adminClient.getSubscription(subscriptionId)
            if (isApiSuccess(response)) {
                const data = (response.data as any)?.subscription || response.data
                setSubscription(data)
                setEditData({
                    status: data.status,
                    plan_id: data.plan_id,
                    expires_at: data.expires_at?.split('T')[0] || '',
                    auto_renew: data.auto_renew
                })
            } else {
                toast({
                    title: "Error",
                    description: "Subscription not found",
                    variant: "destructive"
                })
                router.push('/subscriptions')
            }
        } catch (_error) {
            logger.error('Failed to load subscription', { _error })
            toast({
                title: "Error",
                description: "Failed to load subscription",
                variant: "destructive"
            })
            router.push('/subscriptions')
        } finally {
            setLoading(false)
        }
    }

    const loadPlans = async () => {
        const adminClient = createPlansClient(createAdminApiClient())
        try {
            const response = await adminClient.getPlans({ is_active: true })
            if (isApiSuccess(response)) {
                setPlans((response.data as any)?.plans || response.data as any || [])
            }
        } catch (_error) {
            logger.error('Failed to load plans', { _error })
        }
    }

    const handleUpdate = async () => {
        const adminClient = createPlansClient(createAdminApiClient())
        try {
            setSaving(true)
            const updatePayload: UpdateSubscriptionRequest = {
                ...editData,
                expires_at: editData.expires_at || undefined
            }

            const response = await adminClient.updateSubscription(subscriptionId, updatePayload)

            if (isApiSuccess(response)) {
                setIsEditing(false)
                loadSubscription()
                toast({
                    title: "Success",
                    description: "Subscription updated successfully",
                })
            } else {
                toast({
                    title: "Error",
                    description: response.error?.message || "Failed to update subscription",
                    variant: "destructive"
                })
            }
        } catch (_error) {
            toast({
                title: "Error",
                description: "Failed to update subscription",
                variant: "destructive"
            })
        } finally {
            setSaving(false)
        }
    }

    const handleCancel = async () => {
        if (!confirm('Are you sure you want to cancel this subscription? This action cannot be undone.')) {
            return
        }

        const adminClient = createPlansClient(createAdminApiClient())
        try {
            setSaving(true)
            const response = await adminClient.cancelSubscription(subscriptionId)

            if (isApiSuccess(response)) {
                toast({
                    title: "Success",
                    description: "Subscription cancelled successfully",
                })
                router.push('/subscriptions')
            } else {
                toast({
                    title: "Error",
                    description: response.error?.message || "Failed to cancel subscription",
                    variant: "destructive"
                })
            }
        } catch (_error) {
            toast({
                title: "Error",
                description: "Failed to cancel subscription",
                variant: "destructive"
            })
        } finally {
            setSaving(false)
        }
    }

    const getStatusColor = (status: string) => {
        switch (status.toLowerCase()) {
            case 'active':
                return 'bg-success/10 text-success border border-success/20'
            case 'expired':
                return 'bg-warning/10 text-warning border border-warning/20'
            case 'cancelled':
                return 'bg-destructive/10 text-destructive border border-destructive/20'
            default:
                return 'bg-muted text-muted-foreground border border-border/50'
        }
    }

    const formatDate = (dateString?: string) => {
        if (!dateString) { return 'Never' }
        return new Date(dateString).toLocaleString()
    }

    const formatUsage = (usage: Record<string, any>) => {
        if (!usage || Object.keys(usage).length === 0) { return 'No usage data' }
        return Object.entries(usage)
            .map(([key, value]) => `${key}: ${value}`)
            .join(', ')
    }

    if (loading) {
        return (
            <div className="min-h-screen p-6">
                <div className="max-w-5xl mx-auto">
                    <div className="animate-pulse space-y-6">
                        <div className="h-8 bg-primary/20 rounded-2xl w-1/3"></div>
                        <div className="h-64 bg-card rounded-3xl border border-border/50"></div>
                    </div>
                </div>
            </div>
        )
    }

    if (!subscription) {
        return null
    }

    return (
        <div className="min-h-screen p-6">
            <div className="max-w-5xl mx-auto space-y-6">
                {/* Header */}
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-4">
                        <Link
                            href="/subscriptions"
                            className="p-2 rounded-xl bg-card border border-border/50 hover:border-primary/50 transition-colors"
                        >
                            <ArrowLeft className="h-5 w-5" />
                        </Link>
                        <div>
                            <h1 className="text-2xl font-bold bg-gradient-to-r from-primary via-secondary to-primary bg-clip-text text-transparent">
                                Subscription Details
                            </h1>
                            <div className="flex items-center gap-3 mt-1">
                                <span className="text-lg font-semibold text-foreground">
                                    {subscription.plan_name}
                                </span>
                                <span className={`px-3 py-1 text-sm rounded-full font-semibold border ${getStatusColor(subscription.status)}`}>
                                    {subscription.status.toUpperCase()}
                                </span>
                            </div>
                        </div>
                    </div>
                    <div className="flex gap-3">
                        {!isEditing && subscription.status !== 'cancelled' && (
                            <button
                                onClick={() => setIsEditing(true)}
                                className="px-4 py-2 bg-secondary text-secondary-foreground rounded-xl font-semibold hover:opacity-90 transition-opacity"
                            >
                                Edit
                            </button>
                        )}
                    </div>
                </div>

                {/* Basic Information */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div className="bg-card rounded-3xl p-6 border border-border/50 shadow-sm">
                        <h3 className="font-bold text-foreground mb-4">Basic Information</h3>
                        <div className="space-y-3">
                            <div className="flex justify-between">
                                <span className="text-muted-foreground">Subscription ID:</span>
                                <span className="font-semibold font-mono text-sm">{subscription.id}</span>
                            </div>
                            <div className="flex justify-between">
                                <span className="text-muted-foreground">User ID:</span>
                                <span className="font-semibold font-mono text-sm">{subscription.user_id}</span>
                            </div>
                            <div className="flex justify-between">
                                <span className="text-muted-foreground">Access Context:</span>
                                <span className="font-semibold">{subscription.access_context}</span>
                            </div>
                            {subscription.api_key_name && (
                                <div className="flex justify-between">
                                    <span className="text-muted-foreground">API Key Name:</span>
                                    <span className="font-semibold">{subscription.api_key_name}</span>
                                </div>
                            )}
                            {subscription.api_key && (
                                <div className="flex justify-between">
                                    <span className="text-muted-foreground">API Key:</span>
                                    <span className="font-mono text-sm bg-muted px-2 py-1 rounded">
                                        {subscription.api_key.substring(0, 8)}...
                                    </span>
                                </div>
                            )}
                        </div>
                    </div>

                    <div className="bg-card rounded-3xl p-6 border border-border/50 shadow-sm">
                        <h3 className="font-bold text-foreground mb-4">Timeline</h3>
                        <div className="space-y-3">
                            <div className="flex justify-between">
                                <span className="text-muted-foreground">Started:</span>
                                <span className="font-semibold">{formatDate(subscription.started_at)}</span>
                            </div>
                            <div className="flex justify-between">
                                <span className="text-muted-foreground">Created:</span>
                                <span className="font-semibold">{formatDate(subscription.created_at)}</span>
                            </div>
                            <div className="flex justify-between">
                                <span className="text-muted-foreground">Expires:</span>
                                <span className="font-semibold">{formatDate(subscription.expires_at)}</span>
                            </div>
                            <div className="flex justify-between">
                                <span className="text-muted-foreground">Auto-renew:</span>
                                <span className={`font-semibold ${subscription.auto_renew ? 'text-success' : 'text-destructive'}`}>
                                    {subscription.auto_renew ? 'Yes' : 'No'}
                                </span>
                            </div>
                            {subscription.last_billed_at && (
                                <div className="flex justify-between">
                                    <span className="text-muted-foreground">Last Billed:</span>
                                    <span className="font-semibold">{formatDate(subscription.last_billed_at)}</span>
                                </div>
                            )}
                            {subscription.next_billing_date && (
                                <div className="flex justify-between">
                                    <span className="text-muted-foreground">Next Billing:</span>
                                    <span className="font-semibold">{formatDate(subscription.next_billing_date)}</span>
                                </div>
                            )}
                        </div>
                    </div>
                </div>

                {/* Current Usage */}
                <div className="bg-card rounded-3xl p-6 border border-border/50 shadow-sm">
                    <h3 className="font-bold text-foreground mb-4">Current Usage & Quotas</h3>
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                        <div>
                            <h4 className="font-semibold text-muted-foreground mb-2">Current Usage</h4>
                            <p className="text-sm text-foreground/80 font-mono bg-muted p-3 rounded-xl border border-border/50">
                                {formatUsage(subscription.current_usage || {})}
                            </p>
                        </div>
                        <div>
                            <h4 className="font-semibold text-muted-foreground mb-2">Quota Limits</h4>
                            <p className="text-sm text-foreground/80 font-mono bg-muted p-3 rounded-xl border border-border/50">
                                {formatUsage(subscription.quota_limits || {})}
                            </p>
                        </div>
                    </div>
                </div>

                {/* Edit Form */}
                {isEditing && (
                    <div className="bg-primary/5 rounded-3xl p-6 border border-primary/20">
                        <h3 className="font-bold text-foreground mb-4">Edit Subscription</h3>
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                            <div>
                                <label className="block text-sm font-semibold text-muted-foreground mb-2">
                                    Status
                                </label>
                                <select
                                    value={editData.status}
                                    onChange={(e) => setEditData({ ...editData, status: e.target.value })}
                                    className="w-full px-4 py-3 rounded-xl border border-border bg-card text-foreground focus:ring-2 focus:ring-primary"
                                >
                                    <option value="active">Active</option>
                                    <option value="expired">Expired</option>
                                    <option value="cancelled">Cancelled</option>
                                </select>
                            </div>

                            <div>
                                <label className="block text-sm font-semibold text-muted-foreground mb-2">
                                    Plan
                                </label>
                                <select
                                    value={editData.plan_id}
                                    onChange={(e) => setEditData({ ...editData, plan_id: parseInt(e.target.value) })}
                                    className="w-full px-4 py-3 rounded-xl border border-border bg-card text-foreground focus:ring-2 focus:ring-primary"
                                >
                                    {plans.map(plan => (
                                        <option key={plan.id} value={plan.id}>
                                            {plan.name} - {Number(plan.current_price) === 0 ? 'Free' : `$${plan.current_price} ${plan.currency}`}
                                        </option>
                                    ))}
                                </select>
                            </div>

                            <div>
                                <label className="block text-sm font-semibold text-muted-foreground mb-2">
                                    Expiry Date
                                </label>
                                <input
                                    type="date"
                                    value={editData.expires_at}
                                    onChange={(e) => setEditData({ ...editData, expires_at: e.target.value })}
                                    min={new Date().toISOString().split('T')[0]}
                                    className="w-full px-4 py-3 rounded-xl border border-border bg-card text-foreground focus:ring-2 focus:ring-primary"
                                />
                            </div>

                            <div className="flex items-center justify-center">
                                <label className="flex items-center gap-3 cursor-pointer">
                                    <input
                                        type="checkbox"
                                        checked={editData.auto_renew}
                                        onChange={(e) => setEditData({ ...editData, auto_renew: e.target.checked })}
                                        className="w-5 h-5 text-primary border-border rounded focus:ring-primary"
                                    />
                                    <span className="text-sm font-semibold text-muted-foreground">Auto-renew</span>
                                </label>
                            </div>
                        </div>

                        <div className="flex gap-4 mt-6">
                            <button
                                onClick={() => setIsEditing(false)}
                                className="flex-1 px-6 py-3 rounded-xl border border-border text-foreground font-semibold hover:bg-muted transition-colors"
                            >
                                Cancel
                            </button>
                            <button
                                onClick={handleUpdate}
                                disabled={saving}
                                className="flex-1 px-6 py-3 rounded-xl bg-primary text-primary-foreground font-semibold hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed transition-opacity"
                            >
                                {saving ? 'Updating...' : 'Update Subscription'}
                            </button>
                        </div>
                    </div>
                )}

                {/* Metadata */}
                {subscription.metadata && Object.keys(subscription.metadata).length > 0 && (
                    <div className="bg-card rounded-3xl p-6 border border-border/50 shadow-sm">
                        <h3 className="font-bold text-foreground mb-4">Metadata</h3>
                        <pre className="text-sm text-muted-foreground bg-muted p-4 rounded-xl border border-border/50 font-mono overflow-x-auto">
                            {JSON.stringify(subscription.metadata, null, 2)}
                        </pre>
                    </div>
                )}

                {/* Footer Actions */}
                <div className="flex gap-4 pt-4">
                    <Link href="/subscriptions" className="flex-1">
                        <button className="w-full px-6 py-3 rounded-xl border border-border text-foreground font-semibold hover:bg-muted transition-colors">
                            Back to Subscriptions
                        </button>
                    </Link>

                    {subscription.status === 'active' && !isEditing && (
                        <button
                            onClick={handleCancel}
                            disabled={saving}
                            className="flex-1 px-6 py-3 rounded-xl bg-destructive text-destructive-foreground font-semibold hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed transition-opacity"
                        >
                            {saving ? 'Cancelling...' : 'Cancel Subscription'}
                        </button>
                    )}
                </div>
            </div>
        </div>
    )
}
