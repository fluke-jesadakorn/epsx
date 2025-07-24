'use client';

import React from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { useAuth } from '@/context/auth-context';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { AlertTriangle, ArrowLeft, Shield, Info } from 'lucide-react';

export default function AccessDeniedPage() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const { user } = useAuth();
  const effectivePermissions = null; // TODO: Implement permission checking
  const packageTier = 'FREE'; // TODO: Fetch from backend

  const route = searchParams.get('route') || '/unknown';
  const reason = searchParams.get('reason') || 'Insufficient permissions';

  const handleGoBack = () => {
    router.back();
  };

  const handleGoToDashboard = () => {
    router.push('/dashboard');
  };

  return (
    <div className="min-h-screen bg-gray-50 flex items-center justify-center p-4">
      <Card className="w-full max-w-2xl">
        <CardHeader className="text-center">
          <div className="mx-auto w-16 h-16 bg-red-100 rounded-full flex items-center justify-center mb-4">
            <AlertTriangle className="h-8 w-8 text-red-600" />
          </div>
          <CardTitle className="text-2xl font-bold text-gray-900">
            Access Denied
          </CardTitle>
          <p className="text-gray-600 mt-2">
            You don't have permission to access this page
          </p>
        </CardHeader>

        <CardContent className="space-y-6">
          {/* Access Details */}
          <div className="bg-gray-50 rounded-lg p-4">
            <h3 className="font-medium text-gray-900 mb-2 flex items-center gap-2">
              <Info className="h-4 w-4" />
              Access Details
            </h3>
            <div className="space-y-2 text-sm">
              <div>
                <span className="font-medium text-gray-700">Requested Route:</span>
                <span className="ml-2 text-gray-600 font-mono">{route}</span>
              </div>
              <div>
                <span className="font-medium text-gray-700">Reason:</span>
                <span className="ml-2 text-gray-600">{reason}</span>
              </div>
              <div>
                <span className="font-medium text-gray-700">Your Package:</span>
                <span className="ml-2 text-gray-600 capitalize">{packageTier}</span>
              </div>
            </div>
          </div>

          {/* Template Information */}
          {effectivePermissions && (
            <div className="bg-blue-50 rounded-lg p-4">
              <h3 className="font-medium text-gray-900 mb-3 flex items-center gap-2">
                <Shield className="h-4 w-4" />
                Your Current Permissions
              </h3>
              
              <div className="space-y-3">
                <div>
                  <p className="text-sm text-gray-700 mb-2">
                    <strong>{effectivePermissions?.permissions?.length || 0}</strong> permissions active
                  </p>
                  
                  {effectivePermissions?.templateSources && effectivePermissions.templateSources.length > 0 && (
                    <div className="mb-3">
                      <p className="text-xs font-medium text-gray-600 mb-1">Active Templates:</p>
                      <div className="flex flex-wrap gap-1">
                        {effectivePermissions?.templateSources?.map((source: any) => (
                          <span
                            key={source.templateId}
                            className="px-2 py-1 bg-blue-100 text-blue-800 text-xs rounded-full"
                          >
                            {source.templateName} ({source.contributedPermissions.length})
                          </span>
                        ))}
                      </div>
                    </div>
                  )}

                  {effectivePermissions?.conflicts && effectivePermissions.conflicts.length > 0 && (
                    <div className="mb-3">
                      <p className="text-xs font-medium text-orange-600 mb-1">
                        ⚠️ {effectivePermissions?.conflicts?.length || 0} Permission Conflicts
                      </p>
                      <div className="text-xs text-orange-700">
                        Some permissions may not be working as expected due to template conflicts.
                      </div>
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* Suggestions */}
          <div className="bg-yellow-50 rounded-lg p-4">
            <h3 className="font-medium text-gray-900 mb-2">What you can do:</h3>
            <ul className="text-sm text-gray-700 space-y-1">
              <li>• Contact your administrator to request access to this feature</li>
              <li>• Check if your package tier supports this functionality</li>
              <li>• Review available templates that might grant the required permissions</li>
              {packageTier === 'FREE' && (
                <li>• Consider upgrading your package to access more features</li>
              )}
            </ul>
          </div>

          {/* Action Buttons */}
          <div className="flex gap-3 pt-4">
            <Button
              variant="outline"
              onClick={handleGoBack}
              className="flex items-center gap-2"
            >
              <ArrowLeft className="h-4 w-4" />
              Go Back
            </Button>
            <Button
              onClick={handleGoToDashboard}
              className="flex-1"
            >
              Go to Dashboard
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}