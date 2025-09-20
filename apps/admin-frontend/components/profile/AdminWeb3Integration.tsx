'use client';

import { useState, useEffect } from 'react';
import { useAccount, useDisconnect } from 'wagmi';
import { 
  Wallet, 
  Shield, 
  Link, 
  Unlink, 
  Crown, 
  AlertTriangle, 
  CheckCircle, 
  Copy,
  ExternalLink,
  RefreshCcw
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Separator } from '@/components/ui/separator';
import { AdminWalletAuth } from '@/components/auth/AdminWalletAuth';
import { toast } from 'react-hot-toast';

interface WalletPermission {
  permission: string;
  source: 'manual' | 'nft' | 'token' | 'dao';
  expires_at?: string;
  metadata?: Record<string, any>;
}

interface AdminWeb3IntegrationProps {
  walletAddress?: string;
  email?: string;
  permissions?: string[];
}

export function AdminWeb3Integration({ 
  walletAddress: initialWalletAddress, 
  email: initialEmail,
  permissions: initialPermissions 
}: AdminWeb3IntegrationProps) {
  const { address, isConnected } = useAccount();
  const { disconnect } = useDisconnect();
  
  const [walletAddress, setWalletAddress] = useState(initialWalletAddress);
  const [email, setEmail] = useState(initialEmail);
  const [permissions, setPermissions] = useState<WalletPermission[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [emailLinkStatus, setEmailLinkStatus] = useState<'linked' | 'unlinked' | 'pending'>('unlinked');

  useEffect(() => {
    if (address && isConnected) {
      setWalletAddress(address);
      fetchWalletPermissions(address);
      checkEmailLinkStatus(address);
    }
  }, [address, isConnected]);

  useEffect(() => {
    if (initialPermissions) {
      const walletPerms = initialPermissions.map(permission => ({
        permission,
        source: 'manual' as const,
        expires_at: undefined,
        metadata: {}
      }));
      setPermissions(walletPerms);
    }
  }, [initialPermissions]);

  const fetchWalletPermissions = async (address: string) => {
    try {
      setIsLoading(true);
      const response = await fetch('/api/auth/web3/permissions', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ wallet_address: address }),
      });

      if (response.ok) {
        const data = await response.json();
        const walletPerms = data.permissions?.map((permission: string) => ({
          permission,
          source: 'manual' as const,
          expires_at: undefined,
          metadata: {}
        })) || [];
        setPermissions(walletPerms);
      }
    } catch (error) {
      console.error('Failed to fetch wallet permissions:', error);
      toast.error('Failed to fetch wallet permissions');
    } finally {
      setIsLoading(false);
    }
  };

  const checkEmailLinkStatus = async (address: string) => {
    try {
      const response = await fetch('/api/auth/web3/email-status', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ wallet_address: address }),
      });

      if (response.ok) {
        const data = await response.json();
        setEmailLinkStatus(data.linked ? 'linked' : 'unlinked');
        if (data.email) {
          setEmail(data.email);
        }
      }
    } catch (error) {
      console.error('Failed to check email link status:', error);
    }
  };

  const handleLinkEmail = async () => {
    if (!walletAddress) return;

    try {
      setIsLoading(true);
      setEmailLinkStatus('pending');
      
      const response = await fetch('/api/auth/web3/link-email', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ 
          wallet_address: walletAddress,
          email: email || 'admin@epsx.io' // Default admin email
        }),
      });

      if (response.ok) {
        setEmailLinkStatus('linked');
        toast.success('Email linked to wallet successfully');
      } else {
        setEmailLinkStatus('unlinked');
        toast.error('Failed to link email to wallet');
      }
    } catch (error) {
      setEmailLinkStatus('unlinked');
      toast.error('Failed to link email to wallet');
    } finally {
      setIsLoading(false);
    }
  };

  const handleUnlinkEmail = async () => {
    if (!walletAddress) return;

    try {
      setIsLoading(true);
      
      const response = await fetch('/api/auth/web3/unlink-email', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ wallet_address: walletAddress }),
      });

      if (response.ok) {
        setEmailLinkStatus('unlinked');
        setEmail(undefined);
        toast.success('Email unlinked from wallet');
      } else {
        toast.error('Failed to unlink email from wallet');
      }
    } catch (error) {
      toast.error('Failed to unlink email from wallet');
    } finally {
      setIsLoading(false);
    }
  };

  const copyWalletAddress = () => {
    if (walletAddress) {
      navigator.clipboard.writeText(walletAddress);
      toast.success('Wallet address copied to clipboard');
    }
  };

  const formatAddress = (addr: string) => {
    return `${addr.slice(0, 6)}...${addr.slice(-4)}`;
  };

  const getAdminLevel = () => {
    if (permissions.some(p => p.permission === 'admin:*:*')) return 'Super Admin';
    if (permissions.some(p => p.permission.includes('admin:web3:manage'))) return 'Web3 Manager';
    if (permissions.some(p => p.permission.startsWith('admin:'))) return 'Admin';
    return 'No Admin Access';
  };

  const getAdminLevelColor = () => {
    const level = getAdminLevel();
    switch (level) {
      case 'Super Admin': return 'bg-yellow-100 text-yellow-800 border-yellow-300';
      case 'Web3 Manager': return 'bg-blue-100 text-blue-800 border-blue-300';
      case 'Admin': return 'bg-green-100 text-green-800 border-green-300';
      default: return 'bg-red-100 text-red-800 border-red-300';
    }
  };

  const adminPermissions = permissions.filter(p => p.permission.startsWith('admin:'));

  return (
    <div className="space-y-6">
      {/* Wallet Connection Status */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Wallet className="h-5 w-5" />
            Wallet Connection
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {!isConnected ? (
            <div className="text-center py-6">
              <AdminWalletAuth 
                className="w-full"
                onAuthSuccess={(address) => {
                  setWalletAddress(address);
                  toast.success('Wallet connected successfully');
                }}
                onAuthError={(error) => {
                  toast.error(error);
                }}
              />
            </div>
          ) : (
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div className="space-y-1">
                  <p className="text-sm font-medium">Connected Wallet</p>
                  <div className="flex items-center gap-2">
                    <code className="text-sm bg-gray-100 px-2 py-1 rounded">
                      {formatAddress(walletAddress || '')}
                    </code>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={copyWalletAddress}
                      className="h-6 w-6 p-0"
                    >
                      <Copy className="h-3 w-3" />
                    </Button>
                  </div>
                </div>
                <Button
                  variant="outline"
                  onClick={() => disconnect()}
                  className="text-red-600 hover:text-red-700"
                >
                  Disconnect
                </Button>
              </div>

              <Separator />

              {/* Admin Level Badge */}
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <Crown className="h-4 w-4 text-yellow-500" />
                  <span className="text-sm font-medium">Admin Level</span>
                </div>
                <Badge className={getAdminLevelColor()}>
                  {getAdminLevel()}
                </Badge>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Email Integration */}
      {isConnected && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Link className="h-5 w-5" />
              Email Integration
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <div className="space-y-1">
                <p className="text-sm font-medium">Email Status</p>
                <div className="flex items-center gap-2">
                  {emailLinkStatus === 'linked' && (
                    <>
                      <CheckCircle className="h-4 w-4 text-green-500" />
                      <span className="text-sm text-green-600">
                        {email || 'Email linked'}
                      </span>
                    </>
                  )}
                  {emailLinkStatus === 'unlinked' && (
                    <>
                      <AlertTriangle className="h-4 w-4 text-orange-500" />
                      <span className="text-sm text-orange-600">Not linked</span>
                    </>
                  )}
                  {emailLinkStatus === 'pending' && (
                    <>
                      <RefreshCcw className="h-4 w-4 text-blue-500 animate-spin" />
                      <span className="text-sm text-blue-600">Linking...</span>
                    </>
                  )}
                </div>
              </div>
              
              {emailLinkStatus === 'unlinked' && (
                <Button
                  onClick={handleLinkEmail}
                  disabled={isLoading}
                  className="flex items-center gap-2"
                >
                  <Link className="h-4 w-4" />
                  Link Email
                </Button>
              )}
              
              {emailLinkStatus === 'linked' && (
                <Button
                  variant="outline"
                  onClick={handleUnlinkEmail}
                  disabled={isLoading}
                  className="flex items-center gap-2 text-orange-600 hover:text-orange-700"
                >
                  <Unlink className="h-4 w-4" />
                  Unlink
                </Button>
              )}
            </div>

            {emailLinkStatus === 'unlinked' && (
              <Alert>
                <AlertTriangle className="h-4 w-4" />
                <AlertDescription>
                  Linking your email allows for backup authentication and notifications.
                </AlertDescription>
              </Alert>
            )}
          </CardContent>
        </Card>
      )}

      {/* Admin Permissions */}
      {isConnected && adminPermissions.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Shield className="h-5 w-5" />
              Admin Permissions
              <Badge variant="outline" className="ml-auto">
                {adminPermissions.length}
              </Badge>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {adminPermissions.map((permission, index) => (
                <div
                  key={index}
                  className="flex items-center justify-between p-3 bg-gray-50 rounded-lg"
                >
                  <div className="flex items-center gap-3">
                    <Shield className="h-4 w-4 text-blue-500" />
                    <div>
                      <code className="text-sm font-mono">
                        {permission.permission}
                      </code>
                      {permission.expires_at && (
                        <p className="text-xs text-gray-500 mt-1">
                          Expires: {new Date(permission.expires_at).toLocaleDateString()}
                        </p>
                      )}
                    </div>
                  </div>
                  <Badge variant="outline" className="text-xs">
                    {permission.source}
                  </Badge>
                </div>
              ))}
            </div>

            {adminPermissions.length === 0 && (
              <div className="text-center py-6 text-gray-500">
                <Shield className="h-8 w-8 mx-auto mb-2 opacity-50" />
                <p>No admin permissions found</p>
                <p className="text-sm">Contact your system administrator</p>
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {/* Wallet Security Info */}
      {isConnected && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Shield className="h-5 w-5" />
              Security Information
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
              <div className="space-y-2">
                <h4 className="font-medium">Authentication Method</h4>
                <p className="text-gray-600">SIWE (Sign-In with Ethereum)</p>
              </div>
              <div className="space-y-2">
                <h4 className="font-medium">Security Level</h4>
                <p className="text-gray-600">Cryptographic Signature</p>
              </div>
              <div className="space-y-2">
                <h4 className="font-medium">Session Type</h4>
                <p className="text-gray-600">Wallet-based</p>
              </div>
              <div className="space-y-2">
                <h4 className="font-medium">Network</h4>
                <p className="text-gray-600">Ethereum Mainnet</p>
              </div>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}