'use client';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { type User as WalletType } from '@/shared/types/auth';
import { Database, Mail, Settings, Shield, Wallet } from 'lucide-react';
import { useState } from 'react';
import { DataManagement } from './data-management';
import { EmailManagement } from './email-management';
import { Web3Integration } from './web3-integration';

// Role display helper (uses backend-provided role)
function getRoleDisplayName(role: string): string {
  switch (role) {
    case 'super_admin': return 'Super Admin';
    case 'admin': return 'Admin';
    case 'premium_user': return 'Premium';
    default: return 'user';
  }
}

interface WalletProfileClientProps {
  wallet: WalletType;
}

export function WalletProfileClient({ wallet }: WalletProfileClientProps) {
  const [activeTab, setActiveTab] = useState('web3');

  const formatDate = (timestamp?: number) => {
    if (!timestamp) {return 'Not available';}
    return new Date(timestamp * 1000).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  // Use backend-provided role directly
  const role = (wallet as any).role ?? 'user';
  const roleDisplay = getRoleDisplayName(role);

  return (
    <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
      {/* Wallet Profile Sidebar */}
      <div className="lg:col-span-1">
        <Card className="border-orange-100 dark:border-slate-700">
          <CardHeader className="text-center pb-4">
            <div className="mx-auto mb-4 flex h-20 w-20 items-center justify-center rounded-full bg-gradient-to-br from-orange-400 to-pink-500">
              <Wallet className="h-10 w-10 text-white" />
            </div>
            <CardTitle className="text-lg text-slate-900 dark:text-slate-100">
              {wallet.name ?? 'Wallet'}
            </CardTitle>
            <p className="text-sm text-slate-600 dark:text-slate-400">
              {wallet.email}
            </p>
            <div className="flex justify-center mt-2">
              <Badge
                variant={wallet.verified ? "default" : "secondary"}
                className={wallet.verified ? "bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-300" : ""}
              >
                {wallet.verified ? 'Verified' : 'Unverified'}
              </Badge>
            </div>
          </CardHeader>
          <CardContent className="pt-0">
            <div className="space-y-4">
              <div className="text-center">
                <div className="text-2xl font-bold text-orange-600 dark:text-orange-400">
                  {roleDisplay}
                </div>
                <div className="text-sm text-slate-600 dark:text-slate-400">
                  Access Group
                </div>
              </div>

              <div className="border-t border-slate-200 dark:border-slate-700 pt-4">
                <div className="text-sm space-y-2">
                  <div className="flex justify-between">
                    <span className="text-slate-600 dark:text-slate-400">Permissions:</span>
                    <span className="font-medium text-slate-900 dark:text-slate-100">
                      {wallet.permissions?.length || 0}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-slate-600 dark:text-slate-400">Platform:</span>
                    <span className="font-medium text-slate-900 dark:text-slate-100">
                      {wallet.platform_context ?? 'epsx'}
                    </span>
                  </div>
                  {wallet.permission_last_updated && (
                    <div className="text-xs text-slate-500 dark:text-slate-500">
                      Last updated: {formatDate(wallet.permission_last_updated)}
                    </div>
                  )}
                </div>
              </div>

              <Button
                variant="outline"
                size="sm"
                className="w-full"
                onClick={() => window.location.href = '/plans'}
              >
                Upgrade Access
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Main Content */}
      <div className="lg:col-span-3">
        <Tabs value={activeTab} onValueChange={setActiveTab}>
          <TabsList className="grid w-full grid-cols-4 bg-slate-100 dark:bg-slate-800">
            <TabsTrigger
              value="web3"
              className="flex items-center gap-2 data-[state=active]:bg-white dark:data-[state=active]:bg-slate-700"
            >
              <Shield className="h-4 w-4" />
              <span className="hidden sm:inline">Web3</span>
            </TabsTrigger>
            <TabsTrigger
              value="account"
              className="flex items-center gap-2 data-[state=active]:bg-white dark:data-[state=active]:bg-slate-700"
            >
              <Settings className="h-4 w-4" />
              <span className="hidden sm:inline">Account</span>
            </TabsTrigger>
            <TabsTrigger
              value="email"
              className="flex items-center gap-2 data-[state=active]:bg-white dark:data-[state=active]:bg-slate-700"
            >
              <Mail className="h-4 w-4" />
              <span className="hidden sm:inline">Email</span>
            </TabsTrigger>
            <TabsTrigger
              value="data"
              className="flex items-center gap-2 data-[state=active]:bg-white dark:data-[state=active]:bg-slate-700"
            >
              <Database className="h-4 w-4" />
              <span className="hidden sm:inline">Data</span>
            </TabsTrigger>
          </TabsList>

          <div className="mt-6">
            <TabsContent value="web3">
              <Web3Integration />
            </TabsContent>

            <TabsContent value="account" className="space-y-6">
              <Card className="border-orange-100 dark:border-slate-700">
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Settings className="h-5 w-5 text-orange-500" />
                    Wallet Information
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                      <label className="text-sm font-medium text-slate-700 dark:text-slate-300">
                        Wallet ID
                      </label>
                      <div className="mt-1 p-3 bg-slate-50 dark:bg-slate-800 rounded-lg font-mono text-sm">
                        {wallet.id}
                      </div>
                    </div>
                    <div>
                      <label className="text-sm font-medium text-slate-700 dark:text-slate-300">
                        Email Address
                      </label>
                      <div className="mt-1 p-3 bg-slate-50 dark:bg-slate-800 rounded-lg text-sm">
                        {wallet.email}
                      </div>
                    </div>
                    <div>
                      <label className="text-sm font-medium text-slate-700 dark:text-slate-300">
                        Access Group
                      </label>
                      <div className="mt-1 p-3 bg-slate-50 dark:bg-slate-800 rounded-lg text-sm">
                        <Badge variant="outline">{roleDisplay}</Badge>
                      </div>
                    </div>
                    <div>
                      <label className="text-sm font-medium text-slate-700 dark:text-slate-300">
                        Platform Context
                      </label>
                      <div className="mt-1 p-3 bg-slate-50 dark:bg-slate-800 rounded-lg text-sm">
                        {wallet.platform_context ?? 'epsx'}
                      </div>
                    </div>
                  </div>

                  {wallet.permissions && wallet.permissions.length > 0 && (
                    <div>
                      <label className="text-sm font-medium text-slate-700 dark:text-slate-300 mb-2 block">
                        Current Permissions ({wallet.permissions.length})
                      </label>
                      <div className="max-h-32 overflow-y-auto space-y-1">
                        {wallet.permissions.map((permission) => (
                          <Badge
                            key={permission}
                            variant="secondary"
                            className="mr-2 mb-1 text-xs"
                          >
                            {permission}
                          </Badge>
                        ))}
                      </div>
                    </div>
                  )}
                </CardContent>
              </Card>
            </TabsContent>

            <TabsContent value="email">
              <EmailManagement user={wallet} />
            </TabsContent>

            <TabsContent value="data">
              <DataManagement user={wallet} />
            </TabsContent>
          </div>
        </Tabs>
      </div>
    </div>
  );
}