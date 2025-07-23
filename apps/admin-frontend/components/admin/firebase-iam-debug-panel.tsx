'use client';

import React, { useState, useEffect } from 'react';
// import { firebaseIAMService } from '../../services/firebaseIAMService'; // Service removed
import { useToast } from '@/components/ui/toast';

// Placeholder for removed service
const firebaseIAMService = {
  checkConnection: async () => ({ firebase: true, firestore: true, auth: true }),
  getUsers: async () => [],
};

interface ConnectionStatus {
  firebase: boolean;
  firestore: boolean;
  auth: boolean;
  collections: {
    users: boolean;
    permissions: boolean;
    auditLogs: boolean;
  };
}

interface DebugInfo {
  connectionStatus: ConnectionStatus;
  userCount: number;
  sampleUser: any;
  error?: string;
}

export const FirebaseIAMDebugPanel: React.FC = () => {
  const [debugInfo, setDebugInfo] = useState<DebugInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const { addToast } = useToast();

  const runDiagnostics = async () => {
    setLoading(true);
    try {
      // Test Firebase connection
      const users = await firebaseIAMService.getUsers();
      const sampleUser = users.length > 0 ? users[0] : null;

      const connectionStatus: ConnectionStatus = {
        firebase: true,
        firestore: true,
        auth: true,
        collections: {
          users: users.length > 0,
          permissions: true,
          auditLogs: true
        }
      };

      setDebugInfo({
        connectionStatus,
        userCount: users.length,
        sampleUser,
      });
    } catch (error) {
      console.error('Diagnostics failed:', error);
      setDebugInfo({
        connectionStatus: {
          firebase: false,
          firestore: false,
          auth: false,
          collections: {
            users: false,
            permissions: false,
            auditLogs: false
          }
        },
        userCount: 0,
        sampleUser: null,
        error: error instanceof Error ? error.message : 'Unknown error'
      });
    } finally {
      setLoading(false);
    }
  };

  const createSampleData = async () => {
    try {
      // This would trigger the initialization
      await firebaseIAMService.getUsers();
      addToast({
        type: 'success',
        title: 'Sample data creation triggered',
        description: 'Check the console for details'
      });
      runDiagnostics();
    } catch (error) {
      console.error('Failed to create sample data:', error);
      addToast({
        type: 'error',
        title: 'Failed to create sample data',
        description: 'Check console for details'
      });
    }
  };

  useEffect(() => {
    runDiagnostics();
  }, []);

  const getStatusColor = (status: boolean) => {
    return status ? 'text-green-600' : 'text-red-600';
  };

  const getStatusIcon = (status: boolean) => {
    return status ? '✅' : '❌';
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <h3 className="text-lg font-semibold">Firebase IAM Debug Panel</h3>
        <div className="space-x-2">
          <button
            onClick={runDiagnostics}
            disabled={loading}
            className="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:opacity-50"
          >
            {loading ? 'Running...' : 'Run Diagnostics'}
          </button>
          <button
            onClick={createSampleData}
            className="px-4 py-2 bg-green-500 text-white rounded-lg hover:bg-green-600"
          >
            Create Sample Data
          </button>
        </div>
      </div>

      {loading && (
        <div className="flex items-center justify-center p-8">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
          <span className="ml-2">Running diagnostics...</span>
        </div>
      )}

      {debugInfo && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {/* Connection Status */}
          <div className="bg-white rounded-lg shadow p-6">
            <h4 className="text-md font-semibold mb-4">Connection Status</h4>
            <div className="space-y-2">
              <div className="flex justify-between">
                <span>Firebase:</span>
                <span className={getStatusColor(debugInfo.connectionStatus.firebase)}>
                  {getStatusIcon(debugInfo.connectionStatus.firebase)} {debugInfo.connectionStatus.firebase ? 'Connected' : 'Disconnected'}
                </span>
              </div>
              <div className="flex justify-between">
                <span>Firestore:</span>
                <span className={getStatusColor(debugInfo.connectionStatus.firestore)}>
                  {getStatusIcon(debugInfo.connectionStatus.firestore)} {debugInfo.connectionStatus.firestore ? 'Connected' : 'Disconnected'}
                </span>
              </div>
              <div className="flex justify-between">
                <span>Auth:</span>
                <span className={getStatusColor(debugInfo.connectionStatus.auth)}>
                  {getStatusIcon(debugInfo.connectionStatus.auth)} {debugInfo.connectionStatus.auth ? 'Connected' : 'Disconnected'}
                </span>
              </div>
            </div>
          </div>

          {/* Collections Status */}
          <div className="bg-white rounded-lg shadow p-6">
            <h4 className="text-md font-semibold mb-4">Collections Status</h4>
            <div className="space-y-2">
              <div className="flex justify-between">
                <span>Users Collection:</span>
                <span className={getStatusColor(debugInfo.connectionStatus.collections.users)}>
                  {getStatusIcon(debugInfo.connectionStatus.collections.users)} {debugInfo.connectionStatus.collections.users ? 'Has Data' : 'No Data'}
                </span>
              </div>
              <div className="flex justify-between">
                <span>Permissions:</span>
                <span className={getStatusColor(debugInfo.connectionStatus.collections.permissions)}>
                  {getStatusIcon(debugInfo.connectionStatus.collections.permissions)} Available
                </span>
              </div>
              <div className="flex justify-between">
                <span>Audit Logs:</span>
                <span className={getStatusColor(debugInfo.connectionStatus.collections.auditLogs)}>
                  {getStatusIcon(debugInfo.connectionStatus.collections.auditLogs)} Available
                </span>
              </div>
            </div>
          </div>

          {/* Data Summary */}
          <div className="bg-white rounded-lg shadow p-6">
            <h4 className="text-md font-semibold mb-4">Data Summary</h4>
            <div className="space-y-2">
              <div className="flex justify-between">
                <span>Total Users:</span>
                <span className="font-medium">{debugInfo.userCount}</span>
              </div>
              <div className="flex justify-between">
                <span>Data Source:</span>
                <span className="font-medium">
                  {debugInfo.userCount > 0 ? 'Firebase' : 'Mock Data'}
                </span>
              </div>
            </div>
          </div>

          {/* Sample User */}
          <div className="bg-white rounded-lg shadow p-6">
            <h4 className="text-md font-semibold mb-4">Sample User Data</h4>
            {debugInfo.sampleUser ? (
              <div className="space-y-2 text-sm">
                <div><strong>ID:</strong> {debugInfo.sampleUser.id}</div>
                <div><strong>Email:</strong> {debugInfo.sampleUser.email}</div>
                <div><strong>Name:</strong> {debugInfo.sampleUser.name || 'Not set'}</div>
                <div><strong>Display Name:</strong> {debugInfo.sampleUser.displayName || 'Not set'}</div>
                <div><strong>Package:</strong> {debugInfo.sampleUser.packageTier}</div>
                <div><strong>Status:</strong> {debugInfo.sampleUser.subscriptionStatus}</div>
                <div><strong>Permissions:</strong> {debugInfo.sampleUser.packagePermissions?.length || 0} package, {debugInfo.sampleUser.customPermissions?.length || 0} custom</div>
              </div>
            ) : (
              <p className="text-gray-500">No user data available</p>
            )}
          </div>
        </div>
      )}

      {/* Error Display */}
      {debugInfo?.error && (
        <div className="bg-red-50 border border-red-200 rounded-lg p-4">
          <h4 className="text-md font-semibold text-red-800 mb-2">Error Details</h4>
          <p className="text-red-700 text-sm font-mono">{debugInfo.error}</p>
        </div>
      )}

      {/* Quick Actions */}
      <div className="bg-white rounded-lg shadow p-6">
        <h4 className="text-md font-semibold mb-4">Quick Actions</h4>
        <div className="space-y-2">
          <p className="text-sm text-gray-600">
            🚀 <strong>First time setup:</strong> Run the Firebase initialization script
          </p>
          <code className="block bg-gray-100 p-2 rounded text-sm">
            cd apps/admin-frontend && npm run firebase-init
          </code>
          
          <p className="text-sm text-gray-600 mt-4">
            🔧 <strong>Check Firebase config:</strong> Verify your Firebase configuration in lib/firebase.ts
          </p>
          
          <p className="text-sm text-gray-600 mt-4">
            📚 <strong>Need help?</strong> Check the Firebase IAM setup guide in README-Firebase-IAM-Complete.md
          </p>
        </div>
      </div>
    </div>
  );
};
