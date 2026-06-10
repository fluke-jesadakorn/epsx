import type { PermissionDefinition } from '@/lib/api/permissions-client';

/**
 * Group permissions by backend-provided platform field
 */
export function groupPermissionsByPlatform(
    permissions: PermissionDefinition[]
): Record<string, PermissionDefinition[]> {
    const grouped: Record<string, PermissionDefinition[]> = {};

    permissions.forEach((perm) => {
        const platform = perm.platform || 'other';
        grouped[platform] ??= [];
        grouped[platform].push(perm);
    });

    // Sort platforms: admin first, then alphabetically
    const sortedGrouped: Record<string, PermissionDefinition[]> = {};
    const platforms = Object.keys(grouped).sort((a, b) => {
        if (a === 'admin') { return -1; }
        if (b === 'admin') { return 1; }
        return a.localeCompare(b);
    });

    platforms.forEach((platform) => {
        const platformPerms = grouped[platform];
        if (platformPerms) {
            sortedGrouped[platform] = platformPerms;
        }
    });

    return sortedGrouped;
}

/**
 * Get Tailwind color class for platform
 */
export function getPlatformColorClass(platform: string): string {
    const colorMap: Record<string, string> = {
        admin: 'text-red-400',
        epsx: 'text-cyan-400',
        'epsx-pay': 'text-green-400',
        'epsx-token': 'text-purple-400',
    };

    return colorMap[platform] ?? 'text-slate-400';
}

/**
 * Get platform display name
 */
export function getPlatformDisplayName(platform: string): string {
    const nameMap: Record<string, string> = {
        admin: 'ADMIN',
        epsx: 'EPSX',
        'epsx-pay': 'EPSX PAY',
        'epsx-token': 'EPSX TOKEN',
    };

    return nameMap[platform] ?? platform.toUpperCase();
}

/**
 * Filter permissions by search query
 * Searches in permission_string, name, and description
 */
export function filterPermissions(
    permissions: PermissionDefinition[],
    query: string
): PermissionDefinition[] {
    if (!query.trim()) { return permissions; }

    const lowerQuery = query.toLowerCase();

    return permissions.filter((perm) => {
        return (
            perm.permission_string.toLowerCase().includes(lowerQuery) ||
            (perm.name?.toLowerCase().includes(lowerQuery) ?? false) ||
            (perm.description?.toLowerCase().includes(lowerQuery) ?? false)
        );
    });
}

/**
 * Sort permissions within a group
 */
export function sortPermissions(permissions: PermissionDefinition[]): PermissionDefinition[] {
    return [...permissions].sort((a, b) =>
        a.permission_string.localeCompare(b.permission_string)
    );
}
