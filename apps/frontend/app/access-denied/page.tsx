'use client';

import React, { useEffect, useState } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { useAuth } from '@/context/auth-context';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { AlertTriangle, ArrowLeft, Shield, Info, Loader2 } from 'lucide-react';

interface UserPermissionInfo {
  permissions: string[];
  permissionProfiles: Array<{
    id: string;
    name: string;
    category: string;
    permissions: string[];
  }>;
  packageTier: string;
  role: string;
}

export default function AccessDeniedPage() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const { user } = useAuth();
  const [permissionInfo, setPermissionInfo] = useState<UserPermissionInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchPermissionInfo = async () => {
      if (!user) {
        setLoading(false);
        return;
      }

      try {
        const response = await fetch('/api/v1/authentication/profile', {
          method: 'GET',
          headers: {
            'Content-Type': 'application/json',
          },
        });

        if (response.ok) {
          const data = await response.json();
          setPermissionInfo({
            permissions: data.permissions || [],
            permissionProfiles: data.permissionProfiles || [],
            packageTier: data.packageTier || 'FREE',
            role: data.role || 'User',
          });
        } else {
          setError('Failed to fetch user permissions');
        }
      } catch (err) {
        setError('Error connecting to server');
        console.error('Error fetching permissions:', err);
      } finally {
        setLoading(false);
      }
    };

    fetchPermissionInfo();
  }, [user]);

  const route = searchParams.get('route') || '/unknown';
  const reason = searchParams.get('reason') || 'Insufficient permissions';

  const handleGoBack = () => {
    router.back();
  };

  const handleGoToDashboard = () => {
    router.push('/dashboard');
  };

  const handleUpgrade = () => {
    router.push('/pricing');
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center p-4">
        <Card className="w-full max-w-md">
          <CardContent className="flex items-center justify-center py-8">
            <Loader2 className="h-8 w-8 animate-spin text-gray-600" />
            <span className="ml-2 text-gray-600">Loading permission information...</span>
          </CardContent>
        </Card>
      </div>
    );
  }

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
          {error && (
            <div className="bg-red-50 border border-red-200 rounded-lg p-4">
              <p className="text-red-700 text-sm">{error}</p>
            </div>
          )}

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
                <span className="ml-2 text-gray-600 capitalize">{permissionInfo?.packageTier || 'FREE'}</span>
              </div>
              <div>
                <span className="font-medium text-gray-700">Your Role:</span>
                <span className="ml-2 text-gray-600 capitalize">{permissionInfo?.role || 'User'}</span>
              </div>
            </div>
          </div>

          {/* Profile Information */}
          {permissionInfo && (
            <div className="bg-blue-50 rounded-lg p-4">
              <h3 className="font-medium text-gray-900 mb-3 flex items-center gap-2">
                <Shield className="h-4 w-4" />
                Your Current Permissions
              </h3>
              
              <div className="space-y-3">
                <div>
                  <p className="text-sm text-gray-700 mb-2">
                    <strong>{permissionInfo.permissions.length}</strong> permissions active
                  </p>
                  
                  {permissionInfo.permissionProfiles.length > 0 && (
                    <div className="mb-3">
                      <p className="text-xs font-medium text-gray-600 mb-1">Active Permission Profiles:</p>
                      <div className="flex flex-wrap gap-1">
                        {permissionInfo.permissionProfiles.map((profile) => (
                          <span
                            key={profile.id}
                            className="px-2 py-1 bg-blue-100 text-blue-800 text-xs rounded-full"
                          >
                            {profile.name} ({profile.permissions.length})
                          </span>
                        ))}
                      </div>
                    </div>
                  )}

                  {permissionInfo.permissions.length > 0 && (
                    <div className="mb-3">
                      <p className="text-xs font-medium text-gray-600 mb-1">Your Permissions:</p>
                      <div className="max-h-32 overflow-y-auto">
                        <div className="grid grid-cols-1 sm:grid-cols-2 gap-1 text-xs">
                          {permissionInfo.permissions.map((permission, index) => (
                            <span
                              key={index}
                              className="px-2 py-1 bg-gray-100 text-gray-700 rounded text-xs font-mono"
                            >
                              {permission}
                            </span>
                          ))}
                        </div>
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
              <li>• Review available permission profiles that might grant the required permissions</li>
              {(!permissionInfo?.packageTier || permissionInfo.packageTier === 'FREE') && (
                <li>• Consider upgrading your package to access more features</li>
              )}
              {permissionInfo && permissionInfo.permissions.length === 0 && (
                <li>• Your account may need permission profile assignments - contact support</li>
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
            
            {(!permissionInfo?.packageTier || permissionInfo.packageTier === 'FREE') && (
              <Button
                variant="outline"
                onClick={handleUpgrade}
                className="flex items-center gap-2 border-blue-300 text-blue-600 hover:bg-blue-50"
              >
                Upgrade Package
              </Button>
            )}
            
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