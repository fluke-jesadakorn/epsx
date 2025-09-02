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

  // Transform session data to structured format with null checks
  const user = {
    id: session.user?.id || '',
    email: session.user?.email || '',
    name: session.user?.name || session.user?.email?.split('@')[0] || 'User',
    permissions: session.user?.permissions || ['epsx:analytics:view'],
    package_tier: session.user?.package_tier || 'FREE',
    firebase_uid: session.user?.firebase_uid,
    
    // Cross-platform fields
    platforms: session.user?.platforms || ['epsx'],
    primary_platform: session.user?.primary_platform || 'epsx',
    platform_context: session.user?.platform_context,
  };

  const permissions = {
    role: 'user', // Add required role property
    permissions: user.permissions,
    platforms: user.platforms,
    platform_context: user.platform_context,
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
    permissions: user.permissions,
  });

  return (
    <DashboardClient 
      user={user}
      permissions={permissions}
      dashboardData={dashboardData}
    />
  );
}