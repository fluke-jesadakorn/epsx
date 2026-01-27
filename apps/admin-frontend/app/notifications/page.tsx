'use client';

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
        title="Command Center"
        subtitle="Global broadcast protocol and network alert management"
        icon="Bell"
        gradient="warning"
        centered
      />

      <PageTabs
        tabs={tabs}
        activeTab={activeTab}
        onTabChange={setActiveTab}
      />

      <div className="mt-8">
        {activeTab === 'overview' ? (
          <NotificationManagement currentUser={user} />
        ) : (
          <div className="relative overflow-hidden rounded-[40px] bg-slate-900/40 backdrop-blur-2xl border border-white/5 p-1 shadow-2xl">
            <div className="relative bg-card/60 backdrop-blur-md rounded-[38px] p-8 sm:p-12">
              <div className="max-w-4xl mx-auto">
                <div className="mb-12">
                  <h2 className="text-3xl font-black text-foreground uppercase tracking-tight mb-2">
                    Signal Generator
                  </h2>
                  <p className="text-sm font-bold text-muted-foreground">Construct and transmit high-priority system alerts</p>
                </div>
                <SendNotificationForm
                  onSuccess={() => setActiveTab('overview')}
                  onCancel={() => setActiveTab('overview')}
                />
              </div>
            </div>
          </div>
        )}
      </div>
    </PageLayout>
  );
}
