'use client';

import { Shield, Star, Zap, Crown } from 'lucide-react';
import { cn } from '@/shared/utils/cn';

// Tier color configurations
export const TIER_COLORS = {
  0: {
    bg: 'bg-slate-100',
    text: 'text-slate-700',
    border: 'border-slate-300',
    glow: 'shadow-slate-200',
  },
  1: {
    bg: 'bg-blue-100',
    text: 'text-blue-700',
    border: 'border-blue-300',
    glow: 'shadow-blue-200',
  },
  2: {
    bg: 'bg-purple-100',
    text: 'text-purple-700',
    border: 'border-purple-300',
    glow: 'shadow-purple-200',
  },
  3: {
    bg: 'bg-amber-100',
    text: 'text-amber-700',
    border: 'border-amber-300',
    glow: 'shadow-amber-200',
  },
} as const;

// Tier icon mapping
export const TIER_ICONS = {
  0: Shield,
  1: Star,
  2: Zap,
  3: Crown,
} as const;

type TierLevel = 0 | 1 | 2 | 3;

interface TierBadgeProps {
  tierLevel: number;
  size?: 'sm' | 'md' | 'lg';
  showIcon?: boolean;
  className?: string;
}

export function TierBadge({ tierLevel, size = 'md', showIcon = false, className }: TierBadgeProps) {
  // Normalize tier level to 0-3 range
  const normalizedTier = Math.min(Math.max(tierLevel, 0), 3) as TierLevel;

  const label = tierLevel === 0 ? 'Free Tier' : `Level ${tierLevel}`;
  const colors = TIER_COLORS[normalizedTier];
  const Icon = showIcon ? TIER_ICONS[normalizedTier] : null;

  // Size classes
  const sizeClasses = {
    sm: 'px-2 py-0.5 text-xs gap-1',
    md: 'px-2.5 py-1 text-sm gap-1.5',
    lg: 'px-3 py-1.5 text-base gap-2',
  };

  const iconSizes = {
    sm: 'h-3 w-3',
    md: 'h-4 w-4',
    lg: 'h-5 w-5',
  };

  return (
    <span
      className={cn(
        'inline-flex items-center rounded-md font-medium border',
        colors.bg,
        colors.text,
        colors.border,
        sizeClasses[size],
        className
      )}
    >
      {Icon && <Icon className={iconSizes[size]} />}
      <span>{label}</span>
    </span>
  );
}
