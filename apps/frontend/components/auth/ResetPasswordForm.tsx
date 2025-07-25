'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { resetPasswordAction } from '@/app/actions/auth';
import { calculatePasswordStrength, isPasswordWeak } from '@/lib/password-strength';

interface ResetPasswordFormProps {
  token: string;
}

export function ResetPasswordForm({ token }: ResetPasswordFormProps) {
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [success, setSuccess] = useState(false);
  const [showPassword, setShowPassword] = useState(false);
  const [showConfirmPassword, setShowConfirmPassword] = useState(false);
  const router = useRouter();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    // Security: Clear any previous error state

    // Security: Client-side validation for password matching
    if (password !== confirmPassword) {
      setError('Passwords do not match');
      return;
    }

    // Security: Check password strength
    if (isPasswordWeak(password)) {
      setError('Password is too weak. Please choose a stronger password.');
      return;
    }

    setLoading(true);

    try {
      // Security: Call server action with validated token and password
      const result = await resetPasswordAction(token, password);
      
      if (!result.success) {
        setError(result.error || 'Failed to reset password');
        return;
      }
      
      setSuccess(true);
      setTimeout(() => {
        router.push('/login');
      }, 3000);
    } catch (err) {
      // Security: Safely handle errors without exposing sensitive information
      setError(err instanceof Error ? err.message : 'Failed to reset password');
    } finally {
      setLoading(false);
    }
  };

  if (success) {
    return (
      <div className="text-center space-y-6">
        <div className="mb-6">
          <div className="inline-flex h-16 w-16 items-center justify-center rounded-full bg-gradient-to-r from-green-500 to-emerald-500 mb-4">
            <span className="text-2xl">✅</span>
          </div>
        </div>
        <h2 className="text-2xl font-bold pancake-gradient-text">
          Password Reset Successful!
        </h2>
        <p className="text-muted-foreground">
          Your password has been successfully updated. You will be redirected to the login page shortly.
        </p>
        <a 
          href="/login"
          className="inline-flex items-center gap-2 px-6 py-3 text-sm font-medium bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 text-white rounded-xl shadow-lg hover:shadow-xl transition-all duration-300"
        >
          Continue to Login
        </a>
      </div>
    );
  }

  return (
    <form className="space-y-6" onSubmit={handleSubmit}>
      {/* Security: Display validation errors safely */}
      {error && (
        <div className="rounded-2xl bg-gradient-to-r from-red-50 to-red-100 dark:from-red-900/20 dark:to-red-800/20 p-4 border border-red-200 dark:border-red-800">
          <div className="text-sm text-red-700 dark:text-red-400 font-medium">⚠️ {error}</div>
        </div>
      )}
      
      <div className="space-y-4">
        <div>
          <label htmlFor="password" className="block text-sm font-medium text-foreground mb-2">
            🔒 New Password
          </label>
          <div className="relative">
            <input
              id="password"
              name="password"
              type={showPassword ? 'text' : 'password'}
              autoComplete="new-password"
              required
              minLength={8}
              // Security: Password input with visibility toggle
              className="block w-full px-4 py-3 pr-12 rounded-xl border border-border bg-background/50 text-foreground placeholder-muted-foreground focus:outline-none focus:ring-2 focus:ring-orange-500 focus:border-orange-500 transition-all duration-300"
              placeholder="Enter new password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
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
          {password ? (
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
                      <div className={strength.requirements.length ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}>
                        {strength.requirements.length ? '✓' : '✗'} At least 8 characters
                      </div>
                      <div className={strength.requirements.lowercase ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}>
                        {strength.requirements.lowercase ? '✓' : '✗'} Lowercase letter
                      </div>
                      <div className={strength.requirements.uppercase ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}>
                        {strength.requirements.uppercase ? '✓' : '✗'} Uppercase letter
                      </div>
                      <div className={strength.requirements.numbers ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}>
                        {strength.requirements.numbers ? '✓' : '✗'} Number
                      </div>
                      <div className={strength.requirements.symbols ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}>
                        {strength.requirements.symbols ? '✓' : '✗'} Special character
                      </div>
                    </div>
                  </>
                );
              })()}
            </div>
          ) : (
            <p className="mt-1 text-xs text-muted-foreground">
              Must be at least 8 characters long
            </p>
          )}
        </div>
        
        <div>
          <label htmlFor="confirmPassword" className="block text-sm font-medium text-foreground mb-2">
            🔒 Confirm New Password
          </label>
          <div className="relative">
            <input
              id="confirmPassword"
              name="confirmPassword"
              type={showConfirmPassword ? 'text' : 'password'}
              autoComplete="new-password"
              required
              minLength={8}
              // Security: Password confirmation with visibility toggle
              className="block w-full px-4 py-3 pr-12 rounded-xl border border-border bg-background/50 text-foreground placeholder-muted-foreground focus:outline-none focus:ring-2 focus:ring-orange-500 focus:border-orange-500 transition-all duration-300"
              placeholder="Confirm new password"
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
            />
            <button
              type="button"
              onClick={() => setShowConfirmPassword(!showConfirmPassword)}
              className="absolute right-3 top-1/2 transform -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors duration-200"
              aria-label={showConfirmPassword ? 'Hide password' : 'Show password'}
            >
              <span className="text-lg">{showConfirmPassword ? '🙈' : '👁️'}</span>
            </button>
          </div>
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
              Updating password...
            </>
          ) : (
            <>
              🔑 Update password
            </>
          )}
        </button>
      </div>
    </form>
  );
}