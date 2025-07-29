import { requireGuest } from '@/app/actions/auth-improved';
import { ResetPasswordForm } from '@/components/auth/ResetPasswordForm';
import { redirect } from 'next/navigation';

interface ResetPasswordPageProps {
  searchParams: {
    token?: string;
    error?: string;
  };
}

export default async function ResetPasswordPage({ searchParams }: ResetPasswordPageProps) {
  // Security: Ensure only unauthenticated users can access password reset
  await requireGuest();

  const token = searchParams.token;
  const error = searchParams.error;

  // Security: Validate reset token presence to prevent unauthorized access
  if (!token) {
    redirect('/forgot-password?error=Invalid or missing reset token');
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-background relative overflow-hidden py-12 px-4 sm:px-6 lg:px-8">
      <div className="fixed inset-0 z-0 pointer-events-none">
        <div className="absolute inset-0 bg-gradient-to-br from-blue-50 via-orange-50 to-yellow-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900" />
        <div className="absolute top-20 left-10 w-32 h-32 bg-gradient-to-br from-orange-400/20 to-yellow-400/20 rounded-full animate-float" />
        <div className="absolute bottom-20 right-20 w-24 h-24 bg-gradient-to-br from-blue-400/20 to-cyan-400/20 rounded-full animate-bounce-gentle" />
      </div>

      <div className="relative z-10 max-w-md w-full space-y-8">
        <div className="card-pancake p-8">
          <div className="text-center mb-8">
            <div className="mb-6">
              <div className="inline-flex h-16 w-16 items-center justify-center rounded-full bg-gradient-to-r from-orange-500 to-yellow-500 mb-4">
                <span className="text-2xl">🔐</span>
              </div>
            </div>
            <h2 className="text-3xl font-bold pancake-gradient-text">
              Set New Password
            </h2>
            <p className="mt-2 text-sm text-muted-foreground">
              Enter your new password below
            </p>
          </div>
          
          {/* Security: Safely display server-side errors with proper encoding */}
          {error && (
            <div className="mb-6 rounded-2xl bg-gradient-to-r from-red-50 to-red-100 dark:from-red-900/20 dark:to-red-800/20 p-4 border border-red-200 dark:border-red-800">
              <div className="text-sm text-red-700 dark:text-red-400 font-medium">⚠️ {decodeURIComponent(error)}</div>
            </div>
          )}
          
          <ResetPasswordForm token={token} />
          
          <div className="mt-6 text-center">
            <a href="/login" className="font-medium text-muted-foreground hover:text-foreground hover:underline">
              ← Back to login
            </a>
          </div>
        </div>
      </div>
    </div>
  );
}