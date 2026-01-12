'use client'

import { useRouter } from 'next/navigation'
import { useEffect, useState } from 'react'

import { toast } from '@/hooks/use-toast'
import { createPlansClient, isApiSuccess, type SubscriptionResponse } from '@/shared/api/plans'
import { createAdminApiClient } from '@/shared/utils/api-client'

interface SubscriptionManagementProps {
  currentUser: any
}

/**
 *
 * @param root0
 * @param root0.currentUser
 */
export function SubscriptionManagement({ currentUser: _currentUser }: SubscriptionManagementProps) {
  const router = useRouter()
  const [subscriptions, setSubscriptions] = useState<SubscriptionResponse[]>([])
  const [loading, setLoading] = useState(true)
  const [filterStatus, setFilterStatus] = useState<'all' | 'active' | 'expired' | 'cancelled'>('all')
  const [filterContext, setFilterContext] = useState<'all' | 'internal' | 'external' | 'both'>('all')
  const [searchTerm, setSearchTerm] = useState('')

  useEffect(() => {
    loadSubscriptions()
  }, [])

  const loadSubscriptions = async () => {
    const adminClient = createPlansClient(createAdminApiClient())
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
    } catch (_error) {
      toast({
        title: "Error",
        description: "Failed to load subscriptions",
        variant: "destructive"
      })
    } finally {
      setLoading(false)
    }
  }

  const handleCancelSubscription = async (subscriptionId: string) => {
    if (!confirm('Are you sure you want to cancel this subscription?')) {
      return
    }

    const adminClient = createPlansClient(createAdminApiClient())
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
    } catch (_error) {
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
      (sub.api_key_name?.toLowerCase().includes(searchTerm.toLowerCase()))

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
      <div className="max-w-7xl mx-auto space-y-8 animate-pulse">
        <div className="text-center mb-12">
          <div className="h-16 bg-muted rounded-2xl w-96 mx-auto mb-6"></div>
          <div className="h-6 bg-muted/60 rounded-full w-64 mx-auto"></div>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
          {Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="bg-muted rounded-3xl h-64"></div>
          ))}
        </div>
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          {Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className="bg-muted rounded-3xl h-32"></div>
          ))}
        </div>
      </div>
    )
  }

  return (
    <>
      <div className="max-w-7xl mx-auto space-y-8">
        {/* Hero Section */}
        <div className="text-center mb-12">
          <div className="relative inline-block mb-6">
            <h1 className="text-3xl sm:text-4xl lg:text-5xl font-bold bg-gradient-to-r from-primary via-secondary to-primary bg-clip-text text-transparent">
              📋 Subscription Management
            </h1>
            <div className="absolute -top-2 -right-2 w-4 h-4 bg-primary/20 rounded-full animate-ping"></div>
          </div>
          <p className="text-base sm:text-lg text-muted-foreground max-w-2xl mx-auto">
            Manage user subscriptions, track usage, and handle billing for all plans
          </p>
        </div>

        {/* Action Cards */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
          <div
            className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-success/10 p-0.5 cursor-pointer"
            onClick={() => router.push('/subscriptions/new')}
          >
            <div className="relative bg-success text-success-foreground rounded-2xl sm:rounded-3xl hover:opacity-90 transition-opacity">
              <div className="p-8">
                <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-6">
                  <span className="text-2xl">➕</span>
                </div>
                <h3 className="text-2xl font-bold mb-4">Create Subscription</h3>
                <p className="text-success-foreground/80 mb-6">Assign plans to users with custom configurations</p>
                <div className="bg-white/20 rounded-2xl px-6 py-3 text-center font-semibold">
                  New Subscription
                </div>
              </div>
            </div>
          </div>

          <div
            className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-secondary/10 p-0.5 cursor-pointer"
            onClick={() => loadSubscriptions()}
          >
            <div className="relative bg-secondary text-secondary-foreground rounded-2xl sm:rounded-3xl hover:opacity-90 transition-opacity">
              <div className="p-8">
                <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-6">
                  <span className="text-2xl">🔄</span>
                </div>
                <h3 className="text-2xl font-bold mb-4">Refresh Data</h3>
                <p className="text-secondary-foreground/80 mb-6">Reload subscription data from server</p>
                <div className="bg-white/20 rounded-2xl px-6 py-3 text-center font-semibold">
                  Refresh
                </div>
              </div>
            </div>
          </div>

          <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-primary/10 p-0.5">
            <div className="relative bg-primary text-primary-foreground rounded-2xl sm:rounded-3xl">
              <div className="p-8">
                <div className="bg-white/20 rounded-2xl w-12 h-12 flex items-center justify-center mb-6">
                  <span className="text-2xl">📊</span>
                </div>
                <h3 className="text-2xl font-bold mb-4">Usage Analytics</h3>
                <p className="text-primary-foreground/80 mb-6">View detailed usage and billing analytics</p>
                <div className="bg-white/20 rounded-2xl px-6 py-3 text-center font-semibold">
                  Coming Soon
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Stats Grid */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          <div className="bg-card/80 backdrop-blur-sm rounded-3xl p-6 border border-primary/20 shadow-sm">
            <div className="text-secondary font-semibold mb-2">Total Subscriptions</div>
            <div className="text-3xl font-bold text-foreground mb-1">{subscriptions.length}</div>
            <div className="text-sm text-muted-foreground">All statuses</div>
          </div>

          <div className="bg-card/80 backdrop-blur-sm rounded-3xl p-6 border border-primary/20 shadow-sm">
            <div className="text-primary font-semibold mb-2">Active</div>
            <div className="text-3xl font-bold text-foreground mb-1">{activeSubscriptions.length}</div>
            <div className="text-sm text-muted-foreground">Currently active</div>
          </div>

          <div className="bg-card/80 backdrop-blur-sm rounded-3xl p-6 border border-primary/20 shadow-sm">
            <div className="text-warning font-semibold mb-2">Expired</div>
            <div className="text-3xl font-bold text-foreground mb-1">{expiredSubscriptions.length}</div>
            <div className="text-sm text-muted-foreground">Need renewal</div>
          </div>

          <div className="bg-card/80 backdrop-blur-sm rounded-3xl p-6 border border-primary/20 shadow-sm">
            <div className="text-success font-semibold mb-2">Est. Revenue</div>
            <div className="text-3xl font-bold text-foreground mb-1">${totalRevenue}</div>
            <div className="text-sm text-muted-foreground">Monthly estimate</div>
          </div>
        </div>

        {/* Filters and Search */}
        <div className="bg-card/80 backdrop-blur-sm rounded-3xl p-6 border border-border/50 mb-8 shadow-sm">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div>
              <label className="block text-sm font-semibold text-muted-foreground mb-2">
                Search
              </label>
              <input
                type="text"
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                placeholder="Search by plan, user..."
                className="w-full px-4 py-3 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all"
              />
            </div>

            <div>
              <label className="block text-sm font-semibold text-muted-foreground mb-2">
                Status
              </label>
              <select
                value={filterStatus}
                onChange={(e) => setFilterStatus(e.target.value as any)}
                className="w-full px-4 py-3 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all"
              >
                <option value="all">All Status</option>
                <option value="active">Active</option>
                <option value="expired">Expired</option>
                <option value="cancelled">Cancelled</option>
              </select>
            </div>

            <div>
              <label className="block text-sm font-semibold text-muted-foreground mb-2">
                Access Context
              </label>
              <select
                value={filterContext}
                onChange={(e) => setFilterContext(e.target.value as any)}
                className="w-full px-4 py-3 bg-muted/50 border border-border/50 rounded-xl text-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all"
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
                className="w-full px-6 py-3 bg-primary text-primary-foreground rounded-xl font-semibold hover:opacity-90 transition-opacity border border-border/50 shadow-sm"
              >
                Apply Filters
              </button>
            </div>
          </div>
        </div>

        {/* Subscriptions List */}
        <div className="bg-card/90 backdrop-blur-sm border border-border/30 overflow-hidden rounded-3xl shadow-sm">
          <div className="p-8">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-2xl font-bold bg-gradient-to-r from-primary via-secondary to-primary bg-clip-text text-transparent">
                Subscriptions
              </h2>
              <div className="text-sm text-muted-foreground">
                {filteredSubscriptions.length} subscriptions
              </div>
            </div>

            <div className="space-y-4">
              {filteredSubscriptions.map(subscription => (
                <div
                  key={subscription.id}
                  className="flex items-center justify-between p-6 bg-muted/20 border border-border/50 rounded-2xl hover:bg-muted/40 transition-all cursor-pointer"
                  onClick={() => router.push(`/subscriptions/${subscription.id}`)}
                >
                  <div className="flex items-center gap-6 flex-1">
                    <div className={`h-12 w-12 rounded-2xl flex items-center justify-center text-2xl ${subscription.access_context === 'internal'
                      ? 'bg-secondary text-secondary-foreground'
                      : subscription.access_context === 'external'
                        ? 'bg-primary text-primary-foreground'
                        : 'bg-info text-info-foreground'
                      }`}>
                      {subscription.access_context === 'internal' ? '🖥️' :
                        subscription.access_context === 'external' ? '🔧' : '🔄'}
                    </div>

                    <div className="flex-1">
                      <div className="flex items-center gap-3 mb-2">
                        <h3 className="text-lg font-bold text-foreground">
                          {subscription.plan_name}
                        </h3>
                        <span className={`px-3 py-1 text-xs rounded-full font-semibold ${subscription.status === 'active'
                          ? 'bg-success/10 text-success border border-success/20'
                          : subscription.status === 'expired'
                            ? 'bg-warning/10 text-warning border border-warning/20'
                            : 'bg-destructive/10 text-destructive border border-destructive/20'
                          }`}>
                          {subscription.status.toUpperCase()}
                        </span>
                      </div>
                      <div className="flex items-center gap-4 text-sm text-muted-foreground">
                        <span className="font-semibold text-foreground/80">User: {subscription.user_id}</span>
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
                        className="px-4 py-2 rounded-xl font-semibold bg-destructive/10 text-destructive hover:bg-destructive/20 transition-colors border border-destructive/20"
                      >
                        Cancel
                      </button>
                    )}

                    <button
                      className="px-4 py-2 rounded-xl font-semibold bg-secondary text-secondary-foreground hover:opacity-90 transition-opacity border border-secondary/20 shadow-sm"
                    >
                      View Details
                    </button>
                  </div>
                </div>
              ))}
            </div>

            {filteredSubscriptions.length === 0 && (
              <div className="text-center py-12">
                <div className="h-20 w-20 bg-primary/10 rounded-2xl flex items-center justify-center mx-auto mb-4">
                  <span className="text-4xl text-primary">📋</span>
                </div>
                <h3 className="text-xl font-semibold text-foreground mb-2">
                  No subscriptions found
                </h3>
                <p className="text-muted-foreground">
                  {subscriptions.length === 0
                    ? 'Start by creating your first subscription'
                    : 'Try adjusting your filters or search terms'
                  }
                </p>
              </div>
            )}
          </div>
        </div>
      </div>
    </>
  )
}