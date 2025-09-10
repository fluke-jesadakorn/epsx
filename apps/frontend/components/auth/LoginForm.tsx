'use client';

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Eye, EyeOff, Lock, Mail } from 'lucide-react';

interface LoginFormProps {
  redirectTo?: string;
}

export function LoginForm({ redirectTo = '/dashboard' }: LoginFormProps) {
  const [isLoading, setIsLoading] = useState(false);
  const [showPassword, setShowPassword] = useState(false);
  const [error, setError] = useState('');

  async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setIsLoading(true);
    setError('');

    const formData = new FormData(event.currentTarget);
    const email = formData.get('email') as string;
    const password = formData.get('password') as string;

    try {
      // POST directly to our backend OIDC endpoint
      const backendUrl = process.env.NEXT_PUBLIC_API_URL || 'https://api.epsx.io';
      const response = await fetch(`${backendUrl}/oauth/login-post`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: new URLSearchParams({
          email,
          password,
        }),
        credentials: 'include',
      });

      if (response.ok) {
        const result = await response.json();
        console.log('✅ Login successful:', result);
        
        // Store access token in session API for proper JWT handling
        await fetch('/api/auth/session', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ accessToken: result.access_token }),
          credentials: 'include'
        });
        
        // Redirect to dashboard or callback URL
        window.location.href = redirectTo;
      } else {
        setError('Invalid email or password. Please try again.');
      }
    } catch (error) {
      console.error('❌ Login failed:', error);
      setError('Login failed. Please check your connection and try again.');
    } finally {
      setIsLoading(false);
    }
  }

  return (
    <div className="space-y-6">
      <form onSubmit={handleSubmit} className="space-y-4">
        {/* Error display */}
        {error && (
          <div className="rounded-lg bg-red-50 dark:bg-red-900/20 p-3 border border-red-200 dark:border-red-800">
            <p className="text-sm text-red-700 dark:text-red-400">⚠️ {error}</p>
          </div>
        )}

        {/* Email field - Mobile optimized */}
        <div className="space-y-3">
          <Label htmlFor="email" className="text-sm sm:text-base font-medium flex items-center gap-2">
            <Mail className="h-4 w-4 text-primary" />
            Email Address
          </Label>
          <div className="relative">
            <Input
              id="email"
              name="email"
              type="email"
              required
              className="h-12 sm:h-14 text-base pl-4 pr-4 rounded-xl border-2 focus:border-primary"
              placeholder="your.email@example.com"
              autoComplete="email"
              inputMode="email"
            />
            {/* Removed animated focus indicator for performance */}
          </div>
        </div>

        {/* Password field - Mobile optimized */}
        <div className="space-y-3">
          <Label htmlFor="password" className="text-sm sm:text-base font-medium flex items-center gap-2">
            <Lock className="h-4 w-4 text-primary" />
            Password
          </Label>
          <div className="relative">
            <Input
              id="password"
              name="password"
              type={showPassword ? 'text' : 'password'}
              required
              className="h-12 sm:h-14 text-base pl-4 pr-14 rounded-xl border-2 focus:border-primary"
              placeholder="Enter your password"
              autoComplete="current-password"
            />
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="absolute right-2 top-1/2 -translate-y-1/2 h-8 w-8 sm:h-10 sm:w-10 rounded-lg hover:bg-muted"
              onClick={() => setShowPassword(!showPassword)}
            >
              {showPassword ? (
                <EyeOff className="h-4 w-4 sm:h-5 sm:w-5 text-muted-foreground" />
              ) : (
                <Eye className="h-4 w-4 sm:h-5 sm:w-5 text-muted-foreground" />
              )}
            </Button>
            {/* Removed animated focus indicator for performance */}
          </div>
        </div>

        {/* Submit button - Mobile optimized */}
        <Button
          type="submit"
          disabled={isLoading}
          className="w-full h-12 sm:h-14 text-base sm:text-lg font-semibold bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 text-white border-0 shadow-lg rounded-xl"
        >
          {isLoading ? (
            <>
              <div className="mr-2 h-4 w-4 sm:h-5 sm:w-5 rounded-full border-2 border-white border-t-transparent" />
              <span>Signing in...</span>
            </>
          ) : (
            <div className="flex items-center justify-center gap-2">
              <span className="text-lg sm:text-xl">🚀</span>
              <span>Sign In</span>
            </div>
          )}
        </Button>
      </form>

      {/* Security notice */}
      <div className="text-center">
        <p className="text-xs text-muted-foreground">
          🔒 Secure authentication powered by Firebase Admin
        </p>
      </div>
    </div>
  );
}

export default LoginForm;