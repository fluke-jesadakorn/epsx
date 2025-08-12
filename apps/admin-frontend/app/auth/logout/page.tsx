'use client';

import { useEffect, useState } from 'react';
import { useRouter } from 'next/navigation';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { CheckCircle, Loader2, LogOut, Shield, FileText, Clock } from 'lucide-react';
import { logoutAdminOIDC } from '@/app/actions/admin-oidc-auth';

/**
 * Admin OIDC Logout Callback Page
 * Handles secure admin logout with audit logging and enhanced cleanup
 */
export default function AdminLogoutCallbackPage() {
  const router = useRouter();
  const [status, setStatus] = useState<'processing' | 'success' | 'error'>('processing');
  const [error, setError] = useState<string | null>(null);
  const [auditEvents, setAuditEvents] = useState<string[]>([]);
  const [sessionDuration, setSessionDuration] = useState<number>(0);

  useEffect(() => {
    const handleAdminLogout = async () => {
      try {
        const startTime = Date.now();
        
        // Calculate session duration if available
        const sessionStart = localStorage.getItem('admin_session_start');
        if (sessionStart) {
          setSessionDuration(Date.now() - parseInt(sessionStart));
        }

        // Add audit events
        setAuditEvents(prev => [...prev, 'Initiating admin logout process']);
        await new Promise(resolve => setTimeout(resolve, 500)); // Visual feedback

        // Complete the admin logout process
        await logoutAdminOIDC();
        setAuditEvents(prev => [...prev, 'Admin session terminated']);
        await new Promise(resolve => setTimeout(resolve, 500));
        
        // Clear admin-specific authentication data
        const adminKeys = Object.keys(localStorage).filter(key => key.startsWith('admin_'));
        adminKeys.forEach(key => localStorage.removeItem(key));
        setAuditEvents(prev => [...prev, 'Admin security data cleared']);
        await new Promise(resolve => setTimeout(resolve, 500));
        
        // Clear all session storage
        sessionStorage.clear();
        setAuditEvents(prev => [...prev, 'Session storage cleared']);
        await new Promise(resolve => setTimeout(resolve, 500));

        // Final audit log
        const endTime = Date.now();
        setAuditEvents(prev => [...prev, `Logout completed in ${endTime - startTime}ms`]);
        
        setStatus('success');
        
        // Longer delay for admin to read audit information
        setTimeout(() => {
          router.replace('/login');
        }, 5000);
        
      } catch (err) {
        console.error('Admin logout process failed:', err);
        setError(err instanceof Error ? err.message : 'Admin logout failed');
        setStatus('error');
        setAuditEvents(prev => [...prev, `Error: ${err instanceof Error ? err.message : 'Unknown error'}`]);
      }
    };

    handleAdminLogout();
  }, [router]);

  const handleManualRedirect = () => {
    router.replace('/login');
  };

  const formatDuration = (ms: number): string => {
    const minutes = Math.floor(ms / 60000);
    const seconds = Math.floor((ms % 60000) / 1000);
    return `${minutes}m ${seconds}s`;
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900 p-4">
      {/* Background decoration */}
      <div className="fixed inset-0 z-0 pointer-events-none">
        <div className="absolute inset-0 bg-gradient-to-br from-blue-50 via-gray-50 to-blue-50 dark:from-gray-900 dark:via-gray-800 dark:to-gray-900" />
      </div>

      <div className="relative z-10">
        <Card className="w-full max-w-lg">
          <CardHeader className="text-center space-y-4">
            <div className="flex justify-center">
              {status === 'processing' && (
                <Shield className="h-12 w-12 animate-pulse text-blue-500" />
              )}
              {status === 'success' && (
                <CheckCircle className="h-12 w-12 text-green-500" />
              )}
              {status === 'error' && (
                <LogOut className="h-12 w-12 text-red-500" />
              )}
            </div>
            <div>
              <CardTitle className="text-xl">
                {status === 'processing' && 'Securely Signing Out...'}
                {status === 'success' && 'Admin Logout Complete'}
                {status === 'error' && 'Logout Error'}
              </CardTitle>
              <div className="flex justify-center mt-2">
                <Badge variant="outline" className="text-xs">
                  Administrator Session
                </Badge>
              </div>
            </div>
          </CardHeader>

          <CardContent className="space-y-6">
            {/* Session duration (if available) */}
            {sessionDuration > 0 && (
              <div className="flex items-center justify-center gap-2 text-sm text-muted-foreground">
                <Clock className="h-4 w-4" />
                <span>Session Duration: {formatDuration(sessionDuration)}</span>
              </div>
            )}

            {/* Audit trail */}
            <div className="space-y-3">
              <div className="flex items-center gap-2 text-sm font-medium">
                <FileText className="h-4 w-4" />
                Audit Trail
              </div>
              <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-3 max-h-40 overflow-y-auto">
                <div className="space-y-1">
                  {auditEvents.map((event, index) => (
                    <div key={index} className="flex items-start gap-2 text-xs">
                      <span className="text-muted-foreground font-mono">
                        {new Date().toLocaleTimeString()}
                      </span>
                      <span className="text-muted-foreground">•</span>
                      <span>{event}</span>
                    </div>
                  ))}
                  {status === 'processing' && (
                    <div className="flex items-center gap-2 text-xs">
                      <Loader2 className="h-3 w-3 animate-spin" />
                      <span>Processing...</span>
                    </div>
                  )}
                </div>
              </div>
            </div>

            {status === 'success' && (
              <div className="text-center space-y-4">
                <p className="text-sm text-muted-foreground">
                  Your administrative session has been securely terminated. 
                  All audit events have been logged for compliance.
                </p>
                <Button 
                  onClick={handleManualRedirect}
                  className="w-full"
                >
                  <Shield className="mr-2 h-4 w-4" />
                  Return to Admin Login
                </Button>
              </div>
            )}

            {status === 'error' && (
              <div className="text-center space-y-4">
                <p className="text-sm text-red-600">
                  {error || 'An error occurred during admin logout.'}
                </p>
                <p className="text-xs text-muted-foreground">
                  For security purposes, please clear your browser data and notify your system administrator.
                </p>
                <Button 
                  onClick={handleManualRedirect}
                  variant="outline"
                  className="w-full"
                >
                  Go to Admin Login
                </Button>
              </div>
            )}

            {/* Enhanced security notice for admin */}
            <div className="text-xs text-center text-muted-foreground pt-4 border-t space-y-1">
              <div className="flex items-center justify-center gap-2">
                <Shield className="h-3 w-3" />
                <span>Enhanced Admin Security Active</span>
              </div>
              <div>All administrative access is logged and monitored</div>
              <div>Contact IT Security for any access concerns</div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}