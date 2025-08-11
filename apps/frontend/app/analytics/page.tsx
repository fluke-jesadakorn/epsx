import * as React from 'react';
import type { Metadata } from 'next';

import { AnalyticsRankingDashboard } from '@/components/analytics/AnalyticsDynamic';

// Disable ISR caching for analytics - always fetch fresh data from backend
export const revalidate = 0;

export const metadata: Metadata = {
  title: 'Analytics Dashboard - EPSX',
  description: 'Comprehensive stock ranking analytics and insights - free access for all users',
};

export default function AnalyticsPage() {
  return (
    <div className="min-h-screen bg-background relative overflow-hidden">
      {/* PancakeSwap-style background decorations */}
      <div className="fixed inset-0 z-0 pointer-events-none">
        <div className="absolute top-20 left-10 w-24 h-24 bg-gradient-to-br from-blue-400/10 to-cyan-400/10 rounded-full animate-float" />
        <div className="absolute bottom-20 right-20 w-32 h-32 bg-gradient-to-br from-orange-400/10 to-yellow-400/10 rounded-full animate-bounce-gentle" />
        <div className="absolute top-1/2 right-1/4 w-20 h-20 bg-gradient-to-br from-purple-400/10 to-pink-400/10 rounded-full animate-pulse-gentle" />
      </div>

      <div className="relative z-10 container mx-auto px-4 py-8">
        <div className="mb-8 text-center">
          {/* Header with clean styling */}
          <div className="inline-block animate-slide-up">
            <h1 className="text-4xl sm:text-5xl font-bold tracking-tight mb-4">
              <span className="pancake-gradient-text">📊 Analytics Dashboard</span>
            </h1>
            <p className="text-lg text-muted-foreground max-w-3xl mx-auto leading-relaxed">
              🚀 Comprehensive stock ranking analytics and insights - free access for all users
            </p>
            <div className="w-32 h-1.5 bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 mx-auto rounded-full mt-6" />
          </div>
        </div>
        
        <div className="animate-slide-up-delayed">
          {/* Stock Rankings Dashboard */}
          <AnalyticsRankingDashboard />
        </div>
      </div>
    </div>
  );
}
