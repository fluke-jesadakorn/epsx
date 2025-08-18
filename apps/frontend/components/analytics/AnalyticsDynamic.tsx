'use client';

import dynamic from 'next/dynamic';
import { ComponentType } from 'react';
import { AnalyticsCardSkeleton } from './AnalyticsCardSkeleton';

interface AuthUser {
  user_id: string;
  email: string;
  role: string;
  permissions: string[];
  package_tier: string;
}

interface AnalyticsProps {
  user?: AuthUser | null;
}

// Dynamic import for heavy analytics components to reduce initial bundle size
export const AnalyticsRankingDashboard: ComponentType<AnalyticsProps> = dynamic(
  () => import('./AnalyticsRankingDash').then((mod) => ({ default: mod.AnalyticsRankingDashboard })),
  {
    loading: () => <AnalyticsCardSkeleton index={0} />,
    ssr: false, // Disable SSR for client-heavy analytics
  }
);

export const EPSGrowthChart = dynamic(
  () => import('./eps/EPSGrowthChart'),
  {
    loading: () => <div className="h-64 animate-pulse bg-muted rounded-lg" />,
    ssr: false,
  }
);

export const PatternVisualization = dynamic(
  () => import('./pattern/PatternVisualization'),
  {
    loading: () => <div className="h-96 animate-pulse bg-muted rounded-lg" />,
    ssr: false,
  }
);

export const AnalyticsMetrics = dynamic(
  () => import('./AnalyticsMetrics'),
  {
    loading: () => <div className="grid grid-cols-4 gap-4">
      {Array.from({ length: 4 }).map((_, i) => (
        <div key={i} className="h-24 animate-pulse bg-muted rounded-lg" />
      ))}
    </div>,
    ssr: false,
  }
);

// For EPS Analysis components (heavy computation)
export const EPSAnalysisForm = dynamic(
  () => import('./eps/EPSAnalysisForm'),
  {
    loading: () => <div className="h-32 animate-pulse bg-muted rounded-lg" />,
    ssr: false,
  }
);

export const PatternScanner = dynamic(
  () => import('./pattern/PatternScanner'),
  {
    loading: () => <div className="h-48 animate-pulse bg-muted rounded-lg" />,
    ssr: false,
  }
);

// Chart components that use recharts (heavy library)
export const HistoricalComparison = dynamic(
  () => import('./eps/HistoricalComparison'),
  {
    loading: () => <div className="h-80 animate-pulse bg-muted rounded-lg" />,
    ssr: false,
  }
);

export const ComparisonTable = dynamic(
  () => import('./eps/ComparisonTable'),
  {
    loading: () => <div className="h-64 animate-pulse bg-muted rounded-lg" />,
    ssr: false,
  }
);

