import type { PermissionSource, Platform, WalletData, WalletPermission, WalletSubscription } from '@/components/wallet/types';
import type { WalletSummaryDto } from '@/lib/api/wallet-management-client';

function extractPlans(dto: WalletSummaryDto) {
    if ((dto.plans ?? []).length > 0) {
        return (dto.plans ?? []).filter(p => p.is_active !== false).map(p => ({ planName: p.plan_name }));
    }
    if ((dto.groups ?? []).length > 0) {
        return (dto.groups ?? []).map(g => ({ planName: g.group_name, role: g.role }));
    }
    if (dto.plan_name !== undefined && dto.plan_name !== '') {
        return [{ planName: dto.plan_name }];
    }
    return [];
}

export function mapWalletDtoToData(dto: WalletSummaryDto): WalletData {
    const dtoPermissions = dto.permissions ?? [];

    const permissions: WalletPermission[] = dtoPermissions.map((p, idx) => ({
        id: `perm-${idx}`,
        permission: p.permission,
        platform: (p.platform ?? 'analytics') as Platform,
        source: (p.source ?? 'system') as PermissionSource,
        expiresAt: p.expires_at,
        isActive: p.is_active,
        createdAt: p.created_at ?? dto.created_at,
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

    let status: WalletData['status'] = 'active';
    if (!dto.is_active) { status = 'disabled'; }

    const disableInfo = dto.metadata?.['disable_info'] as WalletData['disableInfo'] | undefined;
    const label = dto.metadata?.['label'] as string | undefined;
    const note = dto.metadata?.['note'] as string | undefined;

    const platforms = (dto.platforms ?? ['analytics']).map(p => p as Platform);

    return {
        walletAddress: dto.wallet_address,
        status,
        disableInfo,
        createdAt: dto.created_at,
        lastAuthAt: dto.last_auth_at,
        platforms,
        permissions,
        subscriptions,
        plans: extractPlans(dto),
        metadata: dto.metadata,
        label,
        note,
    };
}
