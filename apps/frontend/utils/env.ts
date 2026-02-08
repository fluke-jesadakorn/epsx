// Environment utilities (minimal implementation)

export function getCurrentEnvironment(): string {
  return process.env.NODE_ENV;
}

export function getAssetConfig(): Record<string, unknown> {
  return {};
}

export function getDefaultCurrency(): string {
  return 'USD';
}

export function getSupportedCurrencies(): string[] {
  return ['USD', 'USDT'];
}

export function getMusePayApiUrl(): string {
  return process.env.NEXT_PUBLIC_MUSEPAY_API_URL ?? '';
}

export function getDatabaseName(): string {
  return 'epsx_db';
}

export function getLevelNumber(level: string): number {
  const levels: Record<string, number> = {
    'FREE': 0,
    'BRONZE': 1,
    'SILVER': 2,
    'GOLD': 3,
    'PLATINUM': 4,
    'ENTERPRISE': 5
  };
  return levels[level] ?? 0;
}

export function getLevelName(tier: string): string {
  return tier;
}

export function formatLevelAsNumber(level: string): number {
  return getLevelNumber(level);
}

export function getNextLevelName(currentLevel: string): string {
  const levels = ['FREE', 'BRONZE', 'SILVER', 'GOLD', 'PLATINUM', 'ENTERPRISE'];
  const currentIndex = levels.indexOf(currentLevel);
  return levels[currentIndex + 1] ?? currentLevel;
}

export function getLevelColor(level: string): string {
  const colors: Record<string, string> = {
    'FREE': '#9CA3AF',
    'BRONZE': '#CD7F32',
    'SILVER': '#C0C0C0',
    'GOLD': '#FFD700',
    'PLATINUM': '#E5E4E2',
    'ENTERPRISE': '#8B5CF6'
  };
  return colors[level] ?? colors['FREE'];
}