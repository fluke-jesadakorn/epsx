'use client'

import { useState, useEffect } from 'react'
import { 
  PancakeModal, 
  PancakeCard, 
  PancakeButton,
  type PancakeModalProps 
} from '../../../../shared/components'
import { adminClient, SubscriptionResponse, UpdateSubscriptionRequest, isApiSuccess } from '@/lib/api/unified-admin-client'
import { toast } from '@/hooks/use-toast'
import { logger } from '@/lib/logger'

interface SubscriptionDetailsModalProps {
  subscription: SubscriptionResponse
  onClose: () => void
  onUpdate: () => void
  isOpen: boolean
}

export function SubscriptionDetailsModal({ subscription, onClose, onUpdate, isOpen }: SubscriptionDetailsModalProps) {
  const [isEditing, setIsEditing] = useState(false)
  const [loading, setLoading] = useState(false)
  const [plans, setPlans] = useState<any[]>([])
  const [editData, setEditData] = useState<UpdateSubscriptionRequest>({
    status: subscription.status,
    plan_id: subscription.plan_id,
    expires_at: subscription.expires_at?.split('T')[0] || '',
    auto_renew: subscription.auto_renew
  })

  useEffect(() => {
    loadPlans()
  }, [])

  const loadPlans = async () => {
    try {
      const response = await adminClient.getPlans({ is_active: true })
      if (isApiSuccess(response)) {
        setPlans((response.data as any)?.plans || response.data as any || [])
      }
    } catch (error) {
      logger.error('Failed to load plans', { error })
    }
  }

  const handleUpdate = async () => {
    try {
      setLoading(true)
      const updatePayload: UpdateSubscriptionRequest = {
        ...editData,
        expires_at: editData.expires_at || undefined
      }

      const response = await adminClient.updateSubscription(subscription.id, updatePayload)
      
      if (isApiSuccess(response)) {
        onUpdate()
        setIsEditing(false)
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
    } catch (error) {
      toast({
        title: "Error",
        description: "Failed to update subscription",
        variant: "destructive"
      })
    } finally {
      setLoading(false)
    }
  }

  const handleCancel = async () => {
    if (!confirm('Are you sure you want to cancel this subscription? This action cannot be undone.')) {
      return
    }

    try {
      setLoading(true)
      const response = await adminClient.cancelSubscription(subscription.id)
      
      if (isApiSuccess(response)) {
        onUpdate()
        toast({
          title: "Success",
          description: "Subscription cancelled successfully",
        })
        onClose()
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
    } finally {
      setLoading(false)
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

  return (
    <PancakeModal
      isOpen={isOpen}
      onClose={onClose}
      size="xl"
      variant="elevated"
      title={
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold bg-gradient-to-r from-orange-600 to-yellow-600 bg-clip-text text-transparent">
              Subscription Details
            </h2>
            <div className="flex items-center gap-3 mt-2">
              <span className="text-lg font-semibold text-gray-700 dark:text-gray-300">
                {subscription.plan_name}
              </span>
              <span className={`px-3 py-1 text-sm rounded-full font-semibold ${getStatusColor(subscription.status)}`}>
                {subscription.status.toUpperCase()}
              </span>
            </div>
          </div>
        </div>
      }
    >
      <div className="space-y-6">
        {/* Edit Button */}
        <div className="flex items-center justify-end">
          {!isEditing && subscription.status !== 'cancelled' && (
            <PancakeButton
              variant="secondary"
              onClick={() => setIsEditing(true)}
            >
              Edit
            </PancakeButton>
          )}
        </div>

        {/* Basic Information */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <PancakeCard variant="outlined" className="p-6">
              <h3 className="font-bold text-gray-900 dark:text-white mb-4">Basic Information</h3>
              <div className="space-y-3">
                <div className="flex justify-between">
                  <span className="text-gray-600 dark:text-gray-400">Subscription ID:</span>
                  <span className="font-semibold font-mono">{subscription.id}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600 dark:text-gray-400">User ID:</span>
                  <span className="font-semibold font-mono">{subscription.user_id}</span>
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
          <PancakeCard className="bg-gray-50 dark:bg-gray-700 p-6 mb-8">
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
            <PancakeCard className="bg-blue-50 dark:bg-blue-900/20 p-6 mb-8 border border-blue-200 dark:border-blue-800">
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
                        {plan.name} - ${plan.current_price} {plan.currency}
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
                  disabled={loading}
                  className="flex-1 px-6 py-3 rounded-xl bg-gradient-to-r from-emerald-400 to-green-500 text-white font-semibold hover:from-emerald-500 hover:to-green-600 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {loading ? 'Updating...' : 'Update Subscription'}
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

        {/* Footer Buttons */}
        <div className="flex gap-4 pt-4">
          <PancakeButton variant="outline" onClick={onClose} className="flex-1">
            Close
          </PancakeButton>
          
          {subscription.status === 'active' && !isEditing && (
            <PancakeButton 
              variant="destructive" 
              onClick={handleCancel}
              disabled={loading}
              className="flex-1"
            >
              {loading ? 'Cancelling...' : 'Cancel Subscription'}
            </PancakeButton>
          )}
        </div>
      </div>
    </PancakeModal>
  )
}