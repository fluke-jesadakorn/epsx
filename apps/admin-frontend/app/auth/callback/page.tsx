'use client';

import { AdminPureOIDCCallback } from '@/components/auth/AdminPureOIDCCallback';

/**
 * Admin OIDC Authentication Callback Page
 * Handles OAuth2/OIDC authorization code exchange for administrators
 * 
 * Features:
 * - Enterprise-grade security validation
 * - Admin privilege verification
 * - Comprehensive audit logging
 * - Threat detection and monitoring
 * - Enhanced error recovery with escalation
 * - Real-time security metrics
 */
export default function AdminAuthCallbackPage() {
  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      {/* Enhanced background pattern for admin */}
      <div className="fixed inset-0 z-0 pointer-events-none">
        <div className="absolute inset-0 bg-gradient-to-br from-blue-50 via-gray-50 to-blue-50 dark:from-gray-900 dark:via-gray-800 dark:to-gray-900" />
        <div className="absolute top-20 left-10 w-32 h-32 bg-gradient-to-br from-blue-400/15 to-cyan-400/15 rounded-full animate-pulse" />
        <div className="absolute bottom-20 right-20 w-24 h-24 bg-gradient-to-br from-purple-400/15 to-blue-400/15 rounded-full" />
        <div className="absolute top-1/2 left-1/4 w-20 h-20 bg-gradient-to-br from-cyan-400/10 to-blue-400/10 rounded-full" />
      </div>

      <div className="relative z-10">
        <AdminPureOIDCCallback />
      </div>

      {/* Admin security notice */}
      <div className="fixed bottom-4 left-1/2 transform -translate-x-1/2 z-20">
        <div className="bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm rounded-lg px-4 py-2 text-xs text-center text-muted-foreground border border-gray-200 dark:border-gray-700">
          <div className="flex items-center justify-center space-x-2">
            <span>🔒</span>
            <span>Administrator Authentication</span>
            <span>•</span>
            <span>Enhanced Security Validation Active</span>
            <span>•</span>
            <span>All Access Logged</span>
          </div>
        </div>
      </div>
    </div>
  );
}