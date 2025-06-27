'use client';

import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { EmailPasswordForm } from '@/components/auth/EmailPasswordForm';
// import { GoogleSignIn } from "@/components/auth/GoogleSignIn";
import { useAuth } from '@/context/auth-context';
import { redirect } from 'next/navigation';
import { useEffect, useState } from 'react';
import { Button } from '@/components/ui/button';

export default function LoginPage() {
  const { user } = useAuth();
  const [isSignUp, setIsSignUp] = useState(false);

  useEffect(() => {
    if (user) {
      redirect('/dashboard');
    }
  }, [user]);

  return (
    <main className="flex items-center justify-center min-h-screen w-full p-4 md:p-6 lg:p-8 bg-gradient-to-br from-gray-50 to-gray-100">
      <Card className="w-full sm:max-w-md md:max-w-lg hover:scale-105 transition-transform duration-300">
        <CardHeader className="space-y-1">
          <CardTitle className="text-2xl md:text-3xl font-bold mb-4">
            {isSignUp ? 'Create an account' : 'Welcome'}
          </CardTitle>
          {/* <CardDescription>
            Choose your preferred sign in method
          </CardDescription> */}
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {/* <div className="relative">
              <div className="absolute inset-0 flex items-center">
                <span className="w-full border-t" />
              </div>
              <div className="relative flex justify-center text-xs uppercase">
                <span className="bg-background px-2 text-muted-foreground">
                  Or continue with
                </span>
              </div>
            </div> */}

            <EmailPasswordForm isSignUp={isSignUp} />

            <div className="text-center mt-4">
              <Button
                variant="link"
                type="button"
                onClick={() => setIsSignUp(!isSignUp)}
                className="min-h-[48px]"
              >
                {isSignUp
                  ? 'Already have an account? Sign in'
                  : "Don't have an account? Sign up"}
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </main>
  );
}
