import type { PermissionSource, Platform, WalletData, WalletPermission, WalletSubscription } from '@/components/wallet/types';
import type { WalletSummaryDto } from '@/lib/api/wallet-management-client';

export function detectPlatform(permission: string): Platform {
    if (permission.startsWith('epsx:analytics') || permission.startsWith('epsx:rankings')) { return 'analytics'; }
    if (permission.startsWith('epsx-pay:')) { return 'pay'; }
    if (permission.startsWith('epsx-token:')) { return 'token'; }
    if (permission.startsWith('epsx-markets:')) { return 'markets'; }
    return 'analytics';
}

export function mapWalletDtoToData(dto: WalletSummaryDto): WalletData {
    const platforms = new Set<Platform>();
    const dtoPermissions = dto.permissions;
    dtoPermissions.forEach(p => {
        if (p.permission.startsWith('epsx:analytics') || p.permission.startsWith('epsx:rankings')) {
            platforms.add('analytics');
        } else if (p.permission.startsWith('epsx-pay:')) {
            platforms.add('pay');
        } else if (p.permission.startsWith('epsx-token:')) {
            platforms.add('token');
        } else if (p.permission.startsWith('epsx-markets:')) {
            platforms.add('markets');
        }
    });
    if (platforms.size === 0) { platforms.add('analytics'); }

    const permissions: WalletPermission[] = dtoPermissions.map((p, idx) => ({
        id: `perm-${idx}`,
        permission: p.permission,
        platform: detectPlatform(p.permission),
        source: (p.source ?? 'system') as PermissionSource,
        expiresAt: p.expires_at,
        isActive: p.is_active,
        createdAt: dto.created_at,
    }));

    const subscriptions: WalletSubscription[] = (dto.subscriptions ?? []).map((s, idx) => ({
        id: `sub-${idx}`,
        planId: s.plan_id,
        planName: s.plan_name,
        status: s.status as WalletSubscription['status'],
        priceDisplay: '',
        startedAt: s.started_at,
        expiresAt: s.expires_at,
        grantedPermissions: [],
    }));

    let status: 'active' | 'disabled' | 'pending' = 'active';
    if (!dto.is_active) { status = 'disabled'; }

    const disableInfo = dto.metadata?.['disable_info'] as WalletData['disableInfo'] | undefined;
    const label = dto.metadata?.['label'] as string | undefined;
    const note = dto.metadata?.['note'] as string | undefined;

    return {
        walletAddress: dto.wallet_address,
        status,
        disableInfo,
        createdAt: dto.created_at,
        lastAuthAt: dto.last_auth_at,
        platforms: Array.from(platforms),
        permissions,
        subscriptions,
        plans: dto.groups.map(g => ({ planName: g.group_name, role: g.role })),
        metadata: dto.metadata,
        label,
        note,
    };
}
