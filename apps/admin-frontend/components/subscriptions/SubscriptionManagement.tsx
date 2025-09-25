'use client'

import { useState, useEffect } from 'react'
import { PancakeCard } from '@/components/ui/PancakeCard'
import { adminClient, SubscriptionResponse, isApiSuccess } from '@/lib/api/unified-admin-client'
import { CreateSubscriptionForm } from './CreateSubscriptionForm'
import { SubscriptionDetailsModal } from './SubscriptionDetailsModal'
import { toast } from '@/hooks/use-toast'

interface SubscriptionManagementProps {
  currentUser: any
}

export function SubscriptionManagement({ currentUser }: SubscriptionManagementProps) {
  const [subscriptions, setSubscriptions] = useState<SubscriptionResponse[]>([])
  const [loading, setLoading] = useState(true)
  const [selectedSubscription, setSelectedSubscription] = useState<SubscriptionResponse | null>(null)
  const [isCreating, setIsCreating] = useState(false)
  const [filterStatus, setFilterStatus] = useState<'all' | 'active' | 'expired' | 'cancelled'>('all')
  const [filterContext, setFilterContext] = useState<'all' | 'internal' | 'external' | 'both'>('all')
  const [searchTerm, setSearchTerm] = useState('')

  useEffect(() => {
    loadSubscriptions()
  }, [])

  const loadSubscriptions = async () => {
    try {
      setLoading(true)
      const response = await adminClient.getSubscriptions({
        limit: 100,
        status: filterStatus === 'all' ? undefined : filterStatus,
        access_context: filterContext === 'all' ? undefined : filterContext,
      })

      if (isApiSuccess(response)) {
        setSubscriptions((response.data as any)?.subscriptions || response.data as any || [])
      } else {
        toast({
          title: "Error",
          description: response.error || "Failed to load subscriptions",
          variant: "destructive"
        })
      }
    } catch (error) {
      toast({
        title: "Error",
        description: "Failed to load subscriptions",
        variant: "destructive"
      })
    } finally {
      setLoading(false)
    }
  }

  const handleSubscriptionCreated = () => {
    setIsCreating(false)
    loadSubscriptions()
    toast({
      title: "Success",
      description: "Subscription created successfully",
    })
  }

  const handleCancelSubscription = async (subscriptionId: string) => {
    if (!confirm('Are you sure you want to cancel this subscription?')) {
      return
    }

    try {
      const response = await adminClient.cancelSubscription(subscriptionId)
      
      if (isApiSuccess(response)) {
        loadSubscriptions()
        toast({
          title: "Success",
          description: "Subscription cancelled successfully",
        })
      } else {
        toast({
          title: "Error",
          description: response.error || "Failed to cancel subscription",
          variant: "destructive"
        })
      }
    } catch (error) {
      toast({
        title: "Error",
        description: "Failed to cancel subscription",
        variant: "destructive"
      })
    }
  }

  const filteredSubscriptions = subscriptions.filter(sub => {
    const statusMatch = filterStatus === 'all' || sub.status === filterStatus
    const contextMatch = filterContext === 'all' || sub.access_context === filterContext
    const searchMatch = searchTerm === '' || 
      sub.plan_name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      sub.user_id.toLowerCase().includes(searchTerm.toLowerCase()) ||
      (sub.api_key_name && sub.api_key_name.toLowerCase().includes(searchTerm.toLowerCase()))
    
    return statusMatch && contextMatch && searchMatch
  })

  const activeSubscriptions = subscriptions.filter(s => s.status === 'active')
  const expiredSubscriptions = subscriptions.filter(s => s.status === 'expired')
  const cancelledSubscriptions = subscriptions.filter(s => s.status === 'cancelled')
  const totalRevenue = subscriptions.reduce((sum, sub) => {
    // Estimate monthly revenue - this could be enhanced with actual billing data
    return sum + (sub.status === 'active' ? 99 : 0) // placeholder calculation
  }, 0)

  if (loading) {
    return (
      <div className="max-w-7xl mx-auto space-y-8 ">
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
    <>
      {/* Create Subscription Form Modal */}
      {isCreating && (
        <CreateSubscriptionForm 
          onClose={() => setIsCreating(false)}
          onSuccess={handleSubscriptionCreated}
        />
      )}

      {/* Subscription Details Modal */}
      {selectedSubscription && (
        <SubscriptionDetailsModal 
          isOpen={true}
          subscription={selectedSubscription}
          onClose={() => setSelectedSubscription(null)}
          onUpdate={loadSubscriptions}
        />
      )}

      <div className="max-w-7xl mx-auto space-y-8">
        {/* Hero Section */}
        <div className="text-center mb-12">
          <div className="inline-flex items-center gap-3 bg-gradient-to-r from-emerald-400 to-green-500 text-white px-8 py-4 rounded-2xl shadow-xl mb-6">
            <span className="text-3xl">📋</span>
            <h1 className="text-3xl font-bold">Subscription Management</h1>
          </div>
          <p className="text-xl text-gray-600 dark:text-gray-300 max-w-2xl mx-auto">
            Manage user subscriptions, track usage, and handle billing for all plans
          </p>
        </div>

        {/* Action Cards */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
          <PancakeCard 
            className="bg-gradient-to-br from-emerald-400 via-green-500 to-teal-500 text-white cursor-pointer  transition-transform"
            onClick={() => setIsCreating(true)}
          >
            <div className="p-8">
              <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-6">
                <span className="text-2xl">➕</span>
              </div>
              <h3 className="text-2xl font-bold mb-4">Create Subscription</h3>
              <p className="text-white/80 mb-6">Assign plans to users with custom configurations</p>
              <div className="bg-white/20 rounded-2xl px-6 py-3 text-center font-semibold">
                New Subscription
              </div>
            </div>
          </PancakeCard>

          <PancakeCard 
            className="bg-gradient-to-br from-blue-400 via-purple-500 to-pink-500 text-white cursor-pointer  transition-transform"
            onClick={() => loadSubscriptions()}
          >
            <div className="p-8">
              <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-6">
                <span className="text-2xl">🔄</span>
              </div>
              <h3 className="text-2xl font-bold mb-4">Refresh Data</h3>
              <p className="text-white/80 mb-6">Reload subscription data from server</p>
              <div className="bg-white/20 rounded-2xl px-6 py-3 text-center font-semibold">
                Refresh
              </div>
            </div>
          </PancakeCard>

          <PancakeCard className="bg-gradient-to-br from-orange-400 via-red-500 to-pink-500 text-white">
            <div className="p-8">
              <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-6">
                <span className="text-2xl">📊</span>
              </div>
              <h3 className="text-2xl font-bold mb-4">Usage Analytics</h3>
              <p className="text-white/80 mb-6">View detailed usage and billing analytics</p>
              <div className="bg-white/20 rounded-2xl px-6 py-3 text-center font-semibold">
                Coming Soon
              </div>
            </div>
          </PancakeCard>
        </div>

        {/* Stats Grid */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          <PancakeCard className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-white/20">
            <div className="p-6">
              <div className="text-emerald-600 dark:text-emerald-400 font-semibold mb-2">Total Subscriptions</div>
              <div className="text-3xl font-bold text-gray-900 dark:text-white mb-1">{subscriptions.length}</div>
              <div className="text-sm text-gray-500 dark:text-gray-400">All statuses</div>
            </div>
          </PancakeCard>

          <PancakeCard className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-white/20">
            <div className="p-6">
              <div className="text-blue-600 dark:text-blue-400 font-semibold mb-2">Active</div>
              <div className="text-3xl font-bold text-gray-900 dark:text-white mb-1">{activeSubscriptions.length}</div>
              <div className="text-sm text-gray-500 dark:text-gray-400">Currently active</div>
            </div>
          </PancakeCard>

          <PancakeCard className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-white/20">
            <div className="p-6">
              <div className="text-yellow-600 dark:text-yellow-400 font-semibold mb-2">Expired</div>
              <div className="text-3xl font-bold text-gray-900 dark:text-white mb-1">{expiredSubscriptions.length}</div>
              <div className="text-sm text-gray-500 dark:text-gray-400">Need renewal</div>
            </div>
          </PancakeCard>

          <PancakeCard className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-white/20">
            <div className="p-6">
              <div className="text-green-600 dark:text-green-400 font-semibold mb-2">Est. Revenue</div>
              <div className="text-3xl font-bold text-gray-900 dark:text-white mb-1">${totalRevenue}</div>
              <div className="text-sm text-gray-500 dark:text-gray-400">Monthly estimate</div>
            </div>
          </PancakeCard>
        </div>

        {/* Filters and Search */}
        <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-3xl p-6 border border-white/20 mb-8">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div>
              <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                Search
              </label>
              <input
                type="text"
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                placeholder="Search by plan, user, or API key..."
                className="w-full px-4 py-3 rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-emerald-500"
              />
            </div>

            <div>
              <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                Status
              </label>
              <select
                value={filterStatus}
                onChange={(e) => setFilterStatus(e.target.value as any)}
                className="w-full px-4 py-3 rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-emerald-500"
              >
                <option value="all">All Status</option>
                <option value="active">Active</option>
                <option value="expired">Expired</option>
                <option value="cancelled">Cancelled</option>
              </select>
            </div>

            <div>
              <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                Access Context
              </label>
              <select
                value={filterContext}
                onChange={(e) => setFilterContext(e.target.value as any)}
                className="w-full px-4 py-3 rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-emerald-500"
              >
                <option value="all">All Context</option>
                <option value="internal">Internal</option>
                <option value="external">External</option>
                <option value="both">Both</option>
              </select>
            </div>

            <div className="flex items-end">
              <button
                onClick={loadSubscriptions}
                className="w-full px-6 py-3 bg-gradient-to-r from-emerald-400 to-green-500 text-white rounded-xl font-semibold hover:from-emerald-500 hover:to-green-600"
              >
                Apply Filters
              </button>
            </div>
          </div>
        </div>

        {/* Subscriptions List */}
        <PancakeCard className="bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm border border-white/30 overflow-hidden">
          <div className="p-8">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-2xl font-bold bg-gradient-to-r from-emerald-600 to-green-600 bg-clip-text text-transparent">
                Subscriptions
              </h2>
              <div className="text-sm text-gray-500 dark:text-gray-400">
                {filteredSubscriptions.length} subscriptions
              </div>
            </div>

            <div className="space-y-4">
              {filteredSubscriptions.map(subscription => (
                <div
                  key={subscription.id}
                  className="flex items-center justify-between p-6 bg-gradient-to-r from-gray-50 to-gray-100 dark:from-gray-700 dark:to-gray-800 rounded-2xl hover:from-emerald-50 hover:to-green-50 dark:hover:from-gray-600 dark:hover:to-gray-700  cursor-pointer"
                  onClick={() => setSelectedSubscription(subscription)}
                >
                  <div className="flex items-center gap-6 flex-1">
                    <div className={`h-12 w-12 rounded-2xl flex items-center justify-center text-2xl ${
                      subscription.access_context === 'internal' 
                        ? 'bg-gradient-to-br from-blue-400 to-purple-500'
                        : subscription.access_context === 'external'
                        ? 'bg-gradient-to-br from-orange-400 to-red-500'
                        : 'bg-gradient-to-br from-purple-400 to-pink-500'
                    }`}>
                      {subscription.access_context === 'internal' ? '🖥️' : 
                       subscription.access_context === 'external' ? '🔧' : '🔄'}
                    </div>
                    
                    <div className="flex-1">
                      <div className="flex items-center gap-3 mb-2">
                        <h3 className="text-lg font-bold text-gray-900 dark:text-white">
                          {subscription.plan_name}
                        </h3>
                        <span className={`px-3 py-1 text-xs rounded-full font-semibold ${
                          subscription.status === 'active' 
                            ? 'bg-green-100 text-green-600 dark:bg-green-900/20 dark:text-green-400'
                            : subscription.status === 'expired'
                            ? 'bg-yellow-100 text-yellow-600 dark:bg-yellow-900/20 dark:text-yellow-400'
                            : 'bg-red-100 text-red-600 dark:bg-red-900/20 dark:text-red-400'
                        }`}>
                          {subscription.status.toUpperCase()}
                        </span>
                      </div>
                      <div className="flex items-center gap-4 text-sm text-gray-600 dark:text-gray-400">
                        <span className="font-semibold">User: {subscription.user_id}</span>
                        <span>•</span>
                        <span>{subscription.access_context}</span>
                        {subscription.api_key_name && (
                          <>
                            <span>•</span>
                            <span>API Key: {subscription.api_key_name}</span>
                          </>
                        )}
                        {subscription.expires_at && (
                          <>
                            <span>•</span>
                            <span>Expires: {new Date(subscription.expires_at).toLocaleDateString()}</span>
                          </>
                        )}
                      </div>
                    </div>
                  </div>

                  <div className="flex items-center gap-4">
                    {subscription.status === 'active' && (
                      <button
                        onClick={(e) => {
                          e.stopPropagation()
                          handleCancelSubscription(subscription.id)
                        }}
                        className="px-4 py-2 rounded-xl font-semibold bg-red-100 dark:bg-red-900/20 text-red-600 dark:text-red-400 hover:bg-red-200 dark:hover:bg-red-900/40 "
                      >
                        Cancel
                      </button>
                    )}
                    
                    <button
                      className="px-4 py-2 rounded-xl font-semibold bg-gradient-to-r from-blue-400 to-blue-500 text-white  hover:from-blue-500 hover:to-blue-600"
                    >
                      View Details
                    </button>
                  </div>
                </div>
              ))}
            </div>

            {filteredSubscriptions.length === 0 && (
              <div className="text-center py-12">
                <div className="text-6xl mb-4">📋</div>
                <h3 className="text-xl font-semibold text-gray-600 dark:text-gray-400 mb-2">
                  No subscriptions found
                </h3>
                <p className="text-gray-500 dark:text-gray-500">
                  {subscriptions.length === 0 
                    ? 'Start by creating your first subscription'
                    : 'Try adjusting your filters or search terms'
                  }
                </p>
              </div>
            )}
          </div>
        </PancakeCard>
      </div>
    </>
  )
}