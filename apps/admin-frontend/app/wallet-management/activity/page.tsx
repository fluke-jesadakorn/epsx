
import { ActivityLogSection } from '@/components/wallet/ActivityLogSection';
import { fetchActivityLogsAction } from '../actions';

export default async function ActivityPage() {
    const initialActivityLogsRaw = await fetchActivityLogsAction(undefined, 1, 10).catch(() => []);

    // Map activity logs to frontend format
    const initialActivityLogs = (initialActivityLogsRaw || []).map((log: any) => ({
        id: log.id,
        type: (log.details?.action?.includes('grant') ? 'permission_granted' :
            log.details?.action?.includes('revoke') ? 'permission_revoked' : 'wallet_created') || 'wallet_created',
        description: log.details?.description || 'Activity logged',
        timestamp: log.timestamp,
        performedBy: log.wallet_address || 'System',
        metadata: log.details
    }));

    return <ActivityLogSection initialEvents={initialActivityLogs} />;
}
