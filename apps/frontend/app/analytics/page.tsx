import * as React from 'react';
import type { Metadata } from 'next';
import AnalyticsDashboard from '@/components/analytics/AnalyticsDashboard';

// Force dynamic rendering for real-time data
export const dynamic = 'force-dynamic';

export const metadata: Metadata = {
  title: 'Enhanced EPS Rankings - EPSX Analytics',
  description: 'Real-time EPS growth analysis and stock performance rankings with advanced filtering',
};

export default function AnalyticsPage() {
  return <AnalyticsDashboard />;
}
