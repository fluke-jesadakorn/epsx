// User Level Utilities
// Extracted from legacy env.ts, simplified for level formatting

const LVL_MAP = { BRONZE: 1, SILVER: 2, GOLD: 3, PLATINUM: 4, DIAMOND: 5, VIP: 6 };
const LVL_NAMES = { BRONZE: 'Bronze', SILVER: 'Silver', GOLD: 'Gold', PLATINUM: 'Platinum', DIAMOND: 'Diamond', VIP: 'VIP' };
const LVL_COLS = { BRONZE: 'text-amber-600', SILVER: 'text-slate-400', GOLD: 'text-yellow-500', PLATINUM: 'text-purple-500', DIAMOND: 'text-blue-500', VIP: 'text-red-500' };

export const lvlNum = (lvl: string): number => (LVL_MAP as any)[lvl] || 0;
export const lvlName = (lvl: string): string => (LVL_NAMES as any)[lvl] || 'Bronze';
export const formatLevelAsNumber = (lvl: string): string => `Level ${lvlNum(lvl)}`;
export const getNextLevelName = (lvl: string): string => `Level ${lvlNum(lvl) + 1}`;
export const getLevelColor = (lvl: string): string => (LVL_COLS as any)[lvl] || 'text-gray-500';

// Legacy compatibility exports
export const getLevelNumber = lvlNum;
export const getLevelName = lvlName;