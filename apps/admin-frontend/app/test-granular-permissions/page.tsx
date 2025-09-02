'use client';

import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { 
  Table, 
  TableBody, 
  TableCell, 
  TableHead, 
  TableHeader, 
  TableRow 
} from '@/components/ui/table';
import { 
  Shield, 
  ShieldCheck, 
  ShieldAlert, 
  TestTube, 
  CheckCircle, 
  XCircle, 
  AlertTriangle,
  RefreshCw,
  Play,
  Settings
} from 'lucide-react';
import { useAdminGranularPermissions } from '@/hooks/useGranularPermissions';
import { 
  AdminUserPermissionAPI,
  AdminPermissionManagementAPI,
  AdminPermissionAnalyticsAPI,
  AdminPermissionCacheAPI
} from '@/lib/api/granular-permissions-admin-client';
import { 
  GrantPermissionRequest,
  RevokePermissionRequest,
  PermissionSource
} from '@/types/granular-permissions';

interface TestResult {
  name: string;
  description: string;
  status: 'pending' | 'running' | 'success' | 'error';
  error?: string;
  duration?: number;
  data?: any;
}

export default function GranularPermissionsTestPage() {
  const { loading, error } = useAdminGranularPermissions();
  const [testResults, setTestResults] = useState<TestResult[]>([]);
  const [isRunningTests, setIsRunningTests] = useState(false);
  const [selectedUserId, setSelectedUserId] = useState('');
  const [testPermission, setTestPermission] = useState({
    platform: 'epsx',
    resource: 'test',
    action: 'view'
  });

  // Initialize test cases
  const testCases: Omit<TestResult, 'status' | 'duration'>[] = [
    {
      name: 'API Connection',
      description: 'Test basic connectivity to the granular permissions API'
    },
    {
      name: 'Dashboard Data',
      description: 'Load admin permission dashboard data'
    },
    {
      name: 'User Permission Status',
      description: 'Get permission status for a specific user'
    },
    {
      name: 'Grant Permission',
      description: 'Grant a test permission to a user'
    },
    {
      name: 'Validate Permission',
      description: 'Verify the granted permission is active'
    },
    {
      name: 'Revoke Permission',
      description: 'Revoke the test permission'
    },
    {
      name: 'Cache Operations',
      description: 'Test cache invalidation and refresh'
    },
    {
      name: 'System Health',
      description: 'Get system health information'
    }
  ];

  useEffect(() => {
    setTestResults(testCases.map(test => ({ ...test, status: 'pending' })));
  }, []);

  const runSingleTest = async (testIndex: number): Promise<void> => {
    const test = testResults[testIndex];
    
    setTestResults(prev => prev.map((t, i) => 
      i === testIndex ? { ...t, status: 'running' } : t
    ));

    const startTime = Date.now();

    try {
      let result: any;

      switch (testIndex) {
        case 0: // API Connection
          result = await AdminPermissionAnalyticsAPI.getSystemHealth();
          break;

        case 1: // Dashboard Data
          result = await AdminPermissionAnalyticsAPI.getDashboard();
          break;

        case 2: // User Permission Status
          if (!selectedUserId) {
            throw new Error('Please enter a user ID for testing');
          }
          result = await AdminUserPermissionAPI.getUserPermissions(selectedUserId);
          break;

        case 3: // Grant Permission
          if (!selectedUserId) {
            throw new Error('Please enter a user ID for testing');
          }
          const grantRequest: GrantPermissionRequest = {
            user_id: selectedUserId,
            permission: `${testPermission.platform}:${testPermission.resource}:${testPermission.action}`,
            expires_at: Math.floor(Date.now() / 1000) + (24 * 60 * 60), // 24 hours
            source: 'Admin' as PermissionSource,
            reason: 'Integration test permission'
          };
          await AdminPermissionManagementAPI.grantPermission(grantRequest);
          result = { success: true };
          break;

        case 4: // Validate Permission
          if (!selectedUserId) {
            throw new Error('Please enter a user ID for testing');
          }
          result = await AdminUserPermissionAPI.getUserPermissions(selectedUserId);
          const hasTestPermission = Object.keys(result.permissions || {}).some(p => 
            p.includes(`${testPermission.platform}:${testPermission.resource}:${testPermission.action}`)
          );
          if (!hasTestPermission) {
            throw new Error('Test permission not found - grant test may have failed');
          }
          break;

        case 5: // Revoke Permission
          if (!selectedUserId) {
            throw new Error('Please enter a user ID for testing');
          }
          const revokeRequest: RevokePermissionRequest = {
            user_id: selectedUserId,
            permission: `${testPermission.platform}:${testPermission.resource}:${testPermission.action}`,
            reason: 'Integration test cleanup'
          };
          await AdminPermissionManagementAPI.revokePermission(revokeRequest);
          result = { success: true };
          break;

        case 6: // Cache Operations
          const cacheStats = await AdminPermissionCacheAPI.getCacheStatistics();
          if (selectedUserId) {
            await AdminPermissionCacheAPI.invalidateUserPermissionCache(selectedUserId);
          }
          result = { cacheStats, invalidated: !!selectedUserId };
          break;

        case 7: // System Health
          result = await AdminPermissionAnalyticsAPI.getSystemHealth();
          break;

        default:
          throw new Error('Unknown test case');
      }

      const duration = Date.now() - startTime;

      setTestResults(prev => prev.map((t, i) => 
        i === testIndex ? { 
          ...t, 
          status: 'success', 
          duration, 
          data: result 
        } : t
      ));

    } catch (err: any) {
      const duration = Date.now() - startTime;

      setTestResults(prev => prev.map((t, i) => 
        i === testIndex ? { 
          ...t, 
          status: 'error', 
          duration,
          error: err?.message || 'Unknown error'
        } : t
      ));
    }
  };

  const runAllTests = async (): Promise<void> => {
    setIsRunningTests(true);
    
    // Reset all tests to pending
    setTestResults(testCases.map(test => ({ ...test, status: 'pending' })));

    // Run tests sequentially
    for (let i = 0; i < testCases.length; i++) {
      await runSingleTest(i);
      // Small delay between tests
      await new Promise(resolve => setTimeout(resolve, 500));
    }

    setIsRunningTests(false);
  };

  const getStatusIcon = (status: TestResult['status']) => {
    switch (status) {
      case 'pending': return <Shield className="h-4 w-4 text-gray-400" />;
      case 'running': return <RefreshCw className="h-4 w-4 animate-spin text-blue-500" />;
      case 'success': return <CheckCircle className="h-4 w-4 text-green-500" />;
      case 'error': return <XCircle className="h-4 w-4 text-red-500" />;
    }
  };

  const getStatusBadge = (status: TestResult['status']) => {
    switch (status) {
      case 'pending': return <Badge variant="outline">Pending</Badge>;
      case 'running': return <Badge variant="default" className="bg-blue-100 text-blue-800">Running</Badge>;
      case 'success': return <Badge variant="default" className="bg-green-100 text-green-800">Success</Badge>;
      case 'error': return <Badge variant="destructive">Failed</Badge>;
    }
  };

  const successCount = testResults.filter(t => t.status === 'success').length;
  const errorCount = testResults.filter(t => t.status === 'error').length;
  const runningCount = testResults.filter(t => t.status === 'running').length;

  return (
    <div className="container mx-auto p-6 max-w-6xl">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-3xl font-bold flex items-center gap-2">
          <TestTube className="h-8 w-8" />
          Granular Permissions Integration Test
        </h1>
        <p className="text-muted-foreground mt-2">
          Test the integration between frontend components and backend granular permission APIs
        </p>
      </div>

      {/* Test Configuration */}
      <Card className="mb-6">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Settings className="h-5 w-5" />
            Test Configuration
          </CardTitle>
          <CardDescription>
            Configure test parameters before running the integration tests
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <Label htmlFor="userId">Test User ID</Label>
              <Input
                id="userId"
                value={selectedUserId}
                onChange={(e) => setSelectedUserId(e.target.value)}
                placeholder="Enter user ID for permission tests"
              />
              <p className="text-xs text-muted-foreground mt-1">
                Used for user-specific permission operations
              </p>
            </div>
            
            <div className="grid grid-cols-3 gap-2">
              <div>
                <Label>Platform</Label>
                <Select 
                  value={testPermission.platform} 
                  onValueChange={(value) => setTestPermission(prev => ({ ...prev, platform: value }))}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="epsx">EPSX</SelectItem>
                    <SelectItem value="admin">Admin</SelectItem>
                    <SelectItem value="test">Test</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div>
                <Label>Resource</Label>
                <Input
                  value={testPermission.resource}
                  onChange={(e) => setTestPermission(prev => ({ ...prev, resource: e.target.value }))}
                  placeholder="Resource"
                />
              </div>
              <div>
                <Label>Action</Label>
                <Input
                  value={testPermission.action}
                  onChange={(e) => setTestPermission(prev => ({ ...prev, action: e.target.value }))}
                  placeholder="Action"
                />
              </div>
            </div>
          </div>

          <div className="mt-4 p-3 bg-muted rounded">
            <p className="text-sm">
              <strong>Test Permission:</strong> {testPermission.platform}:{testPermission.resource}:{testPermission.action}
            </p>
            <p className="text-xs text-muted-foreground mt-1">
              This permission will be temporarily granted and then revoked during testing
            </p>
          </div>
        </CardContent>
      </Card>

      {/* Test Status Summary */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
        <Card>
          <CardContent className="pt-6">
            <div className="text-center">
              <div className="text-2xl font-bold text-green-600">{successCount}</div>
              <div className="text-sm text-muted-foreground">Passed</div>
            </div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="pt-6">
            <div className="text-center">
              <div className="text-2xl font-bold text-red-600">{errorCount}</div>
              <div className="text-sm text-muted-foreground">Failed</div>
            </div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="pt-6">
            <div className="text-center">
              <div className="text-2xl font-bold text-blue-600">{runningCount}</div>
              <div className="text-sm text-muted-foreground">Running</div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="pt-6">
            <div className="text-center">
              <Button 
                onClick={runAllTests} 
                disabled={isRunningTests}
                className="w-full"
              >
                {isRunningTests ? (
                  <RefreshCw className="h-4 w-4 animate-spin" />
                ) : (
                  <Play className="h-4 w-4" />
                )}
                Run All Tests
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Test Results */}
      <Card>
        <CardHeader>
          <CardTitle>Test Results</CardTitle>
          <CardDescription>
            Individual test results with detailed information
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Status</TableHead>
                <TableHead>Test Name</TableHead>
                <TableHead>Description</TableHead>
                <TableHead>Duration</TableHead>
                <TableHead>Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {testResults.map((test, index) => (
                <TableRow key={test.name}>
                  <TableCell>
                    <div className="flex items-center gap-2">
                      {getStatusIcon(test.status)}
                      {getStatusBadge(test.status)}
                    </div>
                  </TableCell>
                  <TableCell className="font-medium">
                    {test.name}
                  </TableCell>
                  <TableCell>
                    <div>
                      {test.description}
                      {test.error && (
                        <Alert variant="destructive" className="mt-2">
                          <AlertTriangle className="h-4 w-4" />
                          <AlertDescription className="text-xs">
                            {test.error}
                          </AlertDescription>
                        </Alert>
                      )}
                    </div>
                  </TableCell>
                  <TableCell>
                    {test.duration !== undefined && (
                      <span className="text-sm text-muted-foreground">
                        {test.duration}ms
                      </span>
                    )}
                  </TableCell>
                  <TableCell>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => runSingleTest(index)}
                      disabled={test.status === 'running' || isRunningTests}
                    >
                      {test.status === 'running' ? (
                        <RefreshCw className="h-3 w-3 animate-spin" />
                      ) : (
                        <Play className="h-3 w-3" />
                      )}
                      Run Test
                    </Button>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      {/* Hook Status */}
      {(loading || error) && (
        <Card className="mt-6">
          <CardHeader>
            <CardTitle>Hook Status</CardTitle>
          </CardHeader>
          <CardContent>
            {loading && (
              <Alert>
                <RefreshCw className="h-4 w-4 animate-spin" />
                <AlertDescription>
                  Admin permission hook is loading...
                </AlertDescription>
              </Alert>
            )}
            
            {error && (
              <Alert variant="destructive">
                <AlertTriangle className="h-4 w-4" />
                <AlertDescription>
                  Hook Error: {error.message}
                </AlertDescription>
              </Alert>
            )}
          </CardContent>
        </Card>
      )}

      {/* Instructions */}
      <Card className="mt-6">
        <CardHeader>
          <CardTitle>Test Instructions</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-2 text-sm">
            <p>1. <strong>User ID:</strong> Enter a valid user ID to test user-specific operations</p>
            <p>2. <strong>Permission:</strong> Configure the test permission to grant/revoke</p>
            <p>3. <strong>Run Tests:</strong> Click "Run All Tests" or individual "Run Test" buttons</p>
            <p>4. <strong>Cleanup:</strong> The test will automatically clean up any test permissions</p>
          </div>
          
          <Alert className="mt-4">
            <ShieldAlert className="h-4 w-4" />
            <AlertDescription>
              <strong>Note:</strong> This test page requires admin permissions and a running backend server.
              Test permissions are temporary and will be automatically revoked.
            </AlertDescription>
          </Alert>
        </CardContent>
      </Card>
    </div>
  );
}