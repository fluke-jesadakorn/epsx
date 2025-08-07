import { Metadata } from 'next';
import { DeveloperPortal } from '@/components/admin/DeveloperPortal';

export const metadata: Metadata = {
  title: 'Developer Portal | Admin',
  description: 'Manage API keys and third-party integrations',
};

export default function DeveloperPortalPage() {
  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <DeveloperPortal />
    </div>
  );
}
