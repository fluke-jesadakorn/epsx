import { Key } from 'lucide-react';
import { Metadata } from 'next';

import { ApiKeyRequestForm } from '@/components/public/ApiKeyRequestForm';
import { PageHeader, PageLayout } from '@/components/shared';

export const metadata: Metadata = {
  title: 'Request API Access | EPSX',
  description: 'Request access to the EPSX API platform for financial data integration',
};

/**
 * Request Access Page
 * Uses unified page components for consistent design
 */
export default function RequestAccessPage() {
  return (
    <PageLayout maxWidth="5xl">
      <PageHeader
        title="Request API Access"
        subtitle="Submit your request to access the EPSX API platform for financial data integration"
        icon={Key}
        gradient="info"
        centered
      />

      <ApiKeyRequestForm />
    </PageLayout>
  );
}
