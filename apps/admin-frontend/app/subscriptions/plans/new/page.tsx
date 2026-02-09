'use client'

import { useRouter } from 'next/navigation'
import { useState } from 'react'

import { PermissionTransferList } from '@/components/plans/Permissiontransfer-list'
import { PageLoadingSpinner } from '@/components/ui/loading-spinner'
import { toast } from '@/hooks/use-toast'
import { useAvailablePermissions } from '@/hooks/use-plan-permissions'
import { createPlansClient, isApiSuccess } from '@/shared/api/plans'
import { useSharedAuth } from '@/shared/components/auth/Provider'
import { createAdminApiClient } from '@/shared/utils/api-client'
import type { PermissionTemplateName } from '@/types/permission-templates'
import { PERMISSION_TEMPLATE_CONFIGS } from '@/types/permission-templates'

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
  metadata: Record<string, unknown>
}

/**
 * Create New Plan Page
 * Part of the Subscription & Access hub
 */
// eslint-disable-next-line max-lines-per-function
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

  const { permissions: availablePermissions, isLoading: loadingPermissions } = useAvailablePermissions()

  if (authLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-background">
        <PageLoadingSpinner label="Loading..." />
      </div>
    )
  }

  if (!isAuthenticated ?? !user) {
    router.push('/subscriptions/plans')
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
        permission_group_name: formData.name,
        current_price: formData.current_price.toString(),
        currency: formData.currency,
        target_audience: formData.target_audience,
        billing_model: formData.billing_model,
        permissions: formData.permissions,
        metadata: {
          permission_template: formData.template_name,
          features: formData.features,
          ...formData.metadata
        }
      }

      const response = await adminClient.createPlan(planRequest)

      if (isApiSuccess(response)) {
        toast({
          title: "Success",
          description: "Permission template plan created successfully",
        })
        router.push('/subscriptions/plans')
      } else {
        toast({
          title: "Error",
          description: response.error?.message ?? "Failed to create plan",
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

  return (
    <div className="min-h-screen p-6">
      <div className="max-w-4xl mx-auto">
        <div className="bg-card rounded-3xl border border-border/50 shadow-sm overflow-hidden transition-all">
          <div className="p-8">
            <div className="flex items-center justify-between mb-8 pb-4 border-b border-border/30">
              <div>
                <h1 className="text-2xl font-bold bg-gradient-to-r from-primary via-secondary to-primary bg-clip-text text-transparent">
                  Create Permission Template Plan
                </h1>
                <p className="text-sm text-muted-foreground mt-1 text-card-foreground/70">
                  Define a new subscription offering with custom permissions.
                </p>
              </div>
              <button
                onClick={() => router.push('/subscriptions/plans')}
                className="p-2 rounded-xl text-muted-foreground hover:text-foreground hover:bg-muted transition-all"
              >
                <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            <form onSubmit={(e) => void handleSubmit(e)} className="space-y-8">
              {/* Basic Info */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                <div className="space-y-2">
                  <label className="block text-sm font-bold text-muted-foreground uppercase tracking-wider">
                    Plan Name *
                  </label>
                  <input
                    type="text"
                    value={formData.name}
                    onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                    className="w-full px-4 py-4 rounded-xl border border-border bg-card text-foreground focus:ring-2 focus:ring-primary outline-none transition-all placeholder:text-muted-foreground/50"
                    placeholder="e.g., Professional Plan"
                    required
                  />
                </div>

                <div className="space-y-2">
                  <label className="block text-sm font-bold text-muted-foreground uppercase tracking-wider">
                    Price *
                  </label>
                  <div className="relative group">
                    <span className="absolute left-4 top-1/2 -translate-y-1/2 text-muted-foreground font-bold transition-colors group-focus-within:text-primary">
                      $
                    </span>
                    <input
                      type="number"
                      step="0.01"
                      value={formData.current_price}
                      onChange={(e) => setFormData({ ...formData, current_price: parseFloat(e.target.value) ?? 0 })}
                      className="w-full pl-10 pr-4 py-4 rounded-xl border border-border bg-card text-foreground focus:ring-2 focus:ring-primary outline-none transition-all font-mono"
                      placeholder="29.99"
                      required
                    />
                  </div>
                </div>
              </div>

              {/* Permission Template Selection */}
              <div className="space-y-3">
                <label className="block text-sm font-bold text-muted-foreground uppercase tracking-wider">
                  Permission Template *
                </label>
                <select
                  value={formData.template_name}
                  onChange={(e) => handleTemplateChange(e.target.value as PermissionTemplateName)}
                  className="w-full px-4 py-4 rounded-xl border border-border bg-card text-foreground focus:ring-2 focus:ring-primary outline-none transition-all appearance-none cursor-pointer"
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
                <p className="text-xs text-muted-foreground/70 italic bg-primary/5 p-3 rounded-lg border border-primary/10">
                  <span className="font-bold text-primary mr-1">Tip:</span>
                  Selecting a template will automatically populate the recommended permissions and features below.
                </p>
              </div>

              {/* Categories */}
              <div className="space-y-2">
                <label className="block text-sm font-bold text-muted-foreground uppercase tracking-wider">
                  Target Audience
                </label>
                <select
                  value={formData.target_audience}
                  onChange={(e) => setFormData({ ...formData, target_audience: e.target.value })}
                  className="w-full px-4 py-4 rounded-xl border border-border bg-card text-foreground focus:ring-2 focus:ring-primary outline-none transition-all appearance-none cursor-pointer"
                >
                  <option value="web_users">Web Users</option>
                  <option value="api_developers">API Developers</option>
                  <option value="enterprises">Enterprises</option>
                </select>
              </div>

              {/* Description */}
              <div className="space-y-2">
                <label className="block text-sm font-bold text-muted-foreground uppercase tracking-wider">
                  Description
                </label>
                <textarea
                  value={formData.description}
                  onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                  rows={3}
                  className="w-full px-4 py-4 rounded-xl border border-border bg-card text-foreground focus:ring-2 focus:ring-primary outline-none transition-all resize-none placeholder:text-muted-foreground/50"
                  placeholder="Describe your plan features and benefits..."
                />
              </div>

              {/* Permissions Management */}
              <div className="space-y-4">
                <label className="block text-sm font-bold text-muted-foreground uppercase tracking-wider">
                  Plan Permissions
                </label>

                <div className="bg-muted/30 rounded-2xl border border-border/50 p-6">
                  <PermissionTransferList
                    available={availablePermissions}
                    selected={formData.permissions}
                    onChange={(newSelected: string[]) => setFormData({ ...formData, permissions: newSelected })}
                    isLoading={loadingPermissions}
                    systemPermissions={new Set(
                      availablePermissions.filter((p: string) => p.startsWith('system:') ?? p.startsWith('admin:'))
                    )}
                  />
                </div>

                {/* Features Preview */}
                {formData.features.length > 0 && (
                  <div className="animate-in fade-in slide-in-from-top-2 duration-300 bg-success/5 p-6 rounded-2xl border border-success/20">
                    <h4 className="font-bold text-success/80 text-xs uppercase tracking-widest mb-3">Template Features</h4>
                    <div className="flex flex-wrap gap-2">
                      {formData.features.map((feature, index) => (
                        <span key={index} className="px-3 py-1.5 bg-success/10 text-success rounded-xl text-xs font-bold border border-success/20">
                          {feature}
                        </span>
                      ))}
                    </div>
                  </div>
                )}
              </div>

              {/* Submit Buttons */}
              <div className="flex flex-col sm:flex-row gap-4 pt-8 border-t border-border/30">
                <button
                  type="button"
                  onClick={() => router.push('/subscriptions/plans')}
                  className="flex-1 px-8 py-4 rounded-xl border border-border text-foreground font-bold hover:bg-muted transition-all active:scale-[0.98]"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={loading}
                  className="flex-1 px-8 py-4 rounded-xl bg-primary text-primary-foreground font-bold shadow-lg shadow-primary/20 hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed transition-all active:scale-[0.98]"
                >
                  {loading ? 'Creating...' : 'Create Permission Template Plan'}
                </button>
              </div>
            </form>
          </div>
        </div>
      </div>
    </div>
  )
}
