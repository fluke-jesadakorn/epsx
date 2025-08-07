/**
 * Dashboard Server Component
 * Replaces client-side AdminDashboard with server-rendered version
 */

import { Shield, Users, CheckCircle, Crown, Activity, Zap, Database, TrendingUp } from 'lucide-react'
import type { DashboardStats, RecentUser, SystemMetrics } from '@/lib/data/dashboard'
import type { EnhancedAuthUser } from '@/lib/auth/server-auth-enhanced'
import { StatsCard } from '@/components/ui/StatsCard'
import { UserManagementOnly, AnalyticsOnly } from '@/components/auth/RoleGuard'
import { RecentActivity } from './RecentActivity'
import { SystemHealthCard } from './SystemHealthCard'
import Link from 'next/link'

interface DashboardServerProps {
  user: EnhancedAuthUser
  stats: DashboardStats
  recentUsers: RecentUser[]
  systemMetrics: SystemMetrics
}

export function DashboardServer({ user, stats, recentUsers, systemMetrics }: DashboardServerProps) {
  const statCards = [
    {
      title: 'Total Users',
      value: stats.totalUsers,
      description: 'Registered users',
      icon: Users,
      gradient: 'from-blue-500 to-purple-500',
      textColor: 'text-blue-500',
      change: `+${stats.newUsersThisWeek} this week`
    },
    {
      title: 'Email Verified',
      value: `${stats.verificationRate}%`,
      description: `${stats.verifiedUsers} verified users`,
      icon: CheckCircle,
      gradient: 'from-green-500 to-emerald-500',
      textColor: 'text-green-500',
      change: 'Verification rate'
    },
    {
      title: 'Active Today',
      value: stats.activeUsers,
      description: 'Active users today',
      icon: Activity,
      gradient: 'from-orange-500 to-red-500',
      textColor: 'text-orange-500',
      change: `${stats.totalSessions} sessions`
    },
    {
      title: 'Admin Users',
      value: stats.adminUsers,
      description: 'Administrative accounts',
      icon: Crown,
      gradient: 'from-purple-500 to-pink-500',
      textColor: 'text-purple-500',
      change: 'System administrators'
    },
  ]

  const quickActions = [
    {
      title: 'User Management',
      description: 'Manage users, roles, and permissions',
      href: '/users',
      icon: Users,
      color: 'blue'
    },
    {
      title: 'Analytics',
      description: 'View system analytics and reports',
      href: '/analytics',
      icon: TrendingUp,
      color: 'green'
    },
    {
      title: 'Module Management',
      description: 'Configure modules and access',
      href: '/modules',
      icon: Database,
      color: 'purple'
    },
    {
      title: 'System Settings',
      description: 'Configure system settings',
      href: '/settings',
      icon: Zap,
      color: 'orange'
    }
  ]

  return (
    <div className="space-y-8 p-6">
      {/* Enhanced Header */}
      <div className="flex items-center justify-between pancake-card pancake-card-hover p-6">
        <div className="flex items-center gap-4">
          <div className="p-3 rounded-full bg-gradient-to-r from-orange-500 to-yellow-500">
            <Shield className="h-8 w-8 text-white" />
          </div>
          <div>
            <h1 className="text-3xl font-bold bg-gradient-to-r from-orange-500 to-yellow-500 bg-clip-text text-transparent">
              Admin Dashboard
            </h1>
            <p className="text-muted-foreground mt-1">
              Welcome back, {user.displayName || user.email}
            </p>
          </div>
        </div>
        
        {/* System Health Indicator */}
        <div className="flex items-center gap-2">
          <div className={`w-3 h-3 rounded-full ${
            stats.systemHealth === 'good' ? 'bg-green-500' : 
            stats.systemHealth === 'warning' ? 'bg-yellow-500' : 
            'bg-red-500'
          }`} />
          <span className="text-sm text-muted-foreground">
            System {stats.systemHealth}
          </span>
        </div>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {statCards.map((card, index) => (
          <StatsCard
            key={index}
            title={card.title}
            value={card.value}
            description={card.description}
            icon={card.icon}
            gradient={card.gradient}
            textColor={card.textColor}
            change={card.change}
          />
        ))}
      </div>

      {/* Quick Actions - Role-based */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="pancake-card pancake-card-hover p-6">
          <h2 className="text-xl font-semibold mb-4 flex items-center gap-2">
            <Zap className="h-5 w-5" />
            Quick Actions
          </h2>
          <div className="grid grid-cols-1 gap-3">
            {quickActions.map((action, index) => (
              <Link
                key={index}
                href={action.href}
                className="flex items-center gap-3 p-3 rounded-lg hover:bg-muted/50 transition-colors"
              >
                <div className={`p-2 rounded-lg bg-${action.color}-500/10`}>
                  <action.icon className={`h-4 w-4 text-${action.color}-500`} />
                </div>
                <div>
                  <div className="font-medium">{action.title}</div>
                  <div className="text-sm text-muted-foreground">{action.description}</div>
                </div>
              </Link>
            ))}
          </div>
        </div>

        {/* System Metrics - Admin only */}
        <UserManagementOnly>
          <SystemHealthCard metrics={systemMetrics} />
        </UserManagementOnly>
      </div>

      {/* Recent Activity */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Recent Users */}
        <UserManagementOnly>
          <div className="pancake-card pancake-card-hover p-6">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-xl font-semibold flex items-center gap-2">
                <Users className="h-5 w-5" />
                Recent Users
              </h2>
              <Link 
                href="/users" 
                className="text-sm text-blue-500 hover:text-blue-600"
              >
                View all
              </Link>
            </div>
            <div className="space-y-3">
              {recentUsers.map((user) => (
                <div key={user.uid} className="flex items-center justify-between p-3 rounded-lg bg-muted/30">
                  <div className="flex items-center gap-3">
                    <div className={`w-8 h-8 rounded-full flex items-center justify-center text-xs font-medium ${
                      user.emailVerified ? 'bg-green-100 text-green-800' : 'bg-yellow-100 text-yellow-800'
                    }`}>
                      {user.displayName ? user.displayName.charAt(0).toUpperCase() : user.email.charAt(0).toUpperCase()}
                    </div>
                    <div>
                      <div className="font-medium text-sm">{user.displayName || user.email}</div>
                      <div className="text-xs text-muted-foreground">{user.role}</div>
                    </div>
                  </div>
                  <div className="text-xs text-muted-foreground">
                    {new Date(user.createdAt).toLocaleDateString()}
                  </div>
                </div>
              ))}
            </div>
          </div>
        </UserManagementOnly>

        {/* Recent Activity */}
        <AnalyticsOnly>
          <RecentActivity />
        </AnalyticsOnly>
      </div>
    </div>
  )
}