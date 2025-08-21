'use client';

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Eye, EyeOff, Lock, Mail, User } from 'lucide-react';

interface OIDCRegisterFormProps {
  redirectTo?: string;
}

function OIDCRegisterFormComponent({ redirectTo = '/dashboard' }: OIDCRegisterFormProps) {
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
    const display_name = formData.get('display_name') as string;

    try {
      // POST directly to our backend OIDC registration endpoint
      const backendUrl = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8082';
      const response = await fetch(`${backendUrl}/oauth/register`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: new URLSearchParams({
          email,
          password,
          display_name,
        }),
        credentials: 'include',
      });

      if (response.ok) {
        const result = await response.json();
        console.log('✅ Registration successful:', result);
        
        // Redirect to login page with success message
        window.location.href = `/login?message=${encodeURIComponent('Registration successful! Please sign in with your new account.')}`;
      } else {
        const errorText = await response.text();
        setError('Registration failed. Email may already be in use or password too weak.');
      }
    } catch (error) {
      console.error('❌ Registration failed:', error);
      setError('Registration failed. Please check your connection and try again.');
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

        {/* Display Name field */}
        <div className="space-y-2">
          <Label htmlFor="display_name" className="text-sm font-medium">
            👤 Display Name (Optional)
          </Label>
          <div className="relative">
            <User className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              id="display_name"
              name="display_name"
              type="text"
              className="pl-10 h-12"
              placeholder="Your full name"
              autoComplete="name"
            />
          </div>
        </div>

        {/* Email field */}
        <div className="space-y-2">
          <Label htmlFor="email" className="text-sm font-medium">
            📧 Email Address
          </Label>
          <div className="relative">
            <Mail className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              id="email"
              name="email"
              type="email"
              required
              className="pl-10 h-12"
              placeholder="your.email@example.com"
              autoComplete="email"
            />
          </div>
        </div>

        {/* Password field */}
        <div className="space-y-2">
          <Label htmlFor="password" className="text-sm font-medium">
            🔒 Password
          </Label>
          <div className="relative">
            <Lock className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              id="password"
              name="password"
              type={showPassword ? 'text' : 'password'}
              required
              minLength={8}
              className="pl-10 pr-10 h-12"
              placeholder="Choose a strong password"
              autoComplete="new-password"
            />
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="absolute right-0 top-0 h-full px-3 hover:bg-transparent"
              onClick={() => setShowPassword(!showPassword)}
            >
              {showPassword ? (
                <EyeOff className="h-4 w-4 text-muted-foreground" />
              ) : (
                <Eye className="h-4 w-4 text-muted-foreground" />
              )}
            </Button>
          </div>
          <p className="text-xs text-muted-foreground">
            Password must be at least 8 characters long
          </p>
        </div>

        {/* Submit button */}
        <Button
          type="submit"
          disabled={isLoading}
          className="w-full h-12 text-lg font-semibold bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 text-white border-0 shadow-lg"
        >
          {isLoading ? (
            <>
              <div className="mr-2 h-4 w-4 animate-spin rounded-full border-2 border-white border-t-transparent" />
              Creating account...
            </>
          ) : (
            <>
              🚀 Create Account
            </>
          )}
        </Button>
      </form>

      {/* Security notice */}
      <div className="text-center">
        <p className="text-xs text-muted-foreground">
          🔒 Account creation powered by Firebase Admin
        </p>
      </div>
    </div>
  );
}

export default OIDCRegisterFormComponent;
export { OIDCRegisterFormComponent as OIDCRegisterForm };