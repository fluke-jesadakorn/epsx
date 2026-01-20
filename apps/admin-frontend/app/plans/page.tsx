'use client';

import { Package } from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useEffect, useState } from 'react';

import { PlanManagement } from '@/components/plans/PlanManagement';
import { PromotionManagement } from '@/components/promotions/PromotionManagement';
import { PageAuthRequired, PageHeader, PageLayout, PageSkeleton, PageTabs, type TabItem } from '@/components/shared';
import { useSharedAuth } from '@/shared/components/auth/Provider';

type TabType = 'plans' | 'promotions';

const tabs: TabItem[] = [
  { id: 'plans', label: 'Plans Management', prefix: '💳', gradient: 'primary' },
  { id: 'promotions', label: 'Promotions & Deals', prefix: '🎁', gradient: 'success' },
];

/**
 * Plans Page
 * Uses unified page components for consistent design
 */
export default function AdminPlansPage() {
  const { isAuthenticated, isLoading } = useSharedAuth();
  const router = useRouter();
  const [activeTab, setActiveTab] = useState<TabType>('plans');

  // Redirect to auth if not authenticated
  useEffect(() => {
    if (!isLoading && !isAuthenticated) {
      router.push('/auth');
    }
  }, [isAuthenticated, isLoading, router]);

  if (isLoading || !isAuthenticated) {
    return <PageSkeleton showHeader showTabs tabCount={2} stats={4} rows={8} />;
  }

  return (
    <PageLayout>
      <PageHeader
        title="Plans & Promotions"
        subtitle="Create and manage subscription plans and promotional offers"
        icon={Package}
        gradient="primary"
      />

      <PageTabs
        tabs={tabs}
        activeTab={activeTab}
        onTabChange={(id) => setActiveTab(id as TabType)}
      />

      {activeTab === 'plans' ? (
        <PlanManagement />
      ) : (
        <PromotionManagement />
      )}
    </PageLayout>
  );
}
