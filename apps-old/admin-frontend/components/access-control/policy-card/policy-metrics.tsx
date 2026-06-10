import { Clock, TrendingUp, Users } from 'lucide-react';
import { type ReactNode } from 'react';

import { type AccessPolicy } from '@/components/access-control/types';

interface MetricCardProps {
    label: string;
    children: ReactNode;
    icon?: ReactNode;
    variant?: 'default' | 'blue' | 'emerald';
}

function MetricCard({
    label,
    children,
    icon,
    variant = 'default',
}: MetricCardProps) {
    let bgClass = 'bg-muted/40 border-border';
    if (variant === 'blue') {
        bgClass = 'bg-blue-500/10 border-blue-500/20';
    } else if (variant === 'emerald') {
        bgClass = 'bg-emerald-500/10 border-emerald-500/20';
    }

    return (
        <div className={`flex flex-col p-3 rounded-xl border ${bgClass}`}>
            <span className="text-[10px] uppercase text-muted-foreground font-medium tracking-wide mb-1">
                {label}
            </span>
            <div className="flex items-center gap-1.5">
                {icon}
                <span className="text-sm font-semibold text-foreground">{children}</span>
            </div>
        </div>
    );
}

interface PolicyMetricsProps {
    policy: AccessPolicy;
    isSubscription: boolean;
    typeIcon: string;
    typeLabel: string;
}

function SubscriptionMetrics({ policy }: { policy: AccessPolicy }) {
    const isFree = policy.pricing?.amount === 0;
    return (
        <>
            <MetricCard label="Price" variant="blue">
                {isFree ? (
                    <span className="text-emerald-600 dark:text-emerald-400">Free</span>
                ) : (
                    <>
                        ${policy.pricing?.amount.toFixed(2)}
                        <span className="text-muted-foreground font-normal text-xs ml-1">{policy.pricing?.currency}</span>
                    </>
                )}
            </MetricCard>
            <MetricCard label="Revenue (30d)" variant="emerald">
                <TrendingUp className="h-4 w-4 text-emerald-500 inline mr-1.5" />
                <span className="text-emerald-600 dark:text-emerald-400">${(policy.revenue ?? 0).toFixed(2)}</span>
            </MetricCard>
        </>
    );
}

function GroupMetrics({ policy }: { policy: AccessPolicy }) {
    const hasExpiry = policy.expiryDays !== undefined && policy.expiryDays !== 0;
    return (
        <>
            <MetricCard label="Perms">
                {policy.permissions.length}{' '}
                <span className="text-muted-foreground font-normal text-xs">active</span>
            </MetricCard>
            <MetricCard label={hasExpiry ? 'Expiry' : 'Priority'}>
                {hasExpiry ? (
                    <>
                        <Clock className="h-4 w-4 text-amber-500 inline mr-1.5" />
                        {policy.expiryDays === -1 ? 'Permanent' : `${policy.expiryDays ?? 0}d`}
                    </>
                ) : (
                    `Level ${policy.priorityLevel ?? 0}`
                )}
            </MetricCard>
        </>
    );
}

export function PolicyMetrics({
    policy,
    isSubscription,
    typeIcon,
    typeLabel,
}: PolicyMetricsProps) {
    return (
        <div className="grid grid-cols-2 gap-3">
            <MetricCard label="Type">
                <span className="text-base">{typeIcon}</span>
                <span className="ml-1.5">{typeLabel}</span>
            </MetricCard>
            <MetricCard label={isSubscription ? 'Subs' : 'Users'} icon={<Users className="h-4 w-4 text-blue-500" />}>
                {policy.memberCount}
            </MetricCard>
            {isSubscription ? <SubscriptionMetrics policy={policy} /> : <GroupMetrics policy={policy} />}
        </div>
    );
}
