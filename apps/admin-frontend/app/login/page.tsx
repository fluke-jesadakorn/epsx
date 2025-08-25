'use client';

import { useEffect } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';

export default function LoginPage() {
  const router = useRouter();
  const searchParams = useSearchParams();

  useEffect(() => {
    const initiateOAuth = async () => {
      try {
        console.log('🔄 Admin: Initiating OAuth with PKCE parameters...');
        
        // Get the redirect path from URL parameters
        const redirectTo = searchParams.get('redirectTo') || '/';
        
        // Call the PKCE initiation endpoint
        const response = await fetch('/api/auth/initiate', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ redirectTo }),
        });

        if (!response.ok) {
          throw new Error(`Failed to initiate OAuth: ${response.status} ${response.statusText}`);
        }

        const result = await response.json();
        
        if (result.success && result.authorizationUrl) {
          console.log('✅ Admin: PKCE parameters generated, redirecting to OAuth...');
          // Redirect to the OAuth authorization URL with PKCE
          window.location.href = result.authorizationUrl;
        } else {
          throw new Error(result.message || 'Failed to generate OAuth URL');
        }
        
      } catch (error) {
        console.error('❌ Admin: OAuth initiation failed:', error);
        
        // Show error to user instead of infinite redirect
        alert(`Authentication failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
      }
    };

    initiateOAuth();
  }, [searchParams, router]);

  return (
    <div className="flex items-center justify-center min-h-screen bg-gray-100">
      <div className="text-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-4"></div>
        <h1 className="text-xl font-semibold text-gray-700">Redirecting to Admin Login...</h1>
        <p className="text-gray-500 mt-2">Setting up secure authentication...</p>
      </div>
    </div>
  );
}