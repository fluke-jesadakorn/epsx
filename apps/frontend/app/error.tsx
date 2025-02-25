"use client";

import { Alert } from "antd";
import { ErrorBoundary } from "react-error-boundary";

function ErrorFallback({
  error,
  resetErrorBoundary,
}: {
  error: Error;
  resetErrorBoundary: () => void;
}) {
  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white p-6 rounded-lg shadow-lg">
        <Alert
          message="Error"
          description={error.message}
          type="error"
          showIcon
          closable
          onClose={resetErrorBoundary}
          style={{ width: 400 }}
        />
      </div>
    </div>
  );
}

export default function Error({ children }: { children: React.ReactNode }) {
  return (
    <ErrorBoundary FallbackComponent={ErrorFallback}>{children}</ErrorBoundary>
  );
}

// TODO: Add error logging to server
// TODO: Implement different error types handling
// TODO: Add retry mechanism for recoverable errors
// TODO: Consider adding error code mapping for better user messages
