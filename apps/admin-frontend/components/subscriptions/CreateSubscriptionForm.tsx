'use client'

import { useState, useEffect } from 'react'
import { PancakeCard } from '@/components/ui/PancakeCard'
import { adminClient, CreateSubscriptionRequest, isApiSuccess } from '@/lib/api/unified-admin-client'
import { toast } from '@/hooks/use-toast'

interface CreateSubscriptionFormProps {
  onClose: () => void
  onSuccess: () => void
}

export function CreateSubscriptionForm({ onClose, onSuccess }: CreateSubscriptionFormProps) {
  const [loading, setLoading] = useState(false)
  const [plans, setPlans] = useState<any[]>([])
  const [formData, setFormData] = useState<CreateSubscriptionRequest>({
    user_id: '',
    plan_id: 0,
    access_context: 'internal',
    api_key_name: '',
    expires_at: '',
    auto_renew: true,
    metadata: {}
  })
  const [showApiKeyField, setShowApiKeyField] = useState(false)

  useEffect(() => {
    loadPlans()
  }, [])

  const loadPlans = async () => {
    try {
      const response = await adminClient.getPlans({ is_active: true })
      if (isApiSuccess(response)) {
        setPlans(response.data?.plans || [])
      }
    } catch (error) {
      console.error('Failed to load plans:', error)
    }
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    if (!formData.user_id.trim()) {
      toast({
        title: "Error",
        description: "User ID is required",
        variant: "destructive"
      })
      return
    }

    if (formData.plan_id === 0) {
      toast({
        title: "Error",
        description: "Please select a plan",
        variant: "destructive"
      })
      return
    }

    if ((formData.access_context === 'external' || formData.access_context === 'both') && !formData.api_key_name?.trim()) {
      toast({
        title: "Error",
        description: "API key name is required for external access",
        variant: "destructive"
      })
      return
    }

    try {
      setLoading(true)
      const subscriptionData: CreateSubscriptionRequest = {
        ...formData,
        api_key_name: showApiKeyField ? formData.api_key_name : undefined,
        expires_at: formData.expires_at || undefined
      }

      const response = await adminClient.createSubscription(subscriptionData)
      
      if (isApiSuccess(response)) {
        onSuccess()
        toast({
          title: "Success",
          description: "Subscription created successfully",
        })
      } else {
        toast({
          title: "Error",
          description: response.error || "Failed to create subscription",
          variant: "destructive"
        })
      }
    } catch (error) {
      toast({
        title: "Error",
        description: "Failed to create subscription",
        variant: "destructive"
      })
    } finally {
      setLoading(false)
    }
  }

  const handleAccessContextChange = (context: string) => {
    setFormData({ ...formData, access_context: context })
    setShowApiKeyField(context === 'external' || context === 'both')
  }

  // Generate default expiry date (1 year from now)
  const defaultExpiryDate = new Date()
  defaultExpiryDate.setFullYear(defaultExpiryDate.getFullYear() + 1)
  const defaultExpiryString = defaultExpiryDate.toISOString().split('T')[0]

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center p-4 z-50">
      <PancakeCard className="bg-white dark:bg-gray-800 max-w-3xl w-full max-h-[90vh] overflow-y-auto">
        <div className="p-8">
          <div className="flex items-center justify-between mb-6">
            <h2 className="text-2xl font-bold bg-gradient-to-r from-emerald-600 to-green-600 bg-clip-text text-transparent">
              Create New Subscription
            </h2>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
            >
              ✕
            </button>
          </div>

          <form onSubmit={handleSubmit} className="space-y-6">
            {/* User and Plan Selection */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                  User ID *
                </label>
                <input
                  type="text"
                  value={formData.user_id}
                  onChange={(e) => setFormData({ ...formData, user_id: e.target.value })}
                  className="w-full px-4 py-3 rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-emerald-500"
                  placeholder="Enter user ID"
                  required
                />
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  The Firebase UID or email of the user
                </p>
              </div>

              <div>
                <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                  Plan *
                </label>
                <select
                  value={formData.plan_id}
                  onChange={(e) => setFormData({ ...formData, plan_id: parseInt(e.target.value) })}
                  className="w-full px-4 py-3 rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-emerald-500"
                  required
                >
                  <option value={0}>Select a plan</option>
                  {plans.map(plan => (
                    <option key={plan.id} value={plan.id}>
                      {plan.name} - ${plan.current_price} {plan.currency} ({plan.plan_category})
                    </option>
                  ))}
                </select>
              </div>
            </div>

            {/* Access Context */}
            <div>
              <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-4">
                Access Context *
              </label>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                {[
                  { value: 'internal', label: 'Internal', desc: 'Web app access only', icon: '🖥️' },
                  { value: 'external', label: 'External', desc: 'API access with key', icon: '🔧' },
                  { value: 'both', label: 'Both', desc: 'Web + API access', icon: '🔄' }
                ].map(option => (
                  <label
                    key={option.value}
                    className={`relative flex flex-col p-4 rounded-xl border-2 cursor-pointer transition-all ${
                      formData.access_context === option.value
                        ? 'border-emerald-500 bg-emerald-50 dark:bg-emerald-900/20'
                        : 'border-gray-200 dark:border-gray-600 hover:border-gray-300'
                    }`}
                  >
                    <input
                      type="radio"
                      name="access_context"
                      value={option.value}
                      checked={formData.access_context === option.value}
                      onChange={(e) => handleAccessContextChange(e.target.value)}
                      className="sr-only"
                    />
                    <div className="flex items-center gap-3 mb-2">
                      <span className="text-2xl">{option.icon}</span>
                      <span className="font-semibold">{option.label}</span>
                    </div>
                    <span className="text-sm text-gray-500 dark:text-gray-400">{option.desc}</span>
                  </label>
                ))}
              </div>
            </div>

            {/* API Key Name (conditional) */}
            {showApiKeyField && (
              <div>
                <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                  API Key Name *
                </label>
                <input
                  type="text"
                  value={formData.api_key_name}
                  onChange={(e) => setFormData({ ...formData, api_key_name: e.target.value })}
                  className="w-full px-4 py-3 rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-emerald-500"
                  placeholder="e.g., Production API Key"
                  required
                />
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  A descriptive name for the API key
                </p>
              </div>
            )}

            {/* Subscription Settings */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                  Expiry Date (Optional)
                </label>
                <input
                  type="date"
                  value={formData.expires_at}
                  onChange={(e) => setFormData({ ...formData, expires_at: e.target.value })}
                  min={new Date().toISOString().split('T')[0]}
                  className="w-full px-4 py-3 rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-emerald-500"
                />
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  Leave empty for no expiry. Default: {defaultExpiryString}
                </p>
              </div>

              <div className="flex items-center justify-center">
                <label className="flex items-center gap-3 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={formData.auto_renew}
                    onChange={(e) => setFormData({ ...formData, auto_renew: e.target.checked })}
                    className="w-5 h-5 text-emerald-500 border-gray-300 rounded focus:ring-emerald-500"
                  />
                  <div>
                    <span className="text-sm font-semibold text-gray-700 dark:text-gray-300">Auto-renew</span>
                    <p className="text-xs text-gray-500 dark:text-gray-400">Automatically renew when expired</p>
                  </div>
                </label>
              </div>
            </div>

            {/* Advanced Settings */}
            <div className="bg-gray-50 dark:bg-gray-700 p-4 rounded-xl">
              <h3 className="font-semibold text-gray-700 dark:text-gray-300 mb-3">Advanced Settings</h3>
              <div>
                <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                  Metadata (JSON)
                </label>
                <textarea
                  value={JSON.stringify(formData.metadata, null, 2)}
                  onChange={(e) => {
                    try {
                      const parsed = JSON.parse(e.target.value || '{}')
                      setFormData({ ...formData, metadata: parsed })
                    } catch {
                      // Invalid JSON - ignore for now
                    }
                  }}
                  rows={3}
                  className="w-full px-4 py-3 rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-emerald-500 font-mono text-sm"
                  placeholder='{"custom_field": "value"}'
                />
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  Custom metadata in JSON format
                </p>
              </div>
            </div>

            {/* Submit Buttons */}
            <div className="flex gap-4 pt-6">
              <button
                type="button"
                onClick={onClose}
                className="flex-1 px-6 py-3 rounded-xl border border-gray-200 dark:border-gray-600 text-gray-700 dark:text-gray-300 font-semibold hover:bg-gray-50 dark:hover:bg-gray-700"
              >
                Cancel
              </button>
              <button
                type="submit"
                disabled={loading}
                className="flex-1 px-6 py-3 rounded-xl bg-gradient-to-r from-emerald-400 to-green-500 text-white font-semibold hover:from-emerald-500 hover:to-green-600 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {loading ? 'Creating...' : 'Create Subscription'}
              </button>
            </div>
          </form>
        </div>
      </PancakeCard>
    </div>
  )
}