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
import { useAuth } from '@/context/auth-context-improved';

export default function SignupPage() {
  const { user } = useAuth();

  useEffect(() => {
    if (user) {
      redirect('/dashboard');
    }
  }, [user]);

  return (
    <main className="relative min-h-screen overflow-hidden">
      {/* PancakeSwap-style vibrant background */}
      <div className="fixed inset-0 z-0">
        {/* Main gradient background */}
        <div className="absolute inset-0 bg-gradient-to-br from-blue-50 via-orange-50 to-yellow-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900" />

        {/* Floating gradient orbs - PancakeSwap style */}
        <div className="absolute -top-40 -left-40 w-96 h-96 bg-gradient-to-br from-orange-400/30 to-yellow-400/30 rounded-full blur-3xl animate-bounce-slow" />
        <div className="absolute top-20 -right-32 w-80 h-80 bg-gradient-to-br from-blue-400/25 to-cyan-400/25 rounded-full blur-3xl animate-float" />
        <div className="absolute bottom-20 left-20 w-72 h-72 bg-gradient-to-br from-purple-400/20 to-pink-400/20 rounded-full blur-3xl animate-pulse-gentle" />
        <div className="absolute top-1/2 right-1/4 w-64 h-64 bg-gradient-to-br from-green-400/15 to-emerald-400/15 rounded-full blur-3xl animate-float-reverse" />

        {/* Mesh gradient overlays for depth */}
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_25%_25%,_rgba(255,133,27,0.1)_0%,_transparent_50%)] animate-pulse-slow" />
        <div
          className="absolute inset-0 bg-[radial-gradient(circle_at_75%_75%,_rgba(59,130,246,0.08)_0%,_transparent_50%)] animate-pulse-slow"
          style={{ animationDelay: '1s' }}
        />
        <div
          className="absolute inset-0 bg-[radial-gradient(circle_at_50%_50%,_rgba(168,85,247,0.06)_0%,_transparent_60%)] animate-pulse-slow"
          style={{ animationDelay: '2s' }}
        />

        {/* Decorative geometric shapes */}
        <div className="absolute top-1/4 left-1/4 w-32 h-32 bg-gradient-to-br from-orange-300/10 to-yellow-300/10 rounded-2xl rotate-45 animate-spin-slow" />
        <div className="absolute bottom-1/3 right-1/3 w-24 h-24 bg-gradient-to-br from-blue-300/10 to-cyan-300/10 rounded-full animate-bounce-gentle" />
        
        {/* Additional floating elements */}
        <div className="absolute top-10 left-10 w-8 h-8 bg-gradient-to-br from-purple-400/30 to-pink-400/30 rounded-full animate-bounce-gentle" />
        <div className="absolute bottom-10 right-10 w-6 h-6 bg-gradient-to-br from-green-400/30 to-emerald-400/30 rounded-full animate-float" />
        <div className="absolute top-1/3 left-1/2 w-4 h-4 bg-gradient-to-br from-yellow-400/40 to-orange-400/40 rounded-full animate-pulse-gentle" />
      </div>

      {/* Main content area */}
      <div className="relative z-10 flex items-center justify-center min-h-screen w-full p-4 md:p-6 lg:p-8">
        {/* Enhanced signup card with PancakeSwap styling */}
        <div className="relative w-full sm:max-w-md md:max-w-lg animate-slide-up">
          {/* Card background decorations */}
          <div className="absolute -top-8 -left-8 w-16 h-16 bg-gradient-to-br from-orange-400/20 to-yellow-400/20 rounded-full blur-xl animate-pulse-gentle" />
          <div className="absolute -bottom-8 -right-8 w-20 h-20 bg-gradient-to-br from-blue-400/20 to-cyan-400/20 rounded-full blur-xl animate-float" />
          
          <Card className="relative bg-white/90 dark:bg-slate-800/90 backdrop-blur-xl shadow-2xl border border-orange-200/50 dark:border-orange-400/20 hover:shadow-3xl hover:scale-105 transition-all duration-300 overflow-hidden">
            {/* Card background pattern */}
            <div className="absolute inset-0 bg-gradient-to-br from-orange-50/50 via-transparent to-blue-50/50 dark:from-orange-900/10 dark:via-transparent dark:to-blue-900/10" />
            <div className="absolute top-0 right-0 w-32 h-32 bg-gradient-to-br from-orange-400/10 to-yellow-400/10 rounded-full blur-2xl" />
            <div className="absolute bottom-0 left-0 w-24 h-24 bg-gradient-to-br from-blue-400/10 to-cyan-400/10 rounded-full blur-2xl" />
            
            <CardHeader className="relative z-10 space-y-1">
              <CardTitle className="text-3xl md:text-4xl font-bold mb-4 bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 dark:from-orange-400 dark:via-yellow-400 dark:to-orange-500 bg-clip-text text-transparent animate-gradient-x text-center">
                🥞 Join the Community
              </CardTitle>
              
              {/* Decorative elements under title */}
              <div className="flex justify-center items-center gap-4 mt-2">
                <div className="w-2 h-2 bg-orange-400 rounded-full animate-pulse" />
                <div
                  className="w-3 h-3 bg-yellow-400 rounded-full animate-pulse"
                  style={{ animationDelay: '0.5s' }}
                />
                <div
                  className="w-2 h-2 bg-blue-400 rounded-full animate-pulse"
                  style={{ animationDelay: '1s' }}
                />
              </div>
            </CardHeader>
            
            <CardContent className="relative z-10">
              <div className="space-y-4">
                <EmailPasswordForm isSignUp />

                <div className="text-center mt-6">
                  <Button
                    variant="link"
                    type="button"
                    className="min-h-[48px] text-orange-600 dark:text-orange-400 hover:text-orange-700 dark:hover:text-orange-300 font-medium transition-colors duration-200"
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
        </div>
      </div>
    </main>
  );
}
