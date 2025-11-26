'use client'

import { useState } from 'react'
import { useRouter } from 'next/navigation'
import { PancakeCard } from '@/components/ui/PancakeCard'
import { toast } from '@/hooks/use-toast'
import { createPlansClient, isApiSuccess } from '@/shared/api/plans'
import { createAdminApiClient } from '@/shared/utils/api-client'
import { PermissionTemplateName, PERMISSION_TEMPLATE_CONFIGS } from '@/types/permission-templates'
import { useSharedAuth } from '@/shared/components/auth/Provider'

interface CreatePermissionTemplateRequest {
  name: string
  description: string
  template_name: PermissionTemplateName
  permissions: string[]
  current_price: number
  currency: string
  target_audience: string
  billing_model: string
  features: string[]
  metadata: any
}

export default function NewPlanPage() {
  const router = useRouter()
  const { user, isLoading: authLoading, isAuthenticated } = useSharedAuth()
  const [loading, setLoading] = useState(false)
  const [formData, setFormData] = useState<CreatePermissionTemplateRequest>({
    name: '',
    description: '',
    template_name: 'Free Template',
    permissions: [],
    current_price: 0,
    currency: 'USD',
    target_audience: 'web_users',
    billing_model: 'pay_per_use',
    features: [],
    metadata: {}
  })
  const [customPermission, setCustomPermission] = useState('')

  if (authLoading) {
    return <div className="min-h-screen flex items-center justify-center">Loading...</div>
  }

  if (!isAuthenticated || !user) {
    router.push('/plans')
    return null
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    const adminClient = createPlansClient(createAdminApiClient())

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

    if (formData.permissions.length === 0) {
      toast({
        title: "Error",
        description: "At least one permission is required",
        variant: "destructive"
      })
      return
    }

    try {
      setLoading(true)
      const planRequest = {
        name: formData.name,
        description: formData.description,
        plan_type: 'subscription',
        current_price: formData.current_price,
        currency: formData.currency,
        target_audience: formData.target_audience,
        billing_model: formData.billing_model,
        plan_category: 'permission_template',
        features: formData.features.map(feature => ({
          context_name: 'web_app',
          feature_key: feature,
          feature_config: {},
          resource_cost: 1.0,
          is_active: true
        })),
        metadata: {
          permission_template: formData.template_name,
          permissions: formData.permissions,
          ...formData.metadata
        }
      }

      const response = await adminClient.createPlan(planRequest)

      if (isApiSuccess(response)) {
        toast({
          title: "Success",
          description: "Permission template plan created successfully",
        })
        router.push('/plans')
      } else {
        toast({
          title: "Error",
          description: response.error || "Failed to create plan",
          variant: "destructive"
        })
      }
    } catch (_error) {
      toast({
        title: "Error",
        description: "Failed to create plan",
        variant: "destructive"
      })
    } finally {
      setLoading(false)
    }
  }

  const handleTemplateChange = (templateName: PermissionTemplateName) => {
    const template = PERMISSION_TEMPLATE_CONFIGS[templateName]
    setFormData({
      ...formData,
      template_name: templateName,
      permissions: [...template.permissions],
      features: [...template.features],
      current_price: templateName === 'Free Template' ? 0 : formData.current_price
    })
  }

  const addCustomPermission = () => {
    if (!customPermission.trim()) {
      toast({
        title: "Error",
        description: "Permission string is required",
        variant: "destructive"
      })
      return
    }

    if (formData.permissions.includes(customPermission)) {
      toast({
        title: "Error",
        description: "Permission already exists",
        variant: "destructive"
      })
      return
    }

    setFormData({
      ...formData,
      permissions: [...formData.permissions, customPermission]
    })
    setCustomPermission('')
  }

  const removePermission = (permission: string) => {
    setFormData({
      ...formData,
      permissions: formData.permissions.filter(p => p !== permission)
    })
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      <div className="max-w-4xl mx-auto">
        <PancakeCard className="bg-white dark:bg-gray-800">
          <div className="p-8">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-2xl font-bold bg-gradient-to-r from-emerald-600 to-green-600 bg-clip-text text-transparent">
                Create Permission Template Plan
              </h2>
              <button
                onClick={() => router.push('/plans')}
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

              {/* Permission Template Selection */}
              <div>
                <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                  Permission Template *
                </label>
                <select
                  value={formData.template_name}
                  onChange={(e) => handleTemplateChange(e.target.value as PermissionTemplateName)}
                  className="w-full px-4 py-3 rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-emerald-500"
                >
                  {Object.keys(PERMISSION_TEMPLATE_CONFIGS).map(templateName => {
                    const template = PERMISSION_TEMPLATE_CONFIGS[templateName as PermissionTemplateName]
                    return (
                      <option key={templateName} value={templateName}>
                        {template.name} - {template.description}
                      </option>
                    )
                  })}
                </select>
                <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                  Selecting a template will automatically populate permissions and features
                </p>
              </div>

              {/* Categories */}
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

              {/* Permissions Management */}
              <div>
                <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-4">
                  Plan Permissions
                </label>

                {/* Add Custom Permission */}
                <div className="bg-gray-50 dark:bg-gray-700 p-4 rounded-xl mb-4">
                  <div className="flex gap-4">
                    <input
                      type="text"
                      value={customPermission}
                      onChange={(e) => setCustomPermission(e.target.value)}
                      placeholder="e.g., epsx:rankings:view:100"
                      className="flex-1 px-3 py-2 rounded-lg border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                    />
                    <button
                      type="button"
                      onClick={addCustomPermission}
                      className="px-4 py-2 bg-gradient-to-r from-emerald-400 to-green-500 text-white rounded-lg font-semibold hover:from-emerald-500 hover:to-green-600"
                    >
                      Add Permission
                    </button>
                  </div>
                  <p className="text-xs text-gray-500 dark:text-gray-400 mt-2">
                    Format: platform:resource:action or platform:resource:action:limit
                  </p>
                </div>

                {/* Current Template Info */}
                {formData.template_name && (
                  <div className="bg-blue-50 dark:bg-blue-900/20 p-4 rounded-xl mb-4">
                    <h4 className="font-semibold text-blue-800 dark:text-blue-300 mb-2">
                      Current Template: {PERMISSION_TEMPLATE_CONFIGS[formData.template_name].name}
                    </h4>
                    <p className="text-sm text-blue-700 dark:text-blue-400">
                      {PERMISSION_TEMPLATE_CONFIGS[formData.template_name].description}
                    </p>
                  </div>
                )}

                {/* Permission List */}
                {formData.permissions.length > 0 && (
                  <div className="space-y-2">
                    <h4 className="font-medium text-gray-800 dark:text-gray-200">Assigned Permissions ({formData.permissions.length})</h4>
                    <div className="max-h-48 overflow-y-auto space-y-2">
                      {formData.permissions.map((permission, index) => (
                        <div key={index} className="flex items-center justify-between p-3 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-600">
                          <code className="text-sm font-mono text-gray-800 dark:text-gray-200">
                            {permission}
                          </code>
                          <button
                            type="button"
                            onClick={() => removePermission(permission)}
                            className="text-red-500 hover:text-red-700 font-bold ml-2"
                          >
                            ✕
                          </button>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                {/* Features Preview */}
                {formData.features.length > 0 && (
                  <div className="mt-4">
                    <h4 className="font-medium text-gray-800 dark:text-gray-200 mb-2">Template Features</h4>
                    <div className="flex flex-wrap gap-2">
                      {formData.features.map((feature, index) => (
                        <span key={index} className="px-3 py-1 bg-green-100 dark:bg-green-900/20 text-green-800 dark:text-green-300 rounded-full text-sm">
                          {feature}
                        </span>
                      ))}
                    </div>
                  </div>
                )}
              </div>

              {/* Submit Buttons */}
              <div className="flex gap-4 pt-6">
                <button
                  type="button"
                  onClick={() => router.push('/plans')}
                  className="flex-1 px-6 py-3 rounded-xl border border-gray-200 dark:border-gray-600 text-gray-700 dark:text-gray-300 font-semibold hover:bg-gray-50 dark:hover:bg-gray-700"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={loading}
                  className="flex-1 px-6 py-3 rounded-xl bg-gradient-to-r from-emerald-400 to-green-500 text-white font-semibold hover:from-emerald-500 hover:to-green-600 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {loading ? 'Creating...' : 'Create Permission Template Plan'}
                </button>
              </div>
            </form>
          </div>
        </PancakeCard>
      </div>
    </div>
  )
}
