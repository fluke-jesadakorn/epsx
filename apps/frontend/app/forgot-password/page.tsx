import { requireGuest } from '@/app/actions/auth-improved';
import { ForgotPasswordForm } from '@/components/auth/ForgotPasswordForm';

interface ForgotPasswordPageProps {
  searchParams: {
    email?: string;
    success?: string;
    error?: string;
  };
}

export default async function ForgotPasswordPage({ searchParams }: ForgotPasswordPageProps) {
  // Redirect to dashboard if already authenticated
  await requireGuest();

  const email = searchParams.email;
  const success = searchParams.success;
  const error = searchParams.error;

  if (success) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-background relative overflow-hidden py-12 px-4 sm:px-6 lg:px-8">
        <div className="fixed inset-0 z-0 pointer-events-none">
          <div className="absolute inset-0 bg-gradient-to-br from-blue-50 via-orange-50 to-yellow-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900" />
        </div>

        <div className="relative z-10 max-w-md w-full space-y-8">
          <div className="card-pancake p-8 text-center">
            <div className="mb-6">
              <div className="inline-flex h-16 w-16 items-center justify-center rounded-full bg-gradient-to-r from-green-500 to-emerald-500 mb-4">
                <span className="text-2xl">✅</span>
              </div>
            </div>
            <h2 className="text-3xl font-bold pancake-gradient-text mb-4">
              Check Your Email
            </h2>
            <p className="text-muted-foreground mb-6">
              We&apos;ve sent a password reset link to your email address. Please check your inbox and follow the instructions.
            </p>
            <a 
              href="/login"
              className="inline-flex items-center gap-2 px-6 py-3 text-sm font-medium bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 text-white rounded-xl shadow-lg hover:shadow-xl transition-all duration-300"
            >
              Back to Login
            </a>
          </div>
        </div>
      </div>
    );
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
                <span className="text-2xl">🔑</span>
              </div>
            </div>
            <h2 className="text-3xl font-bold pancake-gradient-text">
              Forgot Password?
            </h2>
            <p className="mt-2 text-sm text-muted-foreground">
              Enter your email address and we&apos;ll send you a reset link
            </p>
          </div>
          
          {/* Show server-side error if present */}
          {error && (
            <div className="mb-6 rounded-2xl bg-gradient-to-r from-red-50 to-red-100 dark:from-red-900/20 dark:to-red-800/20 p-4 border border-red-200 dark:border-red-800">
              <div className="text-sm text-red-700 dark:text-red-400 font-medium">⚠️ {decodeURIComponent(error)}</div>
            </div>
          )}
          
          <ForgotPasswordForm prefillEmail={email} />
          
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