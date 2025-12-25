import { UsageMonitor } from '@/components/developer/UsageMonitor';
import { getCurrentUser } from '@/lib/server-actions';

export const dynamic = 'force-dynamic';

export default async function DeveloperUsagePage() {
    const user = await getCurrentUser();

    if (!user) {
        return null; // Layout handles auth guard
    }

    return (
        <div className="space-y-8">
            <UsageMonitor currentUser={user} />
        </div>
    );
}
