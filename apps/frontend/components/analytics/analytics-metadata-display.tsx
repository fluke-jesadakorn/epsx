'use client';

import type { UnifiedAnalyticsRankingsResponse } from '@/types';

interface AnalyticsMetadataDisplayProps {
  data: UnifiedAnalyticsRankingsResponse | null;
  isLoading: boolean;
}

export function AnalyticsMetadataDisplay({ data, isLoading }: AnalyticsMetadataDisplayProps) {
  if (isLoading || !data?.metadata) {
    return null;
  }

  return (
    <div className="mb-8">
      <div className="relative overflow-hidden rounded-3xl border border-purple-200/50 bg-white/80 p-6 shadow-2xl backdrop-blur-xl dark:border-purple-400/20 dark:bg-slate-800/80">
        <div className="absolute inset-0 bg-gradient-to-br from-purple-50/50 via-transparent to-blue-50/50 dark:from-purple-900/10 dark:via-transparent dark:to-blue-900/10" />
        
        <div className="relative z-10">
          <div className="mb-6 text-center sm:text-left">
            <h2 className="mb-3 text-xl font-bold sm:text-2xl">
              <span className="mr-2">🚀</span>
              <span className="animate-gradient-x bg-gradient-to-r from-purple-500 via-blue-500 to-purple-600 bg-clip-text text-transparent">
                Advanced Analytics Engine
              </span>
            </h2>
            <p className="text-gray-600 dark:text-gray-300">
              Powered by Diesel ORM with real-time processing and intelligent caching
            </p>
          </div>

          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
            <MetricCard
              icon={<ProcessingTimeIcon />}
              label="Processing Time"
              value={`${(data.metadata as unknown as Record<string, unknown>).query_time ?? 0}ms`}
              bgColor="green"
            />
            
            <MetricCard
              icon={<RealtimeIcon />}
              label="Real-time"
              value="HTTP Only"
              bgColor="blue"
            />
            
            <MetricCard
              icon={<DatabaseIcon />}
              label="Data Source"
              value="Analytics API"
              bgColor="orange"
            />
            
            <MetricCard
              icon={<MarketsIcon />}
              label="Markets"
              value="Global Markets"
              bgColor="purple"
            />
          </div>
        </div>
      </div>
    </div>
  );
}

interface MetricCardProps {
  icon: React.ReactNode;
  label: string;
  value: string;
  bgColor: 'green' | 'blue' | 'orange' | 'purple';
}

function MetricCard({ icon, label, value, bgColor }: MetricCardProps) {
  const colorClasses = {
    green: {
      border: 'border-green-200/50 dark:border-green-400/20',
      background: 'bg-gradient-to-br from-green-50/80 to-emerald-50/80 dark:from-green-900/20 dark:to-emerald-900/20',
      iconBg: 'bg-gradient-to-r from-green-500 to-emerald-500',
      labelColor: 'text-green-700 dark:text-green-400',
      valueColor: 'text-green-800 dark:text-green-300'
    },
    blue: {
      border: 'border-blue-200/50 dark:border-blue-400/20',
      background: 'bg-gradient-to-br from-blue-50/80 to-cyan-50/80 dark:from-blue-900/20 dark:to-cyan-900/20',
      iconBg: 'bg-gradient-to-r from-blue-500 to-cyan-500',
      labelColor: 'text-blue-700 dark:text-blue-400',
      valueColor: 'text-blue-800 dark:text-blue-300'
    },
    orange: {
      border: 'border-orange-200/50 dark:border-orange-400/20',
      background: 'bg-gradient-to-br from-orange-50/80 to-yellow-50/80 dark:from-orange-900/20 dark:to-yellow-900/20',
      iconBg: 'bg-gradient-to-r from-orange-500 to-yellow-500',
      labelColor: 'text-orange-700 dark:text-orange-400',
      valueColor: 'text-orange-800 dark:text-orange-300'
    },
    purple: {
      border: 'border-purple-200/50 dark:border-purple-400/20',
      background: 'bg-gradient-to-br from-purple-50/80 to-pink-50/80 dark:from-purple-900/20 dark:to-pink-900/20',
      iconBg: 'bg-gradient-to-r from-purple-500 to-pink-500',
      labelColor: 'text-purple-700 dark:text-purple-400',
      valueColor: 'text-purple-800 dark:text-purple-300'
    }
  };

  const colors = colorClasses[bgColor];

  return (
    <div className={`rounded-2xl border ${colors.border} ${colors.background} p-4 backdrop-blur-sm`}>
      <div className="flex items-center gap-3 mb-2">
        <div className={`rounded-xl ${colors.iconBg} p-2`}>
          {icon}
        </div>
        <span className={`text-sm font-semibold ${colors.labelColor}`}>{label}</span>
      </div>
      <p className={`text-lg font-bold ${colors.valueColor}`}>{value}</p>
    </div>
  );
}

// Icon components
function ProcessingTimeIcon() {
  return (
    <svg className="h-4 w-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
    </svg>
  );
}

function RealtimeIcon() {
  return (
    <svg className="h-4 w-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8.111 16.404a5.5 5.5 0 017.778 0M12 20h.01m-7.08-7.071c3.904-3.905 10.236-3.905 14.141 0M1.394 9.393c5.857-5.857 15.355-5.857 21.213 0" />
    </svg>
  );
}

function DatabaseIcon() {
  return (
    <svg className="h-4 w-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4" />
    </svg>
  );
}

function MarketsIcon() {
  return (
    <svg className="h-4 w-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
    </svg>
  );
}