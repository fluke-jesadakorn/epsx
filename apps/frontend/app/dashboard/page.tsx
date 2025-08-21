import { getSessionFromJWT } from '@/lib/server/jwt';
import { DashboardClient } from '@/components/dashboard/DashboardClient';
import { redirect } from 'next/navigation';

// Force dynamic rendering for pages that use authentication
export const dynamic = 'force-dynamic';

export default async function DashboardPage() {
  // Get session data server-side using JWT
  const session = await getSessionFromJWT();
  
  if (!session?.isAuthenticated || !session.user) {
    const { redirectToBackendLogin } = await import('@/lib/server/auth');
    redirectToBackendLogin('/dashboard');
  }

  // Transform custom session data to the expected format
  const user = {
    user_id: session.user.firebase_uid || session.user.id || '',
    email: session.user.email || '',
    role: session.user.role || 'user',
    permissions: session.user.permissions || ['user:read'],
    package_tier: session.user.package_tier || 'FREE',
    admin_modules: [], // Frontend users don't have admin modules
    name: session.user.name || session.user.email || '',
  };

  const permissions = {
    role: user.role,
    permissions: user.permissions,
  };

  // Mock dashboard data for now - you can replace this with API calls using the session
  const dashboardData = {
    success: true,
    data: {
      stats: {
        totalViews: 0,
        totalUsers: 1,
        revenue: 0,
      },
      recentActivity: [],
    },
  };

  console.log('Dashboard: User session loaded for', user.email, {
    package_tier: user.package_tier,
    permissions_count: user.permissions?.length || 0,
    role: user.role,
  });

  return (
    <DashboardClient 
      user={user}
      permissions={permissions}
      dashboardData={dashboardData}
    />
  );
}