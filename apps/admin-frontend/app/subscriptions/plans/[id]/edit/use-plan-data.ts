import type { PlanFormData } from '@/components/plans/edit/types'
import { toast } from '@/hooks/use-toast'
import type { Plan } from '@/shared/api/plans'
import { createPlansClient, isApiSuccess } from '@/shared/api/plans'
import { FREE_PLAN_RANKING_OFFSET } from '@/shared/config/constants'
import { createAdminApiClient } from '@/shared/utils/api-client'
import { useRouter } from 'next/navigation'
import { useEffect, useState } from 'react'

const MANAGED_PREFIXES = [
    'epsx:api:calls:',
    'epsx:rankings:offset:',
    'epsx:analytics:queries:',
    'epsx:export:limit:',
    'epsx:trading:premium',
    'epsx:analytics:premium',
]

const parseLimit = (
    permission: string | undefined,
    defaultValue: number
): number => {
    if (permission === undefined || permission === '') {
        return defaultValue
    }
    const parts = permission.split(':')
    const lastPart = parts[parts.length - 1]
    if (lastPart === 'unlimited') {
        return -1
    }
    const parsed = parseInt(lastPart ?? '0')
    return isNaN(parsed) ? defaultValue : parsed
}

const getPermissionValue = (
    permissions: string[],
    prefix: string
): string | undefined => {
    return permissions.find((p) => p.startsWith(prefix))
}

const getPromoData = (metadata: Record<string, unknown>) => {
    const promo =
        (metadata.promotion as Record<string, unknown> | undefined) ?? {}
    return {
        promo_enabled: (promo.enabled as boolean | undefined) ?? false,
        promo_type:
            (promo.type as 'percentage' | 'fixed' | undefined) ?? 'percentage',
        promo_value: (promo.value as number | undefined) ?? 0,
        promo_price: (promo.price as number | undefined) ?? 0,
        promo_start: (promo.start_date as string | undefined) ?? '',
        promo_end: (promo.end_date as string | undefined) ?? '',
    }
}

const mapPlanToFormData = (planData: Plan): PlanFormData => {
    const permissions = planData.permissions
    const apiCallsPermission = getPermissionValue(permissions, 'epsx:api:calls:')
    const analyticsPermission = getPermissionValue(
        permissions,
        'epsx:analytics:queries:'
    )

    const metadata = planData.metadata ?? {}
    const rankingOffset =
        (metadata.ranking_offset as number | undefined) ?? FREE_PLAN_RANKING_OFFSET

    const promoData = getPromoData(metadata)

    return {
        name: planData.name,
        description: planData.description ?? '',
        current_price:
            typeof planData.current_price === 'string'
                ? parseFloat(planData.current_price)
                : (planData.current_price ?? 0),
        is_active: planData.is_active,
        api_calls_limit: parseLimit(apiCallsPermission, 100),
        ranking_offset: rankingOffset,
        analytics_queries: parseLimit(analyticsPermission, 0),
        premium_features: permissions.some((p) => p.includes('premium')),
        export_limit: 10,
        ...promoData,
    }
}

export function usePlanData(id: string) {
    const router = useRouter()
    const [plan, setPlan] = useState<Plan | null>(null)
    const [loading, setLoading] = useState(true)
    const [formData, setFormData] = useState<PlanFormData>({
        name: '',
        description: '',
        current_price: 0,
        is_active: true,
        api_calls_limit: 100,
        ranking_offset: FREE_PLAN_RANKING_OFFSET,
        analytics_queries: 0,
        premium_features: false,
        export_limit: 10,
        promo_enabled: false,
        promo_type: 'percentage',
        promo_value: 0,
        promo_price: 0,
        promo_start: '',
        promo_end: '',
    })
    const [customPermissions, setCustomPermissions] = useState<string[]>([])

    useEffect(() => {
        const loadPlan = async () => {
            if (!id) {
                return
            }

            try {
                setLoading(true)
                const apiClient = createAdminApiClient()
                const plansClient = createPlansClient(apiClient)
                const response = await plansClient.getPlan(id)

                if (isApiSuccess(response)) {
                    const planData = response.data
                    setPlan(planData)
                    setFormData(mapPlanToFormData(planData))
                } else {
                    toast({
                        title: 'Error',
                        description: 'Failed to load plan',
                        variant: 'destructive',
                    })
                    router.push('/subscriptions/plans')
                }
            } catch {
                toast({
                    title: 'Error',
                    description: 'Failed to load plan',
                    variant: 'destructive',
                })
                router.push('/subscriptions/plans')
            } finally {
                setLoading(false)
            }
        }

        void loadPlan()
    }, [id, router])

    useEffect(() => {
        if (plan?.permissions) {
            const others = plan.permissions.filter(
                (p) => !MANAGED_PREFIXES.some((prefix) => p.startsWith(prefix))
            )
            setCustomPermissions(others)
        }
    }, [plan])

    return {
        plan,
        loading,
        formData,
        setFormData,
        customPermissions,
        setCustomPermissions,
    }
}
