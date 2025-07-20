'use client';

import { useIAM, RoleGate } from '@/context/iam-context';
import { useRouter } from 'next/navigation';
import { useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Users, Shield, Settings, BarChart3, Lock } from 'lucide-react';
import Link from 'next/link';

export default function AdminPage() {
  const { user, permissions } = useIAM();
  const router = useRouter();

  useEffect(() => {
    if (!permissions || permissions.role !== 'admin') {
      router.push('/dashboard');
    }
  }, [permissions, router]);

  if (!user || !permissions || permissions.role !== 'admin') {
    return null;
  }

  return (
    <RoleGate role="admin">
      <div className="min-h-screen bg-gray-50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
          <div className="mb-8">
            <h1 className="text-3xl font-bold text-gray-900">Admin Panel</h1>
            <p className="mt-2 text-gray-600">
              Manage system settings, users, and permissions
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {/* User Management */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center">
                  <Users className="mr-2 h-5 w-5" />
                  User Management
                </CardTitle>
                <CardDescription>
                  Manage user accounts, roles, and permissions
                </CardDescription>
              </CardHeader>
              <CardContent>
                <Link href="/admin/users">
                  <Button className="w-full">Manage Users</Button>
                </Link>
              </CardContent>
            </Card>

            {/* Role Management */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center">
                  <Shield className="mr-2 h-5 w-5" />
                  Role Management
                </CardTitle>
                <CardDescription>
                  Create and manage roles and permissions
                </CardDescription>
              </CardHeader>
              <CardContent>
                <Link href="/admin/roles">
                  <Button className="w-full">Manage Roles</Button>
                </Link>
              </CardContent>
            </Card>

            {/* System Settings */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center">
                  <Settings className="mr-2 h-5 w-5" />
                  System Settings
                </CardTitle>
                <CardDescription>
                  Configure system-wide settings
                </CardDescription>
              </CardHeader>
              <CardContent>
                <Link href="/admin/settings">
                  <Button className="w-full">Configure Settings</Button>
                </Link>
              </CardContent>
            </Card>

            {/* Analytics */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center">
                  <BarChart3 className="mr-2 h-5 w-5" />
                  Analytics
                </CardTitle>
                <CardDescription>
                  View system analytics and reports
                </CardDescription>
              </CardHeader>
              <CardContent>
                <Link href="/admin/analytics">
                  <Button className="w-full">View Analytics</Button>
                </Link>
              </CardContent>
            </Card>

            {/* Security */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center">
                  <Lock className="mr-2 h-5 w-5" />
                  Security
                </CardTitle>
                <CardDescription>
                  Manage security settings and access logs
                </CardDescription>
              </CardHeader>
              <CardContent>
                <Link href="/admin/security">
                  <Button className="w-full">Security Settings</Button>
                </Link>
              </CardContent>
            </Card>
          </div>

          <div className="mt-8">
            <Card>
              <CardHeader>
                <CardTitle>System Overview</CardTitle>
                <CardDescription>
                  Quick overview of system status
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                  <div className="text-center">
                    <div className="text-2xl font-bold text-blue-600">--</div>
                    <div className="text-sm text-gray-600">Total Users</div>
                  </div>
                  <div className="text-center">
                    <div className="text-2xl font-bold text-green-600">--</div>
                    <div className="text-sm text-gray-600">Active Sessions</div>
                  </div>
                  <div className="text-center">
                    <div className="text-2xl font-bold text-purple-600">--</div>
                    <div className="text-sm text-gray-600">System Health</div>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      </div>
    </RoleGate>
  );
}
