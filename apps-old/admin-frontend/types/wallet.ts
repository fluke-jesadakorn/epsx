export type AccessItemType = 'permission' | 'plan';

export interface RawPlanData {
    id: string;
    name: string;
    description?: string;
    permissions?: string[];
    member_count?: number;
    plan_group?: string;
}

export interface AccessItem {
    id: string;
    type: AccessItemType;
    name: string;
    description?: string;
    icon?: string;
    // Permission-specific
    platform?: string;
    category?: string;
    // Plan-specific
    permissionCount?: number;
    permissions?: string[]; // Added: List of permission IDs in the plan
    memberCount?: number;
    planGroup?: string;
    // Assignment info
    expiresAt?: string | null;
    assignedAt?: string;
    source?: string;
}

export interface WalletAccessData {
    // Available items (can be assigned)
    availablePermissions: AccessItem[];
    availablePlans: AccessItem[];
    // Authorized items (currently assigned to wallet)
    authorizedPermissions: AccessItem[];
    authorizedPlans: AccessItem[];
}

export interface UseWalletAccessReturn {
    // Data
    data: WalletAccessData;
    isLoading: boolean;
    error: string | null;
    // Single Actions
    assignPermission: (permissionId: string, expiresAt?: string) => Promise<void>;
    revokePermission: (permissionId: string) => Promise<void>;
    assignPlan: (planId: string, expiresAt?: string) => Promise<void>;
    removePlan: (planId: string) => Promise<void>;
    // Batch Actions
    batchAssignPermissions: (permissionIds: string[], expiresAt?: string) => Promise<void>;
    batchRevokePermissions: (permissionIds: string[]) => Promise<void>;
    batchAssignPlans: (planIds: string[], expiresAt?: string) => Promise<void>;
    batchRemovePlans: (planIds: string[]) => Promise<void>;
    // Refresh
    refresh: () => Promise<void>;
}
