'use client';

import { useRouter } from 'next/navigation';
import { useEffect, useState } from 'react';

import { PaymentAnalytics } from '@/components/payments/PaymentAnalytics';
import { PaymentLinksManagement } from '@/components/payments/PaymentLinksManagement';
import { PaymentsManagement } from '@/components/payments/PaymentsManagement';
import { UserAccessManagement } from '@/components/payments/UserAccessManagement';
import { PageHeader, PageLayout, PageSkeleton, PageTabs, type TabItem } from '@/components/shared';
import { useSharedAuth } from '@/shared/components/auth/Provider';

type TabType = 'payments' | 'user-access' | 'payment-links' | 'analytics';

const tabs: TabItem[] = [
  { id: 'payments', label: 'Payments', icon: 'CreditCard', gradient: 'info' },
  { id: 'user-access', label: 'User Access', icon: 'Users', gradient: 'success' },
  { id: 'payment-links', label: 'Links', icon: 'Link', gradient: 'purple' },
  { id: 'analytics', label: 'Analytics', icon: 'BarChart3', gradient: 'indigo' },
];

/**
 * Payments Page
 * Uses unified page components for consistent design
 */
export default function AdminPaymentsPage() {
  const { isAuthenticated, isLoading } = useSharedAuth();
  const router = useRouter();
  const [activeTab, setActiveTab] = useState<TabType>('payments');

  // Redirect to auth if not authenticated
  useEffect(() => {
    if (!isLoading && !isAuthenticated) {
      router.push('/auth');
    }
  }, [isAuthenticated, isLoading, router]);

  if (isLoading || !isAuthenticated) {
    return <PageSkeleton showHeader showTabs tabCount={4} stats={4} rows={6} />;
  }

  return (
    <PageLayout>
      <PageHeader
        title="Payments Hub"
        subtitle="Manage payments, user access, and payment links"
        icon="CreditCard"
        gradient="primary"
        centered
      />

      <PageTabs
        tabs={tabs}
        activeTab={activeTab}
        onTabChange={(id) => setActiveTab(id as TabType)}
        className="mb-8"
      />

      {activeTab === 'payments' && <PaymentsManagement />}
      {activeTab === 'user-access' && <UserAccessManagement />}
      {activeTab === 'payment-links' && <PaymentLinksManagement />}
      {activeTab === 'analytics' && <PaymentAnalytics />}
    </PageLayout>
  );
}
