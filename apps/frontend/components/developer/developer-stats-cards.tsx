 
'use client';

import { getMyPlansAction } from '@/app/actions/developer';
import { Badge } from '@/components/ui/badge';
import { useQuery } from '@tanstack/react-query';
import type { AuthUser } from '@/lib/server/actions';

interface UserGroupData {
    plans: Array<{
        id: string;
        name: string;
        slug: string;
        description: string;
        plan_type: string;
        permissions: string[];
        expires_at: string | null;
        rate_limit_per_minute: number | null;
        rate_limit_per_day: number | null;
        assigned_at: string;
    }>;
    total_api_keys: number;
    total_requests: number;
}

interface DeveloperStatsCardsProps {
    currentUser: AuthUser;
}

export function DeveloperStatsCards({ currentUser: _currentUser }: DeveloperStatsCardsProps) {
    const { data: response, isLoading } = useQuery({
        queryKey: ['developer-plans'],
        queryFn: getMyPlansAction,
    });

    const userGroupData = response?.success ? (response.data as unknown as UserGroupData) : null;

    const hasGroups = userGroupData?.plans && userGroupData.plans.length > 0;
    const primaryGroup = userGroupData?.plans?.[0];
    const hasApiKeys = (userGroupData?.total_api_keys ?? 0) > 0;
    const accessLevel = hasApiKeys || hasGroups ? 'Active' : 'No API Keys';
    const rateLimit = primaryGroup?.rate_limit_per_minute
        ? `${primaryGroup.rate_limit_per_minute}/min`
        : (hasApiKeys ? 'Default' : 'N/A');

    const getEarliestExpiry = () => {
        if (!userGroupData?.plans?.length) {return null;}
        const expiringGroups = userGroupData.plans.filter(g => g.expires_at);
        if (!expiringGroups.length) {return null;}
        const dates = expiringGroups.map(g => new Date(g.expires_at!));
        return new Date(Math.min(...dates.map(d => d.getTime())));
    };
    const earliestExpiry = getEarliestExpiry();

    const cards = [
        { label: 'API Access', value: accessLevel, color: accessLevel === 'Active' ? 'text-emerald-400' : 'text-amber-400',
          sub: hasGroups && userGroupData ? userGroupData.plans.slice(0, 2).map(g => g.name).join(', ') : undefined },
        { label: 'Rate Limit', value: rateLimit, color: 'text-blue-400',
          sub: primaryGroup?.rate_limit_per_day ? `${primaryGroup.rate_limit_per_day.toLocaleString()}/day` : undefined },
        { label: 'Total Usage', value: isLoading ? '...' : ((userGroupData?.total_requests ?? 0).toLocaleString()), color: 'text-purple-400',
          sub: `${userGroupData?.total_api_keys ?? 0} API Key${(userGroupData?.total_api_keys ?? 0) !== 1 ? 's' : ''}` },
        { label: 'Expires', value: earliestExpiry ? earliestExpiry.toLocaleDateString() : 'Never', color: earliestExpiry ? 'text-amber-400' : 'text-emerald-400',
          sub: earliestExpiry ? `${Math.ceil((earliestExpiry.getTime() - Date.now()) / (1000 * 60 * 60 * 24))} days left` : undefined },
    ];

    return (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
            {cards.map((c) => (
                <div key={c.label} className="rounded-2xl border border-border/20 bg-card p-5 shadow-xl">
                    <p className="text-xs font-medium text-muted-foreground">{c.label}</p>
                    <p className={`mt-2 text-2xl font-bold ${c.color}`}>{c.value}</p>
                    {c.sub && <p className="mt-1 text-[11px] text-muted-foreground/60">{c.sub}</p>}
                </div>
            ))}
        </div>
    );
}
