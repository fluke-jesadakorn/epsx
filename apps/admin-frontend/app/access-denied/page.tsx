import { Shield, Home, RotateCcw } from 'lucide-react';
import Link from 'next/link';

interface AccessDeniedPageProps {
  searchParams: {
    route?: string;
    reason?: string;
    context?: string;
    permission?: string;
  };
}

/**
 *
 * @param root0
 * @param root0.searchParams
 */
export default function AccessDeniedPage({ searchParams }: AccessDeniedPageProps) {
  const route = searchParams.route || '/';
  const reason = searchParams.reason || 'Access denied';
  const context = searchParams.context || 'unknown';
  const permission = searchParams.permission;
  
  return (
    <div className="min-h-screen bg-gray-50 flex flex-col justify-center py-12 sm:px-6 lg:px-8">
      <div className="sm:mx-auto sm:w-full sm:max-w-md">
        <div className="flex justify-center">
          <Shield className="h-12 w-12 text-red-500" />
        </div>
        <h2 className="mt-6 text-center text-3xl font-extrabold text-gray-900">
          Access Denied
        </h2>
        <p className="mt-2 text-center text-sm text-gray-600">
          You don&apos;t have permission to access this resource
        </p>
      </div>

      <div className="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
        <div className="bg-white py-8 px-4 shadow sm:rounded-lg sm:px-10">
          <div className="space-y-4">
            <div>
              <h3 className="text-lg font-medium text-gray-900 mb-3">Error Details</h3>
              <div className="bg-red-50 border border-red-200 rounded-md p-4">
                <div className="text-sm space-y-2">
                  <p><span className="font-medium">Reason:</span> {decodeURIComponent(reason)}</p>
                  <p><span className="font-medium">Requested Route:</span> {decodeURIComponent(route)}</p>
                  <p><span className="font-medium">Context:</span> {context}</p>
                  {permission && (
                    <p><span className="font-medium">Required Permission:</span> {decodeURIComponent(permission)}</p>
                  )}
                </div>
              </div>
            </div>

            <div className="flex space-x-3">
              <Link
                href="/login"
                className="flex-1 flex justify-center items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-red-600 hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500"
              >
                <RotateCcw className="h-4 w-4 mr-2" />
                Login Again
              </Link>
              
              <Link
                href="/"
                className="flex-1 flex justify-center items-center px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500"
              >
                <Home className="h-4 w-4 mr-2" />
                Go Home
              </Link>
            </div>

            {context === 'admin' && (
              <div className="mt-4 p-3 bg-blue-50 border border-blue-200 rounded-md">
                <p className="text-sm text-blue-800">
                  <span className="font-medium">Admin Access Required:</span> Only authorized administrators can access this panel. 
                  If you believe this is an error, please contact your system administrator.
                </p>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}