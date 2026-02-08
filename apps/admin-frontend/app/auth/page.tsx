/**
 * AUTH PAGE - SERVER COMPONENT
 * Verifies session server-side first, then passes to client component
 * This prevents showing "Admin Access Granted" after logout when client state is stale
 */

import { PageSkeleton } from '@/components/shared';
import { logger } from '@/shared/utils/logger';
import { Suspense } from 'react';
import AuthPageClient from './auth-page-client';
import { verifySessionAction } from './actions';

export default async function AuthPage() {
  // Server-side session verification FIRST
  // This is authoritative - if server says no session, don't trust client state
  const session = await verifySessionAction();
  const serverHasSession = session.valid;

  logger.info('[AUTH] Server Page: session verification result', { serverHasSession });

  return (
    <Suspense fallback={<PageSkeleton showHeader={false} stats={0} rows={0} />}>
      <AuthPageClient serverHasSession={serverHasSession} />
    </Suspense>
  );
}
