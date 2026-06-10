import { type PolicyType } from '../types';

export const POLICY_TYPES: {
    value: PolicyType | 'all';
    label: string;
    icon: string;
}[] = [
        { value: 'all', label: 'All Types', icon: '📋' },
        { value: 'subscription', label: 'Subscription', icon: '💳' },
        { value: 'manual', label: 'Manual', icon: '👥' },
        { value: 'web3_asset', label: 'Web3 Asset', icon: '🔗' },
        { value: 'dao', label: 'DAO', icon: '🏛️' },
        { value: 'system', label: 'System', icon: '⚙️' },
    ];

export const SORT_OPTIONS = [
    { value: 'name', label: 'Name' },
    { value: 'members', label: 'Members' },
    { value: 'created_at', label: 'Date Created' },
    { value: 'revenue', label: 'Revenue' },
    { value: 'type', label: 'Type' },
];

export const STATUS_OPTIONS = [
    { value: 'all', label: 'All Status' },
    { value: 'active', label: 'Active' },
    { value: 'inactive', label: 'Inactive' },
];
