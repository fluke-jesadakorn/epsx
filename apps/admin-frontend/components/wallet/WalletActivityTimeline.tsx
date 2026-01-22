/**
 * Wallet Activity Timeline Component
 * Timeline display of wallet activity events
 */
'use client';

import type { WalletActivityEvent } from './types';

import { cn } from '@/lib/utils';

interface WalletActivityTimelineProps {
    events: WalletActivityEvent[];
    maxItems?: number;
    showAll?: boolean;
    onViewAll?: () => void;
    className?: string;
}

const EVENT_CONFIG: Record<WalletActivityEvent['type'], {
    emoji: string;
    label: string;
    color: string;
}> = {
    permission_granted: {
        emoji: '✅',
        label: 'Permission Granted',
        color: 'bg-green-500',
    },
    permission_revoked: {
        emoji: '❌',
        label: 'Permission Revoked',
        color: 'bg-red-500',
    },
    subscription_started: {
        emoji: '📦',
        label: 'Subscription Started',
        color: 'bg-purple-500',
    },
    subscription_cancelled: {
        emoji: '🚫',
        label: 'Subscription Cancelled',
        color: 'bg-orange-500',
    },
    wallet_disabled: {
        emoji: '⚠️',
        label: 'Wallet Disabled',
        color: 'bg-amber-500',
    },
    wallet_enabled: {
        emoji: '🔓',
        label: 'Wallet Enabled',
        color: 'bg-green-500',
    },
    wallet_created: {
        emoji: '🆕',
        label: 'Wallet Created',
        color: 'bg-blue-500',
    },
    login: {
        emoji: '🔑',
        label: 'Login',
        color: 'bg-gray-500',
    },
};

function formatTimestamp(timestamp: string): { date: string; time: string } {
    const d = new Date(timestamp);
    return {
        date: d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' }),
        time: d.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' }),
    };
}

function TimelineItem({ event, isLast }: { event: WalletActivityEvent; isLast: boolean }) {
    const config = EVENT_CONFIG[event.type];
    const { date, time } = formatTimestamp(event.timestamp);

    return (
        <div className="flex gap-4">
            {/* Timeline Line */}
            <div className="flex flex-col items-center">
                <div className={cn('w-3 h-3 rounded-full', config.color)} />
                {!isLast && <div className="w-0.5 flex-1 bg-gray-200 dark:bg-gray-700 my-1" />}
            </div>

            {/* Content */}
            <div className="flex-1 pb-4">
                <div className="flex items-start justify-between gap-4">
                    <div>
                        <p className="text-sm font-medium text-foreground flex items-center gap-2">
                            <span>{config.emoji}</span>
                            {event.description}
                        </p>
                        {event.performedBy && (
                            <p className="text-sm text-muted-foreground mt-0.5">
                                by {event.performedBy}
                            </p>
                        )}
                    </div>
                    <div className="text-right shrink-0">
                        <p className="text-xs font-medium text-muted-foreground">{date}</p>
                        <p className="text-[10px] text-muted-foreground/70 uppercase tracking-wider">{time}</p>
                    </div>
                </div>
            </div>
        </div>
    );
}

/**
 *
 * @param root0
 * @param root0.events
 * @param root0.maxItems
 * @param root0.showAll
 * @param root0.onViewAll
 * @param root0.className
 */
export function WalletActivityTimeline({
    events,
    maxItems = 5,
    showAll = false,
    onViewAll,
    className,
}: WalletActivityTimelineProps) {
    if (events.length === 0) {
        return (
            <div className={cn('text-center py-8 text-gray-500 dark:text-gray-400', className)}>
                <span className="text-3xl mb-2 block">📜</span>
                <p className="text-sm">No activity recorded yet</p>
            </div>
        );
    }

    const displayEvents = showAll ? events : events.slice(0, maxItems);
    const hasMore = !showAll && events.length > maxItems;

    return (
        <div className={className}>
            <div className="divide-y-0">
                {displayEvents.map((event, index) => (
                    <TimelineItem
                        key={event.id}
                        event={event}
                        isLast={index === displayEvents.length - 1 && !hasMore}
                    />
                ))}
            </div>

            {hasMore && onViewAll && (
                <button
                    onClick={onViewAll}
                    className="w-full text-center py-2 text-sm text-blue-600 dark:text-blue-400 hover:text-blue-700 dark:hover:text-blue-300 font-medium"
                >
                    View All ({events.length} events) →
                </button>
            )}
        </div>
    );
}
