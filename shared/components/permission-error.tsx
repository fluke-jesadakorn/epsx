'use client';

interface PermissionErrorProps {
  error: {
    error_type: string;
    message: string;
    user_message: string;
    details: {
      permission?: string;
      required_group?: string;
      current_group?: string;
      wallet_address?: string;
    };
    suggested_actions: string[];
    upgrade_info?: {
      current_group: string;
      required_group: string;
      upgrade_url?: string;
      benefits: string[];
    };
  };
}

export function PermissionError({ error }: PermissionErrorProps) {
  return (
    <div className="bg-red-50 border border-red-200 rounded-lg p-6 max-w-2xl mx-auto">
      <div className="flex items-center mb-4">
        <div className="flex-shrink-0">
          <svg className="h-6 w-6 text-red-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2}
              d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.732-.833-2.464 0L3.34 16.5c-.77.833.192 2.5 1.732 2.5z" />
          </svg>
        </div>
        <div className="ml-3">
          <h3 className="text-lg font-medium text-red-900">Access Restricted</h3>
        </div>
      </div>

      <p className="text-red-700 mb-4">{error.user_message}</p>

      {error.details.permission !== undefined && error.details.permission !== '' && (
        <div className="mb-4">
          <p className="text-sm font-medium text-red-800 mb-1">Required Permission:</p>
          <code className="bg-red-100 px-2 py-1 rounded text-sm text-red-800 font-mono">
            {error.details.permission}
          </code>
        </div>
      )}

      {error.details.required_group !== undefined && error.details.required_group !== '' && (
        <div className="mb-4">
          <p className="text-sm font-medium text-red-800 mb-1">Required Permission Group:</p>
          <span className="bg-red-100 px-2 py-1 rounded text-sm text-red-800 font-medium">
            {error.details.required_group}
          </span>
        </div>
      )}

      {error.details.current_group !== undefined && error.details.current_group !== '' && (
        <div className="mb-4">
          <p className="text-sm font-medium text-red-800 mb-1">Your Current Group:</p>
          <span className="bg-red-100 px-2 py-1 rounded text-sm text-red-800 font-medium">
            {error.details.current_group}
          </span>
        </div>
      )}

      {error.suggested_actions.length > 0 && (
        <div className="mb-4">
          <p className="text-sm font-medium text-red-800 mb-2">Suggested Actions:</p>
          <ul className="list-disc list-inside space-y-1 text-sm text-red-700">
            {error.suggested_actions.map((action) => (
              <li key={action}>{action}</li>
            ))}
          </ul>
        </div>
      )}

      {error.upgrade_info && (
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 mt-4">
          <div className="flex items-center mb-2">
            <svg className="h-5 w-5 text-blue-600 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2}
                d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
            <h4 className="font-medium text-blue-900">Permission Group Upgrade Available</h4>
          </div>

          <p className="text-sm text-blue-800 mb-2">
            Upgrade from <strong>{error.upgrade_info.current_group}</strong> to{' '}
            <strong>{error.upgrade_info.required_group}</strong>
          </p>

          {error.upgrade_info.benefits.length > 0 && (
            <div className="mb-3">
              <p className="text-sm font-medium text-blue-800 mb-1">Benefits:</p>
              <ul className="list-disc list-inside text-sm text-blue-700 space-y-1">
                {error.upgrade_info.benefits.map((benefit) => (
                  <li key={benefit}>{benefit}</li>
                ))}
              </ul>
            </div>
          )}

          {error.upgrade_info.upgrade_url !== undefined && error.upgrade_info.upgrade_url !== '' && (
            <a
              href={error.upgrade_info.upgrade_url}
              className="inline-flex items-center bg-blue-600 text-white px-4 py-2 rounded text-sm font-medium hover:bg-blue-700 transition-colors"
            >
              <svg className="h-4 w-4 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2}
                  d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
              </svg>
              Upgrade Permission Group
            </a>
          )}
        </div>
      )}

      <div className="mt-4 pt-4 border-t border-red-200">
        <p className="text-xs text-red-600">
          Error ID: {error.error_type} | If you believe this is an error, please contact support.
        </p>
      </div>
    </div>
  );
}

export default PermissionError;