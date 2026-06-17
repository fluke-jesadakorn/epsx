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
    category?: string;
    action_raw?: string;
    resource_type_raw?: string;
}

export type ActionType = 'all' | 'permission' | 'wallet' | 'plan' | 'system' | 'payment' | 'auth' | 'developer' | 'notification';

export const ACTION_CATEGORIES: Record<ActionType, { label: string; icon: string; color: string }> = {
    all: { label: 'All Actions', icon: '📋', color: 'gray' },
    permission: { label: 'Permissions', icon: '🔐', color: 'green' },
    wallet: { label: 'Wallets', icon: '👛', color: 'blue' },
    plan: { label: 'Plans', icon: '💳', color: 'purple' },
    payment: { label: 'Payments', icon: '💰', color: 'yellow' },
    system: { label: 'System', icon: '⚙️', color: 'orange' },
    auth: { label: 'Auth', icon: '🔑', color: 'cyan' },
    developer: { label: 'Developer', icon: '🛠', color: 'teal' },
    notification: { label: 'Notifications', icon: '🔔', color: 'pink' },
};
