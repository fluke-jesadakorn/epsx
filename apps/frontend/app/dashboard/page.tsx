import { getSession } from '@/lib/auth';
import { DashboardClient } from '@/components/dashboard/DashboardClient';
import { redirect } from 'next/navigation';

// ISR configuration for dashboard - revalidate every 1 minute for dynamic user data
export const revalidate = 60;

export default async function DashboardPage() {
  // Get session data server-side using custom iron-session
  const session = await getSession();
  
  if (!session?.isLoggedIn || !session.user) {
    redirect('/login');
  }

  // Transform custom session data to the expected format
  const user = {
    user_id: session.user.firebase_uid || session.user.id || '',
    email: session.user.email || '',
    role: session.user.role || 'user',
    permissions: session.user.permissions || ['user:read'],
    subscription_tier: session.user.package_tier || 'FREE',
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
    subscription_tier: user.subscription_tier,
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