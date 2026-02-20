import { redirect } from 'next/navigation';

import { NotificationManagement } from '@/components/notifications/notification-management';
import { getCurrentUser } from '@/lib/auth/server';

export default async function ManageNotificationsPage() {
    const user = await getCurrentUser();

    if (!user) {
        redirect('/auth');
    }

    return <NotificationManagement currentUser={user} />;
}
