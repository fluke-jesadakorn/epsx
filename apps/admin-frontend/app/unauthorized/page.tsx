'use client';

import { AccessDeniedContent } from '@/components/shared';

/**
 * Unauthorized Page
 * Uses unified status page component for consistent design
 */
export default function UnauthorizedPage() {
  return (
    <AccessDeniedContent
      title="Access Denied"
      reason="You don't have permission to access the admin panel. Please contact your administrator if you believe this is an error."
    />
  );
}
