'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { useRouter } from 'next/navigation';
import {
  PACKAGES,
  LEVEL_BENEFITS,
  LEVEL_REQUIREMENTS,
  getPackageByLevel,
} from '@/app/constants/packages';
import type { UserLevelType } from '@/app/constants/packages';
import { useAuth } from '@/context/auth-context-improved';
import { status } from '@/services/pay';
import { Crown, Star, Trophy, Gem, Zap, Lock, ArrowRight } from 'lucide-react';

interface UserLevelDisplayProps {
  className?: string;
}

interface UserStatus {
  level: UserLevelType;
  paymentCount: number;
  isExpired: boolean;
  expirationDate?: Date;
}

const levelIcons = {
  BRONZE: <Star className="h-6 w-6" />,
  SILVER: <Trophy className="h-6 w-6" />,
  GOLD: <Crown className="h-6 w-6" />,
  PLATINUM: <Gem className="h-6 w-6" />,
  DIAMOND: <Zap className="h-6 w-6" />,
  VIP: <Crown className="h-6 w-6" />,
  API_PERSONAL: <Zap className="h-6 w-6" />,
  API_COMPANY: <Zap className="h-6 w-6" />,
  API_PARTNER: <Zap className="h-6 w-6" />,
};

const levelGradients = {
  BRONZE: 'from-amber-400 to-amber-600',
  SILVER: 'from-slate-400 to-slate-600',
  GOLD: 'from-yellow-400 to-orange-500',
  PLATINUM: 'from-purple-500 to-pink-600',
  DIAMOND: 'from-blue-500 to-cyan-600',
  VIP: 'from-red-500 to-pink-600',
  API_PERSONAL: 'from-indigo-500 to-blue-600',
  API_COMPANY: 'from-blue-600 to-cyan-600',
  API_PARTNER: 'from-purple-600 to-indigo-700',
};

export function UserLevelDisplay({ className }: UserLevelDisplayProps) {
  const { user } = useAuth();
  const router = useRouter();
  const [userStatus, setUserStatus] = useState<UserStatus | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const fetchUserStatus = async () => {
      if (!user) {
        setIsLoading(false);
        return;
      }

      try {
        const paymentStatus = await status();
        setUserStatus({
          level: paymentStatus.level as UserLevelType,
          paymentCount: 0, // This would come from your payment history
          isExpired: paymentStatus.expire
            ? new Date() > paymentStatus.expire
            : !paymentStatus.paid,
          expirationDate: paymentStatus.expire,
        });
      } catch (error) {
        console.error('Failed to fetch user status:', error);
        setUserStatus({
          level: 'BRONZE',
          paymentCount: 0,
          isExpired: true,
        });
      } finally {
        setIsLoading(false);
      }
    };

    fetchUserStatus();
  }, [user]);

  if (!user) {
    return (
      <Card className="border-2 border-dashed border-muted-foreground/25">
        <CardContent className="flex items-center justify-center py-8">
          <div className="text-center space-y-3">
            <Lock className="h-12 w-12 text-muted-foreground mx-auto" />
            <h3 className="text-lg font-semibold text-muted-foreground">
              Sign in to view your level
            </h3>
            <p className="text-sm text-muted-foreground/75">
              Access your progress and unlock premium features
            </p>
          </div>
        </CardContent>
      </Card>
    );
  }

  if (isLoading) {
    return (
      <Card className="animate-pulse">
        <CardHeader className="pb-2">
          <div className="flex items-center gap-3">
            <div className="w-12 h-12 rounded-full bg-gray-200 dark:bg-gray-700"></div>
            <div className="space-y-2">
              <div className="h-6 w-32 bg-gray-200 dark:bg-gray-700 rounded"></div>
              <div className="h-4 w-24 bg-gray-200 dark:bg-gray-700 rounded"></div>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            <div className="h-4 w-full bg-gray-200 dark:bg-gray-700 rounded"></div>
            <div className="h-4 w-3/4 bg-gray-200 dark:bg-gray-700 rounded"></div>
          </div>
        </CardContent>
      </Card>
    );
  }

  const currentLevel = userStatus?.level || 'BRONZE';
  const currentPackage = getPackageByLevel(currentLevel);
  const nextLevelOptions = PACKAGES.filter(
    (pkg) =>
      pkg.numericLevel > (currentPackage?.numericLevel || 0) &&
      !pkg.level.startsWith('API'),
  ).slice(0, 2);

  const getProgressToNextLevel = () => {
    const current =
      LEVEL_REQUIREMENTS[currentLevel as keyof typeof LEVEL_REQUIREMENTS];
    const nextLevel = nextLevelOptions[0];

    if (!current || !nextLevel) return 100;

    const progress = Math.min(
      ((userStatus?.paymentCount || 0) / nextLevel.minPayments) * 100,
      100,
    );
    return progress;
  };

  return (
    <Card
      className={`relative overflow-hidden transition-all duration-500 hover:shadow-2xl transform hover:scale-[1.02] bg-gradient-to-br from-card/90 to-card/60 backdrop-blur-lg border-2 ${
        currentLevel === 'GOLD' || currentLevel === 'PLATINUM' 
          ? 'border-primary/40 shadow-primary/20' 
          : 'border-primary/20'
      } ${className}`}
    >
      {/* Enhanced Decorative Background Elements */}
      <div className="absolute inset-0 pointer-events-none">
        <div className="absolute top-0 right-0 w-40 h-40 opacity-15">
          <div
            className={`w-full h-full bg-gradient-to-br ${levelGradients[currentLevel]} rounded-full blur-3xl animate-pulse-slow`}
          ></div>
        </div>
        <div className="absolute bottom-0 left-0 w-32 h-32 opacity-10">
          <div
            className={`w-full h-full bg-gradient-to-tr ${levelGradients[currentLevel]} rounded-full blur-2xl animate-float`}
          ></div>
        </div>
        <div className="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 w-48 h-48 opacity-5">
          <div
            className={`w-full h-full bg-gradient-to-br ${levelGradients[currentLevel]} rounded-full blur-3xl animate-blob1`}
          ></div>
        </div>
        
        {/* Floating Particles for Premium Levels */}
        {(currentLevel === 'GOLD' || currentLevel === 'PLATINUM') && (
          <>
            {[...Array(8)].map((_, i) => (
              <div
                key={i}
                className="absolute w-1 h-1 bg-yellow-400 rounded-full animate-float opacity-60"
                style={{
                  left: `${20 + Math.random() * 60}%`,
                  top: `${20 + Math.random() * 60}%`,
                  animationDelay: `${Math.random() * 3}s`,
                  animationDuration: `${2 + Math.random() * 2}s`,
                }}
              />
            ))}
          </>
        )}
      </div>

      <CardHeader className="relative pb-3 sm:pb-4 border-b border-primary/10">
        {/* Mobile-First Design with Stacked Layout */}
        <div className="space-y-4">
          {/* Top Row: Icon + Title + Badge */}
          <div className="flex items-center gap-3 sm:gap-4">
            {/* Enhanced Level Icon - Responsive Sizing */}
            <div className="relative group flex-shrink-0">
              <div
                className={`relative p-2.5 xs:p-3 sm:p-4 rounded-2xl xs:rounded-3xl bg-gradient-to-br ${levelGradients[currentLevel]} shadow-lg xs:shadow-xl transform transition-all duration-300 group-hover:scale-105 group-hover:rotate-2`}
              >
                <div className="text-white transform transition-transform duration-300 group-hover:scale-110">
                  <div className="w-5 h-5 xs:w-6 xs:h-6 sm:w-7 sm:h-7">
                    {levelIcons[currentLevel]}
                  </div>
                </div>
                
                {/* Premium Level Indicators */}
                {(currentLevel === 'GOLD' || currentLevel === 'PLATINUM') && (
                  <>
                    <div className="absolute -top-1 -right-1 xs:-top-2 xs:-right-2 w-3 h-3 xs:w-4 xs:h-4 sm:w-5 sm:h-5 bg-yellow-400 rounded-full animate-bounce shadow-md">
                      <Star className="h-2 w-2 xs:h-2.5 xs:w-2.5 sm:h-3 sm:w-3 text-yellow-800 m-0.5" />
                    </div>
                    <div className="absolute inset-0 rounded-2xl xs:rounded-3xl bg-yellow-400/20 blur-sm animate-pulse"></div>
                  </>
                )}
                
                {/* Glow Effect */}
                <div className={`absolute inset-0 rounded-2xl xs:rounded-3xl bg-gradient-to-br ${levelGradients[currentLevel]} opacity-0 group-hover:opacity-30 blur-lg transition-opacity duration-300`}></div>
              </div>
            </div>
            
            {/* Level Information - Responsive Layout */}
            <div className="min-w-0 flex-1">
              <div className="flex flex-col xs:flex-row xs:items-center gap-1 xs:gap-2 sm:gap-3">
                <h2 className="text-lg xs:text-xl sm:text-2xl lg:text-3xl font-bold truncate bg-gradient-to-r from-foreground to-foreground/80 bg-clip-text text-transparent leading-tight">
                  {currentPackage?.name || `${currentLevel} Level`}
                </h2>
                <Badge
                  variant="secondary"
                  className={`bg-gradient-to-r ${levelGradients[currentLevel]} text-white border-0 font-bold text-xs px-2 py-0.5 xs:px-3 xs:py-1 shadow-lg animate-pulse w-fit`}
                >
                  {currentLevel}
                </Badge>
              </div>
            </div>
          </div>

          {/* Middle Row: Status Information - Responsive Stack */}
          <div className="flex flex-col xs:flex-row xs:items-center gap-2 xs:gap-3 sm:gap-4 text-xs xs:text-sm sm:text-base">
            <div className="flex items-center gap-2">
              <div className={`w-2 h-2 rounded-full animate-pulse ${
                userStatus?.isExpired ? 'bg-red-500' : 'bg-green-500'
              }`}></div>
              <span className={`font-semibold ${
                userStatus?.isExpired ? 'text-red-600 dark:text-red-400' : 'text-green-600 dark:text-green-400'
              }`}>
                {userStatus?.isExpired ? 'Expired' : 'Active'}
              </span>
            </div>
            
            <div className="flex items-center gap-2 text-muted-foreground">
              <div className="w-1 h-1 bg-muted-foreground rounded-full hidden xs:block"></div>
              <span className="font-medium">{currentPackage?.rankingLimit} stocks access</span>
            </div>
            
            {userStatus?.expirationDate && (
              <div className="flex items-center gap-2 text-muted-foreground">
                <div className="w-1 h-1 bg-muted-foreground rounded-full hidden xs:block"></div>
                <span className="text-xs xs:text-sm">
                  Expires {new Date(userStatus.expirationDate).toLocaleDateString()}
                </span>
              </div>
            )}
          </div>

          {/* Bottom Row: Action Button - Full Width on Mobile */}
          {userStatus?.isExpired && (
            <div className="pt-2">
              <Button
                onClick={() => router.push('/payment')}
                className={`bg-gradient-to-r ${levelGradients[currentLevel]} hover:opacity-90 text-white border-0 shadow-lg hover:shadow-xl transition-all transform hover:scale-105 w-full xs:w-auto animate-pulse h-9 xs:h-10 sm:h-11 text-sm xs:text-base flex items-center gap-2`}
              >
                <Crown className="h-4 w-4" />
                Renew Now
                <ArrowRight className="h-4 w-4" />
              </Button>
            </div>
          )}
        </div>
      </CardHeader>

      <CardContent className="relative space-y-4 sm:space-y-6">
        {/* Enhanced Current Level Benefits - Mobile Optimized */}
        <div className="space-y-3 sm:space-y-4">
          <div className="flex items-center gap-2">
            <div className={`p-1.5 sm:p-2 rounded-lg bg-gradient-to-r ${levelGradients[currentLevel]}/10`}>
              <Gem className="h-3 w-3 xs:h-4 xs:w-4 sm:h-5 sm:w-5 text-primary" />
            </div>
            <h4 className="font-bold text-sm xs:text-base sm:text-lg">Your Premium Benefits</h4>
          </div>
          
          {/* Responsive Grid - Single Column on Mobile */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 sm:gap-3">
            {LEVEL_BENEFITS[currentLevel].slice(0, 6).map((benefit, index) => (
              <div 
                key={index} 
                className="flex items-start gap-2 sm:gap-3 p-2.5 sm:p-3 lg:p-4 rounded-lg sm:rounded-xl bg-gradient-to-r from-primary/5 to-transparent hover:from-primary/10 transition-all duration-300 group"
              >
                <div
                  className={`w-2 h-2 sm:w-3 sm:h-3 rounded-full bg-gradient-to-r ${levelGradients[currentLevel]} animate-pulse mt-1 flex-shrink-0 group-hover:scale-125 transition-transform`}
                ></div>
                <span className="flex-1 text-xs xs:text-sm sm:text-base text-foreground/90 group-hover:text-foreground transition-colors leading-tight">
                  {benefit}
                </span>
              </div>
            ))}
          </div>
        </div>

        {/* Enhanced Progress to Next Level - Mobile Optimized */}
        {nextLevelOptions.length > 0 && userStatus?.paymentCount !== undefined && (
          <div className="space-y-3 sm:space-y-4 p-3 sm:p-4 lg:p-6 rounded-xl sm:rounded-2xl bg-gradient-to-r from-primary/5 via-background/50 to-secondary/5 border border-primary/10">
            <div className="flex flex-col xs:flex-row xs:items-center xs:justify-between gap-2">
              <div className="flex items-center gap-2">
                <Trophy className="h-3 w-3 xs:h-4 xs:w-4 sm:h-5 sm:w-5 text-primary flex-shrink-0" />
                <h4 className="font-bold text-xs xs:text-sm sm:text-base lg:text-lg">
                  Progress to {nextLevelOptions[0].name}
                </h4>
              </div>
              <Badge variant="outline" className="text-xs w-fit">
                {userStatus.paymentCount}/{nextLevelOptions[0].minPayments} payments
              </Badge>
            </div>
            
            <div className="space-y-2">
              <div className="flex justify-between text-xs sm:text-sm text-muted-foreground">
                <span>Progress</span>
                <span>{Math.round(getProgressToNextLevel())}%</span>
              </div>
              <div className="w-full bg-muted/30 rounded-full h-2.5 sm:h-3 lg:h-4 overflow-hidden shadow-inner">
                <div
                  className={`h-full bg-gradient-to-r ${levelGradients[nextLevelOptions[0].level]} transition-all duration-1000 ease-out rounded-full relative overflow-hidden`}
                  style={{ width: `${getProgressToNextLevel()}%` }}
                >
                  <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/30 to-transparent animate-shimmer"></div>
                  <div className="absolute inset-0 bg-white/10 animate-pulse"></div>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Enhanced Upgrade Options - Mobile First */}
        {nextLevelOptions.length > 0 && (
          <div className="p-3 sm:p-4 lg:p-6 rounded-xl sm:rounded-2xl bg-gradient-to-r from-primary/10 via-secondary/5 to-primary/10 border border-primary/20 backdrop-blur-sm">
            <div className="space-y-3 sm:space-y-4">
              <div className="space-y-2">
                <div className="flex items-center gap-2">
                  <ArrowRight className="h-3 w-3 xs:h-4 xs:w-4 sm:h-5 sm:w-5 text-primary animate-bounce-gentle flex-shrink-0" />
                  <h4 className="font-bold text-sm xs:text-base sm:text-lg">Upgrade Available</h4>
                </div>
                <p className="text-xs xs:text-sm sm:text-base text-muted-foreground leading-tight">
                  Unlock premium features with <span className="font-semibold text-primary">{nextLevelOptions[0].name}</span>
                </p>
                <div className="flex items-center gap-2 text-xs sm:text-sm text-muted-foreground">
                  <Star className="h-3 w-3 text-yellow-500 flex-shrink-0" />
                  <span>30-day money-back guarantee</span>
                </div>
              </div>
              
              <Button
                variant="outline"
                onClick={() => router.push('/payment')}
                className="border-primary/40 hover:bg-primary/10 hover:border-primary/60 transition-all transform hover:scale-105 w-full shadow-lg hover:shadow-xl h-9 xs:h-10 sm:h-11 text-sm xs:text-base flex items-center gap-2"
              >
                <Crown className="h-3 w-3 xs:h-4 xs:w-4" />
                Upgrade Now
                <ArrowRight className="h-3 w-3 xs:h-4 xs:w-4" />
              </Button>
            </div>
          </div>
        )}
      </CardContent>

      {/* Enhanced Special Effects for Premium Levels */}
      {(currentLevel === 'GOLD' || currentLevel === 'PLATINUM') && (
        <div className="absolute inset-0 pointer-events-none">
          <div className="absolute top-4 right-4 w-3 h-3 bg-yellow-400 rounded-full animate-ping shadow-lg"></div>
          <div className="absolute bottom-6 left-4 w-2 h-2 bg-yellow-400 rounded-full animate-pulse delay-300 shadow-md"></div>
          <div className="absolute top-1/2 right-8 w-2.5 h-2.5 bg-yellow-400 rounded-full animate-pulse delay-700 shadow-md"></div>
          <div className="absolute top-1/4 left-1/4 w-1.5 h-1.5 bg-yellow-400 rounded-full animate-bounce delay-1000 shadow-sm"></div>
          
          {/* Animated Border Glow */}
          <div className="absolute inset-0 rounded-xl bg-gradient-to-r from-yellow-400/20 via-transparent to-yellow-400/20 animate-gradient-x opacity-50"></div>
        </div>
      )}
    </Card>
  );
}
