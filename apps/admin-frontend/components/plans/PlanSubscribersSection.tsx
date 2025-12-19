'use client'

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { ArrowLeft, Trash2, UserPlus, Users } from 'lucide-react'
import { useState } from 'react'
import { toast } from 'sonner'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { WalletAutocomplete } from '@/components/ui/WalletAutocomplete'
import { createPlansClient, isApiSuccess, type PlanResponse, type SubscriptionResponse } from '@/shared/api/plans'
import { createAdminApiClient } from '@/shared/utils/api-client'

interface PlanSubscribersSectionProps {
    plan: PlanResponse
    onClose: () => void
}

/**
 * Component to manage plan subscribers (add/view/remove)
 * Similar to GroupMembersSection from permissions page
 */
export function PlanSubscribersSection({ plan, onClose }: PlanSubscribersSectionProps) {
    const [newWalletAddress, setNewWalletAddress] = useState('')
    const [showAddSubscriber, setShowAddSubscriber] = useState(false)
    const queryClient = useQueryClient()

    // Fetch plan subscribers
    // Note: plan.id can be number or UUID string depending on backend
    const planIdNum = typeof plan.id === 'number' ? plan.id : parseInt(String(plan.id), 10)
    const planIdValid = !isNaN(planIdNum) ? planIdNum : undefined

    const { data: subscribers = [], isLoading, refetch } = useQuery({
        queryKey: ['plan-subscribers', plan.id],
        queryFn: async () => {
            const plansClient = createPlansClient(createAdminApiClient())
            // Only filter by plan_id if it's a valid number, otherwise fetch all
            const response = await plansClient.getSubscriptions(planIdValid ? { plan_id: planIdValid } : {})
            if (isApiSuccess(response)) {
                const allSubs = (response.data as any)?.subscriptions || response.data || []
                // Client-side filter by plan name if plan_id is UUID
                if (!planIdValid && plan.name) {
                    return allSubs.filter((s: any) => s.plan_name === plan.name)
                }
                return allSubs
            }
            return []
        },
        refetchInterval: 30000,
    })

    // Add subscriber mutation
    const addSubscriberMutation = useMutation({
        mutationFn: async (walletAddress: string) => {
            const plansClient = createPlansClient(createAdminApiClient())
            const response = await plansClient.createSubscription({
                user_id: walletAddress,
                plan_id: planIdValid || 0,
                access_context: 'internal',
                auto_renew: false,
                // Add the permission group name based on the plan name
                permission_group_name: plan.name,
            } as any) // Use 'as any' to allow extra field for backend compatibility
            if (!isApiSuccess(response)) {
                throw new Error((response as any).error || 'Failed to add subscriber')
            }
            return response.data
        },
        onSuccess: () => {
            toast.success('Subscriber added successfully')
            setNewWalletAddress('')
            setShowAddSubscriber(false)
            queryClient.invalidateQueries({ queryKey: ['plan-subscribers', plan.id] })
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
            queryClient.invalidateQueries({ queryKey: ['plan-subscribers', plan.id] })
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

    return (
        <div className="bg-white dark:bg-gray-800 rounded-2xl sm:rounded-3xl p-6 shadow-2xl border-2 border-emerald-300/50 dark:border-emerald-700/50">
            <div className="flex justify-between items-start mb-2">
                <h2 className="text-2xl font-bold text-emerald-600 dark:text-emerald-400 flex items-center gap-2">
                    <Users className="w-6 h-6" />
                    Subscribers of "{plan.name}"
                </h2>
                <Button variant="ghost" size="sm" onClick={onClose} className="h-8 w-8 p-0">
                    <span className="sr-only">Close</span>
                    <ArrowLeft className="h-4 w-4" />
                </Button>
            </div>
            <div className="text-sm text-gray-600 dark:text-gray-400 mb-6">
                Total subscribers: {subscribers.length} ({activeSubscribers.length} active)
                {isLoading && ' (Loading...)'}
            </div>

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
                        <div className="text-center py-8">
                            <div className="h-8 w-8 border-b-2 border-emerald-600 mx-auto mb-4 animate-spin"></div>
                            <p className="text-gray-600 dark:text-gray-300">Loading subscribers...</p>
                        </div>
                    ) : subscribers.length === 0 ? (
                        <div className="text-center py-8">
                            <Users className="w-12 h-12 mx-auto mb-4 text-emerald-400" />
                            <p className="text-lg font-medium text-gray-900 dark:text-white">No subscribers to this plan</p>
                            <p className="text-sm text-gray-600 dark:text-gray-300">Add subscribers to grant them access to this plan's features</p>
                        </div>
                    ) : (
                        <div className="space-y-2 max-h-[400px] overflow-y-auto pr-2">
                            {subscribers.map((subscriber: SubscriptionResponse) => {
                                // Handle both user_id and wallet_address field names
                                const walletAddress = (subscriber as any).wallet_address || subscriber.user_id || 'Unknown';
                                return (
                                    <div key={subscriber.id} className="flex items-center justify-between p-3 bg-white dark:bg-gray-900 border rounded-lg hover:shadow-sm transition-shadow">
                                        <div className="flex items-center gap-3">
                                            <div className="h-8 w-8 rounded-full bg-emerald-100 dark:bg-emerald-900/30 flex items-center justify-center text-emerald-600 dark:text-emerald-400 font-mono text-xs">
                                                {walletAddress.substring(2, 4).toUpperCase()}
                                            </div>
                                            <div>
                                                <div className="font-medium font-mono text-sm">
                                                    {walletAddress}
                                                </div>
                                                <div className="text-xs text-gray-500 flex gap-2">
                                                    <Badge variant={subscriber.status === 'active' ? 'default' : 'secondary'} className="text-xs">
                                                        {subscriber.status}
                                                    </Badge>
                                                    <span>Started: {new Date(subscriber.started_at).toLocaleDateString()}</span>
                                                    {subscriber.expires_at && (
                                                        <span className="text-orange-600">
                                                            Expires: {new Date(subscriber.expires_at).toLocaleDateString()}
                                                        </span>
                                                    )}
                                                </div>
                                            </div>
                                        </div>
                                        {subscriber.status === 'active' && (
                                            <Button
                                                variant="ghost"
                                                size="sm"
                                                className="text-red-500 hover:text-red-700 hover:bg-red-50 dark:hover:bg-red-900/10"
                                                onClick={() => {
                                                    if (confirm('Cancel this subscription?')) {
                                                        cancelSubscriptionMutation.mutate(subscriber.id)
                                                    }
                                                }}
                                                disabled={cancelSubscriptionMutation.isPending}
                                            >
                                                <Trash2 className="w-4 h-4" />
                                            </Button>
                                        )}
                                    </div>
                                );
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
                            <span className="ml-2 font-medium text-emerald-600 dark:text-emerald-300">${plan.current_price} {plan.currency}</span>
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
    )
}

export default PlanSubscribersSection
