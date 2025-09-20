'use client';

import { useEffect, useState } from 'react';
import { useAccount } from 'wagmi';
import { ConnectButton } from '@rainbow-me/rainbowkit';
import { useRouter, useSearchParams } from 'next/navigation';
import { useWeb3Auth } from '@/components/providers/Web3AuthProvider';
import { toast } from 'react-hot-toast';

export default function Web3LoginPage() {
  const { isConnected, address } = useAccount();
  const { isAuthenticated, isLoading, login } = useWeb3Auth();
  const router = useRouter();
  const searchParams = useSearchParams();
  const [isCheckingPermissions, setIsCheckingPermissions] = useState(false);

  const returnUrl = searchParams.get('return_url') || '/permissions/web3';
  const reason = searchParams.get('reason');

  // Handle authentication success
  useEffect(() => {
    if (isAuthenticated && !isLoading) {
      console.log('✅ Web3 Login: Authentication successful, redirecting to:', returnUrl);
      toast.success('Login successful!');
      router.push(returnUrl);
    }
  }, [isAuthenticated, isLoading, returnUrl, router]);

  // Handle wallet connection
  useEffect(() => {
    if (isConnected && address && !isAuthenticated && !isLoading) {
      handleWalletConnected();
    }
  }, [isConnected, address, isAuthenticated, isLoading]);

  const handleWalletConnected = async () => {
    if (!address) return;
    
    setIsCheckingPermissions(true);
    try {
      console.log('🔍 Web3 Login: Checking permissions for wallet:', address);
      
      // Check if wallet has admin permissions
      const response = await fetch('/api/auth/web3/permissions', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ wallet_address: address }),
        credentials: 'include',
      });

      if (!response.ok) {
        throw new Error('Failed to check wallet permissions');
      }

      const data = await response.json();
      
      if (data.has_admin_access) {
        console.log('✅ Web3 Login: Wallet has admin access, authenticating...');
        await login();
      } else {
        console.warn('❌ Web3 Login: Wallet lacks admin permissions');
        toast.error('This wallet does not have admin permissions');
      }
    } catch (error) {
      console.error('💥 Web3 Login: Permission check failed:', error);
      toast.error('Failed to verify wallet permissions');
    } finally {
      setIsCheckingPermissions(false);
    }
  };

  const getReasonMessage = () => {
    switch (reason) {
      case 'no-session':
        return 'Your session has expired. Please connect your wallet to continue.';
      case 'no-permission':
        return 'You need admin permissions to access this area.';
      case 'wallet-disconnected':
        return 'Your wallet was disconnected. Please reconnect to continue.';
      default:
        return 'Connect your Web3 wallet to access the admin dashboard.';
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 flex items-center justify-center p-6">
      <div className="max-w-md w-full">
        {/* Hero Section */}
        <div className="text-center mb-8">
          <div className="relative mb-6">
            <h1 className="text-4xl font-bold bg-gradient-to-r from-yellow-400 via-orange-500 to-pink-500 bg-clip-text text-transparent mb-2">
              🌐 Web3 Admin
            </h1>
            <div className="absolute -top-1 -right-1 text-lg animate-pulse">⚡</div>
          </div>
          <p className="text-gray-600 dark:text-gray-300 text-sm leading-relaxed">
            {getReasonMessage()}
          </p>
        </div>

        {/* Login Card */}
        <div className="bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm rounded-3xl p-8 shadow-2xl border border-white/30">
          <div className="space-y-6">
            {/* Status Display */}
            <div className="space-y-3">
              {isLoading || isCheckingPermissions ? (
                <div className="flex items-center justify-center py-4">
                  <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-orange-500"></div>
                  <span className="ml-3 text-sm text-gray-600 dark:text-gray-400">
                    {isCheckingPermissions ? 'Checking permissions...' : 'Authenticating...'}
                  </span>
                </div>
              ) : isConnected && address ? (
                <div className="text-center py-4">
                  <div className="flex items-center justify-center mb-2">
                    <div className="w-3 h-3 bg-green-500 rounded-full mr-2"></div>
                    <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                      Wallet Connected
                    </span>
                  </div>
                  <p className="text-xs text-gray-500 dark:text-gray-400 font-mono">
                    {address.slice(0, 6)}...{address.slice(-4)}
                  </p>
                </div>
              ) : (
                <div className="text-center py-4">
                  <div className="flex items-center justify-center mb-2">
                    <div className="w-3 h-3 bg-gray-400 rounded-full mr-2"></div>
                    <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                      No Wallet Connected
                    </span>
                  </div>
                  <p className="text-xs text-gray-500 dark:text-gray-400">
                    Please connect your wallet to continue
                  </p>
                </div>
              )}
            </div>

            {/* Connect Button */}
            <div className="flex justify-center">
              <ConnectButton.Custom>
                {({
                  account,
                  chain,
                  openAccountModal,
                  openChainModal,
                  openConnectModal,
                  authenticationStatus,
                  mounted,
                }) => {
                  const ready = mounted && authenticationStatus !== 'loading';
                  const connected =
                    ready &&
                    account &&
                    chain &&
                    (!authenticationStatus ||
                      authenticationStatus === 'authenticated');

                  return (
                    <div
                      {...(!ready && {
                        'aria-hidden': true,
                        'style': {
                          opacity: 0,
                          pointerEvents: 'none',
                          userSelect: 'none',
                        },
                      })}
                      className="w-full"
                    >
                      {(() => {
                        if (!connected) {
                          return (
                            <button
                              onClick={openConnectModal}
                              type="button"
                              className="w-full px-6 py-4 bg-gradient-to-r from-yellow-400 via-orange-500 to-pink-500 text-white font-semibold rounded-2xl hover:from-yellow-500 hover:via-orange-600 hover:to-pink-600 shadow-lg hover:shadow-xl transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed"
                              disabled={isLoading || isCheckingPermissions}
                            >
                              {isLoading || isCheckingPermissions ? (
                                <span className="flex items-center justify-center">
                                  <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
                                  Connecting...
                                </span>
                              ) : (
                                '🔗 Connect Wallet'
                              )}
                            </button>
                          );
                        }

                        if (chain.unsupported) {
                          return (
                            <button
                              onClick={openChainModal}
                              type="button"
                              className="w-full px-6 py-4 bg-gradient-to-r from-red-400 to-red-600 text-white font-semibold rounded-2xl hover:from-red-500 hover:to-red-700 shadow-lg transition-all duration-200"
                            >
                              ⚠️ Wrong Network
                            </button>
                          );
                        }

                        return (
                          <div className="space-y-3">
                            <button
                              onClick={openAccountModal}
                              type="button"
                              className="w-full px-6 py-4 bg-gradient-to-r from-green-400 to-green-600 text-white font-semibold rounded-2xl hover:from-green-500 hover:to-green-700 shadow-lg transition-all duration-200"
                            >
                              ✅ {account.displayName}
                              {account.displayBalance
                                ? ` (${account.displayBalance})`
                                : ''}
                            </button>
                            
                            {!isAuthenticated && (
                              <p className="text-xs text-center text-gray-500 dark:text-gray-400">
                                Verifying admin permissions...
                              </p>
                            )}
                          </div>
                        );
                      })()}
                    </div>
                  );
                }}
              </ConnectButton.Custom>
            </div>

            {/* Help Text */}
            <div className="text-center">
              <p className="text-xs text-gray-500 dark:text-gray-400 leading-relaxed">
                Only wallets with admin permissions can access the admin dashboard. 
                If you believe you should have access, please contact your system administrator.
              </p>
            </div>
          </div>
        </div>

        {/* Return URL Display */}
        {returnUrl !== '/permissions/web3' && (
          <div className="mt-4 text-center">
            <p className="text-xs text-gray-400 dark:text-gray-500">
              You will be redirected to: {returnUrl}
            </p>
          </div>
        )}
      </div>
    </div>
  );
}