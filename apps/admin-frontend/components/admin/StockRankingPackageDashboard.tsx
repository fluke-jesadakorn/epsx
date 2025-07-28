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
    <div className="space-y-8">
      {/* Page Header */}
      <div className="pancake-card pancake-card-hover p-6">
        <div className="flex items-center gap-4">
          <div className="p-3 rounded-full bg-gradient-to-r from-green-500 to-emerald-500">
            <div className="h-8 w-8 text-white font-bold flex items-center justify-center">
              📦
            </div>
          </div>
          <div>
            <h1 className="text-3xl font-bold bg-gradient-to-r from-green-500 to-emerald-500 bg-clip-text text-transparent">
              Stock Ranking Package Management
            </h1>
            <p className="text-muted-foreground mt-1">
              Assign and manage stock ranking access packages for users
            </p>
          </div>
        </div>
      </div>

      {/* Tab Navigation */}
      <div className="pancake-card">
        <div className="border-b border-border">
          <nav className="-mb-px flex">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id as 'assign' | 'manage')}
                className={`
                  flex-1 py-4 px-6 text-center border-b-2 font-medium text-sm transition-all duration-150
                  ${activeTab === tab.id
                    ? 'border-blue-500 text-blue-600 dark:text-blue-400 bg-blue-50 dark:bg-blue-900/20'
                    : 'border-transparent text-muted-foreground hover:text-foreground hover:border-gray-300 dark:hover:border-gray-600'
                  }
                `}
              >
                {tab.label}
                {tab.count && (
                  <span className="ml-2 py-0.5 px-2 rounded-full text-xs bg-muted text-muted-foreground">
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
      <div className="pancake-card p-6">
        <div className="flex items-center gap-3 mb-6">
          <div className="p-2 rounded-lg bg-gradient-to-r from-purple-500/20 to-pink-500/20">
            <div className="h-6 w-6 text-purple-500 font-bold flex items-center justify-center">
              📊
            </div>
          </div>
          <div>
            <h3 className="text-xl font-bold text-foreground">
              Package Overview
            </h3>
            <p className="text-sm text-muted-foreground">
              Available stock ranking packages
            </p>
          </div>
        </div>
        
        <div className="grid grid-cols-2 md:grid-cols-6 gap-4">
          {[
            { tier: 'FREE', color: 'gray', rankings: '3' },
            { tier: 'BRONZE', color: 'orange', rankings: '5' },
            { tier: 'SILVER', color: 'gray', rankings: '25' },
            { tier: 'GOLD', color: 'yellow', rankings: '50' },
            { tier: 'PLATINUM', color: 'purple', rankings: '100' },
            { tier: 'ENTERPRISE', color: 'blue', rankings: '∞' }
          ].map((pkg) => (
            <div key={pkg.tier} className="text-center group">
              <div className={`w-12 h-12 mx-auto rounded-full bg-${pkg.color}-100 dark:bg-${pkg.color}-900/30 flex items-center justify-center mb-2 group-hover:scale-110 transition-transform duration-200`}>
                <span className={`text-${pkg.color}-600 dark:text-${pkg.color}-400 font-bold text-sm`}>
                  {pkg.rankings}
                </span>
              </div>
              <div className="text-sm font-medium text-foreground">{pkg.tier}</div>
              <div className="text-xs text-muted-foreground">Max Rankings</div>
            </div>
          ))}
        </div>
      </div>

      {/* Help Section */}
      <div className="pancake-card p-6 bg-gradient-to-r from-blue-50/50 to-indigo-50/50 dark:from-blue-900/10 dark:to-indigo-900/10 border border-blue-200/50 dark:border-blue-800/50">
        <div className="flex items-center gap-3 mb-4">
          <div className="p-2 rounded-lg bg-gradient-to-r from-blue-500/20 to-indigo-500/20">
            <div className="h-6 w-6 text-blue-500 font-bold flex items-center justify-center">
              💡
            </div>
          </div>
          <h3 className="text-lg font-medium text-blue-900 dark:text-blue-300">
            How to Use Stock Ranking Package Assignment
          </h3>
        </div>
        
        <div className="space-y-3 text-sm text-blue-800 dark:text-blue-200">
          <div className="flex items-start space-x-2">
            <span className="flex-shrink-0 w-5 h-5 bg-blue-200 dark:bg-blue-800 rounded-full flex items-center justify-center text-xs font-bold text-blue-800 dark:text-blue-200">
              1
            </span>
            <div>
              <strong>Select Package Tier:</strong> Choose the appropriate stock ranking package based on user needs. Each tier provides different ranking limits and features.
            </div>
          </div>
          
          <div className="flex items-start space-x-2">
            <span className="flex-shrink-0 w-5 h-5 bg-blue-200 dark:bg-blue-800 rounded-full flex items-center justify-center text-xs font-bold text-blue-800 dark:text-blue-200">
              2
            </span>
            <div>
              <strong>Select Users:</strong> Search and select users to assign the package to. You can select multiple users for bulk assignment.
            </div>
          </div>
          
          <div className="flex items-start space-x-2">
            <span className="flex-shrink-0 w-5 h-5 bg-blue-200 dark:bg-blue-800 rounded-full flex items-center justify-center text-xs font-bold text-blue-800 dark:text-blue-200">
              3
            </span>
            <div>
              <strong>Provide Details:</strong> Enter a reason for the assignment and optionally set an expiration date for temporary access.
            </div>
          </div>
          
          <div className="flex items-start space-x-2">
            <span className="flex-shrink-0 w-5 h-5 bg-blue-200 dark:bg-blue-800 rounded-full flex items-center justify-center text-xs font-bold text-blue-800 dark:text-blue-200">
              4
            </span>
            <div>
              <strong>Review & Assign:</strong> Review the assignment summary and click assign. Users will receive access to stock rankings based on their package tier.
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}