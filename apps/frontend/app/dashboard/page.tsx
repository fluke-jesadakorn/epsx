import { getSessionFromJWT } from '@/lib/server/jwt';
import { DashboardClient } from '@/components/dashboard/DashboardClient';
import { ProgressiveAuthGate } from '@/components/auth/ProgressiveAuthGate';
import { AuthLevel } from '@/types/progressive-auth';

// Force dynamic rendering for pages that use authentication
export const dynamic = 'force-dynamic';

export default async function DashboardPage() {
  // Get session data server-side using JWT (but don't redirect on failure)
  const session = await getSessionFromJWT();
  
  // Transform session data to structured format with null checks
  const user = session?.isAuthenticated && session.user ? {
    id: session.user.id || '',
    email: session.user.email || '',
    name: session.user.name || session.user.email?.split('@')[0] || 'User',
    permissions: session.user.permissions || ['epsx:analytics:view'],
    package_tier: session.user.package_tier || 'FREE',
    firebase_uid: session.user.firebase_uid,
    
    // Cross-platform fields
    platforms: session.user.platforms || ['epsx'],
    primary_platform: session.user.primary_platform || 'epsx',
    platform_context: session.user.platform_context,
  } : null;

  const permissions = user ? {
    role: 'user', // Add required role property
    permissions: user.permissions,
    platforms: user.platforms,
    platform_context: user.platform_context,
  } : null;

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

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-900 dark:to-slate-800">
      <div className="container mx-auto px-4 py-8">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-slate-900 dark:text-slate-100">
            Personal Dashboard
          </h1>
          <p className="text-slate-600 dark:text-slate-400 mt-2">
            Your personalized trading analytics and portfolio overview
          </p>
        </div>

        <ProgressiveAuthGate
          requiredLevel={AuthLevel.AUTHENTICATED}
          actionName="access your personal dashboard"
          authMessage="Sign in with your wallet to view your personalized dashboard with portfolio data, settings, and premium features"
        >
          {user && permissions ? (
            <DashboardClient 
              user={user}
              permissions={permissions}
              dashboardData={dashboardData}
            />
          ) : (
            <div className="text-center p-8">
              <p className="text-slate-600 dark:text-slate-400">
                Loading your dashboard...
              </p>
            </div>
          )}
        </ProgressiveAuthGate>
      </div>
    </div>
  );
}