import type { PlanFormData } from '@/components/plans/edit/types'
import { toast } from '@/hooks/use-toast'
import { createPlansClient, isApiSuccess } from '@/shared/api/plans'
import { createAdminApiClient } from '@/shared/utils/api-client'
import { useRouter } from 'next/navigation'
import { useState } from 'react'

export function usePlanActions(id: string) {
    const router = useRouter()
    const [saving, setSaving] = useState(false)

    const handleSubmit = async (
        e: React.FormEvent,
        formData: PlanFormData,
        customPermissions: string[]
    ) => {
        e.preventDefault()
        const adminClient = createPlansClient(createAdminApiClient())

        try {
            setSaving(true)
            const permissions: string[] = []

            const addLimit = (key: string, value: number) => {
                if (value === -1) {
                    permissions.push(`${key}:unlimited`)
                } else if (value > 0) {
                    permissions.push(`${key}:${value}`)
                }
            }

            addLimit('epsx:api:calls', formData.api_calls_limit)
            addLimit('epsx:analytics:queries', formData.analytics_queries)
            addLimit('epsx:export:limit', formData.export_limit)

            if (formData.premium_features) {
                permissions.push('epsx:trading:premium')
                permissions.push('epsx:analytics:premium')
            }

            permissions.push(...customPermissions)

            const response = await adminClient.updatePlan(id, {
                name: formData.name,
                description: formData.description,
                is_active: formData.is_active,
                permissions,
                metadata: {
                    ranking_offset: formData.ranking_offset,
                    api_limits: {
                        api_calls: formData.api_calls_limit,
                        analytics_queries: formData.analytics_queries,
                        export_limit: formData.export_limit,
                        premium_features: formData.premium_features,
                    },
                    promotion: formData.promo_enabled
                        ? {
                            enabled: true,
                            type: formData.promo_type,
                            value: formData.promo_value,
                            price: formData.promo_price,
                            start_date: formData.promo_start,
                            end_date: formData.promo_end,
                        }
                        : null,
                },
            })

            if (isApiSuccess(response)) {
                toast({ title: 'Success', description: 'Plan updated successfully' })
                router.push('/subscriptions/plans')
            } else {
                toast({
                    title: 'Error',
                    description: response.error?.message ?? 'Failed to update plan',
                    variant: 'destructive',
                })
            }
        } catch {
            toast({
                title: 'Error',
                description: 'Failed to update plan',
                variant: 'destructive',
            })
        } finally {
            setSaving(false)
        }
    }

    const handleDelete = async () => {
        try {
            setSaving(true)
            const apiClient = createAdminApiClient()
            const plansClient = createPlansClient(apiClient)
            const response = await plansClient.deletePlan(id)

            if (isApiSuccess(response)) {
                toast({ title: 'Deleted', description: 'Plan deleted successfully' })
                router.push('/subscriptions/plans')
            } else {
                toast({
                    title: 'Error',
                    description: response.error?.message ?? 'Failed to delete plan',
                    variant: 'destructive',
                })
            }
        } catch {
            toast({
                title: 'Error',
                description: 'Failed to delete plan',
                variant: 'destructive',
            })
        } finally {
            setSaving(false)
        }
    }

    return { saving, handleSubmit, handleDelete }
}
