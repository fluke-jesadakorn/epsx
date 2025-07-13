'use client';

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { useRouter } from 'next/navigation';
import { PACKAGES, LEVEL_BENEFITS } from '@/app/constants/packages';
import type { UserLevelType } from '@/app/constants/packages';
import {
  Check,
  X,
  Star,
  Trophy,
  Crown,
  Gem,
  ArrowRight,
  Sparkles,
} from 'lucide-react';

interface LevelBenefitsComparisonProps {
  currentLevel: UserLevelType;
  className?: string;
}

type MainLevelType = 'BASIC' | 'SILVER' | 'GOLD' | 'PLATINUM';

const levelIcons: Record<MainLevelType, React.ReactNode> = {
  BASIC: <Star className="h-5 w-5" />,
  SILVER: <Trophy className="h-5 w-5" />,
  GOLD: <Crown className="h-5 w-5" />,
  PLATINUM: <Gem className="h-5 w-5" />,
};

const levelGradients: Record<MainLevelType, string> = {
  BASIC: 'from-gray-400 to-gray-600',
  SILVER: 'from-slate-400 to-slate-600',
  GOLD: 'from-yellow-400 to-orange-500',
  PLATINUM: 'from-purple-500 to-pink-600',
};

const levelBorders: Record<MainLevelType, string> = {
  BASIC: 'border-gray-200 dark:border-gray-700',
  SILVER: 'border-slate-200 dark:border-slate-700',
  GOLD: 'border-yellow-200 dark:border-yellow-800',
  PLATINUM: 'border-purple-200 dark:border-purple-800',
};

export function LevelBenefitsComparison({
  currentLevel,
  className,
}: LevelBenefitsComparisonProps) {
  const router = useRouter();

  // Filter to main subscription levels
  const mainLevels = PACKAGES.filter((pkg) =>
    ['BASIC', 'SILVER', 'GOLD', 'PLATINUM'].includes(pkg.level),
  );

  const getCurrentLevelNumeric = () => {
    const pkg = PACKAGES.find((p) => p.level === currentLevel);
    return pkg?.numericLevel || 0;
  };

  const currentLevelNumeric = getCurrentLevelNumeric();

  // Get all unique benefits across all levels
  const allBenefits = Array.from(
    new Set(mainLevels.flatMap((pkg) => LEVEL_BENEFITS[pkg.level])),
  );

  const getPackageBenefitStatus = (
    packageLevel: UserLevelType,
    benefit: string,
  ) => {
    return LEVEL_BENEFITS[packageLevel].includes(benefit);
  };

  return (
    <Card className={`relative overflow-hidden ${className}`}>
      <CardHeader className="pb-4">
        <CardTitle className="flex items-center gap-2 text-lg">
          <Sparkles className="h-5 w-5 text-primary" />
          Level Benefits Comparison
        </CardTitle>
        <p className="text-sm text-muted-foreground">
          See what features are available at each level
        </p>
      </CardHeader>

      <CardContent>
        <div className="overflow-x-auto">
          <div className="min-w-[800px] sm:min-w-full">
            {/* Level Headers */}
            <div className="grid grid-cols-5 gap-2 sm:gap-3 mb-6">
              <div className="text-xs sm:text-sm font-medium text-muted-foreground">
                Features
              </div>
              {mainLevels.map((pkg) => {
                const isCurrentLevel = pkg.level === currentLevel;
                const isHigherLevel = pkg.numericLevel > currentLevelNumeric;

                return (
                  <div
                    key={pkg.level}
                    className={`relative p-2 sm:p-3 rounded-xl border-2 transition-all duration-300 ${
                      isCurrentLevel
                        ? `${levelBorders[pkg.level as MainLevelType]} bg-gradient-to-br ${levelGradients[pkg.level as MainLevelType]}/10 ring-2 ring-primary/20`
                        : levelBorders[pkg.level as MainLevelType]
                    }`}
                  >
                    <div className="text-center space-y-1 sm:space-y-2">
                      <div
                        className={`mx-auto flex justify-center text-white p-1.5 sm:p-2 rounded-lg bg-gradient-to-br ${levelGradients[pkg.level as MainLevelType]}`}
                      >
                        {levelIcons[pkg.level as MainLevelType]}
                      </div>
                      <div>
                        <h3 className="font-semibold text-xs sm:text-sm">
                          {pkg.name}
                        </h3>
                        <div className="flex items-center justify-center gap-1 mt-1">
                          <span className="text-sm sm:text-lg font-bold">
                            ${pkg.price}
                          </span>
                          <span className="text-xs text-muted-foreground">
                            /mo
                          </span>
                        </div>
                        <p className="text-xs text-muted-foreground mt-1">
                          {pkg.rankingLimit} stocks
                        </p>
                      </div>

                      {isCurrentLevel && (
                        <Badge className="text-xs bg-primary text-primary-foreground">
                          Current
                        </Badge>
                      )}

                      {isHigherLevel && (
                        <Button
                          size="sm"
                          onClick={() => router.push('/payment')}
                          className={`w-full text-xs bg-gradient-to-r ${levelGradients[pkg.level as MainLevelType]} hover:opacity-90 text-white border-0`}
                        >
                          <span className="hidden sm:inline">Upgrade</span>
                          <span className="sm:hidden">Up</span>
                          <ArrowRight className="h-3 w-3 ml-1" />
                        </Button>
                      )}
                    </div>

                    {/* Special effects for premium levels */}
                    {(pkg.level === 'GOLD' || pkg.level === 'PLATINUM') && (
                      <div className="absolute top-2 right-2 w-2 h-2 bg-yellow-400 rounded-full animate-pulse"></div>
                    )}
                  </div>
                );
              })}
            </div>

            {/* Benefits Grid */}
            <div className="space-y-1 sm:space-y-2">
              {allBenefits.map((benefit, index) => (
                <div
                  key={index}
                  className="grid grid-cols-5 gap-2 sm:gap-3 py-2 sm:py-3 px-1 sm:px-2 rounded-lg hover:bg-muted/30 transition-colors"
                >
                  <div className="text-xs sm:text-sm font-medium text-left">
                    {benefit}
                  </div>
                  {mainLevels.map((pkg) => {
                    const hasFeature = getPackageBenefitStatus(
                      pkg.level,
                      benefit,
                    );

                    return (
                      <div key={pkg.level} className="flex justify-center">
                        {hasFeature ? (
                          <div
                            className={`p-1 rounded-full bg-gradient-to-br ${levelGradients[pkg.level as MainLevelType]}`}
                          >
                            <Check className="h-3 w-3 sm:h-4 sm:w-4 text-white" />
                          </div>
                        ) : (
                          <div className="p-1 rounded-full bg-gray-200 dark:bg-gray-700">
                            <X className="h-3 w-3 sm:h-4 sm:w-4 text-gray-400" />
                          </div>
                        )}
                      </div>
                    );
                  })}
                </div>
              ))}
            </div>

            {/* Additional Features by Level */}
            <div className="mt-6 sm:mt-8 pt-4 sm:pt-6 border-t border-muted/30">
              <h4 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider mb-4">
                Level-Specific Limits
              </h4>
              <div className="grid grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-4">
                {mainLevels.map((pkg) => {
                  const isCurrentLevel = pkg.level === currentLevel;

                  return (
                    <div
                      key={pkg.level}
                      className={`p-3 sm:p-4 rounded-xl border transition-all duration-300 ${
                        isCurrentLevel
                          ? `${levelBorders[pkg.level as MainLevelType]} bg-gradient-to-br ${levelGradients[pkg.level as MainLevelType]}/10 ring-2 ring-primary/20`
                          : `${levelBorders[pkg.level as MainLevelType]} hover:shadow-md`
                      }`}
                    >
                      <div className="text-center space-y-2">
                        <div
                          className={`mx-auto flex justify-center text-white p-1.5 sm:p-2 rounded-lg bg-gradient-to-br ${levelGradients[pkg.level as MainLevelType]} w-fit`}
                        >
                          {levelIcons[pkg.level as MainLevelType]}
                        </div>
                        <div>
                          <h5 className="font-semibold text-xs sm:text-sm">
                            {pkg.level}
                          </h5>
                          <p className="text-xl sm:text-2xl font-bold text-primary">
                            {pkg.rankingLimit}
                          </p>
                          <p className="text-xs text-muted-foreground">
                            stocks visible
                          </p>
                        </div>

                        {pkg.minPayments > 0 && (
                          <div className="mt-2 pt-2 border-t border-muted/30">
                            <p className="text-xs text-muted-foreground">
                              Requires {pkg.minPayments}+ payments
                            </p>
                          </div>
                        )}
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>

            {/* Call to Action */}
            {currentLevelNumeric < 3 && (
              <div className="mt-6 sm:mt-8 p-4 sm:p-6 rounded-xl bg-gradient-to-r from-primary/10 to-secondary/10 border border-primary/20">
                <div className="text-center space-y-3 sm:space-y-4">
                  <div>
                    <h4 className="text-base sm:text-lg font-semibold">
                      Ready to Upgrade?
                    </h4>
                    <p className="text-xs sm:text-sm text-muted-foreground mt-1">
                      Unlock more features and higher limits with a premium plan
                    </p>
                  </div>
                  <Button
                    onClick={() => router.push('/payment')}
                    size="lg"
                    className="bg-gradient-to-r from-primary to-secondary hover:opacity-90 text-white border-0 shadow-lg hover:shadow-xl transition-all w-full sm:w-auto"
                  >
                    View Upgrade Options <ArrowRight className="h-4 w-4 ml-2" />
                  </Button>
                </div>
              </div>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
