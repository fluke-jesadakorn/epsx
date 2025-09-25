/**
 * Policy Tabs Component
 * Secure replacement for dangerouslySetInnerHTML script in policies page
 * Follows zero animation policy and proper React patterns
 */

'use client';

import { useState } from 'react';
import { ShieldIcon, ActivityIcon } from 'lucide-react';
import PolicyBuilder from './PolicyBuilder';
import PolicyMonitor from './PolicyMonitor';

export function PolicyTabs() {
  const [activeTab, setActiveTab] = useState<'builder' | 'monitor'>('builder');

  const tabBaseClass = "px-4 sm:px-6 py-3 text-sm font-medium rounded-2xl whitespace-nowrap flex-shrink-0 min-h-[44px]";
  const activeTabClass = `${tabBaseClass} bg-gradient-to-r from-purple-600 to-blue-600 text-white shadow-lg`;
  const inactiveTabClass = `${tabBaseClass} text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700`;

  return (
    <div className="space-y-6 sm:space-y-8">
      {/* Tab Navigation */}
      <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-purple-400/20 via-blue-400/20 to-green-400/20 p-0.5">
        <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl">
          <div className="flex space-x-1 p-4">
            <button 
              className={activeTab === 'builder' ? activeTabClass : inactiveTabClass}
              onClick={() => setActiveTab('builder')}
            >
              <ShieldIcon className="h-4 w-4 inline mr-2" />
              Policy Builder
            </button>
            <button 
              className={activeTab === 'monitor' ? activeTabClass : inactiveTabClass}
              onClick={() => setActiveTab('monitor')}
            >
              <ActivityIcon className="h-4 w-4 inline mr-2" />
              Live Monitor
            </button>
          </div>
        </div>
      </div>
      
      {/* Tab Content */}
      {activeTab === 'builder' && (
        <div>
          <PolicyBuilder />
        </div>
      )}
      
      {activeTab === 'monitor' && (
        <div>
          <PolicyMonitor />
        </div>
      )}
    </div>
  );
}