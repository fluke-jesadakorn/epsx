'use client';

import { Button } from '@epsx/ui';
import { Card, CardContent } from '@epsx/ui';
import { Crown, Star, ArrowRight, Lock } from 'lucide-react';
import { useRouter } from 'next/navigation';
import type { UserLevelType } from '@/app/constants/packages';

interface UpgradePromptProps {
  currentLevel: UserLevelType;
  lockedRankings: number;
  className?: string;
}

export function UpgradePrompt({ currentLevel, lockedRankings, className }: UpgradePromptProps) {
  const router = useRouter();
  
  const getNextLevel = () => {
    const levelMap = {
      'BRONZE': 'Level 1',
      'SILVER': 'Level 2', 
      'GOLD': 'Level 3',
      'PLATINUM': 'Level 4'
    };
    return levelMap[currentLevel as keyof typeof levelMap] || 'Level 1';
  };

  const getNextLevelBenefits = () => {
    switch (currentLevel) {
      case 'BRONZE': return '25 rankings + Priority support';
      case 'SILVER': return '50 rankings + Premium features';
      case 'GOLD': return '100 rankings + Custom analytics';
      default: return 'Full access + VIP support';
    }
  };

  const handleUpgrade = () => {
    router.push('/payment');
  };

  return (
    <Card className={`border-2 border-dashed border-yellow-300 bg-gradient-to-br from-yellow-50 to-orange-50 dark:from-yellow-950/20 dark:to-orange-950/20 ${className}`}>
      <CardContent className="p-6 text-center space-y-4">
        <div className="flex justify-center">
          <Crown className="h-12 w-12 text-yellow-500" />
        </div>
        <div>
          <h3 className="text-xl font-bold mb-2">
            {lockedRankings} More Rankings Available
          </h3>
          <p className="text-muted-foreground mb-2">
            Upgrade to {getNextLevel()} to unlock premium rankings
          </p>
          <p className="text-sm text-muted-foreground">
            {getNextLevelBenefits()}
          </p>
        </div>
        <Button onClick={handleUpgrade} size="lg" className="gap-2">
          <Star className="h-4 w-4" />
          Upgrade to {getNextLevel()}
          <ArrowRight className="h-4 w-4" />
        </Button>
      </CardContent>
    </Card>
  );
}

interface LockedRankingCardProps {
  index: number;
  userLevel: UserLevelType;
  onUpgrade: () => void;
}

export function LockedRankingCard({ index, userLevel, onUpgrade }: LockedRankingCardProps) {
  const getNextLevel = () => {
    switch (userLevel) {
      case 'BRONZE': return 'Level 1';
      case 'SILVER': return 'Level 2';
      case 'GOLD': return 'Level 3';
      default: return 'Level 1';
    }
  };

  return (
    <Card className="relative overflow-hidden bg-gradient-to-br from-gray-100 to-gray-200 dark:from-gray-800 dark:to-gray-900 rounded-3xl border-2 border-dashed border-gray-300 dark:border-gray-600">
      <CardContent className="p-6">
        {/* Rank Number Badge */}
        <div className="absolute top-4 left-4 w-10 h-10 rounded-full bg-gray-400 dark:bg-gray-600 flex items-center justify-center text-white text-base font-extrabold shadow-xl border-4 border-white dark:border-gray-700">
          {index + 1}
        </div>
        
        {/* Lock Overlay */}
        <div className="absolute inset-0 bg-black/20 backdrop-blur-sm flex items-center justify-center">
          <div className="text-center space-y-3">
            <Lock className="h-8 w-8 text-gray-500 mx-auto" />
            <p className="text-sm font-medium text-gray-600 dark:text-gray-400">
              Rank #{index + 1} - Upgrade Required
            </p>
            <Button size="sm" variant="default" className="gap-2" onClick={onUpgrade}>
              <Crown className="h-4 w-4" />
              Upgrade to {getNextLevel()}
            </Button>
          </div>
        </div>
        
        {/* Placeholder content */}
        <div className="space-y-4 pt-8">
          <div className="flex justify-between items-start">
            <div>
              <div className="w-16 h-6 bg-gray-300 dark:bg-gray-600 rounded animate-pulse"></div>
              <div className="w-24 h-4 bg-gray-200 dark:bg-gray-700 rounded mt-2 animate-pulse"></div>
            </div>
            <div className="text-right">
              <div className="w-20 h-6 bg-gray-300 dark:bg-gray-600 rounded animate-pulse"></div>
              <div className="w-16 h-4 bg-gray-200 dark:bg-gray-700 rounded mt-2 animate-pulse"></div>
            </div>
          </div>
          <div className="space-y-2">
            <div className="w-full h-4 bg-gray-200 dark:bg-gray-700 rounded animate-pulse"></div>
            <div className="w-3/4 h-4 bg-gray-200 dark:bg-gray-700 rounded animate-pulse"></div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
