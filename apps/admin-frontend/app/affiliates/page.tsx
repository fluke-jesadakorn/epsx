import { notFound } from 'next/navigation';
import { Suspense } from 'react';

import { AffiliateManagement } from '@/components/affiliates/affiliate-management';
import { PageLayout, PageSkeleton } from '@/components/shared';
import { UnifiedAuth } from '@/lib/auth/auth';

export const dynamic = 'force-dynamic';

async function AffiliatesDataWrapper(): Promise<React.JSX.Element> {
  const session = await UnifiedAuth.getSession();

  if (!session.user) {
    notFound();
  }

  // NOTE: Permission enforcement moved to backend
  // If user lacks permission, API calls will return 403 and show Access Denied UI

  return <AffiliateManagement affiliates={[]} currentUser={session.user} />;
}

/**
 * Affiliates Page
 * Uses unified page components for consistent design
 */
export default function AdminAffiliatesPage(): React.JSX.Element {
  return (
    <PageLayout>
      <Suspense fallback={<PageSkeleton stats={4} rows={6} />}>
        <AffiliatesDataWrapper />
      </Suspense>
    </PageLayout>
  );
}
