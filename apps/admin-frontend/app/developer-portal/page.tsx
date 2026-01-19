import { Metadata } from 'next';

import { DeveloperPortalPage } from '@/components/admin/developer-portal';

export const metadata: Metadata = {
  title: 'Developer Portal | Admin',
  description: 'Manage user API keys and third-party integrations',
};

/**
 * Developer Portal Page - Manages user API keys
 */
export default function DeveloperPortalRoute() {
  return (
    <div className="min-h-screen bg-background">
      <DeveloperPortalPage />
    </div>
  );
}

