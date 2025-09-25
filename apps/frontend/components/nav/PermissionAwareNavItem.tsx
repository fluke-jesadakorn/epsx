/**
 * Permission-Aware Navigation Item
 * Shows visual indicators for permission-restricted items and upgrade prompts
 */

'use client';

import React from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { Crown, Lock, Sparkles } from 'lucide-react';
import { navigationService } from '@/services/navigation.service';
import { 
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';

interface PermissionAwareNavItemProps {
  item: {
    label: string;
    href: string;
    key: string;
    requiredPermissions?: string[];
    minimumTier?: 'free' | 'basic' | 'premium' | 'professional';
    upgradePrompt?: string;
    betaFeature?: boolean;
  };
  userPermissions?: string[];
  isLoggedIn?: boolean;
  className?: string;
  children?: React.ReactNode;
  onClick?: () => void;
}

export const PermissionAwareNavItem: React.FC<PermissionAwareNavItemProps> = ({
  item,
  userPermissions = [],
  isLoggedIn = false,
  className = '',
  children,
  onClick
}) => {
  const pathname = usePathname();
  const isActive = pathname === item.href;
  
  // Check if user has access to this route
  const hasAccess = navigationService.isRouteAccessible(item.href, userPermissions, isLoggedIn);
  const upgradeMessage = navigationService.getUpgradeMessageForRoute(item.href);

  // Get visual indicators
  const getPermissionIndicator = () => {
    if (hasAccess) return null;
    
    if (!isLoggedIn && item.requiredPermissions) {
      return <Lock className="h-3 w-3 ml-1 text-gray-400" />;
    }
    
    if (item.minimumTier === 'premium' || item.minimumTier === 'professional') {
      return <Crown className="h-3 w-3 ml-1 text-amber-400" />;
    }
    
    return <Lock className="h-3 w-3 ml-1 text-gray-400" />;
  };

  const getBetaIndicator = () => {
    if (!item.betaFeature) return null;
    return <Sparkles className="h-3 w-3 ml-1 text-blue-400" />;
  };

  const getTierBadge = () => {
    if (!item.minimumTier || item.minimumTier === 'free') return null;
    
    const tierColors = {
      basic: 'bg-green-100 text-green-700 border-green-200',
      premium: 'bg-blue-100 text-blue-700 border-blue-200',
      professional: 'bg-purple-100 text-purple-700 border-purple-200'
    };
    
    const tierLabels = {
      basic: 'Basic',
      premium: 'Premium',
      professional: 'Pro'
    };
    
    return (
      <span className={`text-xs px-1.5 py-0.5 rounded border ${tierColors[item.minimumTier]} ml-2`}>
        {tierLabels[item.minimumTier]}
      </span>
    );
  };

  // Handle click - redirect to billing if no access
  const handleClick = (e: React.MouseEvent) => {
    if (!hasAccess) {
      e.preventDefault();
      if (upgradeMessage) {
        // Show upgrade prompt or redirect to billing
        window.location.href = '/billing';
      }
      return;
    }
    onClick?.();
  };

  const linkClassName = `${className} ${
    !hasAccess ? 'opacity-60 cursor-pointer' : ''
  } ${
    isActive
      ? 'border border-orange-200/50 bg-orange-50/80 text-orange-700 dark:border-orange-700/30 dark:bg-orange-900/20 dark:text-orange-300'
      : 'text-slate-600 hover:bg-slate-50/80 hover:text-slate-700 dark:text-slate-300 dark:hover:bg-slate-800/40 dark:hover:text-slate-200'
  }`;

  const navItem = (
    <Link
      href={hasAccess ? item.href : '#'}
      onClick={handleClick}
      className={linkClassName}
    >
      {children}
      <span>{item.label}</span>
      {getTierBadge()}
      {getBetaIndicator()}
      {getPermissionIndicator()}
    </Link>
  );

  // Wrap with tooltip if there's an upgrade message and no access
  if (!hasAccess && upgradeMessage) {
    return (
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            {navItem}
          </TooltipTrigger>
          <TooltipContent>
            <p className="text-sm">{upgradeMessage}</p>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    );
  }

  // Wrap with tooltip for beta features
  if (item.betaFeature) {
    return (
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            {navItem}
          </TooltipTrigger>
          <TooltipContent>
            <p className="text-sm">Beta Feature - Limited availability</p>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    );
  }

  return navItem;
};

export default PermissionAwareNavItem;