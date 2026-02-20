'use client';

import { cn } from '@/lib/utils';
import { BarChart3, Clock, Plus } from 'lucide-react';
import { useState } from 'react';

import { CreditsManagement } from '@/components/credits/credits-management';

type TabType = 'overview' | 'grant' | 'history';

export default function CreditsPage() {
  const [activeTab, setActiveTab] = useState<TabType>('overview');

  const tabs = [
    {
      id: 'overview' as const,
      label: 'Overview',
      icon: BarChart3,
    },
    {
      id: 'grant' as const,
      label: 'Grant Credits',
      icon: Plus,
    },
    {
      id: 'history' as const,
      label: 'Credit History',
      icon: Clock,
    }
  ];

  return (
    <div>
      <div className="border-b border-gray-200 dark:border-border mb-8">
        <div className="flex gap-8 max-w-3xl mx-auto">
          {tabs.map((tab) => {
            const isActive = activeTab === tab.id;
            const Icon = tab.icon;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={cn(
                  "flex items-center gap-2 px-4 py-4 font-bold text-lg transition-all relative",
                  isActive
                    ? "text-[#1fc7d4]"
                    : "text-muted-foreground hover:text-foreground"
                )}
              >
                <Icon className="w-5 h-5" />
                {tab.label}
                {isActive && (
                  <span className="absolute bottom-0 left-0 right-0 h-[3px] bg-[#1fc7d4]" />
                )}
              </button>
            );
          })}
        </div>
      </div>
      <CreditsManagement activeTab={activeTab} />
    </div>
  );
}
