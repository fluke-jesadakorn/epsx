"use client";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { PACKAGES, LEVEL_REQUIREMENTS } from "@/app/constants/packages";
import type { UserLevelType } from "@/app/constants/packages";
import { 
  Trophy, 
  Crown, 
  Star, 
  Gem, 
  Zap, 
  Shield, 
  Award,
  Target,
  TrendingUp,
  CheckCircle,
  Lock
} from "lucide-react";

interface AchievementBadgesProps {
  currentLevel: UserLevelType;
  paymentCount: number;
  className?: string;
}

interface Achievement {
  id: string;
  title: string;
  description: string;
  icon: React.ReactNode;
  level: UserLevelType;
  isUnlocked: boolean;
  gradient: string;
  glowColor: string;
}

const achievementIcons = {
  BASIC: <Star className="h-6 w-6" />,
  SILVER: <Trophy className="h-6 w-6" />,
  GOLD: <Crown className="h-6 w-6" />,
  PLATINUM: <Gem className="h-6 w-6" />,
  MILESTONE_1: <Award className="h-6 w-6" />,
  MILESTONE_5: <Target className="h-6 w-6" />,
  MILESTONE_10: <TrendingUp className="h-6 w-6" />,
  VIP: <Shield className="h-6 w-6" />,
};

export function AchievementBadges({ 
  currentLevel, 
  paymentCount, 
  className 
}: AchievementBadgesProps) {
  const getCurrentLevelNumeric = () => {
    const pkg = PACKAGES.find(p => p.level === currentLevel);
    return pkg?.numericLevel || 0;
  };

  const currentLevelNumeric = getCurrentLevelNumeric();

  const achievements: Achievement[] = [
    {
      id: 'newcomer',
      title: 'Welcome Aboard!',
      description: 'Started your journey',
      icon: achievementIcons.BASIC,
      level: 'BASIC',
      isUnlocked: true,
      gradient: 'from-gray-400 to-gray-600',
      glowColor: 'shadow-gray-500/25',
    },
    {
      id: 'first_payment',
      title: 'First Step',
      description: 'Made your first payment',
      icon: achievementIcons.SILVER,
      level: 'SILVER',
      isUnlocked: paymentCount >= 1,
      gradient: 'from-slate-400 to-slate-600',
      glowColor: 'shadow-slate-500/25',
    },
    {
      id: 'committed_user',
      title: 'Committed User',
      description: 'Reached Gold level',
      icon: achievementIcons.GOLD,
      level: 'GOLD',
      isUnlocked: currentLevelNumeric >= 2,
      gradient: 'from-yellow-400 to-orange-500',
      glowColor: 'shadow-yellow-500/25',
    },
    {
      id: 'premium_member',
      title: 'Premium Member',
      description: 'Achieved Platinum status',
      icon: achievementIcons.PLATINUM,
      level: 'PLATINUM',
      isUnlocked: currentLevelNumeric >= 3,
      gradient: 'from-purple-500 to-pink-600',
      glowColor: 'shadow-purple-500/25',
    },
    {
      id: 'payment_milestone_5',
      title: 'Loyal Customer',
      description: '5+ successful payments',
      icon: achievementIcons.MILESTONE_5,
      level: 'GOLD',
      isUnlocked: paymentCount >= 5,
      gradient: 'from-blue-500 to-cyan-500',
      glowColor: 'shadow-blue-500/25',
    },
    {
      id: 'payment_milestone_10',
      title: 'VIP Member',
      description: '10+ successful payments',
      icon: achievementIcons.MILESTONE_10,
      level: 'PLATINUM',
      isUnlocked: paymentCount >= 10,
      gradient: 'from-emerald-500 to-green-600',
      glowColor: 'shadow-emerald-500/25',
    },
  ];

  const unlockedAchievements = achievements.filter(a => a.isUnlocked);
  const lockedAchievements = achievements.filter(a => !a.isUnlocked);

  return (
    <Card className={`relative overflow-hidden ${className}`}>
      <CardHeader className="pb-4">
        <CardTitle className="flex items-center gap-2 text-lg">
          <Award className="h-5 w-5 text-primary" />
          Achievements
          <Badge variant="secondary" className="ml-auto">
            {unlockedAchievements.length}/{achievements.length}
          </Badge>
        </CardTitle>
      </CardHeader>
      
      <CardContent>
        <div className="space-y-6">
          {/* Unlocked Achievements */}
          {unlockedAchievements.length > 0 && (
            <div>
              <h4 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider mb-3">
                Unlocked ({unlockedAchievements.length})
              </h4>
              <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
                {unlockedAchievements.map((achievement) => (
                  <div
                    key={achievement.id}
                    className={`relative group p-3 rounded-xl bg-gradient-to-br ${achievement.gradient} ${achievement.glowColor} shadow-lg transition-all duration-300 hover:scale-105 hover:shadow-xl`}
                  >
                    {/* Glow effect */}
                    <div className="absolute inset-0 rounded-xl bg-gradient-to-br from-white/20 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
                    
                    <div className="relative text-center space-y-2">
                      <div className="text-white mx-auto flex justify-center">
                        {achievement.icon}
                      </div>
                      <div>
                        <h5 className="text-xs font-semibold text-white leading-tight">
                          {achievement.title}
                        </h5>
                        <p className="text-xs text-white/80 mt-1">
                          {achievement.description}
                        </p>
                      </div>
                    </div>

                    {/* Sparkle effects for special achievements */}
                    {(achievement.level === 'GOLD' || achievement.level === 'PLATINUM') && achievement.isUnlocked && (
                      <div className="absolute inset-0 pointer-events-none">
                        <div className="absolute top-1 right-1 w-1 h-1 bg-yellow-300 rounded-full animate-ping"></div>
                        <div className="absolute bottom-1 left-1 w-0.5 h-0.5 bg-yellow-300 rounded-full animate-pulse delay-300"></div>
                        <div className="absolute top-1/2 left-1 w-1 h-1 bg-yellow-300 rounded-full animate-pulse delay-700"></div>
                      </div>
                    )}

                    {/* Checkmark for unlocked */}
                    <div className="absolute -top-1 -right-1 w-5 h-5 bg-green-500 rounded-full flex items-center justify-center shadow-lg">
                      <CheckCircle className="h-3 w-3 text-white" />
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Locked Achievements */}
          {lockedAchievements.length > 0 && (
            <div>
              <h4 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider mb-3">
                Locked ({lockedAchievements.length})
              </h4>
              <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
                {lockedAchievements.map((achievement) => (
                  <div
                    key={achievement.id}
                    className="relative group p-3 rounded-xl bg-gradient-to-br from-gray-200 to-gray-300 dark:from-gray-800 dark:to-gray-700 shadow-sm transition-all duration-300 hover:shadow-md opacity-60"
                  >
                    <div className="relative text-center space-y-2">
                      <div className="text-gray-500 dark:text-gray-400 mx-auto flex justify-center">
                        {achievement.icon}
                      </div>
                      <div>
                        <h5 className="text-xs font-semibold text-gray-600 dark:text-gray-300 leading-tight">
                          {achievement.title}
                        </h5>
                        <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                          {achievement.description}
                        </p>
                      </div>
                    </div>

                    {/* Lock icon for locked achievements */}
                    <div className="absolute -top-1 -right-1 w-5 h-5 bg-gray-500 rounded-full flex items-center justify-center shadow-lg">
                      <Lock className="h-3 w-3 text-white" />
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Progress Summary */}
          <div className="pt-4 border-t border-muted/30">
            <div className="flex items-center justify-between text-sm">
              <span className="text-muted-foreground">Achievement Progress</span>
              <span className="font-semibold">
                {Math.round((unlockedAchievements.length / achievements.length) * 100)}% Complete
              </span>
            </div>
            <div className="w-full bg-muted/30 rounded-full h-2 mt-2 overflow-hidden">
              <div 
                className="h-full bg-gradient-to-r from-primary to-secondary transition-all duration-1000 ease-out rounded-full relative"
                style={{ width: `${(unlockedAchievements.length / achievements.length) * 100}%` }}
              >
                <div className="absolute inset-0 bg-white/20 animate-shimmer"></div>
              </div>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
