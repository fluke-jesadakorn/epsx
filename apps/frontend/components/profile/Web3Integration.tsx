'use client';

import { useState, useEffect } from 'react';
import { useAccount } from 'wagmi';
import { Wallet, Link, CheckCircle, AlertCircle, Shield, Eye, EyeOff, Key, Crown, Zap, Users, Settings } from 'lucide-react';
import { type User } from '@/shared/types/auth';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { WalletConnectAuth } from '@/components/auth/WalletConnectAuth';
import { Web3PermissionsDisplay } from '@/components/auth/Web3PermissionsDisplay';
import { ApiKeyManager } from '@/components/auth/ApiKeyManager';
import { useSharedAuth } from '@/shared/components/auth/SharedOpenIDWeb3Provider';
import { UserWalletDisplay, UserTierBadge, UserAuthStatus, UserPermissionsDisplay } from '@/shared/components/display/UserDisplay';

interface Web3IntegrationProps {
  // No props needed - uses shared authentication
}

interface WalletInfo {
  address: string;
  isLinked: boolean;
  linkedEmail?: string;
  permissions: string[];
}

export function Web3Integration(_props: Web3IntegrationProps) {
  const { address, isConnected } = useAccount();
  const { isAuthenticated, user, refreshUser } = useSharedAuth();
  const [activeTab, setActiveTab] = useState('overview');
  const [showAddress, setShowAddress] = useState(false);

  const formatAddress = (addr: string) => {
    if (!addr) return '';
    return showAddress 
      ? addr 
      : `${addr.slice(0, 6)}...${addr.slice(-4)}`;
  };

  // Tier display is now handled by shared components

  return (
    <div className="space-y-6">
      {/* Web3 Status Header */}
      <Card className="border-orange-100 dark:border-slate-700">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Wallet className="h-5 w-5 text-orange-500" />
            Web3 Authentication Dashboard
          </CardTitle>
          <p className="text-sm text-slate-600 dark:text-slate-400">
            Manage your Web3 wallet connection, permissions, and authentication settings
          </p>
        </CardHeader>
        <CardContent>
          {!isConnected ? (
            <div className="text-center p-8 border-2 border-dashed border-orange-200 dark:border-orange-800 rounded-xl bg-gradient-to-br from-orange-50 to-purple-50 dark:from-orange-900/20 dark:to-purple-900/20">
              <div className="p-4 bg-gradient-to-r from-orange-500 to-purple-600 rounded-xl w-fit mx-auto mb-4">
                <Wallet className="h-8 w-8 text-white" />
              </div>
              <h3 className="text-xl font-bold text-slate-900 dark:text-slate-100 mb-2">
                Connect Your Web3 Wallet
              </h3>
              <p className="text-sm text-slate-600 dark:text-slate-400 mb-6 max-w-md mx-auto">
                Unlock the full potential of EPSX with Web3 authentication. Get access to NFT-gated content, 
                token-based permissions, DAO governance, and enterprise API features.
              </p>
              <WalletConnectAuth className="max-w-md mx-auto" />
            </div>
          ) : (
            <div className="space-y-6">
              {/* Connected Wallet Summary */}
              <div className="flex items-center justify-between p-6 bg-gradient-to-r from-green-50 to-emerald-50 dark:from-green-900/20 dark:to-emerald-900/20 border border-green-200 dark:border-green-800 rounded-xl">
                <div className="flex items-center gap-4">
                  <div className="p-3 bg-green-100 dark:bg-green-900/20 rounded-xl">
                    <CheckCircle className="h-6 w-6 text-green-600 dark:text-green-400" />
                  </div>
                  <div>
                    <h3 className="font-bold text-green-900 dark:text-green-100">
                      Wallet Connected & Authenticated
                    </h3>
                    <UserWalletDisplay showFullAddress={showAddress} className="text-sm" />
                  </div>
                </div>
                <div className="flex items-center gap-3">
                  <UserTierBadge />
                  <UserAuthStatus />
                </div>
              </div>

              {/* Web3 Features Tabs */}
              <Tabs value={activeTab} onValueChange={setActiveTab}>
                <TabsList className="grid w-full grid-cols-4 bg-slate-100 dark:bg-slate-800">
                  <TabsTrigger value="overview" className="flex items-center gap-2">
                    <Shield className="h-4 w-4" />
                    <span className="hidden sm:inline">Overview</span>
                  </TabsTrigger>
                  <TabsTrigger value="permissions" className="flex items-center gap-2">
                    <Users className="h-4 w-4" />
                    <span className="hidden sm:inline">Permissions</span>
                  </TabsTrigger>
                  <TabsTrigger value="api" className="flex items-center gap-2">
                    <Key className="h-4 w-4" />
                    <span className="hidden sm:inline">API Keys</span>
                  </TabsTrigger>
                  <TabsTrigger value="settings" className="flex items-center gap-2">
                    <Settings className="h-4 w-4" />
                    <span className="hidden sm:inline">Settings</span>
                  </TabsTrigger>
                </TabsList>

                <div className="mt-6">
                  <TabsContent value="overview" className="space-y-6">
                    {/* Tier Information */}
                    <Card>
                      <CardHeader>
                        <CardTitle className="flex items-center gap-2">
                          <Shield className="h-5 w-5 text-orange-500" />
                          Your Web3 Access Level
                        </CardTitle>
                      </CardHeader>
                      <CardContent>
                        <div className="space-y-4">
                          <div className="p-4 bg-slate-50 dark:bg-slate-800 rounded-lg">
                            <div className="flex items-center justify-between mb-2">
                              <UserWalletDisplay showFullAddress={false} />
                              <UserTierBadge />
                            </div>
                            <UserAuthStatus />
                          </div>

                          {/* User Permissions Summary */}
                          <UserPermissionsDisplay maxDisplay={5} />
                        </div>
                      </CardContent>
                    </Card>

                    {/* Quick Actions */}
                    <Card>
                      <CardHeader>
                        <CardTitle>Quick Actions</CardTitle>
                      </CardHeader>
                      <CardContent>
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                          <Button
                            variant="outline"
                            onClick={refreshUser}
                            className="flex items-center gap-2"
                          >
                            <Shield className="h-4 w-4" />
                            Refresh Permissions
                          </Button>
                          <Button
                            variant="outline"
                            onClick={() => setActiveTab('permissions')}
                            className="flex items-center gap-2"
                          >
                            <Users className="h-4 w-4" />
                            View All Permissions
                          </Button>
                          <Button
                            variant="outline"
                            onClick={() => setActiveTab('api')}
                            className="flex items-center gap-2"
                          >
                            <Key className="h-4 w-4" />
                            Manage API Keys
                          </Button>
                          <Button
                            variant="outline"
                            onClick={() => setActiveTab('settings')}
                            className="flex items-center gap-2"
                          >
                            <Settings className="h-4 w-4" />
                            Web3 Settings
                          </Button>
                        </div>
                      </CardContent>
                    </Card>
                  </TabsContent>

                  <TabsContent value="permissions">
                    <Web3PermissionsDisplay variant="detailed" />
                  </TabsContent>

                  <TabsContent value="api">
                    <ApiKeyManager />
                  </TabsContent>

                  <TabsContent value="settings" className="space-y-6">
                    {/* Wallet Management */}
                    <Card>
                      <CardHeader>
                        <CardTitle className="flex items-center gap-2">
                          <Wallet className="h-5 w-5 text-orange-500" />
                          Wallet Management
                        </CardTitle>
                      </CardHeader>
                      <CardContent>
                        <div className="space-y-4">
                          <div className="p-4 bg-slate-50 dark:bg-slate-800 rounded-lg">
                            <WalletConnectAuth />
                          </div>
                        </div>
                      </CardContent>
                    </Card>

                    {/* Email Integration removed - not needed for wallet-first authentication */}

                    {/* Security Notice */}
                    <Alert>
                      <AlertCircle className="h-4 w-4" />
                      <AlertDescription>
                        <strong>Security Tip:</strong> Web3 authentication is self-sovereign and secure. 
                        Your wallet controls your identity - keep your private keys safe and never share them.
                      </AlertDescription>
                    </Alert>
                  </TabsContent>
                </div>
              </Tabs>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}