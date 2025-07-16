'use client';

import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Mail, X } from 'lucide-react';
import { useAuth } from '@/context/auth-context-improved';
import { useState } from 'react';

export function EmailVerificationBanner() {
  const { user, sendEmailVerification, loading } = useAuth();
  const [isDismissed, setIsDismissed] = useState(false);
  const [isResending, setIsResending] = useState(false);

  // Only show if user is logged in, email is not verified, and banner not dismissed
  if (!user || user.emailVerified || isDismissed) {
    return null;
  }

  const handleResendVerification = async () => {
    try {
      setIsResending(true);
      await sendEmailVerification();
      // Could show a toast here
    } catch (error) {
      console.error('Failed to resend verification email:', error);
    } finally {
      setIsResending(false);
    }
  };

  return (
    <Alert className="border-amber-200 bg-amber-50 dark:border-amber-800 dark:bg-amber-900/20 rounded-none border-l-0 border-r-0 border-t-0">
      <Mail className="h-4 w-4 text-amber-600 dark:text-amber-400" />
      <div className="flex items-center justify-between w-full">
        <AlertDescription className="text-amber-800 dark:text-amber-300 flex-1">
          <strong>Email verification required.</strong> Please check your inbox and verify your email to access all features.
        </AlertDescription>
        <div className="flex items-center space-x-2 ml-4">
          <Button
            variant="outline"
            size="sm"
            onClick={handleResendVerification}
            disabled={loading || isResending}
            className="border-amber-300 text-amber-700 hover:bg-amber-100 dark:border-amber-700 dark:text-amber-300 dark:hover:bg-amber-900/30"
          >
            {isResending ? 'Sending...' : 'Resend Email'}
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setIsDismissed(true)}
            className="text-amber-700 hover:bg-amber-100 dark:text-amber-300 dark:hover:bg-amber-900/30"
          >
            <X className="h-4 w-4" />
          </Button>
        </div>
      </div>
    </Alert>
  );
}
