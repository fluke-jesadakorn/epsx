'use client';

import { ReactNode, useState, MouseEvent } from 'react';
import { Lock, Crown, Shield, Zap } from 'lucide-react';
import { Card } from '@/components/ui';

// Define visual restriction modes
type RestrictionMode = 'blur' | 'opacity' | 'locked' | 'premium' | 'admin';

interface VisualPermissionGuardProps {
  children: ReactNode;
  // Permission checking
  hasPermission: boolean;
  requiredPermission?: string;
  requiredTier?: 'BRONZE' | 'SILVER' | 'GOLD' | 'PLATINUM' | 'ENTERPRISE';
  
  // Visual configuration
  restrictionMode?: RestrictionMode;
  blurIntensity?: 'light' | 'medium' | 'heavy'; // 2px, 4px, 8px
  opacity?: number; // 0.1 to 1.0
  
  // Content
  restrictionReason?: string;
  upgradeMessage?: string;
  
  // Interactions
  onClick?: () => void;
  showTooltip?: boolean;
  allowInteraction?: boolean; // If false, pointer-events: none
  
  // Modal/Request handlers
  onRequestAccess?: () => void;
  onUpgradePrompt?: () => void;
  
  // Styling
  className?: string;
  lockIconSize?: number;
}

export function VisualPermissionGuard({
  children,
  hasPermission,
  requiredPermission,
  requiredTier,
  restrictionMode = 'blur',
  blurIntensity = 'medium',
  opacity = 0.4,
  restrictionReason,
  upgradeMessage,
  onClick,
  showTooltip = true,
  allowInteraction = false,
  onRequestAccess,
  onUpgradePrompt,
  className = '',
  lockIconSize = 24
}: VisualPermissionGuardProps) {
  const [showDetails, setShowDetails] = useState(false);

  // If user has permission, render normally
  if (hasPermission) {
    return <>{children}</>;
  }

  // Generate default messages based on context
  const defaultRestrictionReason = requiredPermission 
    ? `Requires permission: ${requiredPermission}`
    : requiredTier 
    ? `Requires ${requiredTier} tier or higher`
    : 'Access restricted';

  const finalRestrictionReason = restrictionReason || defaultRestrictionReason;
  
  const defaultUpgradeMessage = requiredTier
    ? `Upgrade to ${requiredTier} to unlock this feature`
    : 'Contact admin for access';
  
  const finalUpgradeMessage = upgradeMessage || defaultUpgradeMessage;

  // Generate CSS classes based on restriction mode
  const getRestrictionClasses = (): string => {
    const baseClasses = 'permission-restricted relative';
    
    switch (restrictionMode) {
      case 'blur':
        return `${baseClasses} permission-restricted-blur-${blurIntensity}`;
      case 'opacity':
        return `${baseClasses} permission-restricted-opacity`;
      case 'locked':
        return `${baseClasses} permission-restricted-locked`;
      case 'premium':
        return `${baseClasses} permission-restricted-premium`;
      case 'admin':
        return `${baseClasses} permission-restricted-admin`;
      default:
        return baseClasses;
    }
  };

  // Get icon based on restriction type
  const getRestrictionIcon = () => {
    switch (restrictionMode) {
      case 'premium':
        return <Crown size={lockIconSize} className="text-yellow-500" />;
      case 'admin':
        return <Shield size={lockIconSize} className="text-red-500" />;
      case 'locked':
      default:
        return <Lock size={lockIconSize} className="text-gray-500" />;
    }
  };

  // Handle click events
  const handleClick = (e: MouseEvent) => {
    if (!allowInteraction) {
      e.preventDefault();
      e.stopPropagation();
    }
    
    if (onClick) {
      onClick();
    } else if (requiredTier && onUpgradePrompt) {
      onUpgradePrompt();
    } else if (onRequestAccess) {
      onRequestAccess();
    }
  };

  const restrictedContent = (
    <div 
      className={`${getRestrictionClasses()} ${className} cursor-pointer transition-all duration-200 hover:brightness-110`}
      style={{
        opacity: restrictionMode === 'opacity' ? opacity : undefined,
        pointerEvents: allowInteraction ? 'auto' : 'none'
      }}
      onClick={handleClick}
    >
      {children}
      
      {/* Overlay with lock icon */}
      <div className="absolute inset-0 flex items-center justify-center bg-black/20 opacity-0 hover:opacity-100 transition-opacity duration-200 rounded">
        <div className="bg-white/90 dark:bg-gray-800/90 rounded-full p-3 backdrop-blur-sm">
          {getRestrictionIcon()}
        </div>
      </div>
      
      {/* Corner badge for tier requirements */}
      {requiredTier && (
        <div className="absolute top-2 right-2">
          <div className="bg-gradient-to-r from-yellow-400 to-orange-500 text-white text-xs font-bold px-2 py-1 rounded-full shadow">
            {requiredTier}
          </div>
        </div>
      )}
    </div>
  );

  // Add tooltip info via title attribute if enabled
  if (showTooltip) {
    const tooltipText = `${finalRestrictionReason} - ${finalUpgradeMessage}${(onRequestAccess || onUpgradePrompt) ? ' - Click to learn more' : ''}`;
    return (
      <div title={tooltipText}>
        {restrictedContent}
      </div>
    );
  }

  return restrictedContent;
}

// Convenience components for common use cases
export const BlurredFeature = ({ children, hasPermission, ...props }: Omit<VisualPermissionGuardProps, 'restrictionMode'>) => (
  <VisualPermissionGuard restrictionMode="blur" hasPermission={hasPermission} {...props}>
    {children}
  </VisualPermissionGuard>
);

export const LockedFeature = ({ children, hasPermission, ...props }: Omit<VisualPermissionGuardProps, 'restrictionMode'>) => (
  <VisualPermissionGuard restrictionMode="locked" hasPermission={hasPermission} {...props}>
    {children}
  </VisualPermissionGuard>
);

export const PremiumFeature = ({ children, hasPermission, ...props }: Omit<VisualPermissionGuardProps, 'restrictionMode'>) => (
  <VisualPermissionGuard restrictionMode="premium" hasPermission={hasPermission} {...props}>
    {children}
  </VisualPermissionGuard>
);

export const AdminOnlyFeature = ({ children, hasPermission, ...props }: Omit<VisualPermissionGuardProps, 'restrictionMode'>) => (
  <VisualPermissionGuard restrictionMode="admin" hasPermission={hasPermission} {...props}>
    {children}
  </VisualPermissionGuard>
);

export const FadedFeature = ({ children, hasPermission, ...props }: Omit<VisualPermissionGuardProps, 'restrictionMode' | 'opacity'>) => (
  <VisualPermissionGuard restrictionMode="opacity" opacity={0.3} hasPermission={hasPermission} {...props}>
    {children}
  </VisualPermissionGuard>
);