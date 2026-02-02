import { NotificationTabsNavigation } from '@/components/notifications/NotificationTabsNavigation';
import { PageHeader, PageLayout } from '@/components/shared';

/**
 * Notifications Layout
 *
 * Wraps all notification pages with:
 * 1. Page Header
 * 2. Navigation Tabs
 */
export default async function NotificationsLayout({
    children,
}: {
    children: React.ReactNode;
}) {
    return (
        <PageLayout>
            {/* Page Header */}
            <PageHeader
                title="Command Center"
                subtitle="Global broadcast protocol and network alert management"
                icon="Bell"
                gradient="warning"
                centered
            />

            {/* Navigation Tabs */}
            <NotificationTabsNavigation />

            {/* Page Content */}
            <div className="animate-in fade-in-50 duration-500 pb-12">
                {children}
            </div>
        </PageLayout>
    );
}

// Force dynamic rendering
export const dynamic = 'force-dynamic';
