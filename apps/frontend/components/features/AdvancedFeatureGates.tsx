// ============================================================================
// SIMPLE ADVANCED FEATURE GATES - REPLACES COMPLEX FEATURE LOGIC
// ============================================================================
// Simple stubs - use FeatureGuard components instead

'use client';

import { ReactNode } from 'react';
import { 
  AdminOnly, 
  UserOnly, 
  ViewEpsOnly, 
  ExportDataOnly, 
  RealtimeOnly,
  AdvancedFiltersOnly 
} from '@/components/guards/FeatureGuard';

// Simple stubs for backwards compatibility
export function PremiumGate({ children }: { children: ReactNode }) {
  return <UserOnly>{children}</UserOnly>;
}

export function AdminGate({ children }: { children: ReactNode }) {
  return <AdminOnly>{children}</AdminOnly>;
}

export function AnalyticsGate({ children }: { children: ReactNode }) {
  return <ViewEpsOnly>{children}</ViewEpsOnly>;
}

export function ExportGate({ children }: { children: ReactNode }) {
  return <ExportDataOnly>{children}</ExportDataOnly>;
}

export function RealtimeGate({ children }: { children: ReactNode }) {
  return <RealtimeOnly>{children}</RealtimeOnly>;
}

export function FiltersGate({ children }: { children: ReactNode }) {
  return <AdvancedFiltersOnly>{children}</AdvancedFiltersOnly>;
}