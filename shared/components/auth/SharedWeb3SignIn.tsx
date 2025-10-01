// ============================================================================
// SHARED WEB3 SIGNIN COMPONENT
// Unified Web3 authentication component for both frontend and admin-frontend
// ============================================================================

/**
 * CORE PRINCIPLES:
 * - Same authentication flow for both apps
 * - Web3 wallet signing triggers OpenID token issuance
 * - No permission logic - just authentication
 * - Simple, focused authentication UI
 */

'use client';

import React, { useState, useCallback } from 'react';
import { useRouter } from 'next/navigation';
import { useSharedAuth } from './SharedOpenIDWeb3Provider';
import { requestWalletChallenge, verifyWalletSignature } from '../../auth/direct-web3-api';

// Authentication step states
type AuthStep = 'connect' | 'challenge' | 'signing' | 'authenticating' | 'success' | 'error';

interface SharedWeb3SignInProps {
  redirectTo?: string;
  onSuccess?: () => void;
  className?: string;
  title?: string;
  subtitle?: string;
}

export function SharedWeb3SignIn({ 
  redirectTo = '/dashboard', 
  onSuccess,
  className = '',
  title = 'Authentication',
  subtitle = 'Secure Web3 authentication powered by OpenID Connect'
}: SharedWeb3SignInProps) {
  const router = useRouter();
  const { authenticateWithDirectApi, isLoading } = useSharedAuth();
  
  const [currentStep, setCurrentStep] = useState<AuthStep>('connect');
  const [walletAddress, setWalletAddress] = useState<string>('');
  const [error, setError] = useState<string>('');
  const [challenge, setChallenge] = useState<{ nonce: string; message: string; wallet_address: string } | null>(null);

  // Connect wallet and request challenge
  const handleConnectWallet = useCallback(async () => {
    try {
      setError('');
      setCurrentStep('connect');
      
      // Check if MetaMask is available
      if (typeof window.ethereum === 'undefined') {
        throw new Error('MetaMask is not installed. Please install MetaMask to continue.');
      }

      console.log('Requesting wallet connection');
      
      // Request account access
      const accounts = await window.ethereum.request({
        method: 'eth_requestAccounts'
      });

      if (!accounts || accounts.length === 0) {
        throw new Error('No wallet accounts found. Please connect your wallet.');
      }

      const address = accounts[0];
      setWalletAddress(address);
      
      console.log('Wallet connected', { wallet_address: address });
      
      // Request challenge from backend using direct API
      setCurrentStep('challenge');
      const challengeData = await requestWalletChallenge(address);
      setChallenge(challengeData);
      
      console.log('Challenge received, prompting for signature');
      
      // Automatically proceed to signing
      await handleSignMessage(challengeData, address);
      
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to connect wallet';
      console.error('Wallet connection failed', { error: errorMessage });
      setError(errorMessage);
      setCurrentStep('error');
    }
  }, []);

  // Sign the challenge message
  const handleSignMessage = useCallback(async (
    challengeData: { nonce: string; message: string; wallet_address: string },
    address: string
  ) => {
    try {
      setCurrentStep('signing');
      
      console.log('Requesting message signature', { wallet_address: address });
      
      // Request signature from wallet
      const signature = await window.ethereum.request({
        method: 'personal_sign',
        params: [challengeData.message, address],
      });

      console.log('Signature received, authenticating with backend');
      
      // Authenticate with backend using direct API
      setCurrentStep('authenticating');
      const result = await verifyWalletSignature({
        wallet_address: address,
        signature,
        message: challengeData.message,
        nonce: challengeData.nonce
      });

      if (result.success) {
        console.log('🎉 Authentication successful!', {
          wallet: result.wallet_address,
          tier: result.tier_level,
          permissions: result.permissions?.length || 0,
          isNewUser: result.is_new_user
        });
        
        // Update SharedOpenIDWeb3Provider with authenticated user
        await authenticateWithDirectApi({
          wallet_address: result.wallet_address,
          permissions: result.permissions,
          tier_level: result.tier_level,
          is_new_user: result.is_new_user,
          access_token: result.access_token
        });

        setCurrentStep('success');
        
        // Call onSuccess callback if provided
        if (onSuccess) {
          onSuccess();
        } else {
          // Navigate to redirect URL
          setTimeout(() => {
            router.push(redirectTo);
          }, 1500);
        }
      } else {
        throw new Error(result.error || 'Authentication failed');
      }
      
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Signature or authentication failed';
      console.error('Authentication process failed', { error: errorMessage });
      setError(errorMessage);
      setCurrentStep('error');
    }
  }, [authenticateWithDirectApi, router, redirectTo, onSuccess]);

  // Retry authentication
  const handleRetry = useCallback(() => {
    setError('');
    setChallenge(null);
    setWalletAddress('');
    setCurrentStep('connect');
  }, []);

  // Render step content
  const renderStepContent = () => {
    switch (currentStep) {
      case 'connect':
        return (
          <div className="flex flex-col items-center space-y-4">
            <div className="w-12 h-12 bg-blue-500 rounded-full flex items-center justify-center">
              <svg className="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 9V7a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2m2 4h10a2 2 0 002-2v-6a2 2 0 00-2-2H9a2 2 0 00-2 2v2a2 2 0 002 2z" />
              </svg>
            </div>
            <div className="text-center">
              <h3 className="text-lg font-semibold">Connect Your Wallet</h3>
              <p className="text-sm text-gray-600 mt-1">
                Connect your Web3 wallet to sign in
              </p>
            </div>
            <button 
              onClick={handleConnectWallet} 
              disabled={isLoading}
              className="w-full bg-blue-500 hover:bg-blue-600 disabled:bg-blue-300 text-white px-4 py-2 rounded-lg font-medium"
            >
              {isLoading ? 'Connecting...' : 'Connect Wallet'}
            </button>
          </div>
        );

      case 'challenge':
        return (
          <div className="flex flex-col items-center space-y-4">
            <div className="w-12 h-12 bg-blue-500 rounded-full flex items-center justify-center animate-spin">
              <svg className="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
            </div>
            <div className="text-center">
              <h3 className="text-lg font-semibold">Preparing Authentication</h3>
              <p className="text-sm text-gray-600 mt-1">
                Generating secure challenge for {walletAddress}
              </p>
            </div>
          </div>
        );

      case 'signing':
        return (
          <div className="flex flex-col items-center space-y-4">
            <div className="w-12 h-12 bg-orange-500 rounded-full flex items-center justify-center animate-pulse">
              <svg className="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
              </svg>
            </div>
            <div className="text-center">
              <h3 className="text-lg font-semibold">Sign the Message</h3>
              <p className="text-sm text-gray-600 mt-1">
                Please sign the message in your wallet to continue
              </p>
              <p className="text-xs text-gray-500 mt-2">
                This signature proves you own the wallet address
              </p>
            </div>
          </div>
        );

      case 'authenticating':
        return (
          <div className="flex flex-col items-center space-y-4">
            <div className="w-12 h-12 bg-green-500 rounded-full flex items-center justify-center animate-spin">
              <svg className="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <div className="text-center">
              <h3 className="text-lg font-semibold">Authenticating</h3>
              <p className="text-sm text-gray-600 mt-1">
                Verifying your signature and issuing access tokens
              </p>
            </div>
          </div>
        );

      case 'success':
        return (
          <div className="flex flex-col items-center space-y-4">
            <div className="w-12 h-12 bg-green-500 rounded-full flex items-center justify-center">
              <svg className="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
            </div>
            <div className="text-center">
              <h3 className="text-lg font-semibold text-green-700">Authentication Successful!</h3>
              <p className="text-sm text-gray-600 mt-1">
                Welcome! Redirecting you now...
              </p>
            </div>
          </div>
        );

      case 'error':
        return (
          <div className="flex flex-col items-center space-y-4">
            <div className="w-12 h-12 bg-red-500 rounded-full flex items-center justify-center">
              <svg className="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </div>
            <div className="text-center">
              <h3 className="text-lg font-semibold text-red-700">Authentication Failed</h3>
              <p className="text-sm text-gray-600 mt-1">
                Please try again or check your wallet connection
              </p>
            </div>
            <button 
              onClick={handleRetry}
              className="w-full bg-gray-500 hover:bg-gray-600 text-white px-4 py-2 rounded-lg font-medium"
            >
              Try Again
            </button>
          </div>
        );

      default:
        return null;
    }
  };

  return (
    <div className={`flex items-center justify-center min-h-screen bg-gray-50 p-4 ${className}`}>
      <div className="w-full max-w-md bg-white rounded-lg shadow-lg p-6">
        <div className="text-center mb-6">
          <h1 className="text-2xl font-bold text-gray-900">{title}</h1>
          <p className="text-sm text-gray-600 mt-1">{subtitle}</p>
        </div>
        
        <div className="space-y-6">
          {error && (
            <div className="bg-red-50 border border-red-200 rounded-lg p-3">
              <div className="flex">
                <div className="flex-shrink-0">
                  <svg className="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                    <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
                  </svg>
                </div>
                <div className="ml-3">
                  <p className="text-sm text-red-800">{error}</p>
                </div>
              </div>
            </div>
          )}
          
          {renderStepContent()}
          
          {walletAddress && currentStep !== 'error' && (
            <div className="bg-gray-50 p-3 rounded-lg">
              <p className="text-xs text-gray-500 mb-1">Connected Wallet:</p>
              <p className="text-sm font-mono break-all">{walletAddress}</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export default SharedWeb3SignIn;