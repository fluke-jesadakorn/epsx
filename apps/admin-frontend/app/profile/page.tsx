import { redirect } from 'next/navigation';
import { Suspense } from 'react';

import { AdminProfileClient } from '@/components/profile/admin-profile-client';
import { PageLayout } from '@/components/shared';
import { getServerSessionAdmin } from '@/lib/session';

// Force dynamic rendering since we use cookies for auth
export const dynamic = 'force-dynamic';

export const metadata = {
  title: 'Admin Profile | EPSX Admin',
  description: 'Manage your admin account settings and permissions',
};

/**
 * Admin Profile Page
 * Uses unified page components for consistent design
 */
export default async function AdminProfilePage() {
  const session = await getServerSessionAdmin();

  // Redirect if not authenticated as admin
  if (!session?.isAuthenticated ?? !session.user) {
    redirect('/auth');
  }

  return (
    <PageLayout maxWidth="6xl">
      <Suspense fallback={<ProfileLoadingFallback />}>
        <AdminProfileClient user={session.user} />
      </Suspense>
    </PageLayout>
  );
}

function ProfileLoadingFallback() {
  return (
    <div className="animate-pulse space-y-6">
      <div className="bg-card rounded-2xl p-6 border border-border/30">
        <div className="flex items-center gap-4 mb-6">
          <div className="w-20 h-20 bg-muted rounded-full" />
          <div className="space-y-2">
            <div className="h-6 bg-muted rounded-lg w-48" />
            <div className="h-4 bg-muted/60 rounded-lg w-64" />
          </div>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className="h-16 bg-muted/30 rounded-xl" />
          ))}
        </div>
      </div>
    </div>
  );
}
