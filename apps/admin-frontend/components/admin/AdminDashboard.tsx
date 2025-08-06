'use client';

import { useAdminAuth } from '@/auth/ctx';
import { Shield, Users, CheckCircle, AlertTriangle, UserX, Crown } from 'lucide-react';

interface UserStats {
  totalUsers: number;
  verifiedUsers: number;
  disabledUsers: number;
  adminUsers: number;
  verificationRate: number;
}

interface AdminUser {
  uid: string;
  email: string;
  displayName?: string;
  emailVerified: boolean;
  disabled: boolean;
  customClaims?: {
    role?: string;
  };
  metadata?: {
    creationTime?: string;
  };
}

interface AdminDashboardProps {
  initialStats: UserStats | null;
  initialUsers: AdminUser[];
}

export function AdminDashboard({ initialStats, initialUsers }: AdminDashboardProps) {
  const { user } = useAdminAuth();
  const stats = initialStats;
  const recentUsers = initialUsers;

  // Handle case where server-side data fetch failed
  if (!stats) {
    return (
      <div className="pancake-card pancake-card-hover p-6">
        <div className="flex items-center gap-2 mb-4" style={{ color: 'hsl(var(--pancake-error))' }}>
          <AlertTriangle className="h-6 w-6" />
          <span className="font-semibold">Failed to load dashboard data</span>
        </div>
        <p className="text-muted-foreground">Please refresh the page to try again.</p>
      </div>
    );
  }

  const statCards = [
    {
      title: 'Total Users',
      value: stats?.totalUsers || 0,
      description: 'Registered users',
      icon: Users,
      gradient: 'from-blue-500 to-purple-500',
      textColor: 'text-blue-500',
    },
    {
      title: 'Email Verified',
      value: `${stats?.verificationRate || 0}%`,
      description: `${stats?.verifiedUsers || 0} verified users`,
      icon: CheckCircle,
      gradient: 'from-green-500 to-emerald-500',
      textColor: 'text-green-500',
    },
    {
      title: 'Disabled Users',
      value: stats?.disabledUsers || 0,
      description: 'Disabled accounts',
      icon: UserX,
      gradient: 'from-red-500 to-red-600',
      textColor: 'text-red-500',
    },
    {
      title: 'Admin Users',
      value: stats?.adminUsers || 0,
      description: 'Administrative accounts',
      icon: Crown,
      gradient: 'from-orange-500 to-yellow-500',
      textColor: 'text-orange-500',
    },
  ];

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
              Welcome back, {user?.displayName || user?.email}
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2 px-4 py-2 rounded-full bg-gradient-to-r from-blue-500/10 to-purple-500/10 text-blue-500 font-medium">
          <Shield className="h-5 w-5" />
          <span>Admin Panel</span>
        </div>
      </div>

      {/* Enhanced Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {statCards.map((stat, index) => {
          const Icon = stat.icon;
          return (
            <div
              key={index}
              className="pancake-card pancake-card-hover p-6 relative overflow-hidden group"
            >
              <div className="absolute top-0 right-0 w-20 h-20 -mr-8 -mt-8 rounded-full bg-gradient-to-br from-white/5 to-white/10 group-hover:scale-110 transition-transform duration-500"></div>
              <div className="relative">
                <div className="flex items-center justify-between mb-4">
                  <div className={`p-3 rounded-xl bg-gradient-to-r ${stat.gradient} text-white shadow-lg`}>
                    <Icon className="h-6 w-6" />
                  </div>
                  <div className={`text-right ${stat.textColor}`}>
                    <div className="text-2xl font-bold leading-none">
                      {stat.value}
                    </div>
                  </div>
                </div>
                <div>
                  <h3 className="font-semibold text-foreground mb-1">
                    {stat.title}
                  </h3>
                  <p className="text-sm text-muted-foreground">
                    {stat.description}
                  </p>
                </div>
              </div>
            </div>
          );
        })}
      </div>

      {/* Enhanced Recent Users Table */}
      <div className="pancake-card">
        <div className="px-6 py-6 border-b border-border">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-gradient-to-r from-blue-500/20 to-purple-500/20">
              <Users className="h-6 w-6 text-blue-500" />
            </div>
            <div>
              <h2 className="text-xl font-bold text-foreground">
                Recent Users
              </h2>
              <p className="text-sm text-muted-foreground">
                Latest registered users
              </p>
            </div>
          </div>
        </div>
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-border">
            <thead className="bg-muted/50">
              <tr>
                <th className="px-6 py-4 text-left text-sm font-semibold text-foreground">
                  User
                </th>
                <th className="px-6 py-4 text-left text-sm font-semibold text-foreground">
                  Role
                </th>
                <th className="px-6 py-4 text-left text-sm font-semibold text-foreground">
                  Status
                </th>
                <th className="px-6 py-4 text-left text-sm font-semibold text-foreground">
                  Joined
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-border">
              {recentUsers.map((user, _index) => (
                <tr key={user.uid} className="hover:bg-muted/20 transition-colors group">
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="flex items-center">
                      <div className="h-10 w-10 rounded-full bg-gradient-to-r from-orange-500 to-yellow-500 flex items-center justify-center text-white font-semibold text-sm group-hover:scale-110 transition-transform">
                        {user.email.charAt(0).toUpperCase()}
                      </div>
                      <div className="ml-4">
                        <div className="text-sm font-semibold text-foreground">
                          {user.displayName || 'No name'}
                        </div>
                        <div className="text-sm text-muted-foreground">
                          {user.email}
                        </div>
                      </div>
                    </div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span className={`inline-flex items-center px-3 py-1 rounded-full text-xs font-semibold ${
                      user.customClaims?.role === 'ADMIN' 
                        ? 'bg-gradient-to-r from-orange-500 to-yellow-500 text-white'
                        : 'bg-muted text-muted-foreground'
                    }`}>
                      {user.customClaims?.role === 'ADMIN' && <Crown className="h-3 w-3 mr-1" />}
                      {user.customClaims?.role || 'USER'}
                    </span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="flex items-center gap-2">
                      {user.emailVerified ? (
                        <span className="inline-flex items-center px-3 py-1 rounded-full text-xs font-semibold bg-gradient-to-r from-green-500 to-green-600 text-white">
                          <CheckCircle className="h-3 w-3 mr-1" />
                          Verified
                        </span>
                      ) : (
                        <span className="inline-flex items-center px-3 py-1 rounded-full text-xs font-semibold bg-gradient-to-r from-yellow-500 to-orange-500 text-white">
                          <AlertTriangle className="h-3 w-3 mr-1" />
                          Unverified
                        </span>
                      )}
                      {user.disabled && (
                        <span className="inline-flex items-center px-3 py-1 rounded-full text-xs font-semibold bg-gradient-to-r from-red-500 to-red-600 text-white">
                          <UserX className="h-3 w-3 mr-1" />
                          Disabled
                        </span>
                      )}
                    </div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-muted-foreground font-medium">
                    {user.metadata?.creationTime 
                      ? new Date(user.metadata.creationTime).toLocaleDateString()
                      : 'Unknown'
                    }
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
