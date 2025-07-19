'use client';

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { CheckCircle, Mail, ArrowLeft } from 'lucide-react';

interface SignupSuccessProps {
  email: string;
  onBackToLogin: () => void;
}

export function SignupSuccess({ email, onBackToLogin }: SignupSuccessProps) {
  return (
    <Card className="w-full max-w-md">
      <CardHeader className="text-center">
        <div className="mx-auto w-12 h-12 bg-green-100 dark:bg-green-900/20 rounded-full flex items-center justify-center mb-4">
          <CheckCircle className="h-6 w-6 text-green-600 dark:text-green-400" />
        </div>
        <CardTitle className="text-2xl font-bold text-green-800 dark:text-green-300">
          Account Created Successfully!
        </CardTitle>
        <CardDescription>
          Welcome to EPSX! Your account has been created.
        </CardDescription>
      </CardHeader>

      <CardContent className="space-y-4">
        <Alert className="border-amber-200 bg-amber-50 dark:border-amber-800 dark:bg-amber-900/20">
          <Mail className="h-4 w-4 text-amber-600 dark:text-amber-400" />
          <AlertDescription className="text-amber-800 dark:text-amber-300">
            <strong>Important:</strong> Please verify your email address to
            access all features. We've sent a verification email to{' '}
            <strong>{email}</strong>. We've sent a verification email to{' '}
            <strong>{email}</strong>.
          </AlertDescription>
        </Alert>

        <div className="space-y-3 text-sm text-muted-foreground">
          <p>
            <strong>Next steps:</strong>
          </p>
          <ul className="space-y-1 list-disc list-inside">
            <li>Check your email inbox for a verification message</li>
            <li>Click the verification link in the email</li>
            <li>Return to EPSX and sign in to access all features</li>
          </ul>
        </div>

        <div className="space-y-2">
          <Button onClick={onBackToLogin} className="w-full">
            Continue to Sign In
          </Button>

          <Button variant="ghost" onClick={onBackToLogin} className="w-full">
            <ArrowLeft className="mr-2 h-4 w-4" />
            Back to Login
          </Button>
        </div>

        <div className="text-xs text-muted-foreground">
          <p>
            <strong>Didn't receive the email?</strong>
          </p>
          <p>• Check your spam/junk folder</p>
          <p>• Make sure you entered the correct email address</p>
          <p>• Wait a few minutes and check again</p>
        </div>
      </CardContent>
    </Card>
  );
}
