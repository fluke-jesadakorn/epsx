'use client';

import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Mail, X } from 'lucide-react';
import { useSession } from 'next-auth/react';
import { useState } from 'react';

export function EmailVerificationBanner() {
  const { data: session } = useSession();
  const user = session?.user;
  const [isDismissed, setIsDismissed] = useState(false);

  // Only show if user is logged in, email is not verified, and banner not dismissed
  if (!user || user.emailVerified || isDismissed) {
    return null;
  }

  const handleResendVerification = async () => {
    // TODO: Implement server action for email verification
    console.log('Email verification resend not implemented');
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
            className="border-amber-300 text-amber-700 hover:bg-amber-100 dark:border-amber-700 dark:text-amber-300 dark:hover:bg-amber-900/30"
          >
            Resend Email
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
