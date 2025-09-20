import { Suspense } from 'react';
import { redirect } from 'next/navigation';
import { getServerSessionAdmin } from '@/lib/session-admin';
import { AdminProfileClient } from '@/components/profile/AdminProfileClient';
import { Loader2 } from 'lucide-react';

export const metadata = {
  title: 'Admin Profile | EPSX Admin',
  description: 'Manage your admin account settings and permissions',
};

export default async function AdminProfilePage() {
  const session = await getServerSessionAdmin();

  // Redirect if not authenticated as admin
  if (!session?.isAuthenticated || !session.user) {
    redirect('/login');
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900">
      <div className="container mx-auto px-6 py-8">
        <div className="max-w-6xl mx-auto">
          {/* Header */}
          <div className="mb-8">
            <h1 className="text-3xl font-bold text-slate-900 dark:text-slate-100 mb-2">
              Admin Profile
            </h1>
            <p className="text-slate-600 dark:text-slate-400">
              Manage your administrative account and permissions
            </p>
          </div>

          {/* Profile Content */}
          <Suspense
            fallback={
              <div className="flex items-center justify-center py-12">
                <Loader2 className="h-8 w-8 animate-spin text-orange-500" />
                <span className="ml-2 text-slate-600 dark:text-slate-400">
                  Loading admin profile...
                </span>
              </div>
            }
          >
            <AdminProfileClient user={session.user} />
          </Suspense>
        </div>
      </div>
    </div>
  );
}