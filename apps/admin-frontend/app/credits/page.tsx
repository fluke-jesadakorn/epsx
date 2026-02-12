'use client';

import { useRouter } from 'next/navigation';
import { useEffect, useState } from 'react';

import { CreditsManagement } from '@/components/credits/credits-management';
import { PageHeader, PageLayout, PageSkeleton, PageTabs, type TabItem } from '@/components/shared';
import { useSharedAuth } from '@/shared/components/auth';

type TabType = 'overview' | 'grant' | 'history';

const tabs: TabItem[] = [
  { id: 'overview', label: 'Overview', icon: 'BarChart3', gradient: 'info' },
  { id: 'grant', label: 'Grant Credits', icon: 'Plus', gradient: 'success' },
  { id: 'history', label: 'Credit History', icon: 'History', gradient: 'purple' },
];

export default function AdminCreditsPage() {
  const { isAuthenticated, isLoading } = useSharedAuth();
  const router = useRouter();
  const [activeTab, setActiveTab] = useState<TabType>('overview');

  useEffect(() => {
    if (!isLoading && !isAuthenticated) {
      router.push('/auth');
    }
  }, [isAuthenticated, isLoading, router]);

  if (isLoading || !isAuthenticated) {
    return <PageSkeleton showHeader showTabs tabCount={3} stats={4} rows={6} />;
  }

  return (
    <PageLayout>
      <PageHeader
        title="Credits Management"
        subtitle="Manage user credit balances and grant credits"
        icon="Coins"
        gradient="primary"
        centered
      />

      <PageTabs
        tabs={tabs}
        activeTab={activeTab}
        onTabChange={(id) => setActiveTab(id as TabType)}
        className="mb-8"
      />

      <CreditsManagement activeTab={activeTab} />
    </PageLayout>
  );
}
