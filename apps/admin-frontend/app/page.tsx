import { getRecentWalletsAction } from '@/app/analytics/actions';
import DashboardClient from './DashboardClient';

export const dynamic = 'force-dynamic';

export default async function AdminDashboardPage() {
  let initialRecentWallets = null;

  try {
    // Fetch recent wallets on server
    initialRecentWallets = await getRecentWalletsAction(10, 30);
  } catch (error: any) {
    // Re-throw Next.js redirect errors so they are handled correctly
    if (error?.digest?.startsWith('NEXT_REDIRECT') || error?.message === 'NEXT_REDIRECT') {
      throw error;
    }
    console.error('Failed to pre-fetch recent wallets:', error);
    // Be silent about error, client will switch to loading/error state if initial data missing? 
    // Actually our Client Component sets loading=true if !initialData, so client fetch will retry.
  }

  return <DashboardClient initialRecentWallets={initialRecentWallets} />;
}
