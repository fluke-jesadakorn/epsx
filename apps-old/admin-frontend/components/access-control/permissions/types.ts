export type { CreatePermissionRequest, PermissionDefinition } from '@/lib/api/permissions-client';

export interface PermissionsViewProps {
    className?: string;
}

export interface PermEditFormState {
    name: string;
    description: string;
    category: string;
}

const PLATFORM_CLASSES: Record<string, string> = {
    admin: 'bg-purple-500/15 text-purple-400 border-purple-500/30',
    epsx: 'bg-emerald-500/15 text-emerald-400 border-emerald-500/30',
    'epsx-pay': 'bg-amber-500/15 text-amber-400 border-amber-500/30',
    'epsx-token': 'bg-blue-500/15 text-blue-400 border-blue-500/30',
};

export function platformBadgeClass(platform: string): string {
    return PLATFORM_CLASSES[platform] ?? 'bg-slate-500/15 text-slate-400 border-slate-500/30';
}
