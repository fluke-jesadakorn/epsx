import { NotificationManagement } from '@/components/notifications/NotificationManagement';
import { PageAuthRequired } from '@/components/shared';
import { getCurrentUser } from '@/lib/auth/server';

export default async function ManageNotificationsPage() {
    const user = await getCurrentUser();

    if (!user) {
        return <PageAuthRequired />;
    }

    return <NotificationManagement currentUser={user} />;
}
