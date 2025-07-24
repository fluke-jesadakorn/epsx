'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { requestPasswordResetAction } from '@/app/actions/auth';

interface ForgotPasswordFormProps {
  prefillEmail?: string;
}

export function ForgotPasswordForm({ prefillEmail }: ForgotPasswordFormProps) {
  const [email, setEmail] = useState(prefillEmail || '');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const router = useRouter();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setLoading(true);

    try {
      const result = await requestPasswordResetAction(email);
      
      if (!result.success) {
        setError(result.error || 'Failed to send reset email');
        return;
      }
      
      // Redirect to success page
      router.push('/forgot-password?success=true');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to send reset email');
    } finally {
      setLoading(false);
    }
  };

  return (
    <form className="space-y-6" onSubmit={handleSubmit}>
      {error && (
        <div className="rounded-2xl bg-gradient-to-r from-red-50 to-red-100 dark:from-red-900/20 dark:to-red-800/20 p-4 border border-red-200 dark:border-red-800">
          <div className="text-sm text-red-700 dark:text-red-400 font-medium">⚠️ {error}</div>
        </div>
      )}
      
      <div>
        <label htmlFor="email" className="block text-sm font-medium text-foreground mb-2">
          📧 Email Address
        </label>
        <input
          id="email"
          name="email"
          type="email"
          autoComplete="email"
          required
          className="block w-full px-4 py-3 rounded-xl border border-border bg-background/50 text-foreground placeholder-muted-foreground focus:outline-none focus:ring-2 focus:ring-orange-500 focus:border-orange-500 transition-all duration-300"
          placeholder="Enter your email address"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
        />
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
              Sending reset link...
            </>
          ) : (
            <>
              🔑 Send reset link
            </>
          )}
        </button>
      </div>
    </form>
  );
}