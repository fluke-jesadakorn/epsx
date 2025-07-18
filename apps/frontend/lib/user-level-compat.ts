// Legacy compatibility bridge for user level system
import type { UserLevelType } from '@/app/constants/packages';

// Map legacy level names to new system
export const LEGACY_LEVEL_MAPPING: Record<string, UserLevelType> = {
  'BASIC': 'BRONZE',
  'Basic': 'BRONZE',
  'basic': 'BRONZE',
} as const;

// Helper function to convert legacy level to new level
export function mapLegacyLevel(level: string): UserLevelType {
  return LEGACY_LEVEL_MAPPING[level] || (level as UserLevelType);
}

// Helper function to check if a level is legacy
export function isLegacyLevel(level: string): boolean {
  return level in LEGACY_LEVEL_MAPPING;
}

// Export for backward compatibility
export type { UserLevelType } from '@/app/constants/packages';
export { LEVEL_CONFIGS, LEVEL_BENEFITS } from '@/app/constants/packages';
