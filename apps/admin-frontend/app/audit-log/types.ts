export interface AuditLogEntry {
    id: string;
    action: string;
    wallet_address: string | null;
    resource_type: string;
    resource_id: string | null;
    result: string;
    details: Record<string, unknown> | null;
    additional_data?: Record<string, unknown> | null;
    ip_address?: string;
    user_agent?: string;
    timestamp: string;
    created_at?: string;
}

export type ActionType = 'all' | 'permission' | 'wallet' | 'plan' | 'system';

export const ACTION_CATEGORIES: Record<ActionType, { label: string; icon: string; color: string }> = {
    all: { label: 'All Actions', icon: '📋', color: 'gray' },
    permission: { label: 'Permissions', icon: '🔐', color: 'green' },
    wallet: { label: 'Wallets', icon: '👛', color: 'blue' },
    plan: { label: 'Plans', icon: '💳', color: 'purple' },
    system: { label: 'System', icon: '⚙️', color: 'orange' },
};
