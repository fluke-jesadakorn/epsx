'use client';

import { WalletConnectAuth } from './WalletConnectAuth';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Wallet } from 'lucide-react';

interface UnifiedAuthFormProps {
  redirectTo?: string;
}

export function UnifiedAuthForm({ 
  redirectTo = '/dashboard'
}: UnifiedAuthFormProps) {

  const handleWeb3AuthSuccess = (walletAddress: string) => {
    console.log('Web3 authentication successful:', walletAddress);
    window.location.href = redirectTo;
  };

  const handleWeb3AuthError = (error: string) => {
    console.error('Web3 authentication error:', error);
  };

  return (
    <Card className="w-full max-w-md mx-auto">
      <CardHeader className="text-center">
        <CardTitle className="flex items-center justify-center gap-2">
          <Wallet className="h-6 w-6 text-orange-500" />
          Connect to EPSX
        </CardTitle>
        <p className="text-sm text-muted-foreground">
          Connect your Web3 wallet to get started
        </p>
      </CardHeader>
      
      <CardContent className="space-y-6">
        <WalletConnectAuth
          onAuthSuccess={handleWeb3AuthSuccess}
          onAuthError={handleWeb3AuthError}
        />
      </CardContent>
    </Card>
  );
}