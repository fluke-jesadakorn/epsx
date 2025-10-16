'use client';

import { useRouter } from "next/navigation";

export default function Error({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  const router = useRouter();

  const handleSignOut = async () => {
    try {
      await fetch('/api/auth/logout', { method: 'POST' });
      window.location.href = '/';
    } catch (error) {
      console.error('Logout error:', error);
    }
  };

  return (
    <div className="container mx-auto py-8">
      <div className="max-w-4xl mx-auto">
        <div className="bg-card rounded-lg shadow-sm p-6">
          <h1 className="text-2xl font-bold mb-4">Error</h1>
          
          <div className="space-y-4">
            <div className="p-4 bg-destructive/10 text-destructive rounded-md">
              <h2 className="font-semibold mb-2">Something went wrong</h2>
              <p className="text-sm">{error.message}</p>
            </div>

            <div className="flex items-center gap-4">
              <button
                onClick={() => reset()}
                className="px-4 py-2 text-sm font-medium text-white bg-primary rounded-md hover:bg-primary/90"
              >
                Try again
              </button>
              <button
                onClick={handleSignOut}
                className="px-4 py-2 text-sm font-medium border border-input rounded-md hover:bg-accent hover:text-accent-foreground"
              >
                Sign out
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
