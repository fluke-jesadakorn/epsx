'use client';

import { Crown, Database, Settings, Shield, User } from 'lucide-react';
import { useState } from 'react';

import { AdminPermissions } from './admin-permissions';

import { PageHeader } from '@/components/shared';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { type User as UserType } from '@/shared/types/auth';

interface AdminProfileClientProps {
  user: UserType;
}

/**
 *
 * @param root0
 * @param root0.user
 */
export function AdminProfileClient({ user }: AdminProfileClientProps) {
  const [activeTab, setActiveTab] = useState('account');

  const formatDate = (timestamp?: number) => {
    if (!timestamp) { return 'Not available'; }
    return new Date(timestamp * 1000).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const getAdminLevel = () => {
    const adminPermissions = user.permissions?.filter(p => p.startsWith('admin:')) || [];
    if (adminPermissions.length >= 10) { return 'Super Admin'; }
    if (adminPermissions.length >= 5) { return 'Admin'; }
    return 'Limited Admin';
  };

  const getAdminBadgeColor = () => {
    const level = getAdminLevel();
    switch (level) {
      case 'Super Admin': return 'bg-purple-100 text-purple-800 dark:bg-purple-900/20 dark:text-purple-300';
      case 'Admin': return 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-300';
      default: return 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-300';
    }
  };

  return (
    <>
      <PageHeader
        title="Admin Profile"
        subtitle="Manage your administrative account and permissions"
        icon="user"
        gradient="purple"
      />
      <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
        {/* Admin Profile Sidebar */}
        <div className="lg:col-span-1">
          <Card className="border-yellow-200 dark:border-slate-700">
            <CardHeader className="text-center pb-4">
              <div className="mx-auto mb-4 flex h-20 w-20 items-center justify-center rounded-full bg-gradient-to-br from-yellow-400 via-orange-500 to-pink-500">
                <Crown className="h-10 w-10 text-white" />
              </div>
              <CardTitle className="text-lg text-slate-900 dark:text-slate-100">
                {user.name || 'Admin user'}
              </CardTitle>
              <p className="text-sm text-slate-600 dark:text-slate-400">
                {user.email}
              </p>
              <div className="flex justify-center mt-2">
                <Badge className={getAdminBadgeColor()}>
                  {getAdminLevel()}
                </Badge>
              </div>
            </CardHeader>
            <CardContent className="pt-0">
              <div className="space-y-4">
                <div className="text-center">
                  <div className="text-2xl font-bold text-orange-600 dark:text-orange-400">
                    {user.permissions?.filter(p => p.startsWith('admin:')).length || 0}
                  </div>
                  <div className="text-sm text-slate-600 dark:text-slate-400">
                    Admin Permissions
                  </div>
                </div>

                <div className="border-t border-slate-200 dark:border-slate-700 pt-4">
                  <div className="text-sm space-y-2">
                    <div className="flex justify-between">
                      <span className="text-slate-600 dark:text-slate-400">Total Permissions:</span>
                      <span className="font-medium text-slate-900 dark:text-slate-100">
                        {user.permissions?.length || 0}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-slate-600 dark:text-slate-400">Platform:</span>
                      <span className="font-medium text-slate-900 dark:text-slate-100">
                        {user.platform_context || 'admin'}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-slate-600 dark:text-slate-400">Status:</span>
                      <Badge variant={user.verified ? "default" : "secondary"}>
                        {user.verified ? 'Verified' : 'Unverified'}
                      </Badge>
                    </div>
                    {user.permission_last_updated && (
                      <div className="text-xs text-slate-500 dark:text-slate-500 pt-2">
                        Last updated: {formatDate(user.permission_last_updated)}
                      </div>
                    )}
                  </div>
                </div>

                <div className="pt-4 border-t border-slate-200 dark:border-slate-700">
                  <Button
                    variant="outline"
                    size="sm"
                    className="w-full mb-2"
                    onClick={() => window.location.href = '/permissions'}
                  >
                    <Shield className="h-4 w-4 mr-2" />
                    Manage Permissions
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    className="w-full"
                    onClick={() => window.location.href = '/settings'}
                  >
                    <Settings className="h-4 w-4 mr-2" />
                    System Settings
                  </Button>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Main Content */}
        <div className="lg:col-span-3">
          <Tabs value={activeTab} onValueChange={setActiveTab}>
            <TabsList className="grid w-full grid-cols-3 bg-slate-100 dark:bg-slate-800">
              <TabsTrigger
                value="account"
                className="flex items-center gap-2 data-[state=active]:bg-white dark:data-[state=active]:bg-slate-700"
              >
                <User className="h-4 w-4" />
                <span className="hidden sm:inline">Account</span>
              </TabsTrigger>
              <TabsTrigger
                value="permissions"
                className="flex items-center gap-2 data-[state=active]:bg-white dark:data-[state=active]:bg-slate-700"
              >
                <Shield className="h-4 w-4" />
                <span className="hidden sm:inline">Permissions</span>
              </TabsTrigger>
              <TabsTrigger
                value="settings"
                className="flex items-center gap-2 data-[state=active]:bg-white dark:data-[state=active]:bg-slate-700"
              >
                <Settings className="h-4 w-4" />
                <span className="hidden sm:inline">Settings</span>
              </TabsTrigger>
            </TabsList>

            <div className="mt-6">
              <TabsContent value="account" className="space-y-6">
                <Card className="border-yellow-200 dark:border-slate-700">
                  <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                      <User className="h-5 w-5 text-orange-500" />
                      Admin Account Information
                    </CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                      <div>
                        <label className="text-sm font-medium text-slate-700 dark:text-slate-300">
                          Admin ID
                        </label>
                        <div className="mt-1 p-3 bg-slate-50 dark:bg-slate-800 rounded-lg font-mono text-sm">
                          {user.id}
                        </div>
                      </div>
                      <div>
                        <label className="text-sm font-medium text-slate-700 dark:text-slate-300">
                          Email Address
                        </label>
                        <div className="mt-1 p-3 bg-slate-50 dark:bg-slate-800 rounded-lg text-sm">
                          {user.email}
                        </div>
                      </div>
                      <div>
                        <label className="text-sm font-medium text-slate-700 dark:text-slate-300">
                          Admin Level
                        </label>
                        <div className="mt-1 p-3 bg-slate-50 dark:bg-slate-800 rounded-lg text-sm">
                          <Badge className={getAdminBadgeColor()}>
                            {getAdminLevel()}
                          </Badge>
                        </div>
                      </div>
                      <div>
                        <label className="text-sm font-medium text-slate-700 dark:text-slate-300">
                          Platform Context
                        </label>
                        <div className="mt-1 p-3 bg-slate-50 dark:bg-slate-800 rounded-lg text-sm">
                          {user.platform_context || 'admin'}
                        </div>
                      </div>
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-3 gap-4 pt-4 border-t border-slate-200 dark:border-slate-700">
                      <div className="text-center p-4 bg-gradient-to-br from-purple-50 to-purple-100 dark:from-purple-900/20 dark:to-purple-800/20 rounded-lg">
                        <div className="text-2xl font-bold text-purple-600 dark:text-purple-400">
                          {user.permissions?.filter(p => p.startsWith('admin:')).length || 0}
                        </div>
                        <div className="text-sm text-purple-700 dark:text-purple-300">
                          Admin Permissions
                        </div>
                      </div>
                      <div className="text-center p-4 bg-gradient-to-br from-blue-50 to-blue-100 dark:from-blue-900/20 dark:to-blue-800/20 rounded-lg">
                        <div className="text-2xl font-bold text-blue-600 dark:text-blue-400">
                          {user.permissions?.filter(p => p.includes(':manage') || p.includes(':admin')).length || 0}
                        </div>
                        <div className="text-sm text-blue-700 dark:text-blue-300">
                          Management Rights
                        </div>
                      </div>
                      <div className="text-center p-4 bg-gradient-to-br from-green-50 to-green-100 dark:from-green-900/20 dark:to-green-800/20 rounded-lg">
                        <div className="text-2xl font-bold text-green-600 dark:text-green-400">
                          {user.permissions?.length || 0}
                        </div>
                        <div className="text-sm text-green-700 dark:text-green-300">
                          Total Permissions
                        </div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </TabsContent>

              <TabsContent value="permissions">
                <AdminPermissions user={user} />
              </TabsContent>

              <TabsContent value="settings" className="space-y-6">
                <Card className="border-yellow-200 dark:border-slate-700">
                  <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                      <Settings className="h-5 w-5 text-orange-500" />
                      Admin Preferences
                    </CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div className="space-y-4">
                      <div className="flex items-center justify-between p-4 bg-slate-50 dark:bg-slate-800 rounded-lg">
                        <div>
                          <div className="font-medium text-slate-900 dark:text-slate-100">
                            Security Notifications
                          </div>
                          <div className="text-sm text-slate-600 dark:text-slate-400">
                            Get notified about critical security events
                          </div>
                        </div>
                        <Badge variant="outline" className="text-green-600 border-green-600">
                          Enabled
                        </Badge>
                      </div>

                      <div className="flex items-center justify-between p-4 bg-slate-50 dark:bg-slate-800 rounded-lg">
                        <div>
                          <div className="font-medium text-slate-900 dark:text-slate-100">
                            Admin Dashboard Updates
                          </div>
                          <div className="text-sm text-slate-600 dark:text-slate-400">
                            Real-time updates for admin activities
                          </div>
                        </div>
                        <Badge variant="outline" className="text-green-600 border-green-600">
                          Enabled
                        </Badge>
                      </div>

                      <div className="flex items-center justify-between p-4 bg-slate-50 dark:bg-slate-800 rounded-lg">
                        <div>
                          <div className="font-medium text-slate-900 dark:text-slate-100">
                            System Maintenance Alerts
                          </div>
                          <div className="text-sm text-slate-600 dark:text-slate-400">
                            Notifications about scheduled maintenance
                          </div>
                        </div>
                        <Badge variant="outline" className="text-green-600 border-green-600">
                          Enabled
                        </Badge>
                      </div>
                    </div>

                    <div className="pt-4 border-t border-slate-200 dark:border-slate-700">
                      <h4 className="font-medium text-slate-900 dark:text-slate-100 mb-3">
                        Quick Admin Actions
                      </h4>
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                        <Button
                          variant="outline"
                          onClick={() => window.location.href = '/users'}
                          className="justify-start"
                        >
                          <Database className="h-4 w-4 mr-2" />
                          Manage Users
                        </Button>
                        <Button
                          variant="outline"
                          onClick={() => window.location.href = '/permissions'}
                          className="justify-start"
                        >
                          <Shield className="h-4 w-4 mr-2" />
                          Permission Center
                        </Button>
                        <Button
                          variant="outline"
                          onClick={() => window.location.href = '/analytics'}
                          className="justify-start"
                        >
                          <Database className="h-4 w-4 mr-2" />
                          Analytics Dashboard
                        </Button>
                        <Button
                          variant="outline"
                          onClick={() => window.location.href = '/settings'}
                          className="justify-start"
                        >
                          <Settings className="h-4 w-4 mr-2" />
                          System Settings
                        </Button>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </TabsContent>
            </div>
          </Tabs>
        </div>
      </div>
    </>
  );
}