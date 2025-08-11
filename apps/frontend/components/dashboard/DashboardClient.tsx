'use client';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { signOut } from 'next-auth/react';
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

interface DashboardClientProps {
  user: any;
  permissions: {
    role: string;
    permissions: string[];
  };
  dashboardData: any;
}

export function DashboardClient({ user, permissions, dashboardData: _dashboardData }: DashboardClientProps) {
  const handleLogout = () => signOut({ redirect: true, callbackUrl: '/login' });

  return (
    <div className="min-h-screen relative overflow-hidden">
      {/* PancakeSwap-style vibrant background */}
      <div className="absolute inset-0 bg-gradient-to-br from-blue-50 via-orange-50 to-yellow-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900" />

      {/* Floating gradient orbs */}
      <div className="absolute -top-40 -left-40 w-96 h-96 bg-gradient-to-br from-orange-400/15 to-yellow-400/15 rounded-full blur-3xl animate-bounce-slow" />
      <div className="absolute top-20 -right-32 w-80 h-80 bg-gradient-to-br from-blue-400/12 to-cyan-400/12 rounded-full blur-3xl animate-float" />
      <div className="absolute bottom-20 left-20 w-72 h-72 bg-gradient-to-br from-purple-400/10 to-pink-400/10 rounded-full blur-3xl animate-pulse-gentle" />

      <div className="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Enhanced Header Section */}
        <div className="mb-12 text-center">
          <div className="inline-flex items-center justify-center w-20 h-20 bg-gradient-to-br from-orange-500 to-yellow-500 rounded-3xl mb-6 shadow-2xl animate-bounce-gentle">
            <TrendingUp className="w-10 h-10 text-white" />
          </div>
          <h1 className="text-5xl font-bold bg-gradient-to-r from-orange-600 via-yellow-600 to-orange-600 bg-clip-text text-transparent mb-4 animate-gradient-x">
            🚀 Dashboard
          </h1>
          <p className="text-xl text-gray-600 dark:text-gray-300 mb-4">
            Welcome{user ? ' back' : ''},{' '}
            <span className="font-semibold text-orange-600 dark:text-orange-400">
              {user?.email || 'Guest User'}
            </span>
            ! ✨
          </p>
          <div className="inline-flex items-center gap-3">
            <Badge
              variant="outline"
              className="bg-gradient-to-r from-green-500/10 to-emerald-500/10 border-green-300 dark:border-emerald-400/30 text-green-700 dark:text-emerald-300 px-4 py-2 text-sm font-semibold"
            >
              <Shield className="w-4 h-4 mr-2" />
              Role: {permissions.role}
            </Badge>
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
          {/* User Profile Card */}
          <Card className="relative overflow-hidden bg-white/80 dark:bg-slate-800/80 backdrop-blur-xl border-orange-200/50 dark:border-orange-400/20 shadow-2xl hover:shadow-3xl transition-all duration-300 hover:scale-105 animate-fade-in">
            <div className="absolute top-0 right-0 w-32 h-32 bg-gradient-to-br from-orange-400/10 to-yellow-400/10 rounded-full blur-2xl" />
            <div className="absolute bottom-0 left-0 w-24 h-24 bg-gradient-to-br from-blue-400/8 to-cyan-400/8 rounded-full blur-xl" />
            <CardHeader className="relative z-10">
              <CardTitle className="flex items-center text-orange-600 dark:text-orange-400">
                <div className="w-8 h-8 bg-gradient-to-br from-orange-500 to-yellow-500 rounded-lg flex items-center justify-center mr-3 animate-bounce-gentle">
                  <User className="w-5 h-5 text-white" />
                </div>
                👤 Profile
              </CardTitle>
              <CardDescription className="text-gray-600 dark:text-gray-300">
                Manage your personal information
              </CardDescription>
            </CardHeader>
            <CardContent className="relative z-10">
              <Link href="/profile">
                <Button variant="pancake" className="w-full">
                  <Sparkles className="mr-2 h-4 w-4" />
                  View Profile
                </Button>
              </Link>
            </CardContent>
          </Card>

          {/* Settings Card */}
          <Card className="relative overflow-hidden bg-white/80 dark:bg-slate-800/80 backdrop-blur-xl border-blue-200/50 dark:border-blue-400/20 shadow-2xl hover:shadow-3xl transition-all duration-300 hover:scale-105 animate-fade-in-delayed">
            <div className="absolute top-0 right-0 w-28 h-28 bg-gradient-to-br from-blue-400/10 to-purple-400/10 rounded-full blur-2xl" />
            <div className="absolute bottom-0 left-0 w-20 h-20 bg-gradient-to-br from-green-400/8 to-emerald-400/8 rounded-full blur-xl" />
            <CardHeader className="relative z-10">
              <CardTitle className="flex items-center text-blue-600 dark:text-blue-400">
                <div className="w-8 h-8 bg-gradient-to-br from-blue-500 to-purple-500 rounded-lg flex items-center justify-center mr-3 animate-pulse-gentle">
                  <Settings className="w-5 h-5 text-white" />
                </div>
                ⚙️ Settings
              </CardTitle>
              <CardDescription className="text-gray-600 dark:text-gray-300">
                Configure your preferences
              </CardDescription>
            </CardHeader>
            <CardContent className="relative z-10">
              <Link href="/settings">
                <Button variant="pancake-blue" className="w-full">
                  <Settings className="mr-2 h-4 w-4" />
                  Open Settings
                </Button>
              </Link>
            </CardContent>
          </Card>

          {/* Analytics Card */}
          <Card className="relative overflow-hidden bg-white/80 dark:bg-slate-800/80 backdrop-blur-xl border-green-200/50 dark:border-green-400/20 shadow-2xl hover:shadow-3xl transition-all duration-300 hover:scale-105 animate-fade-in-delayed-2">
            <div className="absolute top-0 right-0 w-30 h-30 bg-gradient-to-br from-green-400/10 to-emerald-400/10 rounded-full blur-2xl" />
            <div className="absolute bottom-0 left-0 w-22 h-22 bg-gradient-to-br from-purple-400/8 to-pink-400/8 rounded-full blur-xl" />
            <CardHeader className="relative z-10">
              <CardTitle className="flex items-center text-green-600 dark:text-green-400">
                <div className="w-8 h-8 bg-gradient-to-br from-green-500 to-emerald-500 rounded-lg flex items-center justify-center mr-3 animate-float-gentle">
                  <BarChart3 className="w-5 h-5 text-white" />
                </div>
                📊 Analytics
              </CardTitle>
              <CardDescription className="text-gray-600 dark:text-gray-300">
                View your data and insights
              </CardDescription>
            </CardHeader>
            <CardContent className="relative z-10">
              <Link href="/analytics">
                <Button variant="pancake-green" className="w-full">
                  <BarChart3 className="mr-2 h-4 w-4" />
                  View Analytics
                </Button>
              </Link>
            </CardContent>
          </Card>

          {/* Premium Content Card */}
          <Card className="relative overflow-hidden bg-white/80 dark:bg-slate-800/80 backdrop-blur-xl border-purple-200/50 dark:border-purple-400/20 shadow-2xl hover:shadow-3xl transition-all duration-300 hover:scale-105 animate-fade-in-delayed-3">
            <div className="absolute top-0 right-0 w-32 h-32 bg-gradient-to-br from-purple-400/10 to-pink-400/10 rounded-full blur-2xl" />
            <div className="absolute bottom-0 left-0 w-24 h-24 bg-gradient-to-br from-yellow-400/8 to-orange-400/8 rounded-full blur-xl" />
            <CardHeader className="relative z-10">
              <CardTitle className="flex items-center text-purple-600 dark:text-purple-400">
                <div className="w-8 h-8 bg-gradient-to-br from-purple-500 to-pink-500 rounded-lg flex items-center justify-center mr-3 animate-glow">
                  <Lock className="w-5 h-5 text-white" />
                </div>
                🔒 Premium Content
              </CardTitle>
              <CardDescription className="text-gray-600 dark:text-gray-300">
                Access exclusive premium features
              </CardDescription>
            </CardHeader>
            <CardContent className="relative z-10">
              <Link href="/premium">
                <Button variant="pancake-purple" className="w-full">
                  <Lock className="mr-2 h-4 w-4" />
                  Access Premium
                </Button>
              </Link>
            </CardContent>
          </Card>

          {/* Moderator Panel */}
          <Card className="relative overflow-hidden bg-white/80 dark:bg-slate-800/80 backdrop-blur-xl border-red-200/50 dark:border-red-400/20 shadow-2xl hover:shadow-3xl transition-all duration-300 hover:scale-105 animate-fade-in-delayed">
            <div className="absolute top-0 right-0 w-28 h-28 bg-gradient-to-br from-red-400/10 to-rose-400/10 rounded-full blur-2xl" />
            <div className="absolute bottom-0 left-0 w-20 h-20 bg-gradient-to-br from-orange-400/8 to-yellow-400/8 rounded-full blur-xl" />
            <CardHeader className="relative z-10">
              <CardTitle className="flex items-center text-red-600 dark:text-red-400">
                <div className="w-8 h-8 bg-gradient-to-br from-red-500 to-rose-500 rounded-lg flex items-center justify-center mr-3 animate-wiggle">
                  <Shield className="w-5 h-5 text-white" />
                </div>
                🛡️ Moderator Panel
              </CardTitle>
              <CardDescription className="text-gray-600 dark:text-gray-300">
                Moderate content and users
              </CardDescription>
            </CardHeader>
            <CardContent className="relative z-10">
              <Link href="/moderator">
                <Button variant="pancake-red" className="w-full">
                  <Shield className="mr-2 h-4 w-4" />
                  Open Moderator Panel
                </Button>
              </Link>
            </CardContent>
          </Card>
        </div>

        <div className="mt-12">
          <Card className="relative overflow-hidden bg-white/80 dark:bg-slate-800/80 backdrop-blur-xl border-indigo-200/50 dark:border-indigo-400/20 shadow-2xl animate-fade-in-delayed-3">
            <div className="absolute top-0 right-0 w-40 h-40 bg-gradient-to-br from-indigo-400/10 to-violet-400/10 rounded-full blur-3xl" />
            <div className="absolute bottom-0 left-0 w-32 h-32 bg-gradient-to-br from-cyan-400/8 to-blue-400/8 rounded-full blur-2xl" />
            <CardHeader className="relative z-10">
              <CardTitle className="flex items-center text-indigo-600 dark:text-indigo-400">
                <div className="w-8 h-8 bg-gradient-to-br from-indigo-500 to-violet-500 rounded-lg flex items-center justify-center mr-3 animate-scale-pulse">
                  <Shield className="w-5 h-5 text-white" />
                </div>
                🔐 Your Permissions
              </CardTitle>
              <CardDescription className="text-gray-600 dark:text-gray-300">
                Current permissions for your account
              </CardDescription>
            </CardHeader>
            <CardContent className="relative z-10">
              <div className="space-y-4">
                <div className="flex items-center gap-3">
                  <span className="text-lg font-semibold text-gray-700 dark:text-gray-200">
                    Role:
                  </span>
                  <Badge className="bg-gradient-to-r from-indigo-500 to-violet-500 text-white px-4 py-2 text-sm font-semibold shadow-lg">
                    {permissions.role}
                  </Badge>
                </div>
                {permissions.permissions &&
                permissions.permissions.length > 0 ? (
                  <div className="space-y-2">
                    <h4 className="font-medium text-gray-700 dark:text-gray-200">
                      🎯 Permissions:
                    </h4>
                    <div className="flex flex-wrap gap-2">
                      {permissions.permissions.map((permission, index) => (
                        <Badge
                          key={permission}
                          variant="secondary"
                          className="bg-gradient-to-r from-cyan-500/10 to-blue-500/10 border-cyan-300 dark:border-cyan-400/30 text-cyan-700 dark:text-cyan-300 hover:scale-105 transition-transform duration-200"
                          style={{ animationDelay: `${index * 0.1}s` }}
                        >
                          {permission}
                        </Badge>
                      ))}
                    </div>
                  </div>
                ) : (
                  <div className="text-center py-4">
                    <p className="text-gray-600 dark:text-gray-400 mb-2">
                      🎭 No specific permissions assigned
                    </p>
                    <div className="w-12 h-12 mx-auto bg-gradient-to-br from-gray-400/20 to-slate-400/20 rounded-full flex items-center justify-center">
                      <span className="text-2xl">⭐</span>
                    </div>
                  </div>
                )}
              </div>
            </CardContent>
          </Card>
        </div>

        <div className="mt-12 text-center">
          {user ? (
            <Button
              onClick={handleLogout}
              variant="pancake-outline"
              className="px-8 py-3 text-lg font-semibold"
            >
              <Lock className="mr-2 h-5 w-5" />
              🚪 Sign Out
            </Button>
          ) : (
            <Link href="/login">
              <Button
                variant="pancake"
                className="px-8 py-3 text-lg font-semibold"
              >
                <Lock className="mr-2 h-5 w-5" />
                🚪 Log In
              </Button>
            </Link>
          )}

          {/* Decorative elements */}
          <div className="flex justify-center items-center gap-4 mt-6">
            <div className="w-2 h-2 bg-orange-400 rounded-full animate-ping" />
            <div
              className="w-3 h-3 bg-yellow-400 rounded-full animate-ping"
              style={{ animationDelay: '0.5s' }}
            />
            <div
              className="w-2 h-2 bg-blue-400 rounded-full animate-ping"
              style={{ animationDelay: '1s' }}
            />
          </div>
        </div>
      </div>
    </div>
  );
}