import AnalyticsDashboard from '@/components/analytics/AnalyticsDashboard';
import type { Metadata } from 'next';

// Force dynamic rendering for real-time data
export const dynamic = 'force-dynamic';

export const metadata: Metadata = {
  title: 'EPS Rankings - EPSX Analytics',
  description:
    'Real-time EPS growth analysis and stock performance rankings with advanced filtering',
};

export default function AnalyticsPage() {
  return <AnalyticsDashboard />;
}
