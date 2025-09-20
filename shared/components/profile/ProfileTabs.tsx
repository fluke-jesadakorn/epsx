'use client';

import { useState, ReactNode } from 'react';

interface TabConfig {
  id: string;
  label: string;
  icon: ReactNode;
  content: ReactNode;
  disabled?: boolean;
}

interface ProfileTabsProps {
  tabs: TabConfig[];
  defaultTab?: string;
  className?: string;
  onTabChange?: (tabId: string) => void;
}

export function ProfileTabs({ 
  tabs, 
  defaultTab, 
  className = '',
  onTabChange 
}: ProfileTabsProps) {
  const [activeTab, setActiveTab] = useState(defaultTab || tabs[0]?.id || '');

  const handleTabChange = (tabId: string) => {
    if (tabs.find(tab => tab.id === tabId)?.disabled) return;
    setActiveTab(tabId);
    onTabChange?.(tabId);
  };

  const activeTabContent = tabs.find(tab => tab.id === activeTab)?.content;

  return (
    <div className={className}>
      {/* Tab Navigation */}
      <div className="border-b border-slate-200 dark:border-slate-700">
        <nav className="flex space-x-8 overflow-x-auto">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => handleTabChange(tab.id)}
              disabled={tab.disabled}
              className={`flex items-center gap-2 whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm ${
                activeTab === tab.id
                  ? 'border-orange-500 text-orange-600 dark:text-orange-400'
                  : 'border-transparent text-slate-500 hover:text-slate-700 hover:border-slate-300 dark:text-slate-400 dark:hover:text-slate-300'
              } ${
                tab.disabled 
                  ? 'opacity-50 cursor-not-allowed' 
                  : 'cursor-pointer'
              }`}
            >
              {tab.icon}
              {tab.label}
            </button>
          ))}
        </nav>
      </div>

      {/* Tab Content */}
      <div className="mt-6">
        {activeTabContent}
      </div>
    </div>
  );
}