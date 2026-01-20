import { notFound } from 'next/navigation';
import { Suspense } from 'react';

import { PageLayout, PageSkeleton } from '@/components/shared';
import { SubscriptionManagement } from '@/components/subscriptions/SubscriptionManagement';
import { UnifiedAuth } from '@/lib/auth/auth';

export const dynamic = 'force-dynamic';

function SubscriptionsHubSkeleton() {
  return <PageSkeleton showHeader stats={4} rows={8} />;
}

async function SubscriptionsDataWrapper() {
  const session = await UnifiedAuth.getSession();

  if (!session?.user) {
    notFound();
  }

  // NOTE: Permission enforcement moved to backend
  // If user lacks permission, API calls will return 403 and show Access Denied UI

  return <SubscriptionManagement currentUser={session.user} />;
}

/**
 * Subscriptions Page
 * Uses unified page components for consistent design
 */
export default function AdminSubscriptionsPage() {
  return (
    <PageLayout>
      <Suspense fallback={<SubscriptionsHubSkeleton />}>
        <SubscriptionsDataWrapper />
      </Suspense>
    </PageLayout>
  );
}
