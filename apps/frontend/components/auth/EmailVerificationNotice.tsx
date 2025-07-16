'use client';

import { useState } from 'react';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { CheckCircle, Mail, RefreshCw } from 'lucide-react';
import { useAuth } from '@/context/auth-context-improved';

interface EmailVerificationNoticeProps {
  onResendSuccess?: () => void;
}

export function EmailVerificationNotice({ 
  onResendSuccess
}: EmailVerificationNoticeProps) {
  const { user, sendEmailVerification, error, loading } = useAuth();
  const [isResending, setIsResending] = useState(false);
  const [resendSuccess, setResendSuccess] = useState(false);
  const [resendCooldown, setResendCooldown] = useState(0);

  const handleResendVerification = async () => {
    if (resendCooldown > 0 || !user) return;

    try {
      setIsResending(true);
      setResendSuccess(false);
      
      await sendEmailVerification();
      
      setResendSuccess(true);
      setResendCooldown(60); // 60 second cooldown
      onResendSuccess?.();

      // Start cooldown timer
      const timer = setInterval(() => {
        setResendCooldown((prev) => {
          if (prev <= 1) {
            clearInterval(timer);
            return 0;
          }
          return prev - 1;
        });
      }, 1000);

      // Auto-hide success message after 5 seconds
      setTimeout(() => setResendSuccess(false), 5000);
    } catch (error) {
      console.error('Failed to resend verification email:', error);
    } finally {
      setIsResending(false);
    }
  };

  const handleRefreshStatus = () => {
    // Force refresh the auth state to check if email has been verified
    window.location.reload();
  };

  if (!user) {
    return null;
  }

  if (user.emailVerified) {
    return (
      <Alert className="border-green-200 bg-green-50 dark:border-green-800 dark:bg-green-900/20">
        <CheckCircle className="h-4 w-4 text-green-600 dark:text-green-400" />
        <AlertDescription className="text-green-800 dark:text-green-300">
          Your email address has been verified!
        </AlertDescription>
      </Alert>
    );
  }

  return (
    <Card className="border-amber-200 bg-amber-50 dark:border-amber-800 dark:bg-amber-900/20">
      <CardHeader className="pb-3">
        <div className="flex items-center space-x-2">
          <Mail className="h-5 w-5 text-amber-600 dark:text-amber-400" />
          <CardTitle className="text-lg text-amber-800 dark:text-amber-300">
            Email Verification Required
          </CardTitle>
        </div>
        <CardDescription className="text-amber-700 dark:text-amber-400">
          Please verify your email address to access all features.
        </CardDescription>
      </CardHeader>
      
      <CardContent className="space-y-4">
        <div className="text-sm text-amber-700 dark:text-amber-400">
          We've sent a verification email to <strong>{user.email}</strong>. 
          Click the link in the email to verify your account.
        </div>

        {error && (
          <Alert className="border-red-200 bg-red-50 dark:border-red-800 dark:bg-red-900/20">
            <AlertDescription className="text-red-800 dark:text-red-300">
              {error}
            </AlertDescription>
          </Alert>
        )}

        {resendSuccess && (
          <Alert className="border-green-200 bg-green-50 dark:border-green-800 dark:bg-green-900/20">
            <CheckCircle className="h-4 w-4 text-green-600 dark:text-green-400" />
            <AlertDescription className="text-green-800 dark:text-green-300">
              Verification email sent successfully! Check your inbox.
            </AlertDescription>
          </Alert>
        )}

        <div className="flex flex-col sm:flex-row gap-2">
          <Button
            onClick={handleResendVerification}
            disabled={isResending || loading || resendCooldown > 0}
            variant="outline"
            className="flex-1 border-amber-300 text-amber-700 hover:bg-amber-100 dark:border-amber-700 dark:text-amber-300 dark:hover:bg-amber-900/30"
          >
            {isResending ? (
              <>
                <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                Sending...
              </>
            ) : resendCooldown > 0 ? (
              `Resend in ${resendCooldown}s`
            ) : (
              <>
                <Mail className="mr-2 h-4 w-4" />
                Resend Email
              </>
            )}
          </Button>

          <Button
            onClick={handleRefreshStatus}
            variant="outline"
            className="flex-1 border-amber-300 text-amber-700 hover:bg-amber-100 dark:border-amber-700 dark:text-amber-300 dark:hover:bg-amber-900/30"
          >
            <RefreshCw className="mr-2 h-4 w-4" />
            I've Verified
          </Button>
        </div>

        <div className="text-xs text-amber-600 dark:text-amber-500">
          <p>• Check your spam/junk folder if you don't see the email</p>
          <p>• Make sure to click the verification link from the same device</p>
          <p>• The verification link expires after 24 hours</p>
        </div>
      </CardContent>
    </Card>
  );
}
