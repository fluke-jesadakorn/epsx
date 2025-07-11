"use client";

import { useState, useEffect } from "react";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { useRouter } from "next/navigation";
import { 
  PACKAGES, 
  LEVEL_BENEFITS, 
  LEVEL_REQUIREMENTS,
  getPackageByLevel 
} from "@/app/constants/packages";
import type { UserLevelType } from "@/app/constants/packages";
import { useAuth } from "@/context/auth-context";
import { status } from "@/services/pay";
import { Crown, Star, Trophy, Gem, Zap, Lock, ArrowRight } from "lucide-react";

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
  BASIC: <Star className="h-6 w-6" />,
  SILVER: <Trophy className="h-6 w-6" />,
  GOLD: <Crown className="h-6 w-6" />,
  PLATINUM: <Gem className="h-6 w-6" />,
  API_PERSONAL: <Zap className="h-6 w-6" />,
  API_COMPANY: <Zap className="h-6 w-6" />,
  API_PARTNER: <Zap className="h-6 w-6" />,
};

const levelGradients = {
  BASIC: "from-gray-400 to-gray-600",
  SILVER: "from-slate-400 to-slate-600",
  GOLD: "from-yellow-400 to-orange-500",
  PLATINUM: "from-purple-500 to-pink-600",
  API_PERSONAL: "from-indigo-500 to-blue-600",
  API_COMPANY: "from-blue-600 to-cyan-600",
  API_PARTNER: "from-purple-600 to-indigo-700",
};

const levelBackgrounds = {
  BASIC: "bg-gradient-to-br from-gray-50 to-gray-100 dark:from-gray-900 dark:to-gray-800",
  SILVER: "bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-900 dark:to-slate-800",
  GOLD: "bg-gradient-to-br from-yellow-50 to-orange-50 dark:from-yellow-900/20 dark:to-orange-900/20",
  PLATINUM: "bg-gradient-to-br from-purple-50 to-pink-50 dark:from-purple-900/20 dark:to-pink-900/20",
  API_PERSONAL: "bg-gradient-to-br from-indigo-50 to-blue-50 dark:from-indigo-900/20 dark:to-blue-900/20",
  API_COMPANY: "bg-gradient-to-br from-blue-50 to-cyan-50 dark:from-blue-900/20 dark:to-cyan-900/20",
  API_PARTNER: "bg-gradient-to-br from-purple-50 to-indigo-50 dark:from-purple-900/20 dark:to-indigo-900/20",
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
          isExpired: paymentStatus.expire ? new Date() > paymentStatus.expire : !paymentStatus.paid,
          expirationDate: paymentStatus.expire,
        });
      } catch (error) {
        console.error("Failed to fetch user status:", error);
        setUserStatus({
          level: "BASIC",
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

  const currentLevel = userStatus?.level || "BASIC";
  const currentPackage = getPackageByLevel(currentLevel);
  const nextLevelOptions = PACKAGES.filter(pkg => 
    pkg.numericLevel > (currentPackage?.numericLevel || 0) && 
    !pkg.level.startsWith('API')
  ).slice(0, 2);

  const getProgressToNextLevel = () => {
    const current = LEVEL_REQUIREMENTS[currentLevel as keyof typeof LEVEL_REQUIREMENTS];
    const nextLevel = nextLevelOptions[0];
    
    if (!current || !nextLevel) return 100;
    
    const progress = Math.min(
      (userStatus?.paymentCount || 0) / nextLevel.minPayments * 100,
      100
    );
    return progress;
  };

  return (
    <Card className={`relative overflow-hidden transition-all duration-300 hover:shadow-xl ${levelBackgrounds[currentLevel]} ${className}`}>
      {/* Decorative Background Elements */}
      <div className="absolute top-0 right-0 w-32 h-32 opacity-10">
        <div className={`w-full h-full bg-gradient-to-br ${levelGradients[currentLevel]} rounded-full blur-3xl animate-pulse-slow`}></div>
      </div>
      <div className="absolute bottom-0 left-0 w-24 h-24 opacity-5">
        <div className={`w-full h-full bg-gradient-to-tr ${levelGradients[currentLevel]} rounded-full blur-2xl animate-float`}></div>
      </div>

      <CardHeader className="relative pb-2">
        <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
          <div className="flex items-center gap-3 sm:gap-4 flex-1 min-w-0">
            <div className={`relative p-2 sm:p-3 rounded-2xl bg-gradient-to-br ${levelGradients[currentLevel]} shadow-lg transform transition-transform hover:scale-105 flex-shrink-0`}>
              <div className="text-white">
                {levelIcons[currentLevel]}
              </div>
              {(currentLevel === 'GOLD' || currentLevel === 'PLATINUM') && (
                <div className="absolute -top-1 -right-1 w-3 h-3 sm:w-4 sm:h-4 bg-yellow-400 rounded-full animate-pulse">
                  <Star className="h-2 w-2 sm:h-3 sm:w-3 text-yellow-800 m-0.5" />
                </div>
              )}
            </div>
            <div className="min-w-0 flex-1">
              <div className="flex flex-col sm:flex-row sm:items-center gap-2">
                <h2 className="text-lg sm:text-xl font-bold truncate">
                  {currentPackage?.name || `${currentLevel} Level`}
                </h2>
                <Badge 
                  variant="secondary" 
                  className={`bg-gradient-to-r ${levelGradients[currentLevel]} text-white border-0 font-semibold animate-shimmer text-xs w-fit`}
                >
                  {currentLevel}
                </Badge>
              </div>
              <p className="text-xs sm:text-sm text-muted-foreground">
                <span className={userStatus?.isExpired ? "text-destructive" : "text-green-600"}>
                  {userStatus?.isExpired ? "Expired" : "Active"}
                </span> • 
                {currentPackage?.rankingLimit} stocks limit
              </p>
            </div>
          </div>
          
          {userStatus?.isExpired && (
            <Button
              onClick={() => router.push('/payment')}
              size="sm"
              className={`bg-gradient-to-r ${levelGradients[currentLevel]} hover:opacity-90 text-white border-0 shadow-lg hover:shadow-xl transition-all w-full sm:w-auto`}
            >
              Renew <ArrowRight className="h-4 w-4 ml-1" />
            </Button>
          )}
        </div>
      </CardHeader>

      <CardContent className="relative space-y-4">
        {/* Current Level Benefits */}
        <div>
          <h4 className="font-semibold mb-3 text-sm text-muted-foreground uppercase tracking-wider">
            Your Benefits
          </h4>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 sm:gap-3">
            {LEVEL_BENEFITS[currentLevel].slice(0, 4).map((benefit, index) => (
              <div key={index} className="flex items-start gap-2 text-sm">
                <div className={`w-2 h-2 rounded-full bg-gradient-to-r ${levelGradients[currentLevel]} animate-pulse mt-1 flex-shrink-0`}></div>
                <span className="flex-1">{benefit}</span>
              </div>
            ))}
          </div>
        </div>

        {/* Progress to Next Level */}
        {nextLevelOptions.length > 0 && userStatus?.paymentCount !== undefined && (
          <div>
            <div className="flex items-center justify-between mb-2">
              <h4 className="font-semibold text-sm text-muted-foreground uppercase tracking-wider">
                Progress to {nextLevelOptions[0].level}
              </h4>
              <span className="text-xs text-muted-foreground">
                {userStatus.paymentCount}/{nextLevelOptions[0].minPayments} payments
              </span>
            </div>
            <div className="w-full bg-muted/30 rounded-full h-2 overflow-hidden">
              <div 
                className={`h-full bg-gradient-to-r ${levelGradients[nextLevelOptions[0].level]} transition-all duration-1000 ease-out rounded-full relative`}
                style={{ width: `${getProgressToNextLevel()}%` }}
              >
                <div className="absolute inset-0 bg-white/20 animate-shimmer"></div>
              </div>
            </div>
          </div>
        )}

        {/* Upgrade Options */}
        {nextLevelOptions.length > 0 && (
          <div className="pt-4 border-t border-muted/30">
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
              <div className="flex-1">
                <h4 className="font-semibold text-sm text-muted-foreground uppercase tracking-wider">
                  Upgrade Available
                </h4>
                <p className="text-xs text-muted-foreground mt-1">
                  Unlock more features with {nextLevelOptions[0].name}
                </p>
              </div>
              <Button
                variant="outline"
                size="sm"
                onClick={() => router.push('/payment')}
                className="hover:bg-primary/10 border-primary/30 w-full sm:w-auto"
              >
                Upgrade
              </Button>
            </div>
          </div>
        )}
      </CardContent>

      {/* Special Effects for Premium Levels */}
      {(currentLevel === 'GOLD' || currentLevel === 'PLATINUM') && (
        <div className="absolute inset-0 pointer-events-none">
          <div className="absolute top-4 right-4 w-2 h-2 bg-yellow-400 rounded-full animate-ping"></div>
          <div className="absolute bottom-4 left-4 w-1 h-1 bg-yellow-400 rounded-full animate-pulse delay-300"></div>
          <div className="absolute top-1/2 right-8 w-1.5 h-1.5 bg-yellow-400 rounded-full animate-pulse delay-700"></div>
        </div>
      )}
    </Card>
  );
}
