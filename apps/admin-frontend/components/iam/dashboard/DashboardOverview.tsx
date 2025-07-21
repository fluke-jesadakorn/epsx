'use client';

import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '../../ui/card';
import { Users, UserCheck, Shield, TrendingUp, TrendingDown } from 'lucide-react';
import { useIAMStats } from '../../../hooks/iam/useIAMStats';

interface StatCardProps {
  title: string;
  value: string | number;
  icon: React.ReactNode;
  trend?: {
    value: number;
    isPositive: boolean;
  };
}

const StatCard: React.FC<StatCardProps> = ({ title, value, icon, trend }) => (
  <Card>
    <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
      <CardTitle className="text-sm font-medium">{title}</CardTitle>
      {icon}
    </CardHeader>
    <CardContent>
      <div className="text-2xl font-bold">{value}</div>
      {trend && (
        <div className={`text-xs flex items-center mt-1 ${trend.isPositive ? 'text-green-600' : 'text-red-600'}`}>
          {trend.isPositive ? <TrendingUp className="w-3 h-3 mr-1" /> : <TrendingDown className="w-3 h-3 mr-1" />}
          {Math.abs(trend.value)}% from last month
        </div>
      )}
    </CardContent>
  </Card>
);

export const DashboardOverview: React.FC = () => {
  const { stats, loading } = useIAMStats();

  if (loading) {
    return (
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
        {[1, 2, 3].map(i => (
          <Card key={i} className="animate-pulse">
            <CardHeader>
              <div className="h-4 bg-gray-200 rounded w-3/4"></div>
            </CardHeader>
            <CardContent>
              <div className="h-8 bg-gray-200 rounded w-1/2"></div>
            </CardContent>
          </Card>
        ))}
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
      <StatCard
        title="Total Users"
        value={stats.totalUsers}
        icon={<Users className="h-4 w-4 text-gray-500" />}
        trend={stats.userGrowth}
      />
      <StatCard
        title="Active Subscriptions"
        value={stats.activeSubscriptions}
        icon={<UserCheck className="h-4 w-4 text-gray-500" />}
        trend={stats.subscriptionGrowth}
      />
      <StatCard
        title="Permission Templates"
        value={stats.permissionTemplates}
        icon={<Shield className="h-4 w-4 text-gray-500" />}
        trend={stats.templateGrowth}
      />
    </div>
  );
};
