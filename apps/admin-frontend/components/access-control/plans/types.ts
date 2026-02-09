import { PermissionPlan } from '@/lib/api/plan-management-client';

export interface PlansViewProps {
    className?: string;
}

export interface PlanEditFormState {
    name: string;
    description: string;
    priority: number;
    price: number;
    expiryDays: number;
    permissions: string[];
    is_public: boolean;
    is_active: boolean;
    features: string[];
}

export const FREE_PLAN_ID = '00000000-0000-0000-0000-000000000000';

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
    setForm: (f: PlanEditFormState) => void;
}
