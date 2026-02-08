import type { Metadata } from 'next';

import { ApiDocumentation } from '@/components/public/api-documentation';

export const metadata: Metadata = {
  title: 'API Documentation | EPSX',
  description: 'Complete guide to integrating with the EPSX module-based API platform',
};

/**
 *
 */
export default function ApiDocumentationPage() {
  return (
    <div className="min-h-screen bg-background py-8">
      <ApiDocumentation />
    </div>
  );
}
