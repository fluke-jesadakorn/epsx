'use client';

import { useRouter } from 'next/navigation';
import { useSession } from 'next-auth/react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { EmailVerificationNotice } from './EmailVerificationNotice';
import { ArrowLeft } from 'lucide-react';

export function EmailVerificationPage() {
  const { data: session } = useSession();
  const user = session?.user;
  const router = useRouter();

  const handleBackToMyData = () => {
    router.push('/my-data');
  };

  const handleResendSuccess = () => {
    // Optionally show a toast or feedback
    // Verification email resent successfully
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 via-orange-50 to-yellow-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900 flex items-center justify-center p-4">
      <div className="w-full max-w-md space-y-6">
        <div className="text-center">
          <h1 className="text-3xl font-bold tracking-tight">Email Verification</h1>
          <p className="text-muted-foreground mt-2">
            Verify your email to unlock all features
          </p>
        </div>

        <EmailVerificationNotice onResendSuccess={handleResendSuccess} />

        {user?.emailVerified && (
          <Card className="border-green-200 bg-green-50 dark:border-green-800 dark:bg-green-900/20">
            <CardHeader>
              <CardTitle className="text-green-800 dark:text-green-300">
                Email Verified!
              </CardTitle>
              <CardDescription className="text-green-700 dark:text-green-400">
                Your email has been successfully verified. You can now access all features.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <Button 
                onClick={handleBackToMyData}
                className="w-full"
              >
                Continue to My Data
              </Button>
            </CardContent>
          </Card>
        )}

        <div className="text-center">
          <Button
            variant="ghost"
            onClick={handleBackToMyData}
            className="text-muted-foreground hover:text-foreground"
          >
            <ArrowLeft className="mr-2 h-4 w-4" />
            Back to Dashboard
          </Button>
        </div>
      </div>
    </div>
  );
}
