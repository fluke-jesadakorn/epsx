'use client';

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
import { useState, useEffect } from 'react';
import { toast } from 'react-hot-toast';
import { useAccount, useDisconnect } from 'wagmi';

import { AdminWalletAuth } from '@/components/auth/AdminWalletAuth';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Separator } from '@/components/ui/separator';
import { apiFetch } from '@/lib/api-fetch';
import { useSharedAuth } from '@/shared/components/auth/SharedOpenIDWeb3Provider';
import { UserWalletDisplay, UserTierBadge, UserAuthStatus, UserPermissionsDisplay } from '@/shared/components/display/UserDisplay';

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

/**
 *
 * @param root0
 * @param root0.walletAddress
 * @param root0.email
 * @param root0.permissions
 */
export function AdminWeb3Integration({ 
  walletAddress: initialWalletAddress, 
  email: initialEmail,
  permissions: initialPermissions 
}: AdminWeb3IntegrationProps) {
  const { address, isConnected } = useAccount();
  const { disconnect } = useDisconnect();
  const { user, isAuthenticated, logout } = useSharedAuth();
  
  const [email, setEmail] = useState(initialEmail || user?.email);
  const [isLoading, setIsLoading] = useState(false);
  const [emailLinkStatus, setEmailLinkStatus] = useState<'linked' | 'unlinked' | 'pending'>('unlinked');

  // Use wallet address from shared auth if available
  const walletAddress = user?.wallet_address || address || initialWalletAddress;

  useEffect(() => {
    if (walletAddress) {
      checkEmailLinkStatus(walletAddress);
    }
  }, [walletAddress, isConnected]);

  // Permission fetching is now handled by shared authentication system

  const checkEmailLinkStatus = async (address: string) => {
    try {
      const data = await apiFetch('/api/auth/web3/email-status', {
        method: 'POST',
        body: JSON.stringify({ wallet_address: address }),
      });

      setEmailLinkStatus(data.linked ? 'linked' : 'unlinked');
      if (data.email) {
        setEmail(data.email);
      }
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Failed to check email link status:', _error);
    }
  };

  const handleLinkEmail = async () => {
    if (!walletAddress) {return;}

    try {
      setIsLoading(true);
      setEmailLinkStatus('pending');

      await apiFetch('/api/auth/web3/link-email', {
        method: 'POST',
        body: JSON.stringify({
          wallet_address: walletAddress,
          email: email || 'admin@epsx.io' // Default admin email
        }),
      });

      setEmailLinkStatus('linked');
      toast.success('Email linked to wallet successfully');
    } catch (_error) {
      setEmailLinkStatus('unlinked');
      toast.error('Failed to link email to wallet');
    } finally {
      setIsLoading(false);
    }
  };

  const handleUnlinkEmail = async () => {
    if (!walletAddress) {return;}

    try {
      setIsLoading(true);

      await apiFetch('/api/auth/web3/unlink-email', {
        method: 'POST',
        body: JSON.stringify({ wallet_address: walletAddress }),
      });

      setEmailLinkStatus('unlinked');
      setEmail(undefined);
      toast.success('Email unlinked from wallet');
    } catch (_error) {
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

  // All display logic is now handled by shared components

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
          {!isAuthenticated ? (
            <div className="text-center py-6">
              <AdminWalletAuth 
                className="w-full"
                onAuthSuccess={(address) => {
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
                  <UserWalletDisplay showFullAddress={false} />
                </div>
                <Button
                  variant="outline"
                  onClick={() => logout()}
                  className="text-red-600 hover:text-red-700"
                >
                  Disconnect
                </Button>
              </div>

              <Separator />

              {/* Admin Level and Status */}
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <Crown className="h-4 w-4 text-yellow-500" />
                  <span className="text-sm font-medium">Admin Level</span>
                </div>
                <div className="flex items-center gap-2">
                  <UserTierBadge />
                  <UserAuthStatus />
                </div>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Email Integration */}
      {isAuthenticated && (
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
      {isAuthenticated && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Shield className="h-5 w-5" />
              Admin Permissions
            </CardTitle>
          </CardHeader>
          <CardContent>
            <UserPermissionsDisplay maxDisplay={20} />
          </CardContent>
        </Card>
      )}

      {/* Wallet Security Info */}
      {isAuthenticated && (
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