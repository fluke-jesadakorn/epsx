'use client';

import {
  calculatePasswordStrength,
  isPasswordWeak,
} from '@/lib/password-strength';
import { useRouter } from 'next/navigation';
import { signIn } from 'next-auth/react';
import React, { useState } from 'react';

interface RegisterFormProps {
  redirectTo?: string;
}

export function RegisterForm({ redirectTo }: RegisterFormProps) {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [displayName, setDisplayName] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [showPassword, setShowPassword] = useState(false);
  const [showConfirmPassword, setShowConfirmPassword] = useState(false);
  const router = useRouter();

  // Component mounted

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    e.stopPropagation();
    // Direct registration handler called
    setError('');
    setLoading(true);

    // Client-side validation
    if (password !== confirmPassword) {
      setError('Passwords do not match');
      setLoading(false);
      return;
    }

    if (isPasswordWeak(password)) {
      setError('Password is too weak. Please choose a stronger password.');
      setLoading(false);
      return;
    }

    try {
      // Use centralized auth API for registration
      
      // Create registration request to backend
      const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
      const registerResponse = await fetch(`${backendUrl}/api/v1/auth/register`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({
          type: 'credentials',
          email,
          password,
          displayName,
        }),
      });

      if (!registerResponse.ok) {
        const errorData = await registerResponse.text();
        throw new Error(errorData || 'Registration failed');
      }

      // Then login automatically using NextAuth
      const result = await signIn('credentials', {
        email,
        password,
        redirect: false,
      });

      if (result?.error) {
        throw new Error(result.error);
      }

      // Registration and login successful
      // Redirect to dashboard
      router.push(redirectTo || '/dashboard');
      router.refresh();
    } catch (err) {
      console.error('❌ Registration error:', err);
      setError(err instanceof Error ? err.message : 'Registration failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <form className="space-y-6" onSubmit={handleSubmit}>
      {/* Security: Display validation errors safely */}
      {error && (
        <div className="rounded-2xl bg-gradient-to-r from-red-50 to-red-100 dark:from-red-900/20 dark:to-red-800/20 p-4 border border-red-200 dark:border-red-800">
          <div className="text-sm text-red-700 dark:text-red-400 font-medium">
            ⚠️ {error}
          </div>
        </div>
      )}

      <div className="space-y-4">
        <div>
          <label
            htmlFor="displayName"
            className="block text-sm font-medium text-foreground mb-2"
          >
            👤 Display Name
          </label>
          <input
            id="displayName"
            name="displayName"
            type="text"
            // Security: Standard text input for display name
            className="block w-full px-4 py-3 rounded-xl border border-border bg-background/50 text-foreground placeholder-muted-foreground focus:outline-none focus:ring-2 focus:ring-orange-500 focus:border-orange-500 transition-all duration-300"
            placeholder="Enter your display name"
            value={displayName}
            onChange={e => setDisplayName(e.target.value)}
          />
        </div>

        <div>
          <label
            htmlFor="email"
            className="block text-sm font-medium text-foreground mb-2"
          >
            📧 Email Address
          </label>
          <input
            id="email"
            name="email"
            type="email"
            autoComplete="email"
            required
            // Security: HTML5 email validation and autocomplete
            className="block w-full px-4 py-3 rounded-xl border border-border bg-background/50 text-foreground placeholder-muted-foreground focus:outline-none focus:ring-2 focus:ring-orange-500 focus:border-orange-500 transition-all duration-300"
            placeholder="Enter your email"
            value={email}
            onChange={e => setEmail(e.target.value)}
          />
        </div>

        <div>
          <label
            htmlFor="password"
            className="block text-sm font-medium text-foreground mb-2"
          >
            🔒 Password
          </label>
          <div className="relative">
            <input
              id="password"
              name="password"
              type={showPassword ? 'text' : 'password'}
              autoComplete="new-password"
              required
              // Security: Password input with visibility toggle
              className="block w-full px-4 py-3 pr-12 rounded-xl border border-border bg-background/50 text-foreground placeholder-muted-foreground focus:outline-none focus:ring-2 focus:ring-orange-500 focus:border-orange-500 transition-all duration-300"
              placeholder="Enter your password"
              value={password}
              onChange={e => setPassword(e.target.value)}
            />
            <button
              type="button"
              onClick={() => setShowPassword(!showPassword)}
              className="absolute right-3 top-1/2 transform -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors duration-200"
              aria-label={showPassword ? 'Hide password' : 'Show password'}
            >
              <span className="text-lg">{showPassword ? '🙈' : '👁️'}</span>
            </button>
          </div>
          {/* Security: Password strength indicator */}
          {password && (
            <div className="mt-2 space-y-2">
              {(() => {
                const strength = calculatePasswordStrength(password);
                return (
                  <>
                    <div className="flex items-center gap-2">
                      <span className="text-sm">{strength.symbol}</span>
                      <span className={`text-sm font-medium ${strength.color}`}>
                        Password strength: {strength.text} ({strength.score}/5)
                      </span>
                    </div>
                    <div className="text-xs text-muted-foreground space-y-1">
                      <div
                        className={
                          strength.requirements.length
                            ? 'text-green-600 dark:text-green-400'
                            : 'text-red-600 dark:text-red-400'
                        }
                      >
                        {strength.requirements.length ? '✓' : '✗'} At least 8
                        characters
                      </div>
                      <div
                        className={
                          strength.requirements.lowercase
                            ? 'text-green-600 dark:text-green-400'
                            : 'text-red-600 dark:text-red-400'
                        }
                      >
                        {strength.requirements.lowercase ? '✓' : '✗'} Lowercase
                        letter
                      </div>
                      <div
                        className={
                          strength.requirements.uppercase
                            ? 'text-green-600 dark:text-green-400'
                            : 'text-red-600 dark:text-red-400'
                        }
                      >
                        {strength.requirements.uppercase ? '✓' : '✗'} Uppercase
                        letter
                      </div>
                      <div
                        className={
                          strength.requirements.numbers
                            ? 'text-green-600 dark:text-green-400'
                            : 'text-red-600 dark:text-red-400'
                        }
                      >
                        {strength.requirements.numbers ? '✓' : '✗'} Number
                      </div>
                      <div
                        className={
                          strength.requirements.symbols
                            ? 'text-green-600 dark:text-green-400'
                            : 'text-red-600 dark:text-red-400'
                        }
                      >
                        {strength.requirements.symbols ? '✓' : '✗'} Special
                        character
                      </div>
                    </div>
                  </>
                );
              })()}
            </div>
          )}
        </div>

        <div>
          <label
            htmlFor="confirmPassword"
            className="block text-sm font-medium text-foreground mb-2"
          >
            🔒 Confirm Password
          </label>
          <div className="relative">
            <input
              id="confirmPassword"
              name="confirmPassword"
              type={showConfirmPassword ? 'text' : 'password'}
              autoComplete="new-password"
              required
              // Security: Password confirmation with visibility toggle
              className={`block w-full px-4 py-3 pr-12 rounded-xl border bg-background/50 text-foreground placeholder-muted-foreground focus:outline-none focus:ring-2 transition-all duration-300 ${
                confirmPassword && password !== confirmPassword
                  ? 'border-red-500 focus:ring-red-500 focus:border-red-500'
                  : 'border-border focus:ring-orange-500 focus:border-orange-500'
              }`}
              placeholder="Confirm your password"
              value={confirmPassword}
              onChange={e => setConfirmPassword(e.target.value)}
            />
            <button
              type="button"
              onClick={() => setShowConfirmPassword(!showConfirmPassword)}
              className="absolute right-3 top-1/2 transform -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors duration-200"
              aria-label={
                showConfirmPassword ? 'Hide password' : 'Show password'
              }
            >
              <span className="text-lg">
                {showConfirmPassword ? '🙈' : '👁️'}
              </span>
            </button>
          </div>
          {/* Security: Real-time password confirmation feedback */}
          {confirmPassword && password !== confirmPassword && (
            <p className="mt-2 text-sm text-red-600 dark:text-red-400">
              Passwords do not match
            </p>
          )}
        </div>
      </div>

      <div className="pt-4">
        <button
          type="submit"
          disabled={loading}
          className="w-full flex justify-center items-center gap-2 py-4 px-6 text-base font-bold bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 text-white rounded-2xl shadow-lg hover:shadow-xl hover:scale-105 transition-all duration-300 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:scale-100"
        >
          {loading ? (
            <>
              <div className="animate-spin rounded-full h-5 w-5 border-2 border-white border-t-transparent"></div>
              Creating account...
            </>
          ) : (
            <>🚀 Create account</>
          )}
        </button>
      </div>
    </form>
  );
}
