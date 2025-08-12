'use client';

import { useEffect, useState } from 'react';
import { useRouter } from 'next/navigation';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { CheckCircle, Loader2, LogOut } from 'lucide-react';
import { logoutOIDC } from '@/app/actions/oidc-auth';

/**
 * OIDC Logout Callback Page
 * Handles post-logout cleanup and provides user feedback
 */
export default function LogoutCallbackPage() {
  const router = useRouter();
  const [status, setStatus] = useState<'processing' | 'success' | 'error'>('processing');
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const handleLogout = async () => {
      try {
        // Complete the logout process
        await logoutOIDC();
        
        // Clear any remaining authentication data
        localStorage.removeItem('trusted_devices');
        localStorage.removeItem('oidc_session_id');
        sessionStorage.clear();
        
        setStatus('success');
        
        // Redirect to login page after 3 seconds
        setTimeout(() => {
          router.replace('/login');
        }, 3000);
        
      } catch (err) {
        console.error('Logout process failed:', err);
        setError(err instanceof Error ? err.message : 'Logout failed');
        setStatus('error');
      }
    };

    handleLogout();
  }, [router]);

  const handleManualRedirect = () => {
    router.replace('/login');
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-background p-4">
      {/* Background decoration */}
      <div className="fixed inset-0 z-0 pointer-events-none">
        <div className="absolute inset-0 bg-gradient-to-br from-gray-50 to-gray-100 dark:from-gray-900 dark:to-gray-800" />
      </div>

      <div className="relative z-10">
        <Card className="w-full max-w-md">
          <CardHeader className="text-center space-y-4">
            <div className="flex justify-center">
              {status === 'processing' && (
                <Loader2 className="h-12 w-12 animate-spin text-blue-500" />
              )}
              {status === 'success' && (
                <CheckCircle className="h-12 w-12 text-green-500" />
              )}
              {status === 'error' && (
                <LogOut className="h-12 w-12 text-red-500" />
              )}
            </div>
            <CardTitle className="text-xl">
              {status === 'processing' && 'Signing Out...'}
              {status === 'success' && 'Successfully Signed Out'}
              {status === 'error' && 'Logout Error'}
            </CardTitle>
          </CardHeader>

          <CardContent className="space-y-4">
            {status === 'processing' && (
              <div className="text-center">
                <p className="text-sm text-muted-foreground">
                  Clearing your session and security data...
                </p>
              </div>
            )}

            {status === 'success' && (
              <div className="text-center space-y-4">
                <p className="text-sm text-muted-foreground">
                  Your session has been securely terminated. You'll be redirected to the login page shortly.
                </p>
                <Button 
                  onClick={handleManualRedirect}
                  className="w-full"
                >
                  Continue to Login
                </Button>
              </div>
            )}

            {status === 'error' && (
              <div className="text-center space-y-4">
                <p className="text-sm text-red-600">
                  {error || 'An error occurred during logout.'}
                </p>
                <p className="text-xs text-muted-foreground">
                  For security purposes, please clear your browser data manually.
                </p>
                <Button 
                  onClick={handleManualRedirect}
                  variant="outline"
                  className="w-full"
                >
                  Go to Login Page
                </Button>
              </div>
            )}

            {/* Security notice */}
            <div className="text-xs text-center text-muted-foreground pt-4 border-t">
              🔐 For your security, all session data has been cleared
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}