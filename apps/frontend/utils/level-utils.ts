import type { UserLevelType } from '@/app/constants/packages';
import { LEVEL_CONFIGS } from '@/app/constants/packages';

/**
 * Convert user level string to numeric level
 */
export function getLevelNumber(level: UserLevelType | string): number {
  const levelConfig = LEVEL_CONFIGS[level as UserLevelType];
  return levelConfig?.numericLevel || 0;
}

/**
 * Get level name from level type
 */
export function getLevelName(level: UserLevelType | string): string {
  const levelConfig = LEVEL_CONFIGS[level as UserLevelType];
  return levelConfig?.name || 'Bronze';
}

/**
 * Format level as "Level X" where X is the numeric level
 */
export function formatLevelAsNumber(level: UserLevelType | string): string {
  const levelNumber = getLevelNumber(level);
  return `Level ${levelNumber}`;
}

/**
 * Get the next level name for upgrade purposes
 */
export function getNextLevelName(currentLevel: UserLevelType | string): string {
  const currentLevelNumber = getLevelNumber(currentLevel);
  const nextLevelNumber = currentLevelNumber + 1;
  
  // Find the level with the next numeric level
  const nextLevelEntry = Object.entries(LEVEL_CONFIGS).find(
    ([_, config]) => config.numericLevel === nextLevelNumber
  );
  
  if (nextLevelEntry) {
    return `Level ${nextLevelNumber}`;
  }
  
  return 'Premium Level';
}

/**
 * Level color mappings
 */
export const levelColors = {
  'BRONZE': 'text-amber-600',
  'SILVER': 'text-slate-400',
  'GOLD': 'text-yellow-500',
  'PLATINUM': 'text-purple-500',
  'DIAMOND': 'text-blue-500',
  'VIP': 'text-red-500',
  'API_PERSONAL': 'text-indigo-500',
  'API_COMPANY': 'text-blue-600',
  'API_PARTNER': 'text-purple-600',
} as const;

export function getLevelColor(level: UserLevelType | string): string {
  return levelColors[level as keyof typeof levelColors] || 'text-gray-500';
}
