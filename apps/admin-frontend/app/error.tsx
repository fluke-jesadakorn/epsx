'use client';

import { ErrorContent } from '@/components/shared';

/**
 * Error Page
 * Uses unified status page component for consistent design
 */
export default function Error({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  return (
    <ErrorContent
      title="Something Went Wrong"
      // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
      message={error.message ?? 'An unexpected error occurred. Please try again.'}
      errorId={error.digest}
      onReset={reset}
    />
  );
}
