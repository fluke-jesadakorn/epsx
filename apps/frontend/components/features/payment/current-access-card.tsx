'use client';

/**
 * CurrentAccessCard Component
 * 
 * Displays the user's current plan, group, or permission access status
 * with expiration info and renewal/upgrade prompts.
 */

import { usePlanAccess } from '@/hooks/use-plan-access';
import { cn } from '@/lib/utils';
import { format } from 'date-fns';
import {
    AlertTriangle,
    Calendar,
    CheckCircle2,
    Clock,
    Crown,
    Loader2,
    ShieldCheck,
    Sparkles,
    TrendingUp
} from 'lucide-react';

export type PaymentType = 'plan' | 'access-plan' | 'group' | 'permission';

interface CurrentAccessCardProps {
    className?: string;
    paymentType?: PaymentType;
}

export function CurrentAccessCard({ className, paymentType = 'plan' }: CurrentAccessCardProps) {
    const { planAccess, loading, error } = usePlanAccess();

    if (loading) {
        return (
            <div className={cn('bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl p-6 shadow-xl border border-purple-200/50 dark:border-purple-700/50', className)}>
                <div className="flex items-center justify-center py-8">
                    <Loader2 className="w-8 h-8 text-purple-500 animate-spin" />
                    <span className="ml-3 text-gray-600 dark:text-gray-400">Loading your access...</span>
                </div>
            </div>
        );
    }

    if (error) {
        return (
            <div className={cn('bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl p-6 shadow-xl border border-red-200/50 dark:border-red-700/50', className)}>
                <div className="flex items-center gap-3 text-red-600 dark:text-red-400">
                    <AlertTriangle className="w-5 h-5" />
                    <span className="text-sm">Unable to load access info</span>
                </div>
            </div>
        );
    }

    const hasActivePlan = planAccess?.status === 'active' || planAccess?.status === 'expiring_soon';
    const isExpired = planAccess?.status === 'expired';
    const isExpiringSoon = planAccess?.status === 'expiring_soon';
    const daysRemaining = planAccess?.days_remaining ?? 0;

    // Format expiration date
    const expiryDate = planAccess?.plan_expires_at
        ? format(new Date(planAccess.plan_expires_at), 'MMM d, yyyy')
        : null;

    // Get status color and icon
    const getStatusConfig = () => {
        if (!planAccess || planAccess.status === 'no_plan') {
            return {
                gradient: 'from-gray-400 to-gray-600',
                bgGradient: 'from-gray-50 to-gray-100 dark:from-gray-800/50 dark:to-gray-900/50',
                borderColor: 'border-gray-200 dark:border-gray-700',
                icon: <Sparkles className="w-6 h-6" />,
                label: 'Free Tier',
                sublabel: 'Upgrade to unlock more features',
            };
        }

        if (isExpired) {
            return {
                gradient: 'from-red-400 to-rose-600',
                bgGradient: 'from-red-50 to-rose-100 dark:from-red-900/20 dark:to-rose-900/20',
                borderColor: 'border-red-200 dark:border-red-800',
                icon: <AlertTriangle className="w-6 h-6" />,
                label: 'Expired',
                sublabel: 'Renew to restore access',
            };
        }

        if (isExpiringSoon) {
            return {
                gradient: 'from-amber-400 to-orange-600',
                bgGradient: 'from-amber-50 to-orange-100 dark:from-amber-900/20 dark:to-orange-900/20',
                borderColor: 'border-amber-200 dark:border-amber-800',
                icon: <Clock className="w-6 h-6" />,
                label: `${daysRemaining} days left`,
                sublabel: 'Renew soon to keep access',
            };
        }

        return {
            gradient: 'from-emerald-400 to-teal-600',
            bgGradient: 'from-emerald-50 to-teal-100 dark:from-emerald-900/20 dark:to-teal-900/20',
            borderColor: 'border-emerald-200 dark:border-emerald-800',
            icon: <CheckCircle2 className="w-6 h-6" />,
            label: 'Active',
            sublabel: expiryDate ? `Valid until ${expiryDate}` : 'Permanent access',
        };
    };

    const config = getStatusConfig();

    // Get type-specific label
    const typeLabel = paymentType === 'plan' ? 'Plan' : paymentType === 'group' ? 'Group' : 'permission';

    return (
        <div className={cn('relative overflow-hidden rounded-2xl shadow-xl', config.borderColor, 'border-2', className)}>
            {/* Background gradient */}
            <div className={cn('absolute inset-0 bg-gradient-to-br opacity-50', config.bgGradient)} />

            {/* Decorative elements */}
            <div className="absolute top-0 right-0 -mr-8 -mt-8 h-32 w-32 rounded-full bg-white/10 blur-2xl" />
            <div className="absolute bottom-0 left-0 -ml-8 -mb-8 h-24 w-24 rounded-full bg-white/10 blur-2xl" />

            <div className="relative z-10 p-6">
                {/* Header */}
                <div className="flex items-start justify-between mb-4">
                    <div className="flex items-center gap-3">
                        <div className={cn('flex h-12 w-12 items-center justify-center rounded-xl bg-gradient-to-br text-white shadow-lg', config.gradient)}>
                            {hasActivePlan || isExpired ? <Crown className="w-6 h-6" /> : config.icon}
                        </div>
                        <div>
                            <h3 className="text-sm font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                                Current {typeLabel}
                            </h3>
                            <p className="text-xl font-black text-gray-900 dark:text-white">
                                {planAccess?.plan_name ?? 'Free Tier'}
                            </p>
                        </div>
                    </div>
                    <div className={cn('flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-bold', `bg-gradient-to-r ${config.gradient} text-white shadow-md`)}>
                        {config.icon}
                        <span>{config.label}</span>
                    </div>
                </div>

                {/* Status info */}
                <div className="flex items-center gap-4 text-sm">
                    {hasActivePlan && expiryDate && (
                        <div className="flex items-center gap-2 text-gray-600 dark:text-gray-400">
                            <Calendar className="w-4 h-4" />
                            <span>Expires {expiryDate}</span>
                        </div>
                    )}
                    {planAccess?.ranking_offset !== undefined && (
                        <div className="flex items-center gap-2 text-gray-600 dark:text-gray-400">
                            <TrendingUp className="w-4 h-4" />
                            <span>
                                {planAccess.ranking_offset === 0
                                    ? 'Full rankings access'
                                    : `Ranks ${planAccess.ranking_offset + 1}+`}
                            </span>
                        </div>
                    )}
                    {planAccess?.can_upgrade && (
                        <div className="flex items-center gap-2 text-purple-600 dark:text-purple-400">
                            <ShieldCheck className="w-4 h-4" />
                            <span className="font-medium">Upgrade available</span>
                        </div>
                    )}
                </div>

                {/* Warning for expiring or expired */}
                {(isExpiringSoon || isExpired) && (
                    <div className={cn(
                        'mt-4 p-3 rounded-lg border flex items-center gap-3',
                        isExpired
                            ? 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800'
                            : 'bg-amber-50 dark:bg-amber-900/20 border-amber-200 dark:border-amber-800'
                    )}>
                        <AlertTriangle className={cn('w-5 h-5', isExpired ? 'text-red-500' : 'text-amber-500')} />
                        <div>
                            <p className={cn('text-sm font-medium', isExpired ? 'text-red-700 dark:text-red-300' : 'text-amber-700 dark:text-amber-300')}>
                                {isExpired
                                    ? 'Your plan has expired. Renew to restore full access.'
                                    : `Only ${daysRemaining} days remaining. Renew now to avoid interruption.`}
                            </p>
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}

export default CurrentAccessCard;
