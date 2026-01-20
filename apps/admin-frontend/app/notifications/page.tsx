'use client';

import { Bell } from 'lucide-react';
import { useState } from 'react';

import { NotificationManagement } from '@/components/notifications/NotificationManagement';
import { SendNotificationForm } from '@/components/notifications/SendNotificationForm';
import { PageAuthRequired, PageHeader, PageLayout, PageSkeleton, PageTabs, type TabItem } from '@/components/shared';
import { useSharedAuth } from '@/shared/components/auth/Provider';

const tabs: TabItem[] = [
  { id: 'overview', label: 'Overview', prefix: '📊', gradient: 'info' },
  { id: 'send', label: 'Send Notification', prefix: '📨', gradient: 'warning' },
];

/**
 * Notifications Page
 * Uses unified page components for consistent design
 */
export default function NotificationsPage() {
  const { user, isLoading, isAuthenticated } = useSharedAuth();
  const [activeTab, setActiveTab] = useState<string>('overview');

  if (isLoading) {
    return <PageSkeleton showHeader showTabs tabCount={2} stats={4} rows={6} />;
  }

  if (!isAuthenticated || !user) {
    return <PageAuthRequired />;
  }

  return (
    <PageLayout>
      <PageHeader
        title="Notifications"
        subtitle="Send notifications and manage user alerts across the platform"
        icon={Bell}
        gradient="info"
      />

      <PageTabs
        tabs={tabs}
        activeTab={activeTab}
        onTabChange={setActiveTab}
      />

      {activeTab === 'overview' ? (
        <NotificationManagement currentUser={user} />
      ) : (
        <div className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-amber-400/10 via-orange-400/10 to-red-400/10 p-0.5">
          <div className="relative bg-card/95 backdrop-blur-xl rounded-2xl p-6 sm:p-8">
            <h2 className="text-xl sm:text-2xl font-bold mb-6 bg-gradient-to-r from-amber-500 via-orange-500 to-red-500 bg-clip-text text-transparent">
              Send Notification
            </h2>
            <div className="max-w-3xl">
              <SendNotificationForm
                onSuccess={() => setActiveTab('overview')}
                onCancel={() => setActiveTab('overview')}
              />
            </div>
          </div>
        </div>
      )}
    </PageLayout>
  );
}
