/**
 * Wallet Activity History Component
 * Timeline of wallet activities and events
 */
'use client';

import { Button } from '@/components/ui/button';
import { formatTimeAgo } from '@/lib/utils/date';
import type { WalletActivityEvent } from './types';

interface WalletActivityHistoryProps {
    events: WalletActivityEvent[];
    limit?: number;
}

function getEventIcon(type: string): string {
    switch (type) {
        case 'permission_granted': return '✅';
        case 'permission_revoked': return '❌';
        case 'subscription_started': return '📦';
        case 'subscription_cancelled': return '🚫';
        case 'wallet_disabled': return '⚠️';
        case 'wallet_enabled': return '🔓';
        case 'wallet_created': return '🆕';
        case 'login': return '🔑';
        default: return '📝';
    }
}

function ActivityItem({ event }: { event: WalletActivityEvent }) {
    return (
        <div className="flex items-start gap-4 py-3 first:pt-0 last:pb-0">
            <div className="flex-shrink-0 w-8 h-8 rounded-full bg-gray-100 dark:bg-gray-800 flex items-center justify-center text-lg shadow-sm">
                {getEventIcon(event.type)}
            </div>
            <div className="flex-1 min-w-0">
                <p className="text-sm font-medium text-gray-900 dark:text-gray-100">
                    {event.description}
                </p>
                <div className="flex items-center gap-2 mt-1">
                    <span className="text-xs text-gray-500 dark:text-gray-400">
                        {formatTimeAgo(event.timestamp)}
                    </span>
                    {event.performedBy && (
                        <>
                            <span className="text-gray-300 dark:text-gray-700">•</span>
                            <span className="text-xs text-gray-500 dark:text-gray-400">
                                by {event.performedBy}
                            </span>
                        </>
                    )}
                </div>
            </div>
        </div>
    );
}

export function WalletActivityHistory({ events, limit = 5 }: WalletActivityHistoryProps) {
    if (events.length === 0) {
        return (
            <div className="text-center py-6 text-gray-500 dark:text-gray-400 text-sm italic">
                No activity history found
            </div>
        );
    }

    const displayedEvents = events.slice(0, limit);

    return (
        <div className="space-y-4">
            <div className="border border-gray-200 dark:border-gray-800 rounded-xl p-4 bg-gray-50/30 dark:bg-gray-900/10">
                <div className="divide-y divide-gray-100 dark:divide-gray-800">
                    {displayedEvents.map((event) => (
                        <ActivityItem key={event.id} event={event} />
                    ))}
                </div>

                {events.length > limit && (
                    <Button
                        variant="ghost"
                        size="sm"
                        className="w-full mt-4 text-xs text-blue-600 dark:text-blue-400 hover:text-blue-700 dark:hover:text-blue-300"
                    >
                        View all activity ({events.length} events)
                    </Button>
                )}
            </div>
        </div>
    );
}
