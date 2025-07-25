'use client';

import React, { useState } from 'react';
import { BulkStockRankingAssignmentResult } from '@epsx/types/src/permission_profile';
import StockRankingPackageAssignment from './StockRankingPackageAssignment';
import StockRankingAssignmentList from './StockRankingAssignmentList';

export default function StockRankingPackageDashboard() {
  const [activeTab, setActiveTab] = useState<'assign' | 'manage'>('assign');
  const [refreshTrigger, setRefreshTrigger] = useState(0);

  const handleAssignmentComplete = (result: BulkStockRankingAssignmentResult) => {
    // Trigger refresh of assignment list
    setRefreshTrigger(prev => prev + 1);
    
    // Switch to manage tab to see results
    setActiveTab('manage');
  };

  const tabs = [
    { id: 'assign', label: 'Assign Packages', count: null },
    { id: 'manage', label: 'Manage Assignments', count: null }
  ];

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Page Header */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900">
            Stock Ranking Package Management
          </h1>
          <p className="mt-2 text-gray-600">
            Assign and manage stock ranking access packages for users
          </p>
        </div>

        {/* Tab Navigation */}
        <div className="bg-white shadow rounded-lg">
          <div className="border-b border-gray-200">
            <nav className="-mb-px flex">
              {tabs.map((tab) => (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id as 'assign' | 'manage')}
                  className={`
                    flex-1 py-4 px-6 text-center border-b-2 font-medium text-sm
                    ${activeTab === tab.id
                      ? 'border-blue-500 text-blue-600 bg-blue-50'
                      : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                    }
                  `}
                >
                  {tab.label}
                  {tab.count && (
                    <span className="ml-2 py-0.5 px-2 rounded-full text-xs bg-gray-100 text-gray-600">
                      {tab.count}
                    </span>
                  )}
                </button>
              ))}
            </nav>
          </div>

          {/* Tab Content */}
          <div className="p-6">
            {activeTab === 'assign' && (
              <StockRankingPackageAssignment 
                onAssignmentComplete={handleAssignmentComplete}
              />
            )}
            
            {activeTab === 'manage' && (
              <StockRankingAssignmentList 
                refreshTrigger={refreshTrigger}
              />
            )}
          </div>
        </div>

        {/* Quick Stats Card */}
        <div className="mt-8 bg-white shadow rounded-lg p-6">
          <h3 className="text-lg font-medium text-gray-900 mb-4">
            Package Overview
          </h3>
          
          <div className="grid grid-cols-2 md:grid-cols-6 gap-4">
            {[
              { tier: 'FREE', color: 'gray', rankings: '3' },
              { tier: 'BRONZE', color: 'orange', rankings: '5' },
              { tier: 'SILVER', color: 'gray', rankings: '25' },
              { tier: 'GOLD', color: 'yellow', rankings: '50' },
              { tier: 'PLATINUM', color: 'purple', rankings: '100' },
              { tier: 'ENTERPRISE', color: 'blue', rankings: '∞' }
            ].map((pkg) => (
              <div key={pkg.tier} className="text-center">
                <div className={`w-12 h-12 mx-auto rounded-full bg-${pkg.color}-100 flex items-center justify-center mb-2`}>
                  <span className={`text-${pkg.color}-600 font-bold text-sm`}>
                    {pkg.rankings}
                  </span>
                </div>
                <div className="text-sm font-medium text-gray-900">{pkg.tier}</div>
                <div className="text-xs text-gray-500">Max Rankings</div>
              </div>
            ))}
          </div>
        </div>

        {/* Help Section */}
        <div className="mt-8 bg-blue-50 rounded-lg p-6">
          <h3 className="text-lg font-medium text-blue-900 mb-3">
            How to Use Stock Ranking Package Assignment
          </h3>
          
          <div className="space-y-3 text-sm text-blue-800">
            <div className="flex items-start space-x-2">
              <span className="flex-shrink-0 w-5 h-5 bg-blue-200 rounded-full flex items-center justify-center text-xs font-bold text-blue-800">
                1
              </span>
              <div>
                <strong>Select Package Tier:</strong> Choose the appropriate stock ranking package based on user needs. Each tier provides different ranking limits and features.
              </div>
            </div>
            
            <div className="flex items-start space-x-2">
              <span className="flex-shrink-0 w-5 h-5 bg-blue-200 rounded-full flex items-center justify-center text-xs font-bold text-blue-800">
                2
              </span>
              <div>
                <strong>Select Users:</strong> Search and select users to assign the package to. You can select multiple users for bulk assignment.
              </div>
            </div>
            
            <div className="flex items-start space-x-2">
              <span className="flex-shrink-0 w-5 h-5 bg-blue-200 rounded-full flex items-center justify-center text-xs font-bold text-blue-800">
                3
              </span>
              <div>
                <strong>Provide Details:</strong> Enter a reason for the assignment and optionally set an expiration date for temporary access.
              </div>
            </div>
            
            <div className="flex items-start space-x-2">
              <span className="flex-shrink-0 w-5 h-5 bg-blue-200 rounded-full flex items-center justify-center text-xs font-bold text-blue-800">
                4
              </span>
              <div>
                <strong>Review & Assign:</strong> Review the assignment summary and click assign. Users will receive access to stock rankings based on their package tier.
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}