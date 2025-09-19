'use client';

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Eye, EyeOff, Lock, Mail, User } from 'lucide-react';
import { URL, URLContext, Service } from '../../../../shared/utils/url-resolver';
import { sanitizeAuthFormData, validatePassword } from '@/lib/xss-protection';

interface OIDCRegisterFormProps {
  redirectTo?: string;
}

function OIDCRegisterFormComponent({ redirectTo = '/dashboard' }: OIDCRegisterFormProps) {
  const [isLoading, setIsLoading] = useState(false);
  const [showPassword, setShowPassword] = useState(false);
  const [error, setError] = useState('');
  const [validationErrors, setValidationErrors] = useState<string[]>([]);

  async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setIsLoading(true);
    setError('');
    setValidationErrors([]);

    const formData = new FormData(event.currentTarget);
    const rawEmail = formData.get('email') as string;
    const rawPassword = formData.get('password') as string;
    const rawDisplayName = formData.get('display_name') as string;

    // Sanitize and validate input data
    const sanitized = sanitizeAuthFormData({
      email: rawEmail,
      display_name: rawDisplayName,
      redirectTo
    });

    const passwordValidation = validatePassword(rawPassword);

    if (!sanitized.isValid || !passwordValidation.isValid) {
      const allErrors = [...sanitized.errors, ...passwordValidation.errors];
      setValidationErrors(allErrors);
      setError('Please fix the validation errors below');
      setIsLoading(false);
      return;
    }

    try {
      // POST directly to our backend OIDC registration endpoint using sanitized data
      const backendUrl = URL.get(Service.BACKEND, URLContext.CLIENT);
      const response = await fetch(`${backendUrl}/oauth/register`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: new URLSearchParams({
          email: sanitized.email,
          password: rawPassword, // Use original password (sanitization not needed for passwords)
          display_name: sanitized.displayName || '',
        }),
        credentials: 'include',
      });

      if (response.ok) {
        const result = await response.json();
        
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

        {/* Validation errors */}
        {validationErrors.length > 0 && (
          <div className="rounded-lg bg-yellow-50 dark:bg-yellow-900/20 p-3 border border-yellow-200 dark:border-yellow-800">
            <p className="text-sm font-medium text-yellow-800 dark:text-yellow-200 mb-2">Please fix these issues:</p>
            <ul className="text-sm text-yellow-700 dark:text-yellow-300 space-y-1">
              {validationErrors.map((error, index) => (
                <li key={index} className="flex items-start gap-2">
                  <span className="text-yellow-600">•</span>
                  <span>{error}</span>
                </li>
              ))}
            </ul>
          </div>
        )}

        {/* Display Name field - Mobile optimized */}
        <div className="space-y-3">
          <Label htmlFor="display_name" className="text-sm sm:text-base font-medium flex items-center gap-2">
            <User className="h-4 w-4 text-primary" />
            Display Name
            <span className="text-xs text-muted-foreground">(Optional)</span>
          </Label>
          <div className="relative">
            <Input
              id="display_name"
              name="display_name"
              type="text"
              className="h-12 sm:h-14 text-base pl-4 pr-4 rounded-xl border-2 focus:border-primary"
              placeholder="Your full name"
              autoComplete="name"
            />
            <div className="absolute inset-0 rounded-xl border-2 border-transparent bg-gradient-to-r from-primary/20 to-secondary/20 opacity-0 focus-within:opacity-100 pointer-events-none" />
          </div>
        </div>

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
            <div className="absolute inset-0 rounded-xl border-2 border-transparent bg-gradient-to-r from-primary/20 to-secondary/20 opacity-0 focus-within:opacity-100 pointer-events-none" />
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
              minLength={8}
              className="h-12 sm:h-14 text-base pl-4 pr-14 rounded-xl border-2 focus:border-primary"
              placeholder="Choose a strong password"
              autoComplete="new-password"
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
            {/* Visual focus indicator */}
            <div className="absolute inset-0 rounded-xl border-2 border-transparent bg-gradient-to-r from-primary/20 to-secondary/20 opacity-0 focus-within:opacity-100 pointer-events-none" />
          </div>
          <p className="text-xs text-muted-foreground">
            Password must be at least 8 characters long
          </p>
        </div>

        {/* Submit button - Mobile optimized */}
        <Button
          type="submit"
          disabled={isLoading}
          className="w-full h-12 sm:h-14 text-base sm:text-lg font-semibold bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 text-white border-0 shadow-lg rounded-xl"
        >
          {isLoading ? (
            <>
              <div className="mr-2 h-4 w-4 sm:h-5 sm:w-5 bg-white rounded" />
              <span>Creating account...</span>
            </>
          ) : (
            <div className="flex items-center justify-center gap-2">
              <span className="text-lg sm:text-xl">🚀</span>
              <span>Create Account</span>
            </div>
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