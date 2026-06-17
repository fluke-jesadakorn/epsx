import { useCallback, useState } from 'react';
import { toast } from 'sonner';

import { updatePlanAction } from '@/app/wallet-management/plan-actions';
import { type PermissionPlan } from '@/lib/api/plan-management-client';

import { FREE_PLAN_ID, getFeatureValue, type PlanEditFormState } from './types';

function rankingFromPermissions(permissions: string[], prefix: string): number | undefined {
    const val = getFeatureValue(permissions, prefix);
    if (val === null) { return undefined; }
    const n = parseInt(val, 10);
    return isNaN(n) ? undefined : n;
}

export const EMPTY_FORM: PlanEditFormState = {
    name: '',
    description: '',
    plan_category: 'base',
    plan_group: 'personal',
    priority: 0,
    price: 0,
    expiryDays: 30,
    gracePeriodHours: 0,
    permissions: [],
    is_public: true,
    is_active: true,
    features: [],
    promoEnabled: false,
    promoType: 'percentage',
    promoValue: 0,
    promoPrice: 0,
    promoStart: '',
    promoEnd: '',
};

function promoFromMeta(promo: Record<string, unknown> | undefined) {
    return {
        promoEnabled: promo?.enabled === true,
        promoType: (promo?.type as ('percentage' | 'fixed') | undefined) ?? 'percentage',
        promoValue: Number(promo?.value) || 0,
        promoPrice: Number(promo?.price) || 0,
        promoStart: toLocal((promo?.start_date as string | undefined) ?? ''),
        promoEnd: toLocal((promo?.end_date as string | undefined) ?? ''),
    };
}

function planToForm(plan: PermissionPlan): PlanEditFormState {
    const meta = plan.plan_metadata;
    const features = Array.isArray(meta?.features) ? (meta.features as string[]) : [];
    const promo = meta?.promotion as Record<string, unknown> | undefined;
    return {
        name: plan.name,
        description: plan.description,
        plan_category: plan.plan_category,
        plan_group: plan.plan_group ?? 'personal',
        priority: plan.tier_level,
        price: Number(plan.price) || 0,
        expiryDays: plan.default_expiry_days ?? 30,
        gracePeriodHours: plan.grace_period_hours ?? 0,
        permissions: plan.permissions,
        is_public: plan.is_public !== false,
        is_active: plan.is_active !== false,
        features,
        ...promoFromMeta(promo),
    };
}

/** datetime-local input value → RFC3339 UTC string for backend */
export function toIso(dt: string): string {
    if (dt === '') { return ''; }
    if (dt.includes('Z') || /[+-]\d{2}:\d{2}$/.test(dt)) { return dt; }
    return dt.length === 16 ? `${dt}:00Z` : `${dt}Z`;
}

/** RFC3339 UTC string → datetime-local input value (YYYY-MM-DDTHH:mm) */
function toLocal(iso: string): string {
    if (iso === '') { return ''; }
    return iso.slice(0, 16);
}

export function usePlanEditForm() {
    const [selectedPlan, setSelectedPlan] = useState<PermissionPlan | null>(null);
    const [form, setForm] = useState<PlanEditFormState>(EMPTY_FORM);
    const [hasChanges, setHasChanges] = useState(false);
    const [isSaving, setIsSaving] = useState(false);

    const selectPlanLogic = useCallback((plan: PermissionPlan | null) => {
        if (plan === null) {
            setSelectedPlan(null);
            setForm(EMPTY_FORM);
        } else {
            setSelectedPlan(plan);
            setForm(planToForm(plan));
        }
        setHasChanges(false);
    }, []);

    const savePlan = async (
        plans: PermissionPlan[],
        setPlans: (p: PermissionPlan[]) => void
    ) => {
        if (selectedPlan === null) { return; }
        setIsSaving(true);
        try {
            const rankingOffset = rankingFromPermissions(form.permissions, 'epsx:rankings:offset');
            const rankingsLimit = rankingFromPermissions(form.permissions, 'epsx:rankings:limit');

            const updated = await updatePlanAction(selectedPlan.id, {
                name: form.name,
                description: form.description,
                plan_category: form.plan_category,
                plan_group: form.plan_group,
                tier_level: form.priority,
                price: selectedPlan.id === FREE_PLAN_ID ? undefined : form.price,
                default_expiry_days: form.expiryDays,
                grace_period_hours: form.gracePeriodHours,
                permissions: form.permissions,
                is_public: form.is_public,
                is_active: form.is_active,
                plan_metadata: {
                    ...selectedPlan.plan_metadata,
                    ...(rankingOffset !== undefined && !isNaN(rankingOffset) ? { ranking_offset: rankingOffset } : {}),
                    ...(rankingsLimit !== undefined && !isNaN(rankingsLimit) ? { rankings_limit: rankingsLimit } : {}),
                    features: form.features,
                    promotion: {
                        enabled: form.promoEnabled,
                        type: form.promoType,
                        value: form.promoValue,
                        price: form.promoPrice,
                        start_date: toIso(form.promoStart),
                        end_date: toIso(form.promoEnd),
                    },
                },
            });
            toast.success('Plan updated');
            setPlans(plans.map((p) => (p.id === updated.id ? updated : p)));
            setSelectedPlan(updated);
            setHasChanges(false);
        } catch (e: unknown) {
            toast.error(e instanceof Error ? e.message : 'Failed to update plan');
        } finally {
            setIsSaving(false);
        }
    };

    const discardChanges = () => {
        if (selectedPlan !== null) {
            selectPlanLogic(selectedPlan);
            toast.info('Changes discarded');
        }
    };

    return {
        selectedPlan,
        form,
        setForm,
        hasChanges,
        setHasChanges,
        isSaving,
        selectPlan: selectPlanLogic,
        savePlan,
        discardChanges,
    };
}
