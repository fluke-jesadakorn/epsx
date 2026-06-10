'use client';

import { AccessDeniedContent } from '@/components/shared';
import { useSearchParams } from 'next/navigation';
import { Suspense } from 'react';

function AccessDeniedPageContent() {
  const searchParams = useSearchParams();

  const route = searchParams.get('route') ?? undefined;
  const reason = searchParams.get('reason') ?? undefined;
  const context = searchParams.get('context') ?? undefined;
  const permission = searchParams.get('permission') ?? undefined;
  const detail = searchParams.get('detail') ?? undefined;

  return (
    <AccessDeniedContent
      reason={reason !== undefined && reason !== '' ? decodeURIComponent(reason) : undefined}
      route={route}
      context={context}
      permission={permission}
      detail={detail !== undefined && detail !== '' ? decodeURIComponent(detail) : undefined}
    />
  );
}

/**
 * Access Denied Page
 * Uses unified status page component for consistent design
 */
export default function AccessDeniedPage() {
  return (
    <Suspense fallback={<AccessDeniedContent />}>
      <AccessDeniedPageContent />
    </Suspense>
  );
}
