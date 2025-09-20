'use client';

import { useEffect } from 'react';
import { AlertCircle, RefreshCw } from 'lucide-react';
import { Button } from '@/components/ui/button';

interface AdminProfileErrorProps {
  error: Error & { digest?: string };
  reset: () => void;
}

export default function AdminProfileError({ error, reset }: AdminProfileErrorProps) {
  useEffect(() => {
    console.error('Admin profile page error:', error);
  }, [error]);

  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900">
      <div className="container mx-auto px-6 py-8">
        <div className="max-w-2xl mx-auto">
          <div className="bg-white dark:bg-slate-800 rounded-2xl border border-yellow-200 dark:border-slate-700 p-8 text-center">
            <div className="flex justify-center mb-6">
              <div className="p-3 bg-red-100 dark:bg-red-900/20 rounded-full">
                <AlertCircle className="h-8 w-8 text-red-600 dark:text-red-400" />
              </div>
            </div>

            <h1 className="text-2xl font-bold text-slate-900 dark:text-slate-100 mb-4">
              Admin Profile Error
            </h1>

            <p className="text-slate-600 dark:text-slate-400 mb-6">
              Something went wrong while loading your admin profile. This might be a temporary issue.
            </p>

            {process.env.NODE_ENV === 'development' && (
              <div className="mb-6 p-4 bg-red-50 dark:bg-red-900/10 rounded-lg text-left">
                <p className="text-sm text-red-700 dark:text-red-300 font-mono">
                  {error.message}
                </p>
                {error.digest && (
                  <p className="text-xs text-red-600 dark:text-red-400 mt-2">
                    Error ID: {error.digest}
                  </p>
                )}
              </div>
            )}

            <div className="flex gap-4 justify-center">
              <Button onClick={reset} className="flex items-center gap-2">
                <RefreshCw className="h-4 w-4" />
                Try Again
              </Button>
              <Button 
                variant="outline" 
                onClick={() => window.location.href = '/'}
              >
                Go to Dashboard
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}