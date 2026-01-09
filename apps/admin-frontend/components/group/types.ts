/**
 * Group Management Types
 * Shared types for the group management system
 */

/** Group type categories */
export type GroupType = 'manual' | 'subscription' | 'web3_asset' | 'dao_membership' | 'admin' | 'system';

/** Group status */
export type GroupStatus = 'active' | 'inactive';

/** Group data structure */
export interface GroupData {
    id: string;
    name: string;
    slug: string;
    description: string;
    groupType: GroupType;
    permissions: string[];
    memberCount: number;
    priorityLevel: number;
    defaultExpiryDays?: number;
    isActive: boolean;
    isSystemGroup: boolean;
    createdAt: string;
    updatedAt: string;
}

/** Stats for dashboard */
export interface GroupStats {
    totalGroups: number;
    activeMemberships: number;
    expiringSoon: number;
    largestGroup: {
        name: string;
        memberCount: number;
    };
}

/** Filter options for group list */
export interface GroupFilters {
    search: string;
    groupType: GroupType | 'all';
    sortBy: 'name' | 'members' | 'permissions' | 'created_at';
    sortOrder: 'asc' | 'desc';
}
