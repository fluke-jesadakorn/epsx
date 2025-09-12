'use client'

import { useState } from 'react'
import { PancakeCard } from '@/components/ui/PancakeCard'
import { adminClient, CreatePlanRequest, PlanFeatureRequest, isApiSuccess } from '@/lib/api/unified-admin-client'
import { toast } from '@/hooks/use-toast'

interface CreatePlanFormProps {
  onClose: () => void
  onSuccess: () => void
}

export function CreatePlanForm({ onClose, onSuccess }: CreatePlanFormProps) {
  const [loading, setLoading] = useState(false)
  const [formData, setFormData] = useState<CreatePlanRequest>({
    name: '',
    description: '',
    plan_type: 'subscription',
    current_price: 0,
    currency: 'USD',
    target_audience: 'web_users',
    billing_model: 'subscription',
    plan_category: 'standard',
    features: [],
    metadata: {}
  })
  const [newFeature, setNewFeature] = useState<PlanFeatureRequest>({
    context_name: 'web_app',
    feature_key: '',
    feature_config: {},
    resource_cost: 1.0,
    is_active: true
  })

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    if (!formData.name.trim()) {
      toast({
        title: "Error",
        description: "Plan name is required",
        variant: "destructive"
      })
      return
    }

    if (formData.current_price <= 0) {
      toast({
        title: "Error", 
        description: "Plan price must be greater than 0",
        variant: "destructive"
      })
      return
    }

    try {
      setLoading(true)
      const response = await adminClient.createPlan(formData)
      
      if (isApiSuccess(response)) {
        onSuccess()
        toast({
          title: "Success",
          description: "Plan created successfully",
        })
      } else {
        toast({
          title: "Error",
          description: response.error || "Failed to create plan",
          variant: "destructive"
        })
      }
    } catch (error) {
      toast({
        title: "Error",
        description: "Failed to create plan",
        variant: "destructive"
      })
    } finally {
      setLoading(false)
    }
  }

  const addFeature = () => {
    if (!newFeature.feature_key.trim()) {
      toast({
        title: "Error",
        description: "Feature key is required",
        variant: "destructive"
      })
      return
    }

    setFormData({
      ...formData,
      features: [...formData.features, { ...newFeature }]
    })

    setNewFeature({
      context_name: 'web_app',
      feature_key: '',
      feature_config: {},
      resource_cost: 1.0,
      is_active: true
    })
  }

  const removeFeature = (index: number) => {
    setFormData({
      ...formData,
      features: formData.features.filter((_, i) => i !== index)
    })
  }

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center p-4 z-50">
      <PancakeCard className="bg-white dark:bg-gray-800 max-w-4xl w-full max-h-[90vh] overflow-y-auto">
        <div className="p-8">
          <div className="flex items-center justify-between mb-6">
            <h2 className="text-2xl font-bold bg-gradient-to-r from-emerald-600 to-green-600 bg-clip-text text-transparent">
              Create New Plan
            </h2>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
            >
              ✕
            </button>
          </div>

          <form onSubmit={handleSubmit} className="space-y-6">
            {/* Basic Info */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                  Plan Name *
                </label>
                <input
                  type="text"
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                  className="w-full px-4 py-3 rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-emerald-500"
                  placeholder="e.g., Professional Plan"
                  required
                />
              </div>

              <div>
                <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                  Price *
                </label>
                <div className="flex">
                  <span className="inline-flex items-center px-3 rounded-l-xl border border-r-0 border-gray-200 dark:border-gray-600 bg-gray-50 dark:bg-gray-600 text-gray-500 dark:text-gray-400">
                    $
                  </span>
                  <input
                    type="number"
                    step="0.01"
                    value={formData.current_price}
                    onChange={(e) => setFormData({ ...formData, current_price: parseFloat(e.target.value) || 0 })}
                    className="flex-1 px-4 py-3 rounded-r-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-emerald-500"
                    placeholder="29.99"
                    required
                  />
                </div>
              </div>
            </div>

            {/* Categories */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
              <div>
                <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                  Plan Category
                </label>
                <select
                  value={formData.plan_category}
                  onChange={(e) => setFormData({ ...formData, plan_category: e.target.value })}
                  className="w-full px-4 py-3 rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-emerald-500"
                >
                  <option value="standard">Standard</option>
                  <option value="api">API</option>
                  <option value="enterprise">Enterprise</option>
                  <option value="custom">Custom</option>
                </select>
              </div>

              <div>
                <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                  Target Audience
                </label>
                <select
                  value={formData.target_audience}
                  onChange={(e) => setFormData({ ...formData, target_audience: e.target.value })}
                  className="w-full px-4 py-3 rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-emerald-500"
                >
                  <option value="web_users">Web Users</option>
                  <option value="api_developers">API Developers</option>
                  <option value="enterprises">Enterprises</option>
                </select>
              </div>

              <div>
                <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                  Billing Model
                </label>
                <select
                  value={formData.billing_model}
                  onChange={(e) => setFormData({ ...formData, billing_model: e.target.value })}
                  className="w-full px-4 py-3 rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-emerald-500"
                >
                  <option value="subscription">Subscription</option>
                  <option value="pay_per_use">Pay Per Use</option>
                  <option value="hybrid">Hybrid</option>
                </select>
              </div>
            </div>

            {/* Description */}
            <div>
              <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                Description
              </label>
              <textarea
                value={formData.description}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                rows={3}
                className="w-full px-4 py-3 rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-emerald-500"
                placeholder="Describe your plan features and benefits..."
              />
            </div>

            {/* Features */}
            <div>
              <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-4">
                Plan Features
              </label>
              
              {/* Add Feature */}
              <div className="bg-gray-50 dark:bg-gray-700 p-4 rounded-xl mb-4">
                <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-4">
                  <select
                    value={newFeature.context_name}
                    onChange={(e) => setNewFeature({ ...newFeature, context_name: e.target.value })}
                    className="px-3 py-2 rounded-lg border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                  >
                    <option value="web_app">Web App</option>
                    <option value="api_access">API Access</option>
                    <option value="admin_interface">Admin Interface</option>
                  </select>
                  
                  <input
                    type="text"
                    value={newFeature.feature_key}
                    onChange={(e) => setNewFeature({ ...newFeature, feature_key: e.target.value })}
                    placeholder="Feature key"
                    className="px-3 py-2 rounded-lg border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                  />
                  
                  <input
                    type="number"
                    step="0.1"
                    value={newFeature.resource_cost}
                    onChange={(e) => setNewFeature({ ...newFeature, resource_cost: parseFloat(e.target.value) || 1.0 })}
                    placeholder="Cost"
                    className="px-3 py-2 rounded-lg border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                  />
                  
                  <button
                    type="button"
                    onClick={addFeature}
                    className="px-4 py-2 bg-gradient-to-r from-emerald-400 to-green-500 text-white rounded-lg font-semibold hover:from-emerald-500 hover:to-green-600"
                  >
                    Add
                  </button>
                </div>
              </div>

              {/* Feature List */}
              {formData.features.length > 0 && (
                <div className="space-y-2">
                  {formData.features.map((feature, index) => (
                    <div key={index} className="flex items-center justify-between p-3 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-600">
                      <div className="flex items-center gap-4">
                        <span className="px-2 py-1 bg-blue-100 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 rounded text-sm">
                          {feature.context_name}
                        </span>
                        <span className="font-medium">{feature.feature_key}</span>
                        <span className="text-sm text-gray-500">Cost: {feature.resource_cost}</span>
                      </div>
                      <button
                        type="button"
                        onClick={() => removeFeature(index)}
                        className="text-red-500 hover:text-red-700 font-bold"
                      >
                        ✕
                      </button>
                    </div>
                  ))}
                </div>
              )}
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
                {loading ? 'Creating...' : 'Create Plan'}
              </button>
            </div>
          </form>
        </div>
      </PancakeCard>
    </div>
  )
}