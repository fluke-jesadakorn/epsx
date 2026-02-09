"use client";

import React from 'react';

interface ErrorPageProps {
  error: Error & { digest?: string };
  reset: () => void;
}

function isBackendConnectivityError(error: Error): boolean {
  const message = error.message.toLowerCase();
  const statusMatch = message.match(/status (404|502|503|504)/);
  const networkError = message.includes('failed to fetch') ||
                       message.includes('network error') ||
                       message.includes('connection refused');
  const backendError = message.includes('/api/') &&
                       (message.includes('404') || message.includes('not found'));

  return Boolean(statusMatch) || networkError || backendError;
}

function isPermissionError(error: Error): boolean {
  const message = error.message.toLowerCase();
  return message.includes('403') ||
         message.includes('forbidden') ||
         message.includes('permission denied');
}

export default function Error({ error, reset }: ErrorPageProps) {
  React.useEffect(() => {
    // Error logging handled by error boundary
  }, [error]);

  const isBackendDown = isBackendConnectivityError(error);
  const isPermissionDenied = isPermissionError(error);

  const handleReload = () => {
    window.location.reload();
  };

  const handleHome = () => {
    window.location.href = '/';
  };

  if (isBackendDown) {
    return (
      <div className="flex min-h-screen flex-col items-center justify-center p-4">
        <div className="max-w-md w-full text-center">
          <div className="mb-6">
            <div className="w-16 h-16 mx-auto mb-4 bg-orange-100 rounded-full flex items-center justify-center">
              <svg className="w-8 h-8 text-orange-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
              </svg>
            </div>
          </div>

          <h1 className="text-3xl font-bold mb-4 text-gray-900">Backend Unavailable</h1>
          <h2 className="text-xl font-semibold mb-4 text-gray-700">Service Temporarily Down</h2>

          <p className="text-gray-600 mb-6 text-sm leading-relaxed">
            We can't connect to our servers right now. This could be due to:
          </p>

          <ul className="text-gray-600 text-sm mb-6 text-left max-w-sm mx-auto space-y-1">
            <li>• Backend service maintenance</li>
            <li>• Network connectivity issues</li>
            <li>• Temporary server problems</li>
          </ul>

          <div className="space-y-3">
            <button
              className="w-full px-6 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600"
              onClick={handleReload}
              type="button"
            >
              Reload Page
            </button>

            <button
              className="w-full px-6 py-3 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200"
              onClick={handleHome}
              type="button"
            >
              Go to Homepage
            </button>
          </div>

          <p className="text-xs text-gray-500 mt-4">
            If the problem persists, please try again in a few minutes.
          </p>
        </div>
      </div>
    );
  }

  if (isPermissionDenied) {
    return (
      <div className="flex min-h-screen flex-col items-center justify-center p-4">
        <div className="max-w-md w-full text-center">
          <div className="mb-6">
            <div className="w-16 h-16 mx-auto mb-4 bg-red-100 rounded-full flex items-center justify-center">
              <svg className="w-8 h-8 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 15v2m0 0v2m0-2h2m-2 0H8m2-12V4a2 2 0 00-2-2H8a2 2 0 00-2 2v2m12 0V4a2 2 0 00-2-2h-2a2 2 0 00-2 2v2m0 0h2a2 2 0 012 2v2a2 2 0 01-2 2H8a2 2 0 01-2-2V8a2 2 0 012-2h2z" />
              </svg>
            </div>
          </div>

          <h1 className="text-3xl font-bold mb-4 text-red-600">Access Denied</h1>
          <h2 className="text-xl font-semibold mb-4 text-gray-700">Permission Required</h2>

          <p className="text-gray-600 mb-6 text-sm leading-relaxed">
            You don't have permission to access this page. This could be because:
          </p>

          <ul className="text-gray-600 text-sm mb-6 text-left max-w-sm mx-auto space-y-1">
            <li>• You need to sign in first</li>
            <li>• Your account lacks the required permissions</li>
            <li>• This page requires a higher subscription tier</li>
          </ul>

          <div className="space-y-3">
            <button
              className="w-full px-6 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600"
              onClick={() => window.location.href = '/auth/login'}
              type="button"
            >
              Sign In
            </button>

            <button
              className="w-full px-6 py-3 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200"
              onClick={handleHome}
              type="button"
            >
              Back to Homepage
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex min-h-screen flex-col items-center justify-center p-4">
      <div className="max-w-md w-full text-center">
        <div className="mb-6">
          <div className="w-16 h-16 mx-auto mb-4 bg-gray-100 rounded-full flex items-center justify-center">
            <svg className="w-8 h-8 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
          </div>
        </div>

        <h1 className="text-3xl font-bold mb-4 text-gray-900">Oops!</h1>
        <h2 className="text-xl font-semibold mb-4 text-gray-700">Something went wrong</h2>

        <p className="text-gray-600 mb-6 text-sm leading-relaxed">
          {error.message || 'An unexpected error occurred while loading this page.'}
        </p>

        <div className="space-y-3">
          <button
            className="w-full px-6 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600"
            onClick={reset}
            type="button"
          >
            Try Again
          </button>

          <button
            className="w-full px-6 py-3 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200"
            onClick={handleHome}
            type="button"
          >
            Go to Homepage
          </button>
        </div>
      </div>
    </div>
  );
}
