'use client';

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { 
  Lock, 
  ExternalLink, 
  CheckCircle, 
  Wallet, 
  Mail,
  Users,
  Shield,
  ArrowRight
} from 'lucide-react';
import { WalletConnectAuth } from './WalletConnectAuth';
import { EmailLinking } from './EmailLinking';
import { useAccount } from 'wagmi';

interface UnifiedAuthFormProps {
  redirectTo?: string;
  defaultTab?: 'web3' | 'oidc';
  showEmailLinking?: boolean;
  web3First?: boolean;
}

export function UnifiedAuthForm({ 
  redirectTo = '/dashboard', 
  defaultTab = 'web3',
  showEmailLinking = true,
  web3First = true
}: UnifiedAuthFormProps) {
  const [isOIDCLoading, setIsOIDCLoading] = useState(false);
  const [oidcError, setOIDCError] = useState('');
  const [oidcStep, setOIDCStep] = useState<'ready' | 'redirecting' | 'complete'>('ready');
  const [currentTab, setCurrentTab] = useState(defaultTab);
  const { isConnected } = useAccount();

  /**
   * Initiate OAuth Authorization Code Flow
   */
  const initiateOAuthFlow = async () => {
    try {
      setIsOIDCLoading(true);
      setOIDCError('');
      setOIDCStep('redirecting');
      
      const response = await fetch('/api/auth/initiate', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          redirectTo: redirectTo
        }),
        credentials: 'include'
      });
      
      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.message || 'Failed to initiate authentication');
      }
      
      const { authorizationUrl } = await response.json();
      window.location.href = authorizationUrl;
      
    } catch (error: any) {
      console.error('OAuth initiation failed:', error);
      setOIDCError(error.message || 'Authentication failed. Please try again.');
      setOIDCStep('ready');
      setIsOIDCLoading(false);
    }
  };

  const handleWeb3AuthSuccess = (walletAddress: string) => {
    console.log('Web3 authentication successful:', walletAddress);
    // Redirect after successful Web3 auth
    window.location.href = redirectTo;
  };

  const handleWeb3AuthError = (error: string) => {
    console.error('Web3 authentication error:', error);
  };

  const getOIDCStepDescription = () => {
    switch (oidcStep) {
      case 'ready': return 'Ready to sign in with traditional account'
      case 'redirecting': return 'Redirecting to secure authentication...'
      case 'complete': return 'Success! Redirecting...'
    }
  };

  return (
    <Card className="w-full max-w-md mx-auto">
      <CardHeader className="text-center">
        <CardTitle className="flex items-center justify-center gap-2">
          <Wallet className="h-6 w-6 text-orange-500" />
          {web3First ? 'Connect to EPSX' : 'Sign In to EPSX'}
        </CardTitle>
        <p className="text-sm text-muted-foreground">
          {web3First 
            ? 'Connect your Web3 wallet for enhanced features and permissions'
            : 'Choose your preferred authentication method'
          }
        </p>
      </CardHeader>
      
      <CardContent>
        <Tabs value={currentTab} onValueChange={(value) => setCurrentTab(value as 'web3' | 'oidc')}>
          <TabsList className="grid w-full grid-cols-2">
            <TabsTrigger value="web3" className="flex items-center gap-2">
              <Wallet className="h-4 w-4" />
              {web3First ? 'Web3 Wallet' : 'Web3 Wallet'}
            </TabsTrigger>
            <TabsTrigger value="oidc" className="flex items-center gap-2">
              <Mail className="h-4 w-4" />
              {web3First ? 'Email/Password' : 'Traditional'}
            </TabsTrigger>
          </TabsList>

          <TabsContent value="oidc" className="space-y-4">
            {/* Fallback Notice */}
            <div className="p-4 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <Mail className="h-4 w-4 text-amber-600 dark:text-amber-400" />
                <span className="text-sm font-medium text-amber-800 dark:text-amber-300">
                  Traditional Authentication
                </span>
              </div>
              <p className="text-xs text-amber-700 dark:text-amber-400">
                Use this option if you don't have a Web3 wallet or prefer traditional email/password authentication.
              </p>
            </div>

            {/* OIDC Authentication Status */}
            {oidcStep !== 'ready' && (
              <div className="p-4 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
                <div className="flex items-center gap-3">
                  {oidcStep === 'redirecting' ? (
                    <div className="h-5 w-5 bg-blue-600 rounded animate-pulse" />
                  ) : (
                    <CheckCircle className="h-5 w-5 text-green-600" />
                  )}
                  <div>
                    <h3 className="font-medium text-blue-900 dark:text-blue-100">Authentication Status</h3>
                    <p className="text-sm text-blue-700 dark:text-blue-300">{getOIDCStepDescription()}</p>
                  </div>
                </div>
                
                <div className="mt-3 text-xs text-blue-600 dark:text-blue-400">
                  <p>✅ OAuth Authorization Code Flow</p>
                  {oidcStep === 'complete' && <p>✅ Secure Token Exchange</p>}
                </div>
              </div>
            )}

            {/* OIDC Error Display */}
            {oidcError && (
              <div className="rounded-lg bg-red-50 dark:bg-red-900/20 p-3 border border-red-200 dark:border-red-800">
                <p className="text-sm text-red-700 dark:text-red-400">⚠️ {oidcError}</p>
              </div>
            )}

            {/* OIDC Sign In Button */}
            <Button
              onClick={initiateOAuthFlow}
              disabled={isOIDCLoading}
              className="w-full h-12 text-base font-semibold bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 text-white border-0 shadow-lg rounded-xl"
            >
              {isOIDCLoading ? (
                <>
                  <div className="h-4 w-4 mr-2 bg-white rounded animate-pulse" />
                  <span>Redirecting to secure authentication...</span>
                </>
              ) : (
                <>
                  <ExternalLink className="h-4 w-4 mr-2" />
                  <span>Sign In with Email</span>
                </>
              )}
            </Button>

            {/* OIDC Features */}
            <div className="grid grid-cols-2 gap-4">
              <div className="text-center p-3 bg-green-50 dark:bg-green-900/20 rounded-lg">
                <Lock className="h-5 w-5 text-green-600 dark:text-green-400 mx-auto mb-1" />
                <p className="text-xs font-medium text-green-700 dark:text-green-300">
                  Secure OAuth
                </p>
              </div>
              <div className="text-center p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
                <CheckCircle className="h-5 w-5 text-blue-600 dark:text-blue-400 mx-auto mb-1" />
                <p className="text-xs font-medium text-blue-700 dark:text-blue-300">
                  OIDC Compliant
                </p>
              </div>
            </div>

            {/* OIDC Security Notice */}
            <div className="text-center space-y-2">
              <p className="text-xs text-muted-foreground">
                🔒 Standard email/password authentication
              </p>
              <div className="flex items-center justify-center gap-2 text-xs text-green-600 dark:text-green-400">
                <CheckCircle className="h-3 w-3" />
                <span>HttpOnly cookies • PKCE protection • Bearer tokens</span>
              </div>
            </div>
          </TabsContent>

          <TabsContent value="web3" className="space-y-6">
            {/* Web3 Hero Section */}
            <div className="text-center space-y-4">
              <div className="p-6 bg-gradient-to-br from-orange-50 to-purple-50 dark:from-orange-900/20 dark:to-purple-900/20 border-2 border-dashed border-orange-200 dark:border-orange-800 rounded-xl">
                <div className="flex justify-center mb-4">
                  <div className="p-3 bg-gradient-to-r from-orange-500 to-purple-600 rounded-xl">
                    <Wallet className="h-8 w-8 text-white" />
                  </div>
                </div>
                <h3 className="text-xl font-bold text-gray-900 dark:text-gray-100 mb-2">
                  Web3-First Authentication
                </h3>
                <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
                  Connect your crypto wallet for instant access to Web3 features, NFT-gated content, token-based permissions, and DAO governance.
                </p>
                
                {/* Supported Wallets */}
                <div className="flex items-center justify-center gap-2 mb-4">
                  <Badge variant="outline" className="bg-orange-100 border-orange-300 text-orange-800 dark:bg-orange-900/20 dark:border-orange-700 dark:text-orange-300">
                    MetaMask
                  </Badge>
                  <Badge variant="outline" className="bg-purple-100 border-purple-300 text-purple-800 dark:bg-purple-900/20 dark:border-purple-700 dark:text-purple-300">
                    WalletConnect
                  </Badge>
                  <Badge variant="outline" className="bg-blue-100 border-blue-300 text-blue-800 dark:bg-blue-900/20 dark:border-blue-700 dark:text-blue-300">
                    Coinbase
                  </Badge>
                </div>

                {/* Web3 Benefits */}
                <div className="grid grid-cols-3 gap-3 mb-4">
                  <div className="text-center p-2 bg-white dark:bg-slate-800 rounded-lg border">
                    <Crown className="h-4 w-4 text-purple-500 mx-auto mb-1" />
                    <p className="text-xs font-medium text-purple-700 dark:text-purple-300">
                      NFT Access
                    </p>
                  </div>
                  <div className="text-center p-2 bg-white dark:bg-slate-800 rounded-lg border">
                    <Zap className="h-4 w-4 text-orange-500 mx-auto mb-1" />
                    <p className="text-xs font-medium text-orange-700 dark:text-orange-300">
                      Token Gates
                    </p>
                  </div>
                  <div className="text-center p-2 bg-white dark:bg-slate-800 rounded-lg border">
                    <Users className="h-4 w-4 text-blue-500 mx-auto mb-1" />
                    <p className="text-xs font-medium text-blue-700 dark:text-blue-300">
                      DAO Voting
                    </p>
                  </div>
                </div>
              </div>
            </div>

            {/* Web3 Authentication Component */}
            <div className="space-y-4">
              <WalletConnectAuth
                onAuthSuccess={handleWeb3AuthSuccess}
                onAuthError={handleWeb3AuthError}
                variant="detailed"
              />
            </div>

            {/* Email Linking for Web3 Users */}
            {showEmailLinking && isConnected && (
              <div className="p-4 bg-slate-50 dark:bg-slate-800 rounded-lg border">
                <div className="flex items-center gap-2 mb-3">
                  <Mail className="h-4 w-4 text-slate-600 dark:text-slate-400" />
                  <span className="text-sm font-medium text-slate-900 dark:text-slate-100">
                    Optional: Link Email Account
                  </span>
                </div>
                <EmailLinking 
                  showAsDialog={false}
                  autoShow={false}
                />
              </div>
            )}

            {/* Security Features */}
            <div className="grid grid-cols-2 gap-4">
              <div className="text-center p-4 bg-green-50 dark:bg-green-900/20 rounded-lg border border-green-200 dark:border-green-800">
                <Shield className="h-6 w-6 text-green-600 dark:text-green-400 mx-auto mb-2" />
                <p className="text-sm font-medium text-green-800 dark:text-green-300 mb-1">
                  Self-Sovereign
                </p>
                <p className="text-xs text-green-600 dark:text-green-400">
                  Your keys, your control
                </p>
              </div>
              <div className="text-center p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg border border-blue-200 dark:border-blue-800">
                <CheckCircle className="h-6 w-6 text-blue-600 dark:text-blue-400 mx-auto mb-2" />
                <p className="text-sm font-medium text-blue-800 dark:text-blue-300 mb-1">
                  SIWE Standard
                </p>
                <p className="text-xs text-blue-600 dark:text-blue-400">
                  Ethereum-native auth
                </p>
              </div>
            </div>

            {/* Web3 Security Notice */}
            <div className="text-center p-4 bg-slate-50 dark:bg-slate-800 rounded-lg border">
              <div className="flex items-center justify-center gap-2 mb-2">
                <Lock className="h-4 w-4 text-slate-600 dark:text-slate-400" />
                <span className="text-sm font-medium text-slate-900 dark:text-slate-100">
                  Cryptographic Authentication
                </span>
              </div>
              <p className="text-xs text-slate-600 dark:text-slate-400">
                Sign a message to prove wallet ownership • No password required • Industry standard SIWE protocol
              </p>
            </div>
          </TabsContent>
        </Tabs>

        {/* Web3-First Notice */}
        <div className="mt-6 p-4 bg-gradient-to-r from-orange-50 to-purple-50 dark:from-orange-900/20 dark:to-purple-900/20 border border-orange-200 dark:border-orange-700 rounded-lg">
          <div className="flex items-center gap-2 mb-2">
            <ArrowRight className="h-4 w-4 text-orange-600" />
            <span className="text-sm font-medium text-orange-900 dark:text-orange-100">
              {web3First ? 'Web3-First Platform' : 'Hybrid Authentication System'}
            </span>
          </div>
          <p className="text-xs text-orange-700 dark:text-orange-300">
            {web3First 
              ? 'EPSX prioritizes Web3 authentication for enhanced features, permissions, and decentralized access. Traditional auth available as fallback.'
              : 'Both methods are OIDC-compliant and provide secure access to your EPSX account. Choose the method that best fits your preferences.'
            }
          </p>
        </div>
      </CardContent>
    </Card>
  );
}