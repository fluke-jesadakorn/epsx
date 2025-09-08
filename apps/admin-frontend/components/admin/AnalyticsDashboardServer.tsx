/**
 * Server-rendered Analytics Dashboard
 * Removes client-side data fetching and uses server component pattern
 */

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Activity,
  BarChart3,
  TrendingUp,
  Users,
  Download,
  Filter,
} from 'lucide-react';
import React from 'react';

interface AnalyticsDashboardServerProps {
  analytics: any;
  systemMetrics: any;
  revenue?: any;
  realtime?: any;
}

function MetricCard({ title, value, change, icon: Icon, color }: {
  title: string;
  value: string | number;
  change?: string;
  icon: React.ElementType;
  color: string;
}) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-sm font-medium">{title}</CardTitle>
        <Icon className={`h-4 w-4 ${color}`} />
      </CardHeader>
      <CardContent>
        <div className="text-2xl font-bold">
          {typeof value === 'number' ? value.toLocaleString() : value}
        </div>
        {change && (
          <p className="text-xs text-muted-foreground">{change}</p>
        )}
      </CardContent>
    </Card>
  );
}

function AnalyticsOverview({ analytics, systemMetrics }: { analytics: any; systemMetrics: any }) {
  return (
    <div className="space-y-6">
      {/* Key Metrics Grid */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <MetricCard
          title="Total Users"
          value={analytics?.total_users || 0}
          change="+20.1% from last month"
          icon={Users}
          color="text-blue-600"
        />
        <MetricCard
          title="Active Sessions"
          value={systemMetrics?.active_users || 0}
          change="+15% from yesterday"
          icon={Activity}
          color="text-green-600"
        />
        <MetricCard
          title="API Response Time"
          value={`${systemMetrics?.api_response_time || 0}ms`}
          change="-2ms from last hour"
          icon={TrendingUp}
          color="text-yellow-600"
        />
        <MetricCard
          title="Memory Usage"
          value={`${systemMetrics?.memory_usage || 0}%`}
          change="Within normal range"
          icon={BarChart3}
          color="text-purple-600"
        />
      </div>

      {/* Analytics Charts Placeholder */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-7">
        <Card className="col-span-4">
          <CardHeader>
            <CardTitle>User Growth</CardTitle>
          </CardHeader>
          <CardContent className="pl-2">
            <div className="h-[200px] bg-muted rounded-lg flex items-center justify-center">
              <p className="text-muted-foreground">Chart component would go here</p>
            </div>
          </CardContent>
        </Card>
        <Card className="col-span-3">
          <CardHeader>
            <CardTitle>Recent Activity</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {[
                { action: 'User registered', time: '2 minutes ago' },
                { action: 'API key generated', time: '5 minutes ago' },
                { action: 'Permission granted', time: '8 minutes ago' },
                { action: 'Export completed', time: '12 minutes ago' },
                { action: 'System backup', time: '30 minutes ago' },
              ].map((activity, i) => (
                <div key={i} className="flex items-center">
                  <div className="ml-4 space-y-1">
                    <p className="text-sm font-medium leading-none">
                      {activity.action}
                    </p>
                    <p className="text-sm text-muted-foreground">
                      {activity.time}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

function UserAnalytics({ analytics }: { analytics: any }) {
  return (
    <div className="space-y-6">
      <div className="grid gap-4 md:grid-cols-3">
        <MetricCard
          title="New Users"
          value={analytics?.recent_users_30_days || 0}
          change="This month"
          icon={Users}
          color="text-green-600"
        />
        <MetricCard
          title="Active Users"
          value={analytics?.active_users || 0}
          change="Currently online"
          icon={Activity}
          color="text-blue-600"
        />
        <MetricCard
          title="Total Users"
          value={analytics?.total_users || 0}
          change="All time"
          icon={TrendingUp}
          color="text-purple-600"
        />
      </div>

      <Card>
        <CardHeader>
          <CardTitle>User Distribution by Tier</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            {analytics?.by_tier && Object.entries(analytics.by_tier).map(([tier, count]: [string, any]) => (
              <div key={tier} className="flex items-center justify-between">
                <span className="capitalize">{tier}</span>
                <div className="flex items-center gap-2">
                  <div className="w-24 bg-muted rounded-full h-2">
                    <div 
                      className="bg-primary h-2 rounded-full" 
                      style={{ 
                        width: `${Math.min((count / analytics.total_users) * 100, 100)}%` 
                      }}
                    />
                  </div>
                  <span className="text-sm text-muted-foreground">{count}</span>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

function PerformanceMetrics({ systemMetrics }: { systemMetrics: any }) {
  return (
    <div className="space-y-6">
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <MetricCard
          title="API Response"
          value={`${systemMetrics?.api_response_time || 0}ms`}
          change="Average response time"
          icon={TrendingUp}
          color="text-green-600"
        />
        <MetricCard
          title="Memory Usage"
          value={`${systemMetrics?.memory_usage || 0}%`}
          change="Current usage"
          icon={BarChart3}
          color="text-yellow-600"
        />
        <MetricCard
          title="Database"
          value={`${systemMetrics?.database_query_time || 0}ms`}
          change="Query time"
          icon={Activity}
          color="text-blue-600"
        />
        <MetricCard
          title="Peak Users"
          value={systemMetrics?.peak_users_today || 0}
          change="Today's peak"
          icon={Users}
          color="text-purple-600"
        />
      </div>

      <Card>
        <CardHeader>
          <CardTitle>System Health</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {[
              { service: 'API Server', status: 'Healthy', uptime: '99.9%' },
              { service: 'Database', status: 'Healthy', uptime: '99.8%' },
              { service: 'Cache', status: 'Healthy', uptime: '100%' },
              { service: 'Analytics', status: 'Healthy', uptime: '99.7%' },
            ].map((service, i) => (
              <div key={i} className="flex items-center justify-between">
                <span className="font-medium">{service.service}</span>
                <div className="flex items-center gap-2">
                  <div className="w-2 h-2 bg-green-500 rounded-full" />
                  <span className="text-sm">{service.status}</span>
                  <span className="text-sm text-muted-foreground">({service.uptime})</span>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// Navigation component for switching views
function AnalyticsNavigation({ activeView, onViewChange }: {
  activeView: string;
  onViewChange: (view: string) => void;
}) {
  const views = [
    { id: 'overview', label: 'Overview', icon: BarChart3 },
    { id: 'users', label: 'Users', icon: Users },
    { id: 'performance', label: 'Performance', icon: TrendingUp },
  ];

  return (
    <div className="flex space-x-1 mb-6">
      {views.map((view) => (
        <button
          key={view.id}
          onClick={() => onViewChange(view.id)}
          className={`flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
            activeView === view.id
              ? 'bg-primary text-primary-foreground'
              : 'text-muted-foreground hover:text-foreground hover:bg-muted'
          }`}
        >
          <view.icon className="h-4 w-4" />
          {view.label}
        </button>
      ))}
    </div>
  );
}

export function AnalyticsDashboardServer({ 
  analytics, 
  systemMetrics, 
  revenue, 
  realtime 
}: AnalyticsDashboardServerProps) {
  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-3xl font-bold tracking-tight">Analytics</h2>
          <p className="text-muted-foreground">
            System performance and user insights
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button className="inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 border border-input bg-background hover:bg-accent hover:text-accent-foreground h-10 px-4 py-2">
            <Filter className="mr-2 h-4 w-4" />
            Filter
          </button>
          <button className="inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-10 px-4 py-2">
            <Download className="mr-2 h-4 w-4" />
            Export
          </button>
        </div>
      </div>

      {/* Always show overview - server-rendered content */}
      <AnalyticsOverview analytics={analytics} systemMetrics={systemMetrics} />

      {/* Additional sections can be shown based on URL params or other server-side logic */}
      <div className="grid gap-6 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>User Analytics Summary</CardTitle>
          </CardHeader>
          <CardContent>
            <UserAnalytics analytics={analytics} />
          </CardContent>
        </Card>
        
        <Card>
          <CardHeader>
            <CardTitle>Performance Summary</CardTitle>
          </CardHeader>
          <CardContent>
            <PerformanceMetrics systemMetrics={systemMetrics} />
          </CardContent>
        </Card>
      </div>
    </div>
  );
}