import { getRecentWalletsAction } from '@/app/analytics/actions';

import type { RecentWalletsData } from '@/hooks/use-analytics-data';
import { logger } from '@/lib/logger';
import DashboardClient from './dashboard-client';

export const dynamic = 'force-dynamic';

export default async function AdminDashboardPage() {

  let initialRecentWallets: RecentWalletsData | undefined = undefined;

  try {
    // Fetch recent wallets on server
    initialRecentWallets = await getRecentWalletsAction(10, 30);
  } catch (err: unknown) {
    if (err instanceof Error && (err as { digest?: string }).digest?.startsWith('NEXT_REDIRECT') === true) {
      throw err;
    }
    logger.error('Failed to pre-fetch recent wallets:', err instanceof Error ? err.message : String(err));
  }

  return <DashboardClient initialRecentWallets={initialRecentWallets} />;
}
