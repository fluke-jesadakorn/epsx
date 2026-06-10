import { DeveloperStatsCards } from '@/components/developer/developer-stats-cards';
import { getCurrentUser } from '@/lib/server/actions';
import nextDynamic from 'next/dynamic';

const APIKeyManager = nextDynamic(
  () => import('@/components/developer/api-key-manager').then(m => ({ default: m.APIKeyManager })),
  { loading: () => <div className="animate-pulse h-64 rounded-2xl bg-muted" /> }
);

export const dynamic = 'force-dynamic';

export default async function DeveloperPage() {
  const user = await getCurrentUser();

  if (!user) {
    return null; // Layout handles auth guard
  }

  return (
    <div className="space-y-8">
      <DeveloperStatsCards currentUser={user} />
      <div className="rounded-2xl border border-border/20 bg-card shadow-xl p-6">
        <APIKeyManager currentUser={user} />
      </div>
    </div>
  );
}
