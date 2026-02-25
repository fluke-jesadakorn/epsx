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
      <div className="border-b border-border/30 mb-6">
        <div className="flex gap-1">
          {tabs.map((tab) => {
            const isActive = activeTab === tab.id;
            const Icon = tab.icon;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={cn(
                  "flex items-center gap-2 px-4 py-3 font-semibold text-sm transition-all relative",
                  isActive
                    ? "text-[#1fc7d4]"
                    : "text-muted-foreground hover:text-foreground"
                )}
              >
                <Icon className="w-4 h-4" />
                {tab.label}
                {isActive && (
                  <span className="absolute bottom-0 left-0 right-0 h-[2px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" />
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
