import { APIKeyManager } from '@/components/developer/APIKeyManager';
import { DeveloperStatsCards } from '@/components/developer/DeveloperStatsCards';
import { getCurrentUser } from '@/lib/server-actions';

export const dynamic = 'force-dynamic';

export default async function DeveloperPage() {
  const user = await getCurrentUser();

  if (!user) {
    return null; // Layout handles auth guard
  }

  return (
    <div className="space-y-8">
      <DeveloperStatsCards currentUser={user} />
      <div className="relative overflow-hidden rounded-2xl bg-gradient-to-br from-white/80 to-white/40 dark:from-gray-800/80 dark:to-gray-800/40 backdrop-blur-xl border border-white/20 dark:border-gray-700/50 shadow-xl p-6">
        <APIKeyManager currentUser={user} />
      </div>
    </div>
  );
}