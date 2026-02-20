 
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

    return (
        <div className="grid md:grid-cols-4 gap-4">
            {/* API Access Status */}
            <div className="group relative overflow-hidden rounded-2xl bg-gradient-to-br from-white/80 to-white/40 dark:from-gray-800/80 dark:to-gray-800/40 backdrop-blur-xl border border-gray-300 dark:border-white/20 dark:border-gray-700/50 shadow-xl hover:shadow-2xl transition-all duration-300 hover:-translate-y-1">
                <div className="absolute inset-0 bg-gradient-to-br from-emerald-500/10 to-teal-500/10 opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
                <div className="relative p-6">
                    <div className="flex items-center gap-3 mb-3">
                        <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-emerald-500 to-teal-500 flex items-center justify-center shadow-lg shadow-emerald-500/25">
                            <svg className="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                            </svg>
                        </div>
                        <span className="text-sm font-medium text-gray-500 dark:text-gray-400">API Access</span>
                    </div>
                    <div className={`text-2xl font-bold mb-2 ${accessLevel === 'Active' ? 'text-emerald-600 dark:text-emerald-400' : 'text-amber-600 dark:text-amber-400'}`}>
                        {accessLevel}
                    </div>
                    {hasGroups && userGroupData && (
                        <div className="flex flex-wrap gap-1">
                            {userGroupData.plans.slice(0, 2).map(g => (
                                <Badge key={g.id} variant="outline" className="text-xs bg-white/50 dark:bg-slate-900">
                                    {g.name}
                                </Badge>
                            ))}
                            {userGroupData.plans.length > 2 && (
                                <Badge variant="outline" className="text-xs bg-white/50 dark:bg-slate-900">
                                    +{userGroupData.plans.length - 2}
                                </Badge>
                            )}
                        </div>
                    )}
                </div>
            </div>

            {/* Rate Limit */}
            <div className="group relative overflow-hidden rounded-2xl bg-gradient-to-br from-white/80 to-white/40 dark:from-gray-800/80 dark:to-gray-800/40 backdrop-blur-xl border border-gray-300 dark:border-white/20 dark:border-gray-700/50 shadow-xl hover:shadow-2xl transition-all duration-300 hover:-translate-y-1">
                <div className="absolute inset-0 bg-gradient-to-br from-blue-500/10 to-indigo-500/10 opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
                <div className="relative p-6">
                    <div className="flex items-center gap-3 mb-3">
                        <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-blue-500 to-indigo-500 flex items-center justify-center shadow-lg shadow-blue-500/25">
                            <svg className="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                            </svg>
                        </div>
                        <span className="text-sm font-medium text-gray-500 dark:text-gray-400">Rate Limit</span>
                    </div>
                    <div className="text-2xl font-bold text-blue-600 dark:text-blue-400 mb-2">{rateLimit}</div>
                    {primaryGroup?.rate_limit_per_day && (
                        <div className="text-xs text-gray-500 dark:text-gray-400">{primaryGroup.rate_limit_per_day.toLocaleString()}/day</div>
                    )}
                </div>
            </div>

            {/* Usage Stats */}
            <div className="group relative overflow-hidden rounded-2xl bg-gradient-to-br from-white/80 to-white/40 dark:from-gray-800/80 dark:to-gray-800/40 backdrop-blur-xl border border-gray-300 dark:border-white/20 dark:border-gray-700/50 shadow-xl hover:shadow-2xl transition-all duration-300 hover:-translate-y-1">
                <div className="absolute inset-0 bg-gradient-to-br from-purple-500/10 to-pink-500/10 opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
                <div className="relative p-6">
                    <div className="flex items-center gap-3 mb-3">
                        <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-purple-500 to-pink-500 flex items-center justify-center shadow-lg shadow-purple-500/25">
                            <svg className="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                            </svg>
                        </div>
                        <span className="text-sm font-medium text-gray-500 dark:text-gray-400">Total Usage</span>
                    </div>
                    <div className="text-2xl font-bold text-purple-600 dark:text-purple-400 mb-2">
                        {isLoading ? '...' : ((userGroupData?.total_requests ?? 0).toLocaleString())}
                    </div>
                    <div className="text-xs text-gray-500 dark:text-gray-400">
                        {userGroupData?.total_api_keys ?? 0} API Key{(userGroupData?.total_api_keys ?? 0) !== 1 ? 's' : ''}
                    </div>
                </div>
            </div>

            {/* Expiry */}
            <div className="group relative overflow-hidden rounded-2xl bg-gradient-to-br from-white/80 to-white/40 dark:from-gray-800/80 dark:to-gray-800/40 backdrop-blur-xl border border-gray-300 dark:border-white/20 dark:border-gray-700/50 shadow-xl hover:shadow-2xl transition-all duration-300 hover:-translate-y-1">
                <div className="absolute inset-0 bg-gradient-to-br from-amber-500/10 to-orange-500/10 opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
                <div className="relative p-6">
                    <div className="flex items-center gap-3 mb-3">
                        <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-amber-500 to-orange-500 flex items-center justify-center shadow-lg shadow-amber-500/25">
                            <svg className="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                            </svg>
                        </div>
                        <span className="text-sm font-medium text-gray-500 dark:text-gray-400">Expires</span>
                    </div>
                    {earliestExpiry ? (
                        <>
                            <div className={`text-2xl font-bold mb-2 ${earliestExpiry.getTime() - Date.now() < 7 * 24 * 60 * 60 * 1000
                                ? 'text-red-600 dark:text-red-400'
                                : 'text-gray-700 dark:text-gray-200'
                                }`}>
                                {earliestExpiry.toLocaleDateString()}
                            </div>
                            <div className="text-xs text-gray-500 dark:text-gray-400">
                                {Math.ceil((earliestExpiry.getTime() - Date.now()) / (1000 * 60 * 60 * 24))} days left
                            </div>
                        </>
                    ) : (
                        <div className="text-2xl font-bold text-emerald-600 dark:text-emerald-400 mb-2">Never</div>
                    )}
                </div>
            </div>
        </div>
    );
}
