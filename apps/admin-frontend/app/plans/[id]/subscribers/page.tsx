'use client'

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { ArrowLeft, Trash2, UserPlus, Users } from 'lucide-react'
import Link from 'next/link'
import { useParams, useRouter } from 'next/navigation'
import { useEffect, useState } from 'react'
import { toast } from 'sonner'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { WalletAutocomplete } from '@/components/ui/WalletAutocomplete'
import { createPlansClient, isApiSuccess, type PlanResponse, type SubscriptionResponse } from '@/shared/api/plans'
import { createAdminApiClient } from '@/shared/utils/api-client'

/**
 *
 */
export default function PlanSubscribersPage() {
    const router = useRouter()
    const params = useParams()
    const planId = params['id'] as string
    const queryClient = useQueryClient()

    const [plan, setPlan] = useState<PlanResponse | null>(null)
    const [planLoading, setPlanLoading] = useState(true)
    const [newWalletAddress, setNewWalletAddress] = useState('')
    const [showAddSubscriber, setShowAddSubscriber] = useState(false)

    // Load plan details
    useEffect(() => {
        const loadPlan = async () => {
            try {
                setPlanLoading(true)
                const plansClient = createPlansClient(createAdminApiClient())
                const response = await plansClient.getPlan(planId)
                if (isApiSuccess(response)) {
                    const backendResponse = response.data as any
                    const planData = backendResponse?.data || backendResponse
                    setPlan(planData)
                } else {
                    toast.error('Plan not found')
                    router.push('/plans')
                }
            } catch (_error) {
                toast.error('Failed to load plan')
                router.push('/plans')
            } finally {
                setPlanLoading(false)
            }
        }
        loadPlan()
    }, [planId, router])

    // Compute plan ID for API calls
    const planIdNum = plan ? (typeof plan.id === 'number' ? plan.id : parseInt(String(plan.id), 10)) : undefined
    const planIdValid = planIdNum && !isNaN(planIdNum) ? planIdNum : undefined

    // Fetch plan subscribers
    const { data: subscribers = [], isLoading, refetch } = useQuery({
        queryKey: ['plan-subscribers', planId],
        queryFn: async () => {
            const plansClient = createPlansClient(createAdminApiClient())
            const response = await plansClient.getSubscriptions(planIdValid ? { plan_id: planIdValid } : {})
            if (isApiSuccess(response)) {
                const allSubs = (response.data as any)?.subscriptions || response.data || []
                // Client-side filter by plan name if plan_id is UUID
                if (!planIdValid && plan?.name) {
                    return allSubs.filter((s: any) => s.plan_name === plan.name)
                }
                return allSubs
            }
            return []
        },
        enabled: !!plan,
        refetchInterval: 10000, // Faster refresh for real-time updates
    })

    // Cancel confirmation modal state
    const [cancelConfirm, setCancelConfirm] = useState<{ isOpen: boolean; subscriptionId: string; walletAddress: string } | null>(null)

    // Add subscriber mutation
    const addSubscriberMutation = useMutation({
        mutationFn: async (walletAddress: string) => {
            const plansClient = createPlansClient(createAdminApiClient())
            const response = await plansClient.createSubscription({
                user_id: walletAddress,
                plan_id: planIdValid || 0,
                access_context: 'internal',
                auto_renew: false,
                permission_group_name: plan?.name,
            } as any)
            if (!isApiSuccess(response)) {
                throw new Error((response as any).error || 'Failed to add subscriber')
            }
            return response.data
        },
        onSuccess: () => {
            toast.success('Subscriber added successfully')
            setNewWalletAddress('')
            setShowAddSubscriber(false)
            queryClient.invalidateQueries({ queryKey: ['plan-subscribers', planId] })
            queryClient.invalidateQueries({ queryKey: ['plans'] })
            refetch()
        },
        onError: (error: any) => {
            toast.error(error.message || 'Failed to add subscriber')
        }
    })

    // Cancel subscription mutation
    const cancelSubscriptionMutation = useMutation({
        mutationFn: async (subscriptionId: string) => {
            const plansClient = createPlansClient(createAdminApiClient())
            const response = await plansClient.cancelSubscription(subscriptionId)
            if (!isApiSuccess(response)) {
                throw new Error((response as any).error || 'Failed to cancel subscription')
            }
            return response.data
        },
        onSuccess: () => {
            toast.success('Subscription cancelled successfully')
            queryClient.invalidateQueries({ queryKey: ['plan-subscribers', planId] })
            queryClient.invalidateQueries({ queryKey: ['plans'] })
            refetch()
        },
        onError: (error: any) => {
            toast.error(error.message || 'Failed to cancel subscription')
        }
    })

    const handleAddSubscriber = (e: React.FormEvent) => {
        e.preventDefault()
        if (!newWalletAddress?.startsWith('0x')) {
            toast.error('Please enter a valid wallet address')
            return
        }
        addSubscriberMutation.mutate(newWalletAddress)
    }

    const activeSubscribers = subscribers.filter((s: SubscriptionResponse) => s.status === 'active')

    if (planLoading) {
        return (
            <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
                <div className="max-w-4xl mx-auto">
                    <div className="animate-pulse space-y-6">
                        <div className="h-8 bg-gray-300 rounded w-1/3"></div>
                        <div className="h-64 bg-gray-200 rounded-2xl"></div>
                    </div>
                </div>
            </div>
        )
    }

    if (!plan) {
        return null
    }

    return (
        <div className="min-h-screen bg-background p-6">
            <div className="max-w-4xl mx-auto space-y-6">
                {/* Header */}
                <div className="flex items-center gap-4 mb-6">
                    <Link
                        href="/plans"
                        className="p-2 rounded-xl bg-white/80 dark:bg-gray-800/80 hover:bg-white dark:hover:bg-gray-800 transition-colors"
                    >
                        <ArrowLeft className="h-5 w-5" />
                    </Link>
                    <div>
                        <h1 className="text-2xl font-bold text-emerald-600 dark:text-emerald-400 flex items-center gap-2">
                            <Users className="w-6 h-6" />
                            Subscribers of "{plan.name}"
                        </h1>
                        <div className="text-sm text-gray-600 dark:text-gray-400">
                            Total subscribers: {subscribers.length} ({activeSubscribers.length} active)
                            {isLoading && ' (Loading...)'}
                        </div>
                    </div>
                </div>

                {/* Main Card */}
                <div className="bg-white dark:bg-gray-800 rounded-2xl sm:rounded-3xl p-6 shadow-2xl border-2 border-emerald-300/50 dark:border-emerald-700/50">
                    <div className="space-y-4">
                        {/* Add Subscriber Section */}
                        <div className="border-b border-gray-200 dark:border-gray-700 pb-4">
                            <div className="flex justify-between items-center mb-3">
                                <h3 className="font-semibold text-emerald-600 dark:text-emerald-400">Manage Subscribers</h3>
                                <Button
                                    onClick={() => setShowAddSubscriber(!showAddSubscriber)}
                                    variant="outline"
                                    size="sm"
                                >
                                    <UserPlus className="w-4 h-4 mr-2" />
                                    Add Subscriber
                                </Button>
                            </div>

                            {showAddSubscriber && (
                                <form onSubmit={handleAddSubscriber} className="flex gap-2">
                                    <WalletAutocomplete
                                        value={newWalletAddress}
                                        onChange={setNewWalletAddress}
                                        placeholder="Enter wallet address (0x...)"
                                        className="flex-1"
                                    />
                                    <Button
                                        type="submit"
                                        disabled={addSubscriberMutation.isPending}
                                        size="sm"
                                    >
                                        {addSubscriberMutation.isPending ? 'Adding...' : 'ADD'}
                                    </Button>
                                    <Button
                                        type="button"
                                        variant="outline"
                                        onClick={() => {
                                            setShowAddSubscriber(false)
                                            setNewWalletAddress('')
                                        }}
                                        size="sm"
                                    >
                                        Cancel
                                    </Button>
                                </form>
                            )}
                        </div>

                        {/* Subscribers List */}
                        <div className="space-y-3">
                            <h3 className="font-semibold text-emerald-600 dark:text-emerald-400">Current Subscribers</h3>

                            {isLoading ? (
                                <div className="text-center py-12">
                                    <div className="h-10 w-10 border-4 border-emerald-200 border-t-emerald-600 rounded-full mx-auto mb-4 animate-spin"></div>
                                    <p className="text-gray-500 dark:text-gray-400">Loading subscribers...</p>
                                </div>
                            ) : subscribers.length === 0 ? (
                                <div className="text-center py-12">
                                    <div className="w-20 h-20 mx-auto mb-4 bg-gradient-to-br from-emerald-100 to-green-100 dark:from-emerald-900/30 dark:to-green-900/30 rounded-full flex items-center justify-center">
                                        <Users className="w-10 h-10 text-emerald-400" />
                                    </div>
                                    <p className="text-lg font-medium text-gray-900 dark:text-white mb-2">No subscribers yet</p>
                                    <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">Add subscribers to grant them access to this plan's features</p>
                                    <Button
                                        onClick={() => setShowAddSubscriber(true)}
                                        className="bg-gradient-to-r from-emerald-400 to-green-500 text-white"
                                    >
                                        <UserPlus className="w-4 h-4 mr-2" />
                                        Add First Subscriber
                                    </Button>
                                </div>
                            ) : (
                                <div className="space-y-3 max-h-[400px] overflow-y-auto pr-2">
                                    {subscribers.map((subscriber: SubscriptionResponse) => {
                                        const walletAddress = (subscriber as any).wallet_address || subscriber.user_id || 'Unknown'
                                        const isActive = subscriber.status === 'active'
                                        const expiresAt = subscriber.expires_at ? new Date(subscriber.expires_at) : null
                                        const isExpiringSoon = expiresAt && (expiresAt.getTime() - Date.now()) < 7 * 24 * 60 * 60 * 1000

                                        return (
                                            <div
                                                key={subscriber.id}
                                                className={`flex items-center justify-between p-4 rounded-xl border-2 transition-all hover:shadow-md ${isActive
                                                    ? 'bg-white dark:bg-gray-800 border-emerald-200 dark:border-emerald-800/50'
                                                    : 'bg-gray-50 dark:bg-gray-800/50 border-gray-200 dark:border-gray-700'
                                                    }`}
                                            >
                                                <div className="flex items-center gap-4">
                                                    {/* Avatar */}
                                                    <div className={`h-12 w-12 rounded-full flex items-center justify-center font-mono text-sm font-bold ${isActive
                                                        ? 'bg-gradient-to-br from-emerald-400 to-green-500 text-white'
                                                        : 'bg-gray-300 dark:bg-gray-600 text-gray-600 dark:text-gray-300'
                                                        }`}>
                                                        {walletAddress.substring(2, 4).toUpperCase()}
                                                    </div>

                                                    {/* Info */}
                                                    <div>
                                                        <div className="font-mono text-sm font-medium text-gray-900 dark:text-white">
                                                            {walletAddress.slice(0, 8)}...{walletAddress.slice(-6)}
                                                        </div>
                                                        <div className="flex items-center gap-2 mt-1 flex-wrap">
                                                            <Badge
                                                                variant={isActive ? 'default' : 'secondary'}
                                                                className={`text-xs ${isActive ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-400' : ''}`}
                                                            >
                                                                {subscriber.status}
                                                            </Badge>
                                                            <span className="text-xs text-gray-500 dark:text-gray-400">
                                                                Since {new Date(subscriber.started_at).toLocaleDateString()}
                                                            </span>
                                                            {expiresAt && (
                                                                <span className={`text-xs font-medium ${isExpiringSoon
                                                                    ? 'text-orange-600 dark:text-orange-400'
                                                                    : 'text-gray-500 dark:text-gray-400'
                                                                    }`}>
                                                                    {isExpiringSoon ? '⚠️ ' : ''}Expires {expiresAt.toLocaleDateString()}
                                                                </span>
                                                            )}
                                                        </div>
                                                    </div>
                                                </div>

                                                {/* Actions */}
                                                {isActive && (
                                                    <Button
                                                        variant="outline"
                                                        size="sm"
                                                        className="text-red-500 hover:text-red-700 hover:bg-red-50 dark:hover:bg-red-900/20 border-red-200"
                                                        onClick={() => {
                                                            setCancelConfirm({
                                                                isOpen: true,
                                                                subscriptionId: subscriber.id,
                                                                walletAddress: walletAddress
                                                            })
                                                        }}
                                                        disabled={cancelSubscriptionMutation.isPending}
                                                    >
                                                        <Trash2 className="w-4 h-4 mr-1" />
                                                        Remove
                                                    </Button>
                                                )}
                                            </div>
                                        )
                                    })}
                                </div>
                            )}
                        </div>

                        {/* Plan Info */}
                        <div className="border-t border-gray-200 dark:border-gray-700 pt-4">
                            <h3 className="font-semibold mb-2 text-emerald-600 dark:text-emerald-400">Plan Details</h3>
                            <div className="grid grid-cols-2 gap-3 text-sm">
                                <div>
                                    <span className="text-gray-500 dark:text-gray-300">Price:</span>
                                    {Number(plan.current_price) === 0 ? (
                                        <span className="ml-2 font-medium bg-gradient-to-r from-emerald-500 to-green-500 bg-clip-text text-transparent">Free</span>
                                    ) : (
                                        <span className="ml-2 font-medium text-emerald-600 dark:text-emerald-300">${plan.current_price} {plan.currency}</span>
                                    )}
                                </div>
                                <div>
                                    <span className="text-gray-500 dark:text-gray-300">Category:</span>
                                    <span className="ml-2 font-medium text-gray-900 dark:text-white">{plan.plan_category || '—'}</span>
                                </div>
                                <div>
                                    <span className="text-gray-500 dark:text-gray-300">Permissions:</span>
                                    <span className="ml-2 font-medium text-emerald-600 dark:text-emerald-300">{plan.permissions?.length || 0}</span>
                                </div>
                                <div>
                                    <span className="text-gray-500 dark:text-gray-300">Status:</span>
                                    <Badge variant={plan.is_active ? 'default' : 'secondary'} className="ml-2">
                                        {plan.is_active ? 'Active' : 'Inactive'}
                                    </Badge>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                {/* Back Button */}
                <div className="flex gap-4">
                    <Link href="/plans" className="flex-1">
                        <Button variant="outline" className="w-full">
                            Back to Plans
                        </Button>
                    </Link>
                    <Link href={`/plans/${planId}/edit`} className="flex-1">
                        <Button className="w-full bg-gradient-to-r from-purple-400 to-purple-500 hover:from-purple-500 hover:to-purple-600">
                            ✏️ Edit Plan
                        </Button>
                    </Link>
                </div>

                {/* Cancel Subscription Confirmation Modal */}
                {cancelConfirm && (
                    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
                        <div className="bg-white dark:bg-gray-800 rounded-2xl p-6 max-w-md w-full mx-4 shadow-2xl border border-gray-200 dark:border-gray-700">
                            <div className="flex items-center gap-3 mb-4">
                                <div className="p-3 bg-red-100 dark:bg-red-900/30 rounded-full">
                                    <Trash2 className="w-6 h-6 text-red-600 dark:text-red-400" />
                                </div>
                                <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                                    Cancel Subscription
                                </h3>
                            </div>
                            <p className="text-gray-600 dark:text-gray-400 mb-2">
                                Are you sure you want to cancel this subscription for:
                            </p>
                            <p className="font-mono text-sm bg-gray-100 dark:bg-gray-700 px-3 py-2 rounded-lg mb-4 break-all">
                                {cancelConfirm.walletAddress}
                            </p>
                            <p className="text-sm text-gray-500 dark:text-gray-400 mb-6">
                                This will revoke their access to the plan's features.
                            </p>
                            <div className="flex gap-3">
                                <Button
                                    variant="outline"
                                    onClick={() => setCancelConfirm(null)}
                                    className="flex-1"
                                >
                                    Keep Subscription
                                </Button>
                                <Button
                                    variant="destructive"
                                    onClick={() => {
                                        cancelSubscriptionMutation.mutate(cancelConfirm.subscriptionId)
                                        setCancelConfirm(null)
                                    }}
                                    disabled={cancelSubscriptionMutation.isPending}
                                    className="flex-1 bg-red-600 hover:bg-red-700 text-white"
                                >
                                    {cancelSubscriptionMutation.isPending ? 'Cancelling...' : 'Cancel Subscription'}
                                </Button>
                            </div>
                        </div>
                    </div>
                )}
            </div>
        </div>
    )
}
