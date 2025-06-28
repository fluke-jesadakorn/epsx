'use client';

// import { GoogleSignIn } from "@/components/auth/GoogleSignIn";
import { redirect } from 'next/navigation';
import { useEffect } from 'react';
import Link from 'next/link';

import { EmailPasswordForm } from '@/components/auth/EmailPasswordForm';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { useAuth } from '@/context/auth-context';

export default function SignupPage() {
  const { user } = useAuth();

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
            Create an account
          </CardTitle>
          {/* <CardDescription>
            Choose your preferred sign up method
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

            <EmailPasswordForm isSignUp={true} />

            <div className="text-center mt-4">
              <Button
                variant="link"
                type="button"
                className="min-h-[48px]"
                asChild
              >
                <Link href="/login">
                  Already have an account? Sign in
                </Link>
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </main>
  );
}
