'use client';

import { useEffect, useState } from 'react';
import { useAccount } from 'wagmi';
import { ConnectButton } from '@rainbow-me/rainbowkit';
import { useRouter, useSearchParams } from 'next/navigation';
import { useSharedAuth } from '@/shared/components/auth/SharedOpenIDWeb3Provider';
import { useAdminWeb3Context } from '../../providers/Web3Provider';
import { toast } from 'react-hot-toast';

export default function AuthPage() {
  const { isInitialized } = useAdminWeb3Context();
  const { requestChallenge, authenticateWithWallet, isLoading, hasPermissionForDisplay } = useSharedAuth();
  const router = useRouter();
  const searchParams = useSearchParams();
  
  // Always call useAccount but handle when Web3 isn't ready
  let accountData;
  try {
    accountData = useAccount();
  } catch (error) {
    accountData = { isConnected: false, address: undefined };
  }
  
  const { isConnected, address } = accountData;
  const isWeb3Ready = isInitialized && isConnected;
  
  // 3-step authentication state
  const [authStep, setAuthStep] = useState<'connect' | 'sign' | 'authenticating' | 'success'>('connect');
  const [challenge, setChallenge] = useState<{nonce: string, message: string, wallet_address: string} | null>(null);
  const [error, setError] = useState<string>('');
  const [mounted, setMounted] = useState(false);

  const returnUrl = searchParams.get('return_url') || '/dashboard';
  const reason = searchParams.get('reason');

  useEffect(() => {
    setMounted(true);
  }, []);

  // Check if user already has admin permissions and redirect
  useEffect(() => {
    if (mounted && isInitialized && isConnected && hasPermissionForDisplay('admin:*:*')) {
      router.push('/dashboard');
    }
  }, [mounted, isInitialized, isConnected, hasPermissionForDisplay, router]);

  // Step 1: When wallet connects, move to sign step
  useEffect(() => {
    if (isConnected && address && authStep === 'connect') {
      console.log('🔗 Step 1 Complete: Wallet connected:', address);
      setAuthStep('sign');
      setError('');
    }
  }, [isConnected, address, authStep]);

  // Step 2: Request challenge and sign message
  const handleSignMessage = async () => {
    if (!address) return;
    
    try {
      setError('');
      console.log('🔑 Step 2: Requesting challenge for:', address);
      
      // Request challenge from backend
      const challengeData = await requestChallenge(address);
      setChallenge(challengeData);
      
      console.log('📝 Step 2: Challenge received, requesting signature');
      
      // Request signature from wallet
      if (typeof window === 'undefined' || !window.ethereum) {
        throw new Error('MetaMask is not available. Please install MetaMask to continue.');
      }
      
      const signature = await window.ethereum.request({
        method: 'personal_sign',
        params: [challengeData.message, address],
      });

      console.log('✍️ Step 2: Signature received, authenticating...');
      setAuthStep('authenticating');
      
      // Step 3: Authenticate with backend
      const result = await authenticateWithWallet(
        address,
        signature,
        challengeData.message,
        challengeData.nonce
      );

      if (result.success) {
        console.log('✅ Step 3 Complete: Authentication successful');
        setAuthStep('success');
        toast.success('Admin authentication successful!');
        
        setTimeout(() => {
          router.push(returnUrl);
        }, 1000);
      } else {
        throw new Error(result.error || 'Authentication failed');
      }
      
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Authentication failed';
      console.error('❌ Authentication error:', errorMessage);
      setError(errorMessage);
      setAuthStep('sign');
      toast.error(errorMessage);
    }
  };

  // Reset authentication flow
  const resetAuthFlow = () => {
    setAuthStep('connect');
    setChallenge(null);
    setError('');
  };

  const getReasonMessage = () => {
    switch (reason) {
      case 'no-session':
        return 'Your session has expired. Please connect your wallet to continue.';
      case 'no-admin-permissions':
        return 'Your current wallet does not have admin permissions. Please connect a wallet with admin access.';
      default:
        return 'Connect your Web3 wallet and sign to verify admin permissions.';
    }
  };

  if (!mounted || !isInitialized) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-gray-600 dark:text-gray-400">
          {!mounted ? "Loading..." : "Initializing Web3..."}
        </div>
      </div>
    );
  }

  return (
    <div className="flex items-center justify-center min-h-screen p-6">
      <div className="max-w-md w-full">
        {/* Hero Section */}
        <div className="text-center mb-8">
          <div className="relative mb-6">
            <h1 className="text-4xl font-bold bg-gradient-to-r from-yellow-400 via-orange-500 to-pink-500 bg-clip-text text-transparent mb-2">
              🔐 Admin Access
            </h1>
            <div className="absolute -top-1 -right-1 text-lg">⚡</div>
          </div>
          <p className="text-gray-600 dark:text-gray-300 text-sm leading-relaxed">
            {getReasonMessage()}
          </p>
        </div>

        {/* Authentication Card */}
        <div className="bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm rounded-3xl p-8 shadow-2xl border border-white/30">
          <div className="space-y-6">
            {/* 3-Step Status Display */}
            <div className="space-y-4">
              {/* Step Progress */}
              <div className="flex justify-center space-x-4 mb-6">
                <div className={`flex items-center space-x-2 ${authStep !== 'connect' ? 'text-green-600' : 'text-gray-500'}`}>
                  <div className={`w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold ${authStep !== 'connect' ? 'bg-green-500 text-white' : 'bg-gray-300 text-gray-600'}`}>1</div>
                  <span className="text-sm font-medium">Connect</span>
                </div>
                <div className={`flex items-center space-x-2 ${authStep === 'sign' || authStep === 'authenticating' || authStep === 'success' ? 'text-blue-600' : 'text-gray-500'}`}>
                  <div className={`w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold ${authStep === 'sign' || authStep === 'authenticating' || authStep === 'success' ? 'bg-blue-500 text-white' : 'bg-gray-300 text-gray-600'}`}>2</div>
                  <span className="text-sm font-medium">Sign</span>
                </div>
                <div className={`flex items-center space-x-2 ${authStep === 'success' ? 'text-green-600' : 'text-gray-500'}`}>
                  <div className={`w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold ${authStep === 'success' ? 'bg-green-500 text-white' : 'bg-gray-300 text-gray-600'}`}>3</div>
                  <span className="text-sm font-medium">Access</span>
                </div>
              </div>

              {/* Current Step Status */}
              <div className="text-center py-4">
                {authStep === 'connect' && (
                  <>
                    <div className="flex items-center justify-center mb-2">
                      <div className="w-3 h-3 bg-gray-400 rounded-full mr-2"></div>
                      <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                        Step 1: Connect Your Admin Wallet
                      </span>
                    </div>
                    <p className="text-xs text-gray-500 dark:text-gray-400">
                      Connect your Web3 wallet that has admin permissions
                    </p>
                  </>
                )}

                {authStep === 'sign' && address && (
                  <>
                    <div className="flex items-center justify-center mb-2">
                      <div className="w-3 h-3 bg-green-500 rounded-full mr-2"></div>
                      <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                        Step 2: Sign Admin Authentication
                      </span>
                    </div>
                    <p className="text-xs text-gray-500 dark:text-gray-400 font-mono mb-2">
                      Connected: {address.slice(0, 6)}...{address.slice(-4)}
                    </p>
                    <p className="text-xs text-gray-500 dark:text-gray-400">
                      Sign the message to verify admin permissions
                    </p>
                  </>
                )}

                {authStep === 'authenticating' && (
                  <>
                    <div className="flex items-center justify-center mb-2">
                      <div className="w-3 h-3 bg-blue-500 rounded-full mr-2 animate-pulse"></div>
                      <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                        Step 3: Verifying Admin Access...
                      </span>
                    </div>
                    <p className="text-xs text-gray-500 dark:text-gray-400">
                      Checking admin permissions with backend
                    </p>
                  </>
                )}

                {authStep === 'success' && (
                  <>
                    <div className="flex items-center justify-center mb-2">
                      <div className="w-3 h-3 bg-green-500 rounded-full mr-2"></div>
                      <span className="text-sm font-medium text-green-700 dark:text-green-300">
                        ✅ Admin Access Granted!
                      </span>
                    </div>
                    <p className="text-xs text-gray-500 dark:text-gray-400">
                      Redirecting to admin dashboard...
                    </p>
                  </>
                )}
              </div>

              {/* Error Display */}
              {error && (
                <div className="bg-red-50 border border-red-200 rounded-lg p-4 text-center space-y-3">
                  <p className="text-sm text-red-800">{error}</p>
                  <button
                    onClick={resetAuthFlow}
                    type="button"
                    className="px-4 py-2 bg-red-500 text-white text-sm font-medium rounded-lg hover:bg-red-600"
                  >
                    Try Again
                  </button>
                </div>
              )}
            </div>

            {/* Action Button */}
            <div className="flex justify-center">
              {authStep === 'connect' && (
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
                        <button
                          onClick={openConnectModal}
                          type="button"
                          className="w-full px-6 py-4 bg-gradient-to-r from-yellow-400 via-orange-500 to-pink-500 text-white font-semibold rounded-2xl hover:from-yellow-500 hover:via-orange-600 hover:to-pink-600 shadow-lg hover:shadow-xl"
                        >
                          🔗 Connect Admin Wallet
                        </button>
                      </div>
                    );
                  }}
                </ConnectButton.Custom>
              )}

              {authStep === 'sign' && (
                <button
                  onClick={handleSignMessage}
                  type="button"
                  className="w-full px-6 py-4 bg-gradient-to-r from-blue-400 to-blue-600 text-white font-semibold rounded-2xl hover:from-blue-500 hover:to-blue-700 shadow-lg hover:shadow-xl"
                >
                  ✍️ Sign Admin Message
                </button>
              )}

              {authStep === 'authenticating' && (
                <button
                  type="button"
                  disabled
                  className="w-full px-6 py-4 bg-gradient-to-r from-blue-400 to-blue-600 opacity-50 text-white font-semibold rounded-2xl shadow-lg cursor-not-allowed"
                >
                  <span className="flex items-center justify-center">
                    <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
                    Verifying Admin Access...
                  </span>
                </button>
              )}

              {authStep === 'success' && (
                <button
                  type="button"
                  disabled
                  className="w-full px-6 py-4 bg-gradient-to-r from-green-400 to-green-600 text-white font-semibold rounded-2xl shadow-lg cursor-not-allowed"
                >
                  ✅ Admin Access Granted!
                </button>
              )}
            </div>

            {/* Help Text */}
            <div className="text-center">
              <p className="text-xs text-gray-500 dark:text-gray-400 leading-relaxed">
                {authStep === 'connect' && 'Only wallets with admin permissions can access the dashboard.'}
                {authStep === 'sign' && 'This signature verifies your admin permissions. No gas fees required.'}
                {authStep === 'authenticating' && 'Please wait while we verify your admin access rights.'}
                {authStep === 'success' && 'Admin authentication complete! Welcome to the dashboard.'}
              </p>
              {(authStep === 'connect' || authStep === 'sign') && (
                <p className="text-xs text-gray-400 dark:text-gray-500 mt-2">
                  Need admin access? Contact your system administrator.
                </p>
              )}
            </div>
          </div>
        </div>

        {/* Return URL Display */}
        {returnUrl !== '/dashboard' && (
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