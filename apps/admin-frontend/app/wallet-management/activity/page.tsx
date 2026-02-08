
import { ActivityLogSection } from '@/components/wallet/activity-log-section';
import { fetchActivityLogsAction } from '../actions';

export default async function ActivityPage() {
    const initialActivityLogsRaw = await fetchActivityLogsAction(undefined, 1, 10).catch(() => []);

    // Map activity logs to frontend format
    const initialActivityLogs = (initialActivityLogsRaw || []).map((log) => {
        const details = log.details as Record<string, unknown> | undefined;
        const action = (details?.action as string | undefined) ?? log.action;

        let type: 'permission_granted' | 'permission_revoked' | 'wallet_created' = 'wallet_created';
        if (action.includes('grant')) {
            type = 'permission_granted';
        } else if (action.includes('revoke')) {
            type = 'permission_revoked';
        }

        return {
            id: log.id,
            type,
            description: (details?.description as string | undefined) ?? 'Activity logged',
            timestamp: log.timestamp,
            performedBy: log.wallet_address ?? 'System',
            metadata: details
        };
    });

    return <ActivityLogSection initialEvents={initialActivityLogs} />;
}
