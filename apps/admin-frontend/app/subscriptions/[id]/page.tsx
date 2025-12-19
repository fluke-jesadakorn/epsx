'use client'

import { ArrowLeft } from 'lucide-react'
import Link from 'next/link'
import { useParams, useRouter } from 'next/navigation'
import { useEffect, useState } from 'react'

import { toast } from '@/hooks/use-toast'
import { logger } from '@/lib/logger'
import { createPlansClient, isApiSuccess, type SubscriptionResponse, type UpdateSubscriptionRequest } from '@/shared/api/plans'
import { PancakeButton, PancakeCard } from '@/shared/components'
import { createAdminApiClient } from '@/shared/utils/api-client'

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
                    description: response.error || "Failed to update subscription",
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
                    description: response.error || "Failed to cancel subscription",
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
        switch (status) {
            case 'active':
                return 'bg-green-100 text-green-600 dark:bg-green-900/20 dark:text-green-400'
            case 'expired':
                return 'bg-yellow-100 text-yellow-600 dark:bg-yellow-900/20 dark:text-yellow-400'
            case 'cancelled':
                return 'bg-red-100 text-red-600 dark:bg-red-900/20 dark:text-red-400'
            default:
                return 'bg-gray-100 text-gray-600 dark:bg-gray-900/20 dark:text-gray-400'
        }
    }

    const formatDate = (dateString?: string) => {
        if (!dateString) return 'Never'
        return new Date(dateString).toLocaleString()
    }

    const formatUsage = (usage: Record<string, any>) => {
        if (!usage || Object.keys(usage).length === 0) return 'No usage data'
        return Object.entries(usage)
            .map(([key, value]) => `${key}: ${value}`)
            .join(', ')
    }

    if (loading) {
        return (
            <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
                <div className="max-w-5xl mx-auto">
                    <div className="animate-pulse space-y-6">
                        <div className="h-8 bg-gray-300 rounded w-1/3"></div>
                        <div className="h-64 bg-gray-200 rounded-2xl"></div>
                    </div>
                </div>
            </div>
        )
    }

    if (!subscription) {
        return null
    }

    return (
        <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
            <div className="max-w-5xl mx-auto space-y-6">
                {/* Header */}
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-4">
                        <Link
                            href="/subscriptions"
                            className="p-2 rounded-xl bg-white/80 dark:bg-gray-800/80 hover:bg-white dark:hover:bg-gray-800 transition-colors"
                        >
                            <ArrowLeft className="h-5 w-5" />
                        </Link>
                        <div>
                            <h1 className="text-2xl font-bold bg-gradient-to-r from-orange-600 to-yellow-600 bg-clip-text text-transparent">
                                Subscription Details
                            </h1>
                            <div className="flex items-center gap-3 mt-1">
                                <span className="text-lg font-semibold text-gray-700 dark:text-gray-300">
                                    {subscription.plan_name}
                                </span>
                                <span className={`px-3 py-1 text-sm rounded-full font-semibold ${getStatusColor(subscription.status)}`}>
                                    {subscription.status.toUpperCase()}
                                </span>
                            </div>
                        </div>
                    </div>
                    <div className="flex gap-3">
                        {!isEditing && subscription.status !== 'cancelled' && (
                            <PancakeButton
                                variant="secondary"
                                onClick={() => setIsEditing(true)}
                            >
                                Edit
                            </PancakeButton>
                        )}
                    </div>
                </div>

                {/* Basic Information */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <PancakeCard variant="outlined" className="p-6">
                        <h3 className="font-bold text-gray-900 dark:text-white mb-4">Basic Information</h3>
                        <div className="space-y-3">
                            <div className="flex justify-between">
                                <span className="text-gray-600 dark:text-gray-400">Subscription ID:</span>
                                <span className="font-semibold font-mono text-sm">{subscription.id}</span>
                            </div>
                            <div className="flex justify-between">
                                <span className="text-gray-600 dark:text-gray-400">User ID:</span>
                                <span className="font-semibold font-mono text-sm">{subscription.user_id}</span>
                            </div>
                            <div className="flex justify-between">
                                <span className="text-gray-600 dark:text-gray-400">Access Context:</span>
                                <span className="font-semibold">{subscription.access_context}</span>
                            </div>
                            {subscription.api_key_name && (
                                <div className="flex justify-between">
                                    <span className="text-gray-600 dark:text-gray-400">API Key Name:</span>
                                    <span className="font-semibold">{subscription.api_key_name}</span>
                                </div>
                            )}
                            {subscription.api_key && (
                                <div className="flex justify-between">
                                    <span className="text-gray-600 dark:text-gray-400">API Key:</span>
                                    <span className="font-mono text-sm bg-gray-200 dark:bg-gray-600 px-2 py-1 rounded">
                                        {subscription.api_key.substring(0, 8)}...
                                    </span>
                                </div>
                            )}
                        </div>
                    </PancakeCard>

                    <PancakeCard className="bg-gray-50 dark:bg-gray-700 p-6">
                        <h3 className="font-bold text-gray-900 dark:text-white mb-4">Timeline</h3>
                        <div className="space-y-3">
                            <div className="flex justify-between">
                                <span className="text-gray-600 dark:text-gray-400">Started:</span>
                                <span className="font-semibold">{formatDate(subscription.started_at)}</span>
                            </div>
                            <div className="flex justify-between">
                                <span className="text-gray-600 dark:text-gray-400">Created:</span>
                                <span className="font-semibold">{formatDate(subscription.created_at)}</span>
                            </div>
                            <div className="flex justify-between">
                                <span className="text-gray-600 dark:text-gray-400">Expires:</span>
                                <span className="font-semibold">{formatDate(subscription.expires_at)}</span>
                            </div>
                            <div className="flex justify-between">
                                <span className="text-gray-600 dark:text-gray-400">Auto-renew:</span>
                                <span className={`font-semibold ${subscription.auto_renew ? 'text-green-600' : 'text-red-600'}`}>
                                    {subscription.auto_renew ? 'Yes' : 'No'}
                                </span>
                            </div>
                            {subscription.last_billed_at && (
                                <div className="flex justify-between">
                                    <span className="text-gray-600 dark:text-gray-400">Last Billed:</span>
                                    <span className="font-semibold">{formatDate(subscription.last_billed_at)}</span>
                                </div>
                            )}
                            {subscription.next_billing_date && (
                                <div className="flex justify-between">
                                    <span className="text-gray-600 dark:text-gray-400">Next Billing:</span>
                                    <span className="font-semibold">{formatDate(subscription.next_billing_date)}</span>
                                </div>
                            )}
                        </div>
                    </PancakeCard>
                </div>

                {/* Current Usage */}
                <PancakeCard className="bg-gray-50 dark:bg-gray-700 p-6">
                    <h3 className="font-bold text-gray-900 dark:text-white mb-4">Current Usage & Quotas</h3>
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                        <div>
                            <h4 className="font-semibold text-gray-700 dark:text-gray-300 mb-2">Current Usage</h4>
                            <p className="text-sm text-gray-600 dark:text-gray-400 font-mono bg-gray-200 dark:bg-gray-600 p-3 rounded">
                                {formatUsage(subscription.current_usage)}
                            </p>
                        </div>
                        <div>
                            <h4 className="font-semibold text-gray-700 dark:text-gray-300 mb-2">Quota Limits</h4>
                            <p className="text-sm text-gray-600 dark:text-gray-400 font-mono bg-gray-200 dark:bg-gray-600 p-3 rounded">
                                {formatUsage(subscription.quota_limits)}
                            </p>
                        </div>
                    </div>
                </PancakeCard>

                {/* Edit Form */}
                {isEditing && (
                    <PancakeCard className="bg-blue-50 dark:bg-blue-900/20 p-6 border border-blue-200 dark:border-blue-800">
                        <h3 className="font-bold text-gray-900 dark:text-white mb-4">Edit Subscription</h3>
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                            <div>
                                <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                                    Status
                                </label>
                                <select
                                    value={editData.status}
                                    onChange={(e) => setEditData({ ...editData, status: e.target.value })}
                                    className="w-full px-4 py-3 rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-emerald-500"
                                >
                                    <option value="active">Active</option>
                                    <option value="expired">Expired</option>
                                    <option value="cancelled">Cancelled</option>
                                </select>
                            </div>

                            <div>
                                <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                                    Plan
                                </label>
                                <select
                                    value={editData.plan_id}
                                    onChange={(e) => setEditData({ ...editData, plan_id: parseInt(e.target.value) })}
                                    className="w-full px-4 py-3 rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-emerald-500"
                                >
                                    {plans.map(plan => (
                                        <option key={plan.id} value={plan.id}>
                                            {plan.name} - {Number(plan.current_price) === 0 ? 'Free' : `$${plan.current_price} ${plan.currency}`}
                                        </option>
                                    ))}
                                </select>
                            </div>

                            <div>
                                <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                                    Expiry Date
                                </label>
                                <input
                                    type="date"
                                    value={editData.expires_at}
                                    onChange={(e) => setEditData({ ...editData, expires_at: e.target.value })}
                                    min={new Date().toISOString().split('T')[0]}
                                    className="w-full px-4 py-3 rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-emerald-500"
                                />
                            </div>

                            <div className="flex items-center justify-center">
                                <label className="flex items-center gap-3 cursor-pointer">
                                    <input
                                        type="checkbox"
                                        checked={editData.auto_renew}
                                        onChange={(e) => setEditData({ ...editData, auto_renew: e.target.checked })}
                                        className="w-5 h-5 text-emerald-500 border-gray-300 rounded focus:ring-emerald-500"
                                    />
                                    <span className="text-sm font-semibold text-gray-700 dark:text-gray-300">Auto-renew</span>
                                </label>
                            </div>
                        </div>

                        <div className="flex gap-4 mt-6">
                            <button
                                onClick={() => setIsEditing(false)}
                                className="flex-1 px-6 py-3 rounded-xl border border-gray-200 dark:border-gray-600 text-gray-700 dark:text-gray-300 font-semibold hover:bg-gray-50 dark:hover:bg-gray-700"
                            >
                                Cancel
                            </button>
                            <button
                                onClick={handleUpdate}
                                disabled={saving}
                                className="flex-1 px-6 py-3 rounded-xl bg-gradient-to-r from-emerald-400 to-green-500 text-white font-semibold hover:from-emerald-500 hover:to-green-600 disabled:opacity-50 disabled:cursor-not-allowed"
                            >
                                {saving ? 'Updating...' : 'Update Subscription'}
                            </button>
                        </div>
                    </PancakeCard>
                )}

                {/* Metadata */}
                {subscription.metadata && Object.keys(subscription.metadata).length > 0 && (
                    <PancakeCard variant="outlined" className="p-6">
                        <h3 className="font-bold text-gray-900 dark:text-white mb-4">Metadata</h3>
                        <pre className="text-sm text-gray-600 dark:text-gray-400 bg-gray-200 dark:bg-gray-600 p-3 rounded font-mono overflow-x-auto">
                            {JSON.stringify(subscription.metadata, null, 2)}
                        </pre>
                    </PancakeCard>
                )}

                {/* Footer Actions */}
                <div className="flex gap-4 pt-4">
                    <Link href="/subscriptions" className="flex-1">
                        <PancakeButton variant="outline" className="w-full">
                            Back to Subscriptions
                        </PancakeButton>
                    </Link>

                    {subscription.status === 'active' && !isEditing && (
                        <PancakeButton
                            variant="destructive"
                            onClick={handleCancel}
                            disabled={saving}
                            className="flex-1"
                        >
                            {saving ? 'Cancelling...' : 'Cancel Subscription'}
                        </PancakeButton>
                    )}
                </div>
            </div>
        </div>
    )
}
