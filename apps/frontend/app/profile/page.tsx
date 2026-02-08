import { Suspense } from 'react';
import { redirect } from 'next/navigation';
import { getServerSession } from '@/lib/session';
import { WalletProfileClient } from '@/components/profile/wallet-profile-client';
import { Loader2 } from 'lucide-react';

export const metadata = {
  title: 'Profile | EPSX',
  description: 'Manage your account settings, preferences, and data',
};

export default async function ProfilePage() {
  const session = await getServerSession();

  // Redirect if not authenticated
  if (!session?.isAuthenticated || !session.user) {
    redirect('/login');
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-orange-50 via-yellow-50 to-pink-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900">
      <div className="container mx-auto px-6 py-8">
        <div className="max-w-6xl mx-auto">
          {/* Header */}
          <div className="mb-8">
            <h1 className="text-3xl font-bold text-slate-900 dark:text-slate-100 mb-2">
              Profile & Settings
            </h1>
            <p className="text-slate-600 dark:text-slate-400">
              Manage your authentication, permissions, API keys, and account preferences
            </p>
          </div>

          {/* Profile Content */}
          <Suspense
            fallback={
              <div className="flex items-center justify-center py-12">
                <Loader2 className="h-8 w-8 animate-spin text-orange-500" />
                <span className="ml-2 text-slate-600 dark:text-slate-400">
                  Loading profile...
                </span>
              </div>
            }
          >
            <WalletProfileClient wallet={session.user} />
          </Suspense>
        </div>
      </div>
    </div>
  );
}