import { Suspense } from 'react';
import AuthPageClient from './auth-page-client';

export default function AuthPage() {
  return (
    <Suspense fallback={
      <div className="fixed inset-0 z-50 flex items-center justify-center bg-background">
        <div className="h-8 w-8 animate-spin rounded-full border-t-2 border-purple-500" />
      </div>
    }>
      <AuthPageClient />
    </Suspense>
  );
}
