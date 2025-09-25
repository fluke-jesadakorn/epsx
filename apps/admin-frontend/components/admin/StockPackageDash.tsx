'use client';

import React, { useState } from 'react';
type BulkStockRankingAssignmentResult = any;
import StockRankingPackageAssignment from './StockRankingPackageAssignment';
import StockRankingAssignmentList from './StockRankingAssignmentList';
import { adminCardVariants, adminButtonVariants, adminBadgeVariants } from '@/design-system';
import { cn } from '@/lib/utils';

export default function StockRankingPackageDashboard() {
  const [activeTab, setActiveTab] = useState<'assign' | 'manage'>('assign');
  const [refreshTrigger, setRefreshTrigger] = useState(0);

  const handleAssignmentComplete = (_result: BulkStockRankingAssignmentResult) => {
    // Trigger refresh of assignment list
    setRefreshTrigger(prev => prev + 1);
    
    // Switch to manage tab to see results
    setActiveTab('manage');
  };

  const tabs = [
    { id: 'assign', label: 'Assign Permissions', count: null },
    { id: 'manage', label: 'Manage Permissions', count: null }
  ];

  return (
    <div className="space-y-8">
      {/* Page Header */}
      <div className={cn(adminCardVariants({ variant: 'pancake', hover: 'both' }))}>
        <div className="flex items-center gap-4">
          <div className="p-3 rounded-full bg-gradient-to-r from-green-500 to-emerald-500">
            <div className="h-8 w-8 text-white font-bold flex items-center justify-center">
              📦
            </div>
          </div>
          <div>
            <h1 className="text-3xl font-bold bg-gradient-to-r from-green-500 to-emerald-500 bg-clip-text text-transparent">
              Stock Ranking Permission Management
            </h1>
            <p className="text-muted-foreground mt-1">
              Assign and manage stock ranking access permissions for users
            </p>
          </div>
        </div>
      </div>

      {/* Tab Navigation */}
      <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <div className="border-b border-border">
          <nav className="-mb-px flex">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id as 'assign' | 'manage')}
                className={cn(
                  adminButtonVariants({ 
                    variant: activeTab === tab.id ? 'primary' : 'ghost',
                    size: 'default'
                  }),
                  'flex-1 border-b-2 rounded-none',
                  activeTab === tab.id 
                    ? 'border-primary-500' 
                    : 'border-transparent hover:border-neutral-300'
                )}
              >
                {tab.label}
                {tab.count && (
                  <span className={cn(adminBadgeVariants({ variant: 'default', size: 'sm' }), 'ml-2')}>
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
      <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <div className="flex items-center gap-3 mb-6">
          <div className="p-2 rounded-lg bg-gradient-to-r from-purple-500/20 to-pink-500/20">
            <div className="h-6 w-6 text-purple-500 font-bold flex items-center justify-center">
              📊
            </div>
          </div>
          <div>
            <h3 className="text-xl font-bold text-foreground">
              Permission Tiers Overview
            </h3>
            <p className="text-sm text-muted-foreground">
              Available ranking permission levels (displayed as familiar tiers)
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
              <div className={cn(
                'w-12 h-12 mx-auto rounded-full flex items-center justify-center mb-2 group-hover:scale-110 transition-transform duration-200',
                pkg.color === 'gray' && 'bg-neutral-100 dark:bg-neutral-900/30',
                pkg.color === 'orange' && 'bg-warning-100 dark:bg-warning-900/30',
                pkg.color === 'yellow' && 'bg-warning-100 dark:bg-warning-900/30',
                pkg.color === 'purple' && 'bg-info-100 dark:bg-info-900/30',
                pkg.color === 'blue' && 'bg-primary-100 dark:bg-primary-900/30'
              )}>
                <span className={cn(
                  'font-bold text-sm',
                  pkg.color === 'gray' && 'text-neutral-600 dark:text-neutral-400',
                  pkg.color === 'orange' && 'text-warning-600 dark:text-warning-400',
                  pkg.color === 'yellow' && 'text-warning-600 dark:text-warning-400',
                  pkg.color === 'purple' && 'text-info-600 dark:text-info-400',
                  pkg.color === 'blue' && 'text-primary-600 dark:text-primary-400'
                )}>
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
      <div className={cn(
        adminCardVariants({ variant: 'pancake' }),
        'bg-gradient-to-r from-primary-50/50 to-info-50/50 dark:from-primary-900/10 dark:to-info-900/10 border border-primary-200/50 dark:border-primary-800/50'
      )}>
        <div className="flex items-center gap-3 mb-4">
          <div className="p-2 rounded-lg bg-gradient-to-r from-blue-500/20 to-indigo-500/20">
            <div className="h-6 w-6 text-blue-500 font-bold flex items-center justify-center">
              💡
            </div>
          </div>
          <h3 className="text-lg font-medium text-blue-900 dark:text-blue-300">
            How to Use Stock Ranking Permission Assignment
          </h3>
        </div>
        
        <div className="space-y-3 text-sm text-blue-800 dark:text-blue-200">
          <div className="flex items-start space-x-2">
            <span className="flex-shrink-0 w-5 h-5 bg-blue-200 dark:bg-blue-800 rounded-full flex items-center justify-center text-xs font-bold text-blue-800 dark:text-blue-200">
              1
            </span>
            <div>
              <strong>Select Permission Tier:</strong> Choose the appropriate stock ranking permission level based on user needs. Each tier represents different permission sets with specific ranking limits.
            </div>
          </div>
          
          <div className="flex items-start space-x-2">
            <span className="flex-shrink-0 w-5 h-5 bg-blue-200 dark:bg-blue-800 rounded-full flex items-center justify-center text-xs font-bold text-blue-800 dark:text-blue-200">
              2
            </span>
            <div>
              <strong>Select Users:</strong> Search and select users to assign the permission tier to. You can select multiple users for bulk permission assignment.
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
              <strong>Review & Assign:</strong> Review the assignment summary and click assign. Users will receive access to stock rankings based on their permission tier.
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}