'use client';

import { fetchActivityLogsAction } from '@/app/wallet-management/actions';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Skeleton } from '@/components/ui/skeleton';
import { logger } from '@/lib/logger';
import { cn } from '@/lib/utils';
import { ExternalLink, RefreshCw, Search } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import type { WalletActivityEvent } from './types';
import { WalletActivityTimeline } from './wallet-activity-timeline';

interface ActivityLogSectionProps {
    className?: string;
    initialEvents?: WalletActivityEvent[];
}

interface ActivityLogEntry {
    id: string;
    action: string;
    timestamp: string;
    wallet_address: string;
    details: unknown;
}

export function ActivityLogSection({ className, initialEvents }: ActivityLogSectionProps) {
    const [events, setEvents] = useState<WalletActivityEvent[]>(initialEvents ?? []);
    const [isLoading, setIsLoading] = useState(!initialEvents);

    const fetchLogs = useCallback(async () => {
        setIsLoading(true);
        try {
            // Use Server Action
            const logs = await fetchActivityLogsAction(undefined, 1, 10) as ActivityLogEntry[];

            // Map to WalletActivityEvent
            const mappedEvents: WalletActivityEvent[] = logs.map((log) => ({
                id: log.id,
                type: mapActionToEventType(log.action),
                description: formatActionDescription(log.action),
                timestamp: log.timestamp,
                performedBy: log.wallet_address ?? 'System',
                metadata: log.details && typeof log.details === 'object' ? (log.details as Record<string, unknown>) : undefined
            }));
            setEvents(mappedEvents);
        } catch (err) {
            logger.error('Failed to fetch activity logs:', { err });
        } finally {
            setIsLoading(false);
        }
    }, []);

    useEffect(() => {
        fetchLogs();
    }, [fetchLogs]);

    return (
        <div className={cn("flex flex-col h-full bg-slate-900/40 backdrop-blur-2xl border border-white/5 rounded-[32px] shadow-xl overflow-hidden", className)}>
            {/* Filter Bar */}
            <div className="p-6 border-b border-white/5 flex flex-col sm:flex-row gap-4 justify-between items-start sm:items-center bg-white/5">
                <div className="flex items-center gap-3 w-full sm:w-auto flex-1">
                    <div className="relative w-full sm:max-w-md">
                        <Search className="absolute left-4 top-3 h-4 w-4 text-muted-foreground" />
                        <Input
                            placeholder="Search activity logs..."
                            className="pl-11 h-12 bg-white/5 border-white/5 focus:bg-white/10 transition-all rounded-2xl placeholder:text-muted-foreground/50 font-medium"
                        />
                    </div>
                </div>

                <div className="flex items-center gap-2">
                    <Button variant="outline" size="sm" className="h-12 border-dashed rounded-2xl bg-white/5 border-white/10 hover:bg-white/10 font-bold" onClick={fetchLogs} disabled={isLoading}>
                        <RefreshCw className={cn("h-4 w-4 mr-2", isLoading && "animate-spin")} />
                        Refresh
                    </Button>
                </div>
            </div>

            {/* Content */}
            <div className="flex-1 p-6 bg-muted/5 min-h-[500px]">
                {isLoading ? (
                    <div className="space-y-8 max-w-3xl mx-auto pt-8">
                        {[1, 2, 3, 4].map(i => <div key={i} className="flex gap-4">
                            <Skeleton className="h-4 w-4 rounded-full mt-2" />
                            <div className="space-y-3 flex-1">
                                <Skeleton className="h-5 w-1/3" />
                                <Skeleton className="h-4 w-2/3" />
                                <Skeleton className="h-20 w-full rounded-xl" />
                            </div>
                        </div>)}
                    </div>
                ) : (
                    <div className="max-w-4xl mx-auto">
                        <WalletActivityTimeline
                            events={events}
                            maxItems={50}
                            showAll={true}
                            className="space-y-8"
                        />
                    </div>
                )}
            </div>

            {/* Simple Footer */}
            <div className="p-3 border-t border-border bg-muted/20 flex justify-center">
                <Button variant="link" className="text-xs text-muted-foreground hover:text-primary">
                    View Complete Audit Log Archive <ExternalLink className="ml-1 h-3 w-3" />
                </Button>
            </div>
        </div>
    );
}

// Helpers
function mapActionToEventType(action: string): WalletActivityEvent['type'] {
    if (action.includes('grant') || action.includes('permission')) { return 'permission_granted'; }
    if (action.includes('revoke')) { return 'permission_revoked'; }
    if (action.includes('disable')) { return 'wallet_disabled'; }
    if (action.includes('enable')) { return 'wallet_enabled'; }
    if (action.includes('login')) { return 'login'; }
    return 'wallet_created'; // fallback
}

function formatActionDescription(action: string): string {
    return action.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ');
}
