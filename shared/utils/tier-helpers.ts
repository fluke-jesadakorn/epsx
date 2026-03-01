/**
 * Tier utility functions for plan tier management
 */

/**
 * Get display label for tier level
 */
export function getTierLabel(tierLevel: number): string {
  return tierLevel === 0 ? 'Free Tier' : `Level ${tierLevel}`;
}

/**
 * Get tier color configuration
 */
export function getTierColor(tierLevel: number) {
  const normalizedTier = Math.min(Math.max(tierLevel, 0), 3) as 0 | 1 | 2 | 3;

  const colors = {
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
  };

  return colors[normalizedTier];
}

/**
 * Get tier icon name (for lucide-react)
 */
export function getTierIconName(tierLevel: number): string {
  const normalizedTier = Math.min(Math.max(tierLevel, 0), 3);
  const icons = ['Shield', 'Star', 'Zap', 'Crown'];
  return icons[normalizedTier] ?? 'Shield';
}

/**
 * Sort plans by tier level ascending
 */
export function sortPlansByTier<T extends { tier_level?: number | null }>(plans: T[]): T[] {
  return [...plans].sort((a, b) => {
    const tierA = a.tier_level ?? 0;
    const tierB = b.tier_level ?? 0;
    return tierA - tierB;
  });
}

/**
 * Check if a plan is an upgrade from current tier
 */
export function isUpgrade(currentTier: number, targetTier: number): boolean {
  return targetTier > currentTier;
}

/**
 * Check if a plan is a downgrade from current tier
 */
export function isDowngrade(currentTier: number, targetTier: number): boolean {
  return targetTier < currentTier;
}
