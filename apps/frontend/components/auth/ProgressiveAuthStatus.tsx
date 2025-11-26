/**
 * Simplified Auth Status Component  
 * No authentication checks - just renders wallet auth
 */
'use client';

import { WalletConnectAuth } from './WalletConnectAuth';

interface ProgressiveAuthStatusProps {
  compact?: boolean;
  showUpgradeHints?: boolean;
  className?: string;
}

export function ProgressiveAuthStatus({ 
  compact = false, 
  className = ''
}: ProgressiveAuthStatusProps) {
  // Just render wallet auth component
  return (
    <div className={`flex items-center gap-3 ${className}`}>
      <WalletConnectAuth 
        className="text-sm" 
        compact={compact}
      />
    </div>
  );
}