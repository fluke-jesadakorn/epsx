"use client";

import React from 'react';

interface ErrorPageProps {
  error: Error & { digest?: string };
  reset: () => void;
}

export default function Error({ error, reset }: ErrorPageProps) {
  React.useEffect(() => {
    console.error('Application error:', error);
  }, [error]);

  return (
    <div className="flex min-h-screen flex-col items-center justify-center p-4">
      <div className="max-w-md w-full text-center">
        <h1 className="text-4xl font-bold mb-4 text-red-600">Oops!</h1>
        <h2 className="text-xl font-semibold mb-4">Something went wrong</h2>
        <p className="text-gray-600 mb-6 text-sm">
          {error.message || 'An unexpected error occurred'}
        </p>
        <button
          className="px-6 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors"
          onClick={reset}
          type="button"
        >
          Try again
        </button>
      </div>
    </div>
  );
}
