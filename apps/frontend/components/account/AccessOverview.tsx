
import { Badge } from '@/components/ui';
import { PermissionBadge } from '@/components/ui/PermissionBadge';
import { AccessOverviewData, createUsersClient } from '@/shared/api/users';
import { createFrontendApiClient } from '@/shared/utils/api-client';
import { differenceInDays, format } from 'date-fns';
import {
    AlertTriangle,
    Check,
    ShieldCheck,
    Sparkles,
    X
} from 'lucide-react';
import Link from 'next/link';

export async function AccessOverview() {
    const client = createFrontendApiClient();
    const usersApi = createUsersClient(client);

    let data: AccessOverviewData | null = null;
    let error: string | null = null;

    try {
        // NOTE: Endpoint is /api/wallet/access-overview in backend (unified_user_handlers.rs)
        // usersApi uses unified client which might suffix, but let's be explicit or add method alias
        // Backend struct AccessOverviewData has 'plans', frontend interface has 'groups'.
        // We need to map response.
        const response = await usersApi.getAccessOverview();

        if (response.success && response.data) {
            const responseData = response.data as any;
            data = {
                current_tier: responseData.current_tier,
                // Map backend 'plans' to frontend 'groups'
                groups: responseData.groups || responseData.plans || [],
                direct_permissions: responseData.direct_permissions || []
            };
        } else {
            // Show empty state instead of mock data - users should see their real access level
            data = {
                current_tier: "Free User",
                groups: [],
                direct_permissions: []
            };
            if (!response.success && response.error) {
                console.error('Error fetching access overview detailed:', JSON.stringify(response.error, null, 2));
                // also try logging keys just in case
                console.error('Error keys:', Object.keys(response.error));
                console.error('Error message:', response.error.message);
                console.error('Error code:', response.error.code);
                error = 'Unable to load access details.';
            }
        }
    } catch (err) {
        console.error('Error fetching access overview:', err);
        error = 'Unable to load access details.';
    }

    const getExpiryStatus = (dateStr?: string) => {
        if (!dateStr) return { label: 'Permanent', color: 'text-green-600 bg-green-50/50 dark:bg-green-900/20 dark:text-green-400 border-green-200 dark:border-green-800' };

        const days = differenceInDays(new Date(dateStr), new Date());

        if (days < 0) return { label: 'Expired', color: 'text-red-600 bg-red-50/50 dark:bg-red-900/20 dark:text-red-400 border-red-200 dark:border-red-800' };
        if (days <= 7) return { label: `Expires in ${days} days`, color: 'text-orange-600 bg-orange-50/50 dark:bg-orange-900/20 dark:text-orange-400 border-orange-200 dark:border-orange-800' };
        return { label: `Expires ${format(new Date(dateStr), 'MMM d, yyyy')}`, color: 'text-indigo-600 bg-indigo-50/50 dark:bg-indigo-900/20 dark:text-indigo-400 border-indigo-200 dark:border-indigo-800' };
    };

    const FeatureRow = ({ label, included }: { label: string; included: boolean }) => (
        <div className="flex items-center justify-between py-3 border-b border-gray-100 dark:border-gray-800 last:border-0 group">
            <span className={`text-sm font-medium transition-colors ${included ? 'text-gray-900 dark:text-white' : 'text-gray-400 dark:text-gray-500'}`}>
                {label}
            </span>
            {included ? (
                <div className="bg-green-100 dark:bg-green-900/30 p-1 rounded-full">
                    <Check className="w-4 h-4 text-green-600 dark:text-green-400" />
                </div>
            ) : (
                <div className="bg-gray-100 dark:bg-gray-800 p-1 rounded-full">
                    <X className="w-4 h-4 text-gray-300 dark:text-gray-600" />
                </div>
            )}
        </div>
    );

    if (error || !data) {
        return (
            <div className="rounded-2xl border-2 border-red-200 bg-red-50/50 p-6 dark:border-red-900/50 dark:bg-red-900/20">
                <div className="flex items-center gap-3">
                    <AlertTriangle className="h-6 w-6 text-red-500" />
                    <p className="text-sm font-semibold text-red-700 dark:text-red-300">{error || 'No data available'}</p>
                </div>
            </div>
        );
    }

    return (
        <div className="space-y-8">
            {/* Current Tier Header */}
            <div className="relative overflow-hidden rounded-[2rem] bg-gradient-to-r from-yellow-500 via-orange-500 to-pink-500 p-8 text-white shadow-2xl group">
                <div className="relative z-10 flex flex-col md:flex-row md:items-center md:justify-between gap-6">
                    <div>
                        <div className="flex items-center gap-2 text-white/80 mb-2 font-bold uppercase tracking-wider text-xs">
                            <ShieldCheck className="w-4 h-4" />
                            Current Access Level
                        </div>
                        <h2 className="text-4xl md:text-5xl font-black tracking-tight">{data.current_tier}</h2>
                    </div>
                    <Link href="/plans" className="px-8 py-4 bg-white text-orange-600 rounded-2xl font-black text-sm hover:scale-105 transition-all shadow-xl hover:shadow-orange-500/25 flex items-center gap-2">
                        <Sparkles className="w-4 h-4" /> UPGRADE ACCESS
                    </Link>
                </div>
                {/* Decorative circles */}
                <div className="absolute top-0 right-0 -mr-16 -mt-16 h-64 w-64 rounded-full bg-white/20 blur-3xl group-hover:bg-white/30 transition-colors"></div>
                <div className="absolute bottom-0 left-0 -ml-16 -mb-16 h-48 w-48 rounded-full bg-black/10 blur-3xl"></div>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-12 gap-8">
                {/* Left Col: Permission Sources */}
                <div className="lg:col-span-8 space-y-6">
                    <div className="flex items-center justify-between">
                        <h3 className="text-xl font-bold text-gray-900 dark:text-white flex items-center gap-3">
                            <span className="flex h-8 w-8 items-center justify-center rounded-xl bg-indigo-100 dark:bg-indigo-900/30 text-indigo-600 dark:text-indigo-400 text-sm font-bold shadow-sm">A</span>
                            Active Permissions
                        </h3>
                        <Badge variant="outline" className="bg-indigo-50/50 dark:bg-indigo-900/20 text-indigo-600 border-indigo-200">
                            {data.groups.length} Active Groups
                        </Badge>
                    </div>

                    <div className="grid sm:grid-cols-2 gap-6">
                        {data.groups.length > 0 ? data.groups.map((group, idx) => {
                            const status = getExpiryStatus(group.expires_at);
                            const isPlan = group.source_type === 'plan';
                            // Use backend-provided days_remaining if available, otherwise calculate
                            const daysRemaining = group.days_remaining ?? (group.expires_at ? differenceInDays(new Date(group.expires_at), new Date()) : null);
                            const isExpiringSoon = daysRemaining !== null && daysRemaining >= 0 && daysRemaining <= 7;
                            const canRenew = group.can_renew ?? (isPlan && daysRemaining !== null && daysRemaining <= 30);

                            return (
                                <div key={idx} className={`bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-6 shadow-xl border-2 transition-all duration-300 hover:shadow-2xl flex flex-col ${isPlan ? 'border-blue-300/50 dark:border-blue-700/50' : 'border-purple-300/50 dark:border-purple-700/50'}`}>
                                    <div className="flex items-center justify-between mb-4">
                                        <div className="flex items-center gap-2">
                                            <div className="text-3xl">{isPlan ? '💎' : '👥'}</div>
                                            {/* Source type badge */}
                                            <Badge variant="outline" className="text-[9px] font-bold uppercase tracking-wider bg-gray-50 dark:bg-gray-800 border-gray-200 dark:border-gray-700">
                                                {group.source_type === 'plan' ? '💳 Paid' :
                                                    group.source_type === 'manual' ? '🔧 Manual' : '👑 Admin'}
                                            </Badge>
                                        </div>
                                        <Badge variant="outline" className={`text-[10px] font-bold uppercase tracking-wider ${status.color}`}>
                                            {daysRemaining !== null && daysRemaining >= 0 ? `${daysRemaining}d left` : status.label}
                                        </Badge>
                                    </div>

                                    <div className="mb-4">
                                        <h4 className="text-xl font-bold text-gray-900 dark:text-white mb-1">{group.name}</h4>
                                        <p className="text-xs text-gray-500 dark:text-gray-400 line-clamp-2 leading-relaxed">
                                            {group.description || `Includes ${group.permissions.length} specialized permissions`}
                                        </p>
                                        {/* Show assigned date if available */}
                                        {group.assigned_at && (
                                            <p className="text-[10px] text-gray-400 dark:text-gray-500 mt-1">
                                                📅 Since {format(new Date(group.assigned_at), 'MMM d, yyyy')}
                                            </p>
                                        )}
                                    </div>

                                    {/* Countdown timer for expiring plans with renewal info */}
                                    {isExpiringSoon && (
                                        <div className="mb-4 flex items-center gap-2 px-3 py-2 rounded-lg bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800">
                                            <AlertTriangle className="w-4 h-4 text-amber-600 dark:text-amber-400" />
                                            <span className="text-xs font-medium text-amber-700 dark:text-amber-300">
                                                {daysRemaining === 0 ? 'Expires today!' :
                                                    daysRemaining === 1 ? 'Expires tomorrow!' :
                                                        `Only ${daysRemaining} days left`}
                                            </span>
                                            <Link href="/plans" className="ml-auto text-[10px] font-bold text-amber-600 dark:text-amber-400 hover:underline">
                                                Renew {group.renewal_price ? `$${group.renewal_price}` : ''} →
                                            </Link>
                                        </div>
                                    )}

                                    {/* Renewal CTA for plans that can be renewed */}
                                    {canRenew && !isExpiringSoon && group.renewal_price && (
                                        <div className="mb-4 flex items-center justify-between px-3 py-2 rounded-lg bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800">
                                            <span className="text-xs text-blue-700 dark:text-blue-300">
                                                💳 Renewal: ${group.renewal_price}{group.billing_cycle ? `/${group.billing_cycle}` : ''}
                                            </span>
                                            <Link href="/plans" className="text-[10px] font-bold text-blue-600 dark:text-blue-400 hover:underline">
                                                Renew Now →
                                            </Link>
                                        </div>
                                    )}

                                    <div className="mt-auto space-y-2">
                                        <div className="text-[10px] font-bold text-gray-400 uppercase tracking-widest mb-1">Permissions</div>
                                        <div className="flex flex-wrap gap-1.5">
                                            {group.permissions.slice(0, 3).map((perm: string) => (
                                                <PermissionBadge key={perm} permission={perm} size="sm" showNote />
                                            ))}
                                        </div>
                                        {group.permissions.length > 3 && (
                                            <div className="text-[10px] text-indigo-500 font-bold ml-1">
                                                + {group.permissions.length - 3} more permissions
                                            </div>
                                        )}
                                    </div>
                                </div>
                            );
                        }) : (
                            <div className="col-span-full py-16 bg-gradient-to-br from-indigo-50/50 via-purple-50/30 to-pink-50/50 dark:from-gray-800/50 dark:via-purple-900/20 dark:to-gray-800/50 rounded-3xl border-2 border-dashed border-indigo-200/50 dark:border-indigo-700/30 text-center">
                                <div className="flex justify-center mb-4">
                                    <div className="p-4 bg-white/80 dark:bg-gray-800 rounded-2xl shadow-lg">
                                        <Sparkles className="w-8 h-8 text-indigo-500" />
                                    </div>
                                </div>
                                <h4 className="text-lg font-bold text-gray-900 dark:text-white mb-2">No Active Plans</h4>
                                <p className="text-sm text-gray-500 dark:text-gray-400 mb-6 max-w-sm mx-auto">
                                    Unlock premium features, advanced analytics, and more by upgrading your access level.
                                </p>
                                <a href="/plans" className="inline-flex items-center gap-2 px-6 py-3 bg-gradient-to-r from-indigo-500 to-purple-500 text-white rounded-xl font-bold text-sm hover:from-indigo-600 hover:to-purple-600 transition-all shadow-lg hover:shadow-indigo-500/25">
                                    <Sparkles className="w-4 h-4" />
                                    Explore Plans
                                </a>
                            </div>
                        )}
                    </div>

                    {data.direct_permissions.length > 0 && (
                        <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-3xl border-2 border-indigo-100 dark:border-indigo-900/50 p-6">
                            <h4 className="font-bold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
                                <span className="text-xl">✨</span> External / Direct Access
                            </h4>
                            <div className="grid sm:grid-cols-2 gap-3">
                                {data.direct_permissions.map((perm, idx) => {
                                    const status = getExpiryStatus(perm.expires_at);
                                    const daysLabel = perm.days_remaining !== undefined && perm.days_remaining !== null && perm.days_remaining >= 0
                                        ? `${perm.days_remaining}d left`
                                        : status.label;
                                    return (
                                        <div key={idx} className="flex items-center justify-between text-[11px] p-3 rounded-xl bg-gray-50/50 dark:bg-gray-900/40 border border-gray-100 dark:border-gray-800 group hover:border-indigo-200 dark:hover:border-indigo-800 transition-colors">
                                            <div className="flex items-center gap-2 truncate mr-2">
                                                <PermissionBadge permission={perm.permission} size="sm" showNote />
                                                {perm.source && (
                                                    <span className={`text-[8px] px-1 py-0.5 rounded ${perm.source === 'system' ? 'bg-purple-100 text-purple-600 dark:bg-purple-900/30 dark:text-purple-400' : 'bg-gray-100 text-gray-500 dark:bg-gray-800 dark:text-gray-400'}`}>
                                                        {perm.source === 'system' ? '⚙️' : '🔧'}
                                                    </span>
                                                )}
                                            </div>
                                            <Badge className="text-[9px] font-bold px-1.5 py-0 whitespace-nowrap" variant="outline">
                                                {daysLabel}
                                            </Badge>
                                        </div>
                                    );
                                })}
                            </div>
                        </div>
                    )}
                </div>

                {/* Right Col: Feature Checklist */}
                <div className="lg:col-span-4 space-y-6">
                    <h3 className="text-xl font-bold text-gray-900 dark:text-white flex items-center gap-3">
                        <span className="flex h-8 w-8 items-center justify-center rounded-xl bg-green-100 dark:bg-green-900/30 text-green-600 dark:text-green-400 text-sm font-bold shadow-sm">F</span>
                        Feature Access
                    </h3>
                    <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-xl rounded-3xl border-2 border-gray-100 dark:border-gray-800 p-6 shadow-xl relative overflow-hidden group">
                        <div className="relative z-10 space-y-1">
                            <FeatureRow label="Market Rankings" included={data.groups.some(g => g.permissions.some(p => p.includes('rankings:view')))} />
                            <FeatureRow label="Advanced Analytics" included={data.groups.some(g => g.permissions.some(p => p.includes('analytics:advanced')))} />
                            <FeatureRow label="Direct API Access" included={data.groups.some(g => g.permissions.some(p => p.includes('api:access')))} />
                            <FeatureRow label="CSV Data Export" included={data.groups.some(g => g.permissions.some(p => p.includes('export')))} />
                            <FeatureRow label="Custom Notifications" included={true} />
                            <FeatureRow label="Beta Lab Access" included={data.groups.some(g => g.permissions.some(p => p.includes('beta')))} />
                        </div>
                        <div className="mt-6 pt-6 border-t border-gray-100 dark:border-gray-800 relative z-10">
                            <div className="text-[10px] text-center text-gray-400 font-bold uppercase tracking-widest leading-relaxed">
                                Managed by protocol permissions
                            </div>
                        </div>

                        {/* Subtle background decoration */}
                        <div className="absolute -bottom-8 -right-8 h-24 w-24 bg-green-500/5 rounded-full blur-2xl group-hover:bg-green-500/10 transition-colors"></div>
                    </div>

                    <div className="p-6 rounded-3xl bg-indigo-50 dark:bg-indigo-900/20 border-2 border-indigo-100 dark:border-indigo-900/50">
                        <h5 className="font-bold text-indigo-900 dark:text-indigo-100 text-xs mb-2 uppercase tracking-wider">Pro Tip</h5>
                        <p className="text-[11px] text-indigo-700/80 dark:text-indigo-300/80 leading-relaxed">
                            Upgrade to a higher tier to unlock advanced trading signals and historical data exports.
                        </p>
                    </div>
                </div>
            </div>
        </div>
    );
}
