/**
 * Simplified Navigation Item
 * No permission checking - backend handles access control
 */

'use client';

import React from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { Sparkles } from 'lucide-react';
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
  className = '',
  children,
  onClick
}) => {
  const pathname = usePathname();
  const isActive = pathname === item.href;

  const getBetaIndicator = () => {
    if (!item.betaFeature) return null;
    return <Sparkles className="h-3 w-3 ml-1 text-blue-400" />;
  };

  const linkClassName = `${className} ${
    isActive
      ? 'border border-orange-200/50 bg-orange-50/80 text-orange-700 dark:border-orange-700/30 dark:bg-orange-900/20 dark:text-orange-300'
      : 'text-slate-600 hover:bg-slate-50/80 hover:text-slate-700 dark:text-slate-300 dark:hover:bg-slate-800/40 dark:hover:text-slate-200'
  }`;

  const navItem = (
    <Link
      href={item.href}
      onClick={onClick}
      className={linkClassName}
    >
      {children}
      <span>{item.label}</span>
      {getBetaIndicator()}
    </Link>
  );

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