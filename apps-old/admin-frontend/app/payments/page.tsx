'use client';

import { useSearchParams } from 'next/navigation';

import { PaymentLinksManagement } from '@/components/payments/payment-links-management';
import { PaymentsManagement } from '@/components/payments/payments-management';
import { UserAccessManagement } from '@/components/payments/user-access-management';
import { PageHeader, PageLayout } from '@/components/shared';

type TabType = 'payments' | 'user-access' | 'payment-links';

export default function AdminPaymentsPage() {
  const searchParams = useSearchParams();
  const activeTab = (searchParams.get('tab') ?? 'payments') as TabType;

  return (
    <PageLayout>
      <PageHeader
        title="Payments Hub"
        subtitle="Manage payments, user access, and payment links"
        icon="CreditCard"
        gradient="primary"
        centered
      />

      {activeTab === 'payments' && <PaymentsManagement />}
      {activeTab === 'user-access' && <UserAccessManagement />}
      {activeTab === 'payment-links' && <PaymentLinksManagement />}
    </PageLayout>
  );
}
