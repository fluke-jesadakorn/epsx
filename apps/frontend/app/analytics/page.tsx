import * as React from 'react';
import type { Metadata } from 'next';

import { AuthGuard } from '@/components/auth/AuthGuard';
import { AnalyticsRankingDashboard } from '@/components/analytics/AnalyticsRankingDashboard';

export const metadata: Metadata = {
  title: 'Analytics Dashboard - EPSX',
  description: 'Comprehensive stock ranking analytics and insights based on your subscription level',
};

export default function AnalyticsPage() {
  return (
    <AuthGuard requireAuth>
      <div className="min-h-screen bg-background">
        <div className="container mx-auto px-4 py-8">
          <div className="mb-8">
            <h1 className="text-3xl font-bold tracking-tight">Analytics Dashboard</h1>
            <p className="text-muted-foreground mt-2">
              Comprehensive stock ranking analytics and insights based on your subscription level
            </p>
          </div>
          
          <AnalyticsRankingDashboard />
        </div>
      </div>
    </AuthGuard>
  );
}
