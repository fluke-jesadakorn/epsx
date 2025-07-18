export enum UserLevel {
  BRONZE = 'BRONZE',
  SILVER = 'SILVER',
  GOLD = 'GOLD',
  PLATINUM = 'PLATINUM',
  DIAMOND = 'DIAMOND',
  VIP = 'VIP'
}

export interface UserLevelConfig {
  level: UserLevel;
  name: string;
  description: string;
  color: string;
  benefits: string[];
  tokenMultiplier: number;
  maxTokens: number;
  priority: number;
}

export const USER_LEVEL_CONFIGS: Record<UserLevel, UserLevelConfig> = {
  [UserLevel.BRONZE]: {
    level: UserLevel.BRONZE,
    name: 'Bronze',
    description: 'Basic user level',
    color: 'bg-amber-100 text-amber-800',
    benefits: ['Basic features', 'Standard support'],
    tokenMultiplier: 1,
    maxTokens: 1000,
    priority: 1
  },
  [UserLevel.SILVER]: {
    level: UserLevel.SILVER,
    name: 'Silver',
    description: 'Intermediate user level',
    color: 'bg-gray-100 text-gray-800',
    benefits: ['Enhanced features', 'Priority support'],
    tokenMultiplier: 1.2,
    maxTokens: 2500,
    priority: 2
  },
  [UserLevel.GOLD]: {
    level: UserLevel.GOLD,
    name: 'Gold',
    description: 'Advanced user level',
    color: 'bg-yellow-100 text-yellow-800',
    benefits: ['Premium features', 'Fast support', 'API access'],
    tokenMultiplier: 1.5,
    maxTokens: 5000,
    priority: 3
  },
  [UserLevel.PLATINUM]: {
    level: UserLevel.PLATINUM,
    name: 'Platinum',
    description: 'Premium user level',
    color: 'bg-purple-100 text-purple-800',
    benefits: ['All features', 'Dedicated support', 'Custom integrations'],
    tokenMultiplier: 2,
    maxTokens: 10000,
    priority: 4
  },
  [UserLevel.DIAMOND]: {
    level: UserLevel.DIAMOND,
    name: 'Diamond',
    description: 'Elite user level',
    color: 'bg-blue-100 text-blue-800',
    benefits: ['Enterprise features', '24/7 support', 'White-label options'],
    tokenMultiplier: 3,
    maxTokens: 25000,
    priority: 5
  },
  [UserLevel.VIP]: {
    level: UserLevel.VIP,
    name: 'VIP',
    description: 'Highest user level',
    color: 'bg-red-100 text-red-800',
    benefits: ['Unlimited features', 'Personal account manager', 'Custom solutions'],
    tokenMultiplier: 5,
    maxTokens: -1, // Unlimited
    priority: 6
  }
};

export interface UserLevelAssignment {
  uid: string;
  userLevel: UserLevel;
  assignedBy: string;
  assignedAt: Date;
  reason?: string;
  previousLevel?: UserLevel;
}
