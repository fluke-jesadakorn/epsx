'use client';

import { ConnectButton } from '@rainbow-me/rainbowkit';
import { useRouter, useSearchParams } from 'next/navigation';
import { useEffect, useRef, useState } from 'react';
import { toast } from 'sonner';
import { useAccount } from 'wagmi';

import { useAdminWeb3Context } from '../../providers/AuthProvider';

import {
    requestWalletChallenge,
    verifyWalletSignature,
} from '@/shared/auth/api';
import { OIDC_KEYS } from '@/shared/auth/storage';
import { useSharedAuth } from '@/shared/components/auth/Provider';

/**
 *
 */
export default function AuthPage() {
  const { isInitialized } = useAdminWeb3Context();
  const {
    authenticateWithDirectApi,
    isLoading,
    hasPermissionForDisplay,
    isAuthenticated,
    user,
  } = useSharedAuth();
  const router = useRouter();
  const searchParams = useSearchParams();

  // Always call useAccount but handle when Web3 isn't ready
  let accountData;
  try {
    // eslint-disable-next-line react-hooks/rules-of-hooks -- Defensive pattern for when WAGMI is not available
    accountData = useAccount();
  } catch (_error) {
    accountData = { isConnected: false, address: undefined };
  }

  const { isConnected, address } = accountData;

  // 3-step authentication state
  const [authStep, setAuthStep] = useState<
    'connect' | 'sign' | 'authenticating' | 'success'
  >('connect');

  // Track if authentication has been completed to prevent re-triggering
  const authCompletedRef = useRef(false);

  // Guarded setAuthStep to prevent unwanted state changes once success is reached
  const setAuthStepGuarded = (
    newStep: 'connect' | 'sign' | 'authenticating' | 'success'
  ) => {
    // If authentication has been completed, don't allow going backwards
    if (authCompletedRef.current && newStep !== 'success') {
      return;
    }
    // If we've already reached success, don't allow going backwards
    if (authStep === 'success' && newStep !== 'success') {
      return;
    }
    setAuthStep(newStep);

    // Mark authentication as completed when we reach success
    if (newStep === 'success') {
      authCompletedRef.current = true;
    }
  };
  const [challenge, setChallenge] = useState<{
    nonce: string;
    message: string;
    wallet_address: string;
  } | null>(null);
  const [error, setError] = useState<string>('');
  const [mounted, setMounted] = useState(false);

  const returnUrl = searchParams.get('return_url') || '/';

  // Decode the return URL if it's URL encoded
  const decodedReturnUrl = decodeURIComponent(returnUrl);

  // Final return URL - ensure we don't redirect to auth pages
  const finalReturnUrl =
    decodedReturnUrl === '/' ||
    decodedReturnUrl === '/auth' ||
    decodedReturnUrl === '/login'
      ? '/'
      : decodedReturnUrl;
  const reason = searchParams.get('reason');

  useEffect(() => {
    setMounted(true);
  }, []);

  // Check if user is already authenticated with admin permissions and redirect
  useEffect(() => {
    if (
      mounted &&
      isAuthenticated &&
      user &&
      hasPermissionForDisplay('admin:*:*')
    ) {
      setAuthStepGuarded('success');

      setTimeout(() => {
        window.location.href = finalReturnUrl;
      }, 1000);
    }
  }, [
    mounted,
    isAuthenticated,
    user,
    hasPermissionForDisplay,
    router,
    finalReturnUrl,
  ]);

  // Step 1: When wallet connects, move to sign step (only if currently on connect step and not authenticated)
  useEffect(() => {
    // Only transition to sign step if we're currently on connect step and wallet is connected but not authenticated
    // Guard: Only proceed if we haven't already completed authentication successfully
    if (
      isConnected &&
      address &&
      authStep === 'connect' &&
      !isAuthenticated &&
      !authCompletedRef.current
    ) {
      setAuthStepGuarded('sign');
      setError('');
    }
  }, [isConnected, address, authStep, isAuthenticated]); // Include all dependencies

  // Separate effect to handle authenticated state changes
  useEffect(() => {
    if (
      isAuthenticated &&
      authStep !== 'success' &&
      !authCompletedRef.current
    ) {
      setAuthStepGuarded('success');
    }
  }, [isAuthenticated, authStep]);

  // Step 2: Request challenge and sign message
  const handleSignMessage = async () => {
    if (!address) {
      return;
    }

    try {
      setError('');

      const challengeData = await requestWalletChallenge(address);
      setChallenge(challengeData);

      if (typeof window === 'undefined' || !window.ethereum) {
        throw new Error(
          'MetaMask is not available. Please install MetaMask to continue.'
        );
      }

      const signature = await window.ethereum.request({
        method: 'personal_sign',
        params: [challengeData.message, address],
      });

      setAuthStepGuarded('authenticating');

      // Step 3: Authenticate with backend using direct API
      const result = await verifyWalletSignature({
        wallet_address: address,
        signature,
        message: challengeData.message,
        nonce: challengeData.nonce,
      });

      if (result.success) {
        // Store access token in unified OpenID localStorage for session validation
        if (result.access_token) {
          try {
            localStorage.setItem(OIDC_KEYS.ACCESS_TOKEN, result.access_token);
          } catch (_error) {
            // Ignore storage errors
          }
        }

        setAuthStepGuarded('success');
        toast.success('Admin authentication successful!');

        try {
          // Update SharedOpenIDWeb3Provider with authenticated user
          await authenticateWithDirectApi({
            wallet_address: result.wallet_address,
            permissions: result.permissions,
            is_new_user: result.is_new_user,
            access_token: result.access_token,
          });

          setTimeout(() => {
            window.location.href = finalReturnUrl;
          }, 1000);
        } catch (authProviderError) {
          setTimeout(() => {
            window.location.href = finalReturnUrl;
          }, 1000);
        }
      } else {
        throw new Error(result.error || 'Authentication failed');
      }
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : 'Authentication failed';
      setError(errorMessage);
      setAuthStepGuarded('sign');
      toast.error(errorMessage);
    }
  };

  // Reset authentication flow
  const resetAuthFlow = () => {
    authCompletedRef.current = false;
    setAuthStepGuarded('connect');
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
        return 'Connect your wallet and sign to verify admin permissions.';
    }
  };

  if (!mounted || !isInitialized) {
    return (
      <div className="flex min-h-screen items-center justify-center">
        <div className="text-gray-600 dark:text-gray-400">
          {!mounted ? 'Loading...' : 'Initializing Web3...'}
        </div>
      </div>
    );
  }

  // If user is already authenticated with admin permissions, show redirecting message
  if (isAuthenticated && user && hasPermissionForDisplay('admin:*:*')) {
    return (
      <div className="flex min-h-screen items-center justify-center">
        <div className="text-center">
          <h2 className="mb-2 text-xl font-semibold text-green-700">
            ✅ Admin Access Granted!
          </h2>
          <p className="text-gray-600 dark:text-gray-400">
            Redirecting to admin dashboard...
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex min-h-screen items-center justify-center p-6">
      <div className="w-full max-w-md">
        {/* Hero Section */}
        <div className="mb-8 text-center">
          <div className="relative mb-6">
            <h1 className="mb-2 bg-gradient-to-r from-yellow-400 via-orange-500 to-pink-500 bg-clip-text text-4xl font-bold text-transparent">
              🔐 Admin Access
            </h1>
            <div className="absolute -top-1 -right-1 text-lg">⚡</div>
          </div>
          <p className="text-sm leading-relaxed text-gray-600 dark:text-gray-300">
            {getReasonMessage()}
          </p>
        </div>

        {/* Authentication Card */}
        <div className="rounded-3xl border border-white/30 bg-white/90 p-8 shadow-2xl backdrop-blur-sm dark:bg-gray-800/90">
          <div className="space-y-6">
            {/* 3-Step Status Display */}
            <div className="space-y-4">
              {/* Step Progress */}
              <div className="mb-6 flex justify-center space-x-4">
                <div
                  className={`flex items-center space-x-2 ${authStep !== 'connect' ? 'text-green-600' : 'text-gray-500'}`}
                >
                  <div
                    className={`flex h-8 w-8 items-center justify-center rounded-full text-xs font-bold ${authStep !== 'connect' ? 'bg-green-500 text-white' : 'bg-gray-300 text-gray-600'}`}
                  >
                    1
                  </div>
                  <span className="text-sm font-medium">Connect</span>
                </div>
                <div
                  className={`flex items-center space-x-2 ${authStep === 'sign' || authStep === 'authenticating' || authStep === 'success' ? 'text-blue-600' : 'text-gray-500'}`}
                >
                  <div
                    className={`flex h-8 w-8 items-center justify-center rounded-full text-xs font-bold ${authStep === 'sign' || authStep === 'authenticating' || authStep === 'success' ? 'bg-blue-500 text-white' : 'bg-gray-300 text-gray-600'}`}
                  >
                    2
                  </div>
                  <span className="text-sm font-medium">Sign</span>
                </div>
                <div
                  className={`flex items-center space-x-2 ${authStep === 'success' ? 'text-green-600' : 'text-gray-500'}`}
                >
                  <div
                    className={`flex h-8 w-8 items-center justify-center rounded-full text-xs font-bold ${authStep === 'success' ? 'bg-green-500 text-white' : 'bg-gray-300 text-gray-600'}`}
                  >
                    3
                  </div>
                  <span className="text-sm font-medium">Access</span>
                </div>
              </div>

              {/* Current Step Status */}
              <div className="py-4 text-center">
                {authStep === 'connect' && (
                  <>
                    <div className="mb-2 flex items-center justify-center">
                      <div className="mr-2 h-3 w-3 rounded-full bg-gray-400"></div>
                      <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                        Step 1: Connect Your Admin Wallet
                      </span>
                    </div>
                    <p className="text-xs text-gray-500 dark:text-gray-400">
                      Connect your wallet that has admin permissions
                    </p>
                  </>
                )}

                {authStep === 'sign' && address && (
                  <>
                    <div className="mb-2 flex items-center justify-center">
                      <div className="mr-2 h-3 w-3 rounded-full bg-green-500"></div>
                      <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                        Step 2: Sign Admin Authentication
                      </span>
                    </div>
                    <p className="mb-2 font-mono text-xs text-gray-500 dark:text-gray-400">
                      Connected: {address.slice(0, 6)}...{address.slice(-4)}
                    </p>
                    <p className="text-xs text-gray-500 dark:text-gray-400">
                      Sign the message to verify admin permissions
                    </p>
                  </>
                )}

                {authStep === 'authenticating' && (
                  <>
                    <div className="mb-2 flex items-center justify-center">
                      <div className="mr-2 h-3 w-3 animate-pulse rounded-full bg-blue-500"></div>
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
                    <div className="mb-2 flex items-center justify-center">
                      <div className="mr-2 h-3 w-3 rounded-full bg-green-500"></div>
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
                <div className="space-y-3 rounded-lg border border-red-200 bg-red-50 p-4 text-center">
                  <p className="text-sm text-red-800">{error}</p>
                  <button
                    onClick={resetAuthFlow}
                    type="button"
                    className="rounded-lg bg-red-500 px-4 py-2 text-sm font-medium text-white hover:bg-red-600"
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
                          style: {
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
                          className="w-full rounded-2xl bg-gradient-to-r from-yellow-400 via-orange-500 to-pink-500 px-6 py-4 font-semibold text-white shadow-lg hover:from-yellow-500 hover:via-orange-600 hover:to-pink-600 hover:shadow-xl"
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
                  className="w-full rounded-2xl bg-gradient-to-r from-blue-400 to-blue-600 px-6 py-4 font-semibold text-white shadow-lg hover:from-blue-500 hover:to-blue-700 hover:shadow-xl"
                >
                  ✍️ Sign Admin Message
                </button>
              )}

              {authStep === 'authenticating' && (
                <button
                  type="button"
                  disabled
                  className="w-full cursor-not-allowed rounded-2xl bg-gradient-to-r from-blue-400 to-blue-600 px-6 py-4 font-semibold text-white opacity-50 shadow-lg"
                >
                  <span className="flex items-center justify-center">
                    <div className="mr-2 h-4 w-4 animate-spin rounded-full border-b-2 border-white"></div>
                    Verifying Admin Access...
                  </span>
                </button>
              )}

              {authStep === 'success' && (
                <button
                  type="button"
                  disabled
                  className="w-full cursor-not-allowed rounded-2xl bg-gradient-to-r from-green-400 to-green-600 px-6 py-4 font-semibold text-white shadow-lg"
                >
                  ✅ Admin Access Granted!
                </button>
              )}
            </div>

            {/* Help Text */}
            <div className="text-center">
              <p className="text-xs leading-relaxed text-gray-500 dark:text-gray-400">
                {authStep === 'connect' &&
                  'Only wallets with admin permissions can access the dashboard.'}
                {authStep === 'sign' &&
                  'This signature verifies your admin permissions. No gas fees required.'}
                {authStep === 'authenticating' &&
                  'Please wait while we verify your admin access rights.'}
                {authStep === 'success' &&
                  'Admin authentication complete! Welcome to the dashboard.'}
              </p>
              {(authStep === 'connect' || authStep === 'sign') && (
                <p className="mt-2 text-xs text-gray-400 dark:text-gray-500">
                  Need admin access? Contact your system administrator.
                </p>
              )}
            </div>
          </div>
        </div>

        {/* Return URL Display */}
        {finalReturnUrl !== '/' && (
          <div className="mt-4 text-center">
            <p className="text-xs text-gray-400 dark:text-gray-500">
              You will be redirected to: {finalReturnUrl}
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
