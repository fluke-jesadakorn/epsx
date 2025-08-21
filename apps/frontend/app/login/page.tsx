'use client';

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { useSearchParams } from 'next/navigation';

export default function LoginPage() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');
  const searchParams = useSearchParams();
  
  const redirectTo = searchParams.get('redirect') || searchParams.get('callbackUrl') || '/dashboard';
  const errorParam = searchParams.get('error');

  async function handleOAuthLogin() {
    setIsLoading(true);
    setError('');

    try {
      // Call initiate endpoint to set up PKCE parameters
      const response = await fetch('/api/auth/initiate', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ redirectTo }),
        credentials: 'include'
      });

      if (response.ok) {
        const { authorizationUrl } = await response.json();
        console.log('✅ PKCE parameters set, redirecting to:', authorizationUrl);
        
        // Redirect to authorization URL
        window.location.href = authorizationUrl;
      } else {
        setError('Failed to initialize authentication. Please try again.');
      }
    } catch (err) {
      console.error('❌ OAuth initiation failed:', err);
      setError('Connection error. Please check your network and try again.');
    } finally {
      setIsLoading(false);
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-orange-50 to-yellow-50 dark:from-gray-900 dark:to-gray-800 p-4">
      <div className="max-w-md w-full space-y-8 bg-white dark:bg-gray-900 p-8 rounded-2xl shadow-xl border border-gray-200 dark:border-gray-700">
        <div className="text-center">
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
            Welcome to EPSX
          </h1>
          <p className="text-gray-600 dark:text-gray-400">
            Sign in to access your trading platform
          </p>
        </div>

        {/* Error display */}
        {(error || errorParam) && (
          <div className="rounded-lg bg-red-50 dark:bg-red-900/20 p-4 border border-red-200 dark:border-red-800">
            <p className="text-sm text-red-700 dark:text-red-400">
              ⚠️ {error || errorParam}
            </p>
          </div>
        )}

        {/* OAuth Sign In Button */}
        <div className="space-y-4">
          <Button
            onClick={handleOAuthLogin}
            disabled={isLoading}
            className="w-full h-14 text-lg font-semibold bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 text-white border-0 shadow-lg transform transition-transform hover:scale-105"
          >
            {isLoading ? (
              <>
                <div className="mr-3 h-5 w-5 animate-spin rounded-full border-2 border-white border-t-transparent" />
                Connecting...
              </>
            ) : (
              <>
                🚀 Continue with EPSX
              </>
            )}
          </Button>
        </div>

        <div className="text-center">
          <p className="text-xs text-gray-500 dark:text-gray-400">
            🔒 Secure OAuth 2.0 authentication with PKCE
          </p>
        </div>
      </div>
    </div>
  );
}
