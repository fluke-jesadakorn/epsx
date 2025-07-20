'use client';

import React, { useState, useEffect } from 'react';
import { EnhancedUserList } from './EnhancedUserList';
import { FirebaseIAMDebugPanel } from '../debug/FirebaseIAMDebugPanel';
import { usePaymentIntegration } from '../../hooks/usePaymentIntegration';
import { iamService } from '../../services/iamService';
import { PackageTier } from '../../types/admin/iam-enhanced';

export const IAMDashboard: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'users' | 'permissions' | 'audit' | 'testing' | 'debug'>('users');
  const [stats, setStats] = useState({
    totalUsers: 0,
    activeSubscriptions: 0,
    customPermissionsGranted: 0,
    packageUpgradesToday: 0,
  });
  const [loading, setLoading] = useState(true);
  const [hasFirebaseError, setHasFirebaseError] = useState(false);

  // Initialize payment integration hooks
  const { triggerPaymentSuccess, triggerPackageDowngrade, triggerSubscriptionExpiry } = usePaymentIntegration();

  useEffect(() => {
    loadDashboardStats();
  }, []);

  const loadDashboardStats = async () => {
    try {
      setLoading(true);
      setHasFirebaseError(false);
      // In a real implementation, these would be separate API endpoints
      const users = await iamService.getUsers();
      
      setStats({
        totalUsers: users.length,
        activeSubscriptions: users.filter(u => u.subscriptionStatus === 'active').length,
        customPermissionsGranted: users.reduce((acc, u) => acc + (u.customPermissions?.length || 0), 0),
        packageUpgradesToday: 0, // This would come from audit logs
      });
    } catch (error) {
      console.error('Failed to load dashboard stats:', error);
      setHasFirebaseError(true);
      // Set default values for demo purposes
      setStats({
        totalUsers: 3,
        activeSubscriptions: 2,
        customPermissionsGranted: 1,
        packageUpgradesToday: 0,
      });
    } finally {
      setLoading(false);
    }
  };

  const handleTestPaymentSuccess = () => {
    const testUserId = prompt('Enter User ID for testing:');
    const testTier = prompt('Enter Package Tier (bronze, silver, gold, platinum, enterprise):') as PackageTier;
    const testTransactionId = `test-${Date.now()}`;
    
    if (testUserId && testTier) {
      triggerPaymentSuccess(testUserId, testTier, testTransactionId);
    }
  };

  const handleTestDowngrade = () => {
    const testUserId = prompt('Enter User ID for testing:');
    const oldTier = prompt('Enter Old Tier:') as PackageTier;
    const newTier = prompt('Enter New Tier:') as PackageTier;
    
    if (testUserId && oldTier && newTier) {
      triggerPackageDowngrade(testUserId, oldTier, newTier);
    }
  };

  const handleTestExpiry = () => {
    const testUserId = prompt('Enter User ID for testing:');
    
    if (testUserId) {
      triggerSubscriptionExpiry(testUserId);
    }
  };

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
        {/* Header */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900">IAM Dashboard</h1>
          <p className="mt-2 text-gray-600">
            Manage user permissions, package tiers, and access control
          </p>
        </div>

        {/* Stats Cards */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-6 mb-8">
          <div className="bg-white overflow-hidden shadow rounded-lg">
            <div className="p-5">
              <div className="flex items-center">
                <div className="flex-shrink-0">
                  <div className="w-8 h-8 bg-blue-500 rounded-md flex items-center justify-center">
                    <span className="text-white text-sm font-bold">U</span>
                  </div>
                </div>
                <div className="ml-5 w-0 flex-1">
                  <dl>
                    <dt className="text-sm font-medium text-gray-500 truncate">
                      Total Users
                    </dt>
                    <dd className="text-lg font-medium text-gray-900">
                      {loading ? '...' : stats.totalUsers}
                    </dd>
                  </dl>
                </div>
              </div>
            </div>
          </div>

          <div className="bg-white overflow-hidden shadow rounded-lg">
            <div className="p-5">
              <div className="flex items-center">
                <div className="flex-shrink-0">
                  <div className="w-8 h-8 bg-green-500 rounded-md flex items-center justify-center">
                    <span className="text-white text-sm font-bold">A</span>
                  </div>
                </div>
                <div className="ml-5 w-0 flex-1">
                  <dl>
                    <dt className="text-sm font-medium text-gray-500 truncate">
                      Active Subscriptions
                    </dt>
                    <dd className="text-lg font-medium text-gray-900">
                      {loading ? '...' : stats.activeSubscriptions}
                    </dd>
                  </dl>
                </div>
              </div>
            </div>
          </div>

          <div className="bg-white overflow-hidden shadow rounded-lg">
            <div className="p-5">
              <div className="flex items-center">
                <div className="flex-shrink-0">
                  <div className="w-8 h-8 bg-purple-500 rounded-md flex items-center justify-center">
                    <span className="text-white text-sm font-bold">P</span>
                  </div>
                </div>
                <div className="ml-5 w-0 flex-1">
                  <dl>
                    <dt className="text-sm font-medium text-gray-500 truncate">
                      Custom Permissions
                    </dt>
                    <dd className="text-lg font-medium text-gray-900">
                      {loading ? '...' : stats.customPermissionsGranted}
                    </dd>
                  </dl>
                </div>
              </div>
            </div>
          </div>

          <div className="bg-white overflow-hidden shadow rounded-lg">
            <div className="p-5">
              <div className="flex items-center">
                <div className="flex-shrink-0">
                  <div className="w-8 h-8 bg-orange-500 rounded-md flex items-center justify-center">
                    <span className="text-white text-sm font-bold">↗</span>
                  </div>
                </div>
                <div className="ml-5 w-0 flex-1">
                  <dl>
                    <dt className="text-sm font-medium text-gray-500 truncate">
                      Upgrades Today
                    </dt>
                    <dd className="text-lg font-medium text-gray-900">
                      {loading ? '...' : stats.packageUpgradesToday}
                    </dd>
                  </dl>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Navigation Tabs */}
        <div className="border-b border-gray-200 mb-6">
          <nav className="-mb-px flex space-x-8">
            <button
              onClick={() => setActiveTab('users')}
              className={`py-2 px-1 border-b-2 font-medium text-sm ${
                activeTab === 'users'
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              User Management
            </button>
            <button
              onClick={() => setActiveTab('permissions')}
              className={`py-2 px-1 border-b-2 font-medium text-sm ${
                activeTab === 'permissions'
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              Permission Templates
            </button>
            <button
              onClick={() => setActiveTab('audit')}
              className={`py-2 px-1 border-b-2 font-medium text-sm ${
                activeTab === 'audit'
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              Audit Logs
            </button>
            <button
              onClick={() => setActiveTab('testing')}
              className={`py-2 px-1 border-b-2 font-medium text-sm ${
                activeTab === 'testing'
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              Testing Tools
            </button>
            <button
              onClick={() => setActiveTab('debug')}
              className={`py-2 px-1 border-b-2 font-medium text-sm ${
                activeTab === 'debug'
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              🔥 Debug
            </button>
          </nav>
        </div>

        {/* Tab Content */}
        <div className="tab-content">
          {activeTab === 'users' && <EnhancedUserList />}
          
          {activeTab === 'permissions' && (
            <div className="bg-white rounded-lg shadow p-6">
              <h3 className="text-lg font-semibold mb-4">Permission Templates</h3>
              <p className="text-gray-600">
                Permission template management will be implemented here.
                This will allow admins to create, edit, and apply permission templates.
              </p>
            </div>
          )}
          
          {activeTab === 'audit' && (
            <div className="bg-white rounded-lg shadow p-6">
              <h3 className="text-lg font-semibold mb-4">Audit Logs</h3>
              <p className="text-gray-600">
                Audit log viewer will be implemented here.
                This will show all permission changes, package upgrades, and administrative actions.
              </p>
            </div>
          )}
          
          {activeTab === 'testing' && (
            <div className="bg-white rounded-lg shadow p-6">
              <h3 className="text-lg font-semibold mb-4">Testing Tools</h3>
              <p className="text-gray-600 mb-6">
                Use these tools to test payment integration and permission assignment.
              </p>
              
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <button
                  onClick={handleTestPaymentSuccess}
                  className="bg-green-600 text-white px-4 py-2 rounded-lg hover:bg-green-700"
                >
                  Test Payment Success
                </button>
                
                <button
                  onClick={handleTestDowngrade}
                  className="bg-orange-600 text-white px-4 py-2 rounded-lg hover:bg-orange-700"
                >
                  Test Package Downgrade
                </button>
                
                <button
                  onClick={handleTestExpiry}
                  className="bg-red-600 text-white px-4 py-2 rounded-lg hover:bg-red-700"
                >
                  Test Subscription Expiry
                </button>
              </div>
              
              <div className="mt-6 p-4 bg-yellow-50 rounded-lg">
                <h4 className="font-medium text-yellow-800">Usage Instructions:</h4>
                <ul className="mt-2 text-sm text-yellow-700 space-y-1">
                  <li>• <strong>Payment Success:</strong> Simulates a successful payment and auto-applies package permissions</li>
                  <li>• <strong>Package Downgrade:</strong> Simulates a user downgrading their package tier</li>
                  <li>• <strong>Subscription Expiry:</strong> Simulates a subscription expiring and downgrades to FREE tier</li>
                </ul>
              </div>
            </div>
          )}

          {activeTab === 'debug' && (
            <div className="space-y-6">
              {hasFirebaseError && (
                <div className="bg-red-50 border border-red-200 rounded-lg p-4">
                  <h3 className="text-red-800 font-medium">⚠️ Firebase Connection Issue Detected</h3>
                  <p className="text-red-600 mt-1">
                    The IAM system is currently using mock data. Use the debug panel below to troubleshoot.
                  </p>
                </div>
              )}
              
              <FirebaseIAMDebugPanel />
              
              <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
                <h3 className="text-blue-800 font-medium">💡 Common Firebase Setup Issues</h3>
                <div className="mt-2 text-blue-700 text-sm space-y-2">
                  <p><strong>1. Environment Variables:</strong> Check that Firebase config is in .env.local</p>
                  <p><strong>2. Firestore Database:</strong> Ensure Firestore is enabled in Firebase Console</p>
                  <p><strong>3. Security Rules:</strong> Make sure Firestore rules allow admin access</p>
                  <p><strong>4. Collections:</strong> IAM collections may need to be created first</p>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
