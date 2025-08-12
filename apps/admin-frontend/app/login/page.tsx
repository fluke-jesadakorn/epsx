'use client';

import { AdminOIDCLoginButton } from '@/components/auth/AdminOIDCLoginButton';
import { useSearchParams } from 'next/navigation';

/**
 * Admin Login Page - Uses OIDC Authentication
 * Features enterprise-grade security with threat detection and audit logging
 */
export default function AdminLoginPage() {
  const searchParams = useSearchParams();
  
  // Extract redirect URL and any error from query parameters
  const redirectTo = searchParams.get('callbackUrl') || searchParams.get('redirect') || '/';
  const error = searchParams.get('error');

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      {/* Background pattern */}
      <div className="fixed inset-0 z-0 pointer-events-none">
        <div className="absolute inset-0 bg-gradient-to-br from-blue-50 via-gray-50 to-blue-50 dark:from-gray-900 dark:via-gray-800 dark:to-gray-900" />
        <div className="absolute top-20 left-10 w-32 h-32 bg-gradient-to-br from-blue-400/10 to-cyan-400/10 rounded-full blur-xl" />
        <div className="absolute bottom-20 right-20 w-24 h-24 bg-gradient-to-br from-purple-400/10 to-blue-400/10 rounded-full blur-xl" />
        <div className="absolute top-1/2 left-1/4 w-20 h-20 bg-gradient-to-br from-cyan-400/5 to-blue-400/5 rounded-full blur-xl" />
      </div>

      <div className="relative z-10 flex items-center justify-center min-h-screen">
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