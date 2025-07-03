'use client';

import { LEVEL_REQUIREMENTS } from '@/app/constants/packages';
import type { UserLevelType } from '@/app/constants/packages';

interface UserLevelBadgeProps {
  level: UserLevelType;
  className?: string;
}

export function UserLevelBadge({ level, className = '' }: UserLevelBadgeProps) {
  const getLevelColor = (level: UserLevelType) => {
    switch (level) {
      case 'PLATINUM':
        return 'bg-purple-100 text-purple-800 ring-purple-600/20 dark:bg-purple-900/30 dark:text-purple-300';
      case 'GOLD':
        return 'bg-yellow-100 text-yellow-800 ring-yellow-600/20 dark:bg-yellow-900/30 dark:text-yellow-300';
      case 'SILVER':
        return 'bg-slate-100 text-slate-800 ring-slate-600/20 dark:bg-slate-900/30 dark:text-slate-300';
      default:
        return 'bg-gray-100 text-gray-800 ring-gray-600/20 dark:bg-gray-900/30 dark:text-gray-300';
    }
  };

  return (
    <span
      className={`
        inline-flex items-center rounded-md px-2 py-1 text-sm font-medium 
        ring-1 ring-inset ${getLevelColor(level)} ${className}
      `}
    >
      {level}
    </span>
  );
}
