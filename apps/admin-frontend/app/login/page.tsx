'use client';

import { AdminOIDCLoginButton } from '@/components/auth/AdminOIDCLoginButton';
import { useSearchParams } from 'next/navigation';
import { AlertTriangle } from 'lucide-react';

/**
 * JWT Admin Login Page
 * Features enterprise-grade security with threat detection and audit logging
 */
export default function AdminLoginPage() {
  const searchParams = useSearchParams();
  
  // Extract redirect URL and any error from query parameters
  const redirectTo = searchParams.get('callbackUrl') || searchParams.get('redirect') || '/';
  const error = searchParams.get('error');

  // Map NextAuth error codes to user-friendly messages
  const getErrorMessage = (error: string | null) => {
    switch (error) {
      case 'Configuration':
        return 'There is a problem with the server configuration.';
      case 'AccessDenied':
        return 'You do not have permission to sign in.';
      case 'Verification':
        return 'The verification token has expired or is invalid.';
      case 'OAuthSignin':
        return 'Error in constructing an authorization URL.';
      case 'OAuthCallback':
        return 'Error in handling the response from an OAuth provider.';
      case 'OAuthCreateAccount':
        return 'Could not create OAuth provider user in the database.';
      case 'EmailCreateAccount':
        return 'Could not create email provider user in the database.';
      case 'Callback':
        return 'Error in the OAuth callback handler route.';
      case 'OAuthAccountNotLinked':
        return 'The account is already associated with another user.';
      case 'EmailSignin':
        return 'Sending the e-mail with the verification token failed.';
      case 'CredentialsSignin':
        return 'Authorization failed. Check your credentials and try again.';
      case 'SessionRequired':
        return 'Please sign in to access this page.';
      case 'insufficient_admin_access':
        return 'Administrative privileges required for this application.';
      default:
        return error ? 'Authentication error occurred. Please try again.' : null;
    }
  };

  const errorMessage = getErrorMessage(error);

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      {/* Background pattern */}
      <div className="fixed inset-0 z-0 pointer-events-none">
        <div className="absolute inset-0 bg-gradient-to-br from-blue-50 via-gray-50 to-blue-50 dark:from-gray-900 dark:via-gray-800 dark:to-gray-900" />
        <div className="absolute top-20 left-10 w-32 h-32 bg-gradient-to-br from-blue-400/10 to-cyan-400/10 rounded-full blur-xl" />
        <div className="absolute bottom-20 right-20 w-24 h-24 bg-gradient-to-br from-purple-400/10 to-blue-400/10 rounded-full blur-xl" />
        <div className="absolute top-1/2 left-1/4 w-20 h-20 bg-gradient-to-br from-cyan-400/5 to-blue-400/5 rounded-full blur-xl" />
      </div>

      <div className="relative z-10 flex flex-col items-center justify-center min-h-screen gap-6">
        {/* Error message */}
        {errorMessage && (
          <div className="w-full max-w-md mx-auto">
            <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
              <div className="flex items-center">
                <AlertTriangle className="h-5 w-5 text-red-600 dark:text-red-400 mr-2" />
                <div className="text-sm text-red-800 dark:text-red-200">
                  {errorMessage}
                </div>
              </div>
            </div>
          </div>
        )}

        <AdminOIDCLoginButton
          redirectTo={redirectTo}
          requireMFA={true}
          enableThreatDetection={true}
          enableSessionMonitoring={true}
          maxFailedAttempts={3}
        />
      </div>

      {/* Admin notice */}
      <div className="fixed bottom-4 left-1/2 transform -translate-x-1/2 z-20">
        <div className="bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm rounded-lg px-4 py-2 text-xs text-center text-muted-foreground border border-gray-200 dark:border-gray-700">
          🔒 Administrative Access Only • All attempts are logged and monitored
        </div>
      </div>
    </div>
  );
}