import type { Metadata } from 'next';

import { DeveloperPortalPage } from '@/components/admin/developer-portal';
import { PageLayout } from '@/components/shared';

export const metadata: Metadata = {
  title: 'Developer Portal | Admin',
  description: 'Manage user API keys and third-party integrations',
};

/**
 * Developer Portal Page
 * Uses unified page components for consistent design
 */
export default function DeveloperPortalRoute() {
  return (
    <PageLayout>
      <DeveloperPortalPage />
    </PageLayout>
  );
}
