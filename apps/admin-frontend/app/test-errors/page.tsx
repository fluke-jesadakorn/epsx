'use client';

import { useState } from 'react';
import { PermissionErrorUI } from '@/components/errors/PermissionErrorUI';
import { AdminErrorBoundary } from '@/components/errors/AdminErrorBoundary';
import useAdminApi from '@/hooks/useAdminApi';
import { ApiError } from '@/lib/api/response-handler';

/**
 * Test page for demonstrating backend-only permission error handling
 * This page shows how different permission errors are displayed to users
 */
export default function TestErrorsPage() {
  const { loading, error, clearError, getUsers, getSystemHealth } = useAdminApi();
  const [selectedErrorType, setSelectedErrorType] = useState<string>('');

  // Mock different types of permission errors for demonstration
  const mockErrors: Record<string, ApiError> = {
    permission_denied: {
      success: false,
      error: {
        type: 'PERMISSION_DENIED',
        code: 'PERMISSION_DENIED',
        message: 'Permission denied',
        user_message: 'You don\'t have permission to access this resource.',
        details: {},
        suggested_actions: ['Contact your administrator to request access'],
        permission: 'admin:users:manage',
        required_permissions: ['admin:users:manage'],
        user_permissions: ['admin:analytics:view'],
        access_level_required: 'admin',
        current_access_level: 'viewer'
      }
    },
    insufficient_tier: {
      success: false,
      error: {
        type: 'INSUFFICIENT_TIER',
        code: 'INSUFFICIENT_TIER',
        message: 'Insufficient tier level',
        user_message: 'This feature requires a higher tier subscription.',
        details: {},
        suggested_actions: ['Upgrade your subscription'],
        current_tier: 'basic',
        required_tier: 'premium',
        missing_permissions: ['admin:advanced:*'],
        upgrade_info: {
          current_tier: 'basic',
          required_tier: 'premium',
          upgrade_url: '/upgrade',
          benefits: ['Advanced user management', 'System analytics', 'Bulk operations']
        }
      }
    },
    permission_expired: {
      success: false,
      error: {
        type: 'PERMISSION_EXPIRED',
        code: 'PERMISSION_EXPIRED',
        message: 'Permission has expired',
        user_message: 'Your access to this feature has expired.',
        details: {},
        suggested_actions: ['Renew your subscription'],
        expired_permissions: [
          {
            permission: 'admin:users:manage',
            expired_at: new Date(Date.now() - 86400000).toISOString() // 1 day ago
          }
        ],
        renewal_url: '/renew'
      }
    },
    rate_limit: {
      success: false,
      error: {
        type: 'RATE_LIMIT_EXCEEDED',
        code: 'RATE_LIMIT_EXCEEDED',
        message: 'Usage limit exceeded',
        user_message: 'You have exceeded your usage limits.',
        details: {},
        suggested_actions: ['Wait for limits to reset', 'Upgrade for higher limits'],
        rate_limit: {
          limit: 100,
          remaining: 0,
          reset_at: new Date(Date.now() + 3600000).toISOString(), // 1 hour from now
          window_size: 'hour'
        },
        upgrade_for_higher_limits: {
          tier: 'premium',
          new_limit: 1000
        }
      }
    }
  };

  const testRealApiCall = async () => {
    await getUsers();
  };

  const testSystemHealth = async () => {
    await getSystemHealth();
  };

  return (
    <div className="p-6 space-y-8">
      <div className="bg-white rounded-lg shadow p-6">
        <h1 className="text-2xl font-bold text-gray-900 mb-4">
          Backend-Only Permission Error Testing
        </h1>
        <p className="text-gray-600 mb-6">
          This page demonstrates how the admin frontend handles different types of permission errors from the backend.
          All permission validation is now handled by the backend, and the frontend only displays appropriate error messages.
        </p>

        {/* Mock Error Demonstrations */}
        <div className="space-y-4 mb-8">
          <h2 className="text-lg font-semibold text-gray-800">Mock Error Examples</h2>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-2">
            {Object.keys(mockErrors).map((errorType) => (
              <button
                key={errorType}
                onClick={() => setSelectedErrorType(errorType)}
                className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 text-sm"
              >
                {errorType.replace('_', ' ')}
              </button>
            ))}
            <button
              onClick={() => setSelectedErrorType('')}
              className="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-700 text-sm"
            >
              Clear
            </button>
          </div>
        </div>

        {/* Display Selected Mock Error */}
        {selectedErrorType && mockErrors[selectedErrorType] && (
          <div className="mb-8">
            <h3 className="text-md font-medium text-gray-800 mb-4">
              Mock Error: {selectedErrorType.replace('_', ' ')}
            </h3>
            <PermissionErrorUI
              error={mockErrors[selectedErrorType]}
              onRetry={() => console.log('Retry clicked')}
              onLogin={() => console.log('Login clicked')}
              onUpgrade={(tier) => console.log('Upgrade clicked:', tier)}
              onSupport={(context) => console.log('Support clicked:', context)}
              variant="card"
            />
          </div>
        )}

        {/* Real API Testing */}
        <div className="space-y-4">
          <h2 className="text-lg font-semibold text-gray-800">Real API Testing</h2>
          <p className="text-gray-600">
            Test real API calls to see how backend permission errors are handled.
          </p>
          
          <div className="flex space-x-4">
            <button
              onClick={testRealApiCall}
              disabled={loading}
              className="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700 disabled:opacity-50"
            >
              {loading ? 'Loading...' : 'Test Get Users'}
            </button>
            <button
              onClick={testSystemHealth}
              disabled={loading}
              className="px-4 py-2 bg-purple-600 text-white rounded hover:bg-purple-700 disabled:opacity-50"
            >
              {loading ? 'Loading...' : 'Test System Health'}
            </button>
            {error && (
              <button
                onClick={clearError}
                className="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700"
              >
                Clear Error
              </button>
            )}
          </div>

          {/* Display Real API Errors */}
          {error && (
            <div className="mt-6">
              <h3 className="text-md font-medium text-gray-800 mb-4">
                Real API Error Response
              </h3>
              <AdminErrorBoundary>
                <PermissionErrorUI
                  error={error}
                  onRetry={clearError}
                  onLogin={() => window.location.href = '/login'}
                  onSupport={(context) => console.log('Support requested:', context)}
                  variant="card"
                />
              </AdminErrorBoundary>
            </div>
          )}
        </div>

        {/* Implementation Notes */}
        <div className="mt-8 p-4 bg-blue-50 rounded-lg">
          <h3 className="text-md font-medium text-blue-800 mb-2">Implementation Notes</h3>
          <ul className="text-sm text-blue-700 space-y-1">
            <li>• All permission validation is now handled by the backend</li>
            <li>• Frontend only displays appropriate error messages based on backend responses</li>
            <li>• Error UI automatically adapts based on error type (permission denied, tier insufficient, etc.)</li>
            <li>• Components use AdminErrorBoundary and useAdminApi for consistent error handling</li>
            <li>• No client-side permission checking or caching</li>
          </ul>
        </div>
      </div>
    </div>
  );
}