/**
 * AUTH PAGE - SERVER COMPONENT
 * Verifies session server-side first, then passes to client component
 * This prevents showing "Admin Access Granted" after logout when client state is stale
 */

import { logger } from '@/shared/utils/logger';
import { Suspense } from 'react';
import AuthPageClient from './auth-page-client';
import { verifySessionAction } from './actions';

export default async function AuthPage() {
  const session = await verifySessionAction();
  const serverHasSession = session.valid;

  logger.info('[AUTH] Server Page: session verification result', { serverHasSession });

  return (
    <Suspense fallback={
      <div className="fixed inset-0 z-50 flex items-center justify-center bg-[#0a0118]">
        <div className="h-8 w-8 animate-spin rounded-full border-t-2 border-purple-500" />
      </div>
    }>
      <AuthPageClient serverHasSession={serverHasSession} />
    </Suspense>
  );
}
