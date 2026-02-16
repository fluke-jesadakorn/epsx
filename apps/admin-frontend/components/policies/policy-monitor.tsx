'use client';

import { useState } from 'react';

import {
  AlertCenter,
  ChartsGrid,
  HeaderSection,
  LiveEvaluations,
  LoadingState,
  StatsOverview,
} from '@/components/policies/policy-monitor-sections';
import { usePolicyMonitor } from '@/hooks/use-policy-monitor';

export default function PolicyMonitor() {
  const [isLiveMode, setIsLiveMode] = useState(true);

  const { liveEvaluations, stats, loading, lastUpdate, loadInitialData } = usePolicyMonitor({
    isLiveMode,
    setIsLiveMode,
  });

  if (loading) {
    return <LoadingState />;
  }

  return (
    <div className="space-y-4 sm:space-y-6">
      <HeaderSection
        isLiveMode={isLiveMode}
        lastUpdate={lastUpdate}
        onToggleLiveMode={() => setIsLiveMode(!isLiveMode)}
        onRefresh={() => void loadInitialData()}
      />

      {stats && <StatsOverview stats={stats} />}

      <LiveEvaluations evaluations={liveEvaluations} isLiveMode={isLiveMode} />

      {stats && <ChartsGrid stats={stats} />}

      <AlertCenter />
    </div>
  );
}