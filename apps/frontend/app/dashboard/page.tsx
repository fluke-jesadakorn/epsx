import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import {
  BarChart3,
  Lock,
  Settings,
  Shield,
  Sparkles,
  TrendingUp,
  User,
} from 'lucide-react';
import Link from 'next/link';
import { getCurrentUser, getDashboardData, getUserFeatures } from '@epsx/server-actions';
import { DashboardClient } from '@/components/dashboard/DashboardClient';

// ISR configuration for dashboard - revalidate every 1 minute for dynamic user data
export const revalidate = 60;

export default async function DashboardPage() {
  // Fetch all data server-side
  const [userResult, dashboardResult, featuresResult] = await Promise.allSettled([
    getCurrentUser(),
    getCurrentUser().then(user => user ? getDashboardData(user.user_id || user.id) : null),
    getUserFeatures()
  ]);

  const user = userResult.status === 'fulfilled' ? userResult.value : null;
  const dashboardData = dashboardResult.status === 'fulfilled' ? dashboardResult.value : null;
  const features = featuresResult.status === 'fulfilled' ? featuresResult.value : [];
  
  // Extract permissions from user data
  const permissions = {
    role: user?.role || 'user',
    permissions: user?.permissions || features || []
  };

  return (
    <DashboardClient 
      user={user}
      permissions={permissions}
      dashboardData={dashboardData}
    />
  );
}
