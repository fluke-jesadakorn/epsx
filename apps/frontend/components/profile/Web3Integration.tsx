'use client';

import { useState, useEffect } from 'react';
import { useAccount } from 'wagmi';
import { Wallet, Link, CheckCircle, AlertCircle, Shield, Eye, EyeOff, Key, Crown, Zap, Users, Settings } from 'lucide-react';
import { type User } from '../../../../shared/types/auth';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { WalletConnectAuth } from '@/components/auth/WalletConnectAuth';
import { EmailLinking } from '@/components/auth/EmailLinking';
import { Web3PermissionsDisplay } from '@/components/auth/Web3PermissionsDisplay';
import { ApiKeyManager } from '@/components/auth/ApiKeyManager';
import { useWeb3AuthContext } from '@/providers/Web3AuthProvider';

interface Web3IntegrationProps {
  user: User;
}

interface WalletInfo {
  address: string;
  isLinked: boolean;
  linkedEmail?: string;
  permissions: string[];
}

export function Web3Integration({ user }: Web3IntegrationProps) {
  const { address, isConnected } = useAccount();
  const {
    isAuthenticated,
    walletAddress,
    permissions,
    userTier,
    hasApiAccess,
    refreshPermissions
  } = useWeb3AuthContext();
  const [activeTab, setActiveTab] = useState('overview');
  const [showAddress, setShowAddress] = useState(false);

  const formatAddress = (addr: string) => {
    if (!addr) return '';
    return showAddress 
      ? addr 
      : `${addr.slice(0, 6)}...${addr.slice(-4)}`;
  };

  const getTierInfo = () => {
    switch (userTier) {
      case 'nft':
        return {
          icon: <Crown className="h-5 w-5 text-purple-500" />,
          title: 'NFT Tier',
          description: 'Enhanced access through NFT ownership',
          color: 'bg-purple-100 text-purple-800 dark:bg-purple-900/20 dark:text-purple-300'
        };
      case 'token':
        return {
          icon: <Zap className="h-5 w-5 text-orange-500" />,
          title: 'Token Tier',
          description: 'Token-gated premium features',
          color: 'bg-orange-100 text-orange-800 dark:bg-orange-900/20 dark:text-orange-300'
        };
      case 'dao':
        return {
          icon: <Users className="h-5 w-5 text-blue-500" />,
          title: 'DAO Tier',
          description: 'Governance access and voting rights',
          color: 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-300'
        };
      case 'enterprise':
        return {
          icon: <Shield className="h-5 w-5 text-green-500" />,
          title: 'Enterprise Tier',
          description: 'Full API access and team management',
          color: 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-300'
        };
      default:
        return {
          icon: <Shield className="h-5 w-5 text-slate-500" />,
          title: 'Basic Tier',
          description: 'Standard platform access',
          color: 'bg-slate-100 text-slate-800 dark:bg-slate-900/20 dark:text-slate-300'
        };
    }
  };

  const tierInfo = getTierInfo();

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
                    <div className="flex items-center gap-2 text-sm text-green-700 dark:text-green-300">
                      <code className="font-mono bg-green-100 dark:bg-green-900/30 px-2 py-1 rounded">
                        {formatAddress(walletAddress || address || '')}
                      </code>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => setShowAddress(!showAddress)}
                        className="h-6 w-6 p-0"
                      >
                        {showAddress ? <EyeOff className="h-3 w-3" /> : <Eye className="h-3 w-3" />}
                      </Button>
                    </div>
                  </div>
                </div>
                <div className="flex items-center gap-3">
                  <Badge className={tierInfo.color}>
                    {tierInfo.icon}
                    <span className="ml-2">{tierInfo.title}</span>
                  </Badge>
                  {hasApiAccess && (
                    <Badge className="bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-300">
                      <Key className="h-3 w-3 mr-1" />
                      API Access
                    </Badge>
                  )}
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
                          {tierInfo.icon}
                          Your Web3 Access Level
                        </CardTitle>
                      </CardHeader>
                      <CardContent>
                        <div className="space-y-4">
                          <div className="p-4 bg-slate-50 dark:bg-slate-800 rounded-lg">
                            <div className="flex items-center justify-between mb-2">
                              <h4 className="font-semibold text-slate-900 dark:text-slate-100">
                                {tierInfo.title}
                              </h4>
                              <Badge className={tierInfo.color}>
                                Active
                              </Badge>
                            </div>
                            <p className="text-sm text-slate-600 dark:text-slate-400">
                              {tierInfo.description}
                            </p>
                          </div>

                          {/* Quick Stats */}
                          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                            <div className="text-center p-3 bg-purple-50 dark:bg-purple-900/20 rounded-lg">
                              <Crown className="h-6 w-6 text-purple-500 mx-auto mb-1" />
                              <div className="text-sm font-medium text-purple-900 dark:text-purple-100">
                                {permissions.filter(p => p.source === 'nft').length}
                              </div>
                              <div className="text-xs text-purple-600 dark:text-purple-400">
                                NFT Perms
                              </div>
                            </div>
                            <div className="text-center p-3 bg-orange-50 dark:bg-orange-900/20 rounded-lg">
                              <Zap className="h-6 w-6 text-orange-500 mx-auto mb-1" />
                              <div className="text-sm font-medium text-orange-900 dark:text-orange-100">
                                {permissions.filter(p => p.source === 'token').length}
                              </div>
                              <div className="text-xs text-orange-600 dark:text-orange-400">
                                Token Perms
                              </div>
                            </div>
                            <div className="text-center p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
                              <Users className="h-6 w-6 text-blue-500 mx-auto mb-1" />
                              <div className="text-sm font-medium text-blue-900 dark:text-blue-100">
                                {permissions.filter(p => p.source === 'dao').length}
                              </div>
                              <div className="text-xs text-blue-600 dark:text-blue-400">
                                DAO Perms
                              </div>
                            </div>
                            <div className="text-center p-3 bg-green-50 dark:bg-green-900/20 rounded-lg">
                              <Shield className="h-6 w-6 text-green-500 mx-auto mb-1" />
                              <div className="text-sm font-medium text-green-900 dark:text-green-100">
                                {permissions.length}
                              </div>
                              <div className="text-xs text-green-600 dark:text-green-400">
                                Total Perms
                              </div>
                            </div>
                          </div>
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
                            onClick={refreshPermissions}
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
                          {hasApiAccess && (
                            <Button
                              variant="outline"
                              onClick={() => setActiveTab('api')}
                              className="flex items-center gap-2"
                            >
                              <Key className="h-4 w-4" />
                              Manage API Keys
                            </Button>
                          )}
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

                    {/* Email Integration */}
                    <Card>
                      <CardHeader>
                        <CardTitle className="flex items-center gap-2">
                          <Link className="h-5 w-5 text-orange-500" />
                          Email Integration
                        </CardTitle>
                        <p className="text-sm text-slate-600 dark:text-slate-400">
                          Link your email account for enhanced account recovery and notifications.
                        </p>
                      </CardHeader>
                      <CardContent>
                        <EmailLinking 
                          showAsDialog={false}
                          onLinkSuccess={() => refreshPermissions()}
                        />
                      </CardContent>
                    </Card>

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