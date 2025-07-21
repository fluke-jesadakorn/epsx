'use client';

import { useIAM, PermissionGate } from '@/context/iam-context';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Shield, User, Settings, BarChart3, Lock } from 'lucide-react';
import Link from 'next/link';

export default function DashboardPage() {
  const { user, permissions, signOut } = useIAM();

  if (!user || !permissions) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <h2 className="text-2xl font-bold text-gray-900">Access Denied</h2>
          <p className="mt-2 text-gray-600">Please log in to access the dashboard.</p>
          <Link href="/login">
            <Button className="mt-4">Go to Login</Button>
          </Link>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900">Dashboard</h1>
          <p className="mt-2 text-gray-600">
            Welcome back, {user.email || 'User'}!
          </p>
          <div className="mt-2">
            <Badge variant="outline">Role: {permissions.role}</Badge>
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {/* User Profile Card */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center">
                <User className="mr-2 h-5 w-5" />
                Profile
              </CardTitle>
              <CardDescription>Manage your personal information</CardDescription>
            </CardHeader>
            <CardContent>
              <Link href="/profile">
                <Button className="w-full">View Profile</Button>
              </Link>
            </CardContent>
          </Card>

          {/* Settings Card */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center">
                <Settings className="mr-2 h-5 w-5" />
                Settings
              </CardTitle>
              <CardDescription>Configure your preferences</CardDescription>
            </CardHeader>
            <CardContent>
              <Link href="/settings">
                <Button className="w-full">Open Settings</Button>
              </Link>
            </CardContent>
          </Card>

          {/* Analytics Card */}
          <PermissionGate permission="read:own_data">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center">
                  <BarChart3 className="mr-2 h-5 w-5" />
                  Analytics
                </CardTitle>
                <CardDescription>View your data and insights</CardDescription>
              </CardHeader>
              <CardContent>
                <Link href="/analytics">
                  <Button className="w-full">View Analytics</Button>
                </Link>
              </CardContent>
            </Card>
          </PermissionGate>

          {/* Premium Content Card */}
          <PermissionGate permission="read:premium_content">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center">
                  <Lock className="mr-2 h-5 w-5" />
                  Premium Content
                </CardTitle>
                <CardDescription>Access exclusive premium features</CardDescription>
              </CardHeader>
              <CardContent>
                <Link href="/premium">
                  <Button className="w-full">Access Premium</Button>
                </Link>
              </CardContent>
            </Card>
          </PermissionGate>

          {/* Moderator Panel */}
          <PermissionGate permission="moderate:content">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center">
                  <Shield className="mr-2 h-5 w-5" />
                  Moderator Panel
                </CardTitle>
                <CardDescription>Moderate content and users</CardDescription>
              </CardHeader>
              <CardContent>
                <Link href="/moderator">
                  <Button className="w-full">Open Moderator Panel</Button>
                </Link>
              </CardContent>
            </Card>
          </PermissionGate>
        </div>

        <div className="mt-8">
          <Card>
            <CardHeader>
              <CardTitle>Your Permissions</CardTitle>
              <CardDescription>
                Current permissions for your account
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                <h3 className="font-semibold">Role: {permissions.role}</h3>
                {permissions.permissions && permissions.permissions.length > 0 ? (
                  <div className="flex flex-wrap gap-2">
                    {permissions.permissions.map((permission) => (
                      <Badge key={permission} variant="secondary">
                        {permission}
                      </Badge>
                    ))}
                  </div>
                ) : (
                  <p className="text-gray-600">No specific permissions assigned</p>
                )}
              </div>
            </CardContent>
          </Card>
        </div>

        <div className="mt-8">
          <Button onClick={signOut} variant="outline">
            Sign Out
          </Button>
        </div>
      </div>
    </div>
  );
}
