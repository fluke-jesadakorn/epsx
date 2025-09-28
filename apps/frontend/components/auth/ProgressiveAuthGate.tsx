/**
 * Simplified Gate Component
 * No authentication checks - just renders children
 */
'use client';

interface ProgressiveAuthGateProps {
  children: React.ReactNode;
  fallback?: React.ReactNode;
  [key: string]: any; // Accept any other props to maintain compatibility
}

export function ProgressiveAuthGate({
  children,
  fallback,
  ...otherProps
}: ProgressiveAuthGateProps) {
  // No authentication logic - just render children
  return <>{children}</>;
}