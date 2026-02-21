import type { PlanCategory, PlanGroup, PermissionPlan } from '@/lib/api/plan-management-client';

export interface FeaturePermDef {
    prefix: string;
    label: string;
    type: 'numeric' | 'boolean';
    tooltip?: string;
    placeholder?: string;
}

export const FEATURE_PERMISSIONS: FeaturePermDef[] = [
    { prefix: 'epsx:rankings:offset', label: 'Ranking Offset', type: 'numeric', tooltip: 'Starting rank. 1 = all, 100 = skip first 99.', placeholder: '1' },
    { prefix: 'epsx:rankings:limit', label: 'Rankings Limit', type: 'numeric', tooltip: 'Max rankings shown.', placeholder: '3' },
    { prefix: 'epsx:api:calls_limit', label: 'API Calls Limit', type: 'numeric', tooltip: 'Max API calls.', placeholder: '100' },
    { prefix: 'epsx:api:ratelimit_min', label: 'Rate Limit /min', type: 'numeric', tooltip: 'Requests per minute.', placeholder: '60' },
    { prefix: 'epsx:api:ratelimit_hour', label: 'Rate Limit /hour', type: 'numeric', tooltip: 'Requests per hour.', placeholder: '1000' },
    { prefix: 'epsx:api:ratelimit_day', label: 'Rate Limit /day', type: 'numeric', tooltip: 'Requests per day.', placeholder: '10000' },
    { prefix: 'epsx:api:burst', label: 'Burst Capacity', type: 'numeric', tooltip: 'Max burst request count.', placeholder: '10' },
    { prefix: 'epsx:analytics:enabled', label: 'Analytics Enabled', type: 'boolean', tooltip: 'Enable advanced analytics.' },
    { prefix: 'epsx:support:premium', label: 'Premium Support', type: 'boolean', tooltip: 'Premium support tier.' },
];

export function getFeatureValue(permissions: string[], prefix: string): string | null {
    const match = permissions.find(p => p.startsWith(`${prefix  }:`) || p === prefix);
    if (!match) {return null;}
    if (match === prefix) {return 'true';}
    return match.slice(prefix.length + 1);
}

export function setFeatureValue(permissions: string[], prefix: string, value: string | null): string[] {
    const filtered = permissions.filter(p => p !== prefix && !p.startsWith(`${prefix  }:`));
    if (value === null || value === '') {return filtered;}
    if (value === 'true') {return [...filtered, prefix];}
    return [...filtered, `${prefix}:${value}`];
}

export interface PlansViewProps {
    className?: string;
}

export interface PlanEditFormState {
    name: string;
    description: string;
    plan_category: PlanCategory;
    plan_group: PlanGroup;
    priority: number;
    price: number;
    expiryDays: number;
    gracePeriodHours: number;
    permissions: string[];
    is_public: boolean;
    is_active: boolean;
    features: string[];
    promoEnabled: boolean;
    promoType: 'percentage' | 'fixed';
    promoValue: number;
    promoPrice: number;
    promoStart: string;
    promoEnd: string;
}

export const FREE_PLAN_ID = '00000000-0000-0000-0000-000000000000';

export const SYSTEM_PLAN_IDS = [
    '00000000-0000-0000-0001-000000000001', // Super Admin
    '00000000-0000-0000-0001-000000000002', // Moderator
    '00000000-0000-0000-0001-000000000003', // Support
];

export function isSystemPlan(plan: { id: string; is_system_plan?: boolean }): boolean {
    return plan.is_system_plan === true || SYSTEM_PLAN_IDS.includes(plan.id);
}

export interface PlanDeletionContext {
    plans: PermissionPlan[];
    setPlans: (p: PermissionPlan[]) => void;
    selectedPlan: PermissionPlan | null;
    setSelectedPlan: (p: PermissionPlan | null) => void;
}

export interface DragDropContext {
    plans: PermissionPlan[];
    setPlans: (p: PermissionPlan[]) => void;
    selectedPlan: PermissionPlan | null;
    setForm: React.Dispatch<React.SetStateAction<PlanEditFormState>>;
}

const CATEGORY_CLASSES: Record<string, string> = {
    base: 'bg-blue-500/15 text-blue-400 border-blue-500/30',
    addon: 'bg-amber-500/15 text-amber-400 border-amber-500/30',
    system: 'bg-purple-500/15 text-purple-400 border-purple-500/30',
    exclusive: 'bg-emerald-500/15 text-emerald-400 border-emerald-500/30',
};

export function categoryBadgeClass(cat: string): string {
    return CATEGORY_CLASSES[cat] ?? 'bg-blue-500/15 text-blue-400 border-blue-500/30';
}
