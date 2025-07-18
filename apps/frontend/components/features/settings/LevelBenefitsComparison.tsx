'use client';

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { useRouter } from 'next/navigation';
import { PACKAGES } from '@/app/constants/packages';
import type { UserLevelType } from '@/app/constants/packages';
import { useState } from 'react';
import {
  Check,
  X,
  Star,
  Trophy,
  Crown,
  Gem,
  ArrowRight,
  Sparkles,
  Info,
  TrendingUp,
  Database,
  Zap,
  Users,
  ChevronDown,
  ChevronUp,
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


export function LevelBenefitsComparison({
  currentLevel,
  className,
}: LevelBenefitsComparisonProps) {
  const router = useRouter();
  const [activeTab, setActiveTab] = useState('overview');
  const [expandedCategories, setExpandedCategories] = useState<string[]>(['Trading Features']);

  // Filter to main subscription levels
  const mainLevels = PACKAGES.filter((pkg) =>
    ['BASIC', 'SILVER', 'GOLD', 'PLATINUM'].includes(pkg.level),
  );

  const getCurrentLevelNumeric = () => {
    const pkg = PACKAGES.find((p) => p.level === currentLevel);
    return pkg?.numericLevel || 0;
  };

  const currentLevelNumeric = getCurrentLevelNumeric();

  // Categorized benefits with descriptions and icons
  const benefitCategories = {
    'Analytics Features': {
      icon: <TrendingUp className="h-4 w-4" />,
      color: 'text-green-600',
      bgColor: 'bg-green-50 dark:bg-green-950/30',
      benefits: [
        {
          name: 'Basic Data Screening',
          description: 'Access to fundamental data screening tools and basic filters',
          levels: ['BASIC', 'SILVER', 'GOLD', 'PLATINUM']
        },
        {
          name: 'Advanced Analytics',
          description: 'In-depth data analysis with advanced charting and technical indicators',
          levels: ['SILVER', 'GOLD', 'PLATINUM']
        },
        {
          name: 'Real-time Data',
          description: 'Live data feeds and real-time updates',
          levels: ['GOLD', 'PLATINUM']
        },
        {
          name: 'AI-Powered Insights',
          description: 'Machine learning-based predictions and data sentiment analysis',
          levels: ['PLATINUM']
        }
      ]
    },
    'Data Access': {
      icon: <Database className="h-4 w-4" />,
      color: 'text-blue-600',
      bgColor: 'bg-blue-50 dark:bg-blue-950/30',
      benefits: [
        {
          name: 'Historical Data (1 Year)',
          description: 'Access to 12 months of historical entity data and trends',
          levels: ['BASIC', 'SILVER', 'GOLD', 'PLATINUM']
        },
        {
          name: 'Extended Historical Data (5 Years)',
          description: 'Access to 5 years of comprehensive historical data',
          levels: ['SILVER', 'GOLD', 'PLATINUM']
        },
        {
          name: 'Full Historical Data (10+ Years)',
          description: 'Complete historical dataset with decades of information',
          levels: ['GOLD', 'PLATINUM']
        },
        {
          name: 'Alternative Data Sources',
          description: 'Access to satellite data, social sentiment, and other alternative datasets',
          levels: ['PLATINUM']
        }
      ]
    },
    'Premium Tools': {
      icon: <Zap className="h-4 w-4" />,
      color: 'text-purple-600',
      bgColor: 'bg-purple-50 dark:bg-purple-950/30',
      benefits: [
        {
          name: 'Basic Performance Tracking',
          description: 'Track your analytics with basic performance management tools',
          levels: ['BASIC', 'SILVER', 'GOLD', 'PLATINUM']
        },
        {
          name: 'Advanced Performance Analytics',
          description: 'Detailed performance metrics, risk analysis, and optimization suggestions',
          levels: ['SILVER', 'GOLD', 'PLATINUM']
        },
        {
          name: 'Custom Alerts & Notifications',
          description: 'Set personalized value alerts and data event notifications',
          levels: ['GOLD', 'PLATINUM']
        },
        {
          name: 'Backtesting Platform',
          description: 'Test your analysis strategies against historical data',
          levels: ['PLATINUM']
        }
      ]
    },
    'Support & Community': {
      icon: <Users className="h-4 w-4" />,
      color: 'text-orange-600',
      bgColor: 'bg-orange-50 dark:bg-orange-950/30',
      benefits: [
        {
          name: 'Community Access',
          description: 'Join our analyst community forums and discussion groups',
          levels: ['BASIC', 'SILVER', 'GOLD', 'PLATINUM']
        },
        {
          name: 'Priority Support',
          description: 'Get faster response times and dedicated customer support',
          levels: ['SILVER', 'GOLD', 'PLATINUM']
        },
        {
          name: 'Expert Webinars',
          description: 'Access to monthly webinars with data experts and analysts',
          levels: ['GOLD', 'PLATINUM']
        },
        {
          name: 'Personal Account Manager',
          description: 'Dedicated account manager for personalized data guidance',
          levels: ['PLATINUM']
        }
      ]
    }
  };

  const getPackageBenefitStatus = (packageLevel: UserLevelType, benefit: any) => {
    return benefit.levels.includes(packageLevel);
  };

  const toggleCategory = (categoryName: string) => {
    setExpandedCategories(prev => 
      prev.includes(categoryName) 
        ? prev.filter(cat => cat !== categoryName)
        : [...prev, categoryName]
    );
  };

  const getRecommendedLevel = () => {
    if (currentLevelNumeric === 0) return 'SILVER';
    if (currentLevelNumeric === 1) return 'GOLD';
    if (currentLevelNumeric === 2) return 'PLATINUM';
    return null;
  };

  const recommendedLevel = getRecommendedLevel();

  return (
    <Card className={`relative overflow-hidden bg-gradient-to-br from-card/80 to-card/40 backdrop-blur-sm border-primary/20 ${className}`}>
      {/* Background Decorations */}
      <div className="absolute inset-0 pointer-events-none">
        <div className="absolute top-4 right-4 w-20 h-20 bg-primary/5 rounded-full blur-2xl animate-pulse-slow"></div>
        <div className="absolute bottom-4 left-4 w-16 h-16 bg-secondary/5 rounded-full blur-xl animate-float"></div>
        <div className="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 w-32 h-32 bg-primary/3 rounded-full blur-3xl"></div>
      </div>

      <CardHeader className="relative pb-4 border-b border-primary/10">
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="flex items-center gap-3 text-xl sm:text-2xl">
              <div className="p-2 bg-primary/10 rounded-lg">
                <Sparkles className="h-5 w-5 sm:h-6 sm:w-6 text-primary animate-pulse" />
              </div>
              Level Benefits Comparison
            </CardTitle>
            <p className="text-sm sm:text-base text-muted-foreground mt-2">
              Compare features across all subscription tiers
            </p>
          </div>
          <div className="hidden sm:flex items-center gap-2">
            <div className="w-2 h-2 bg-primary/40 rounded-full animate-ping"></div>
            <div className="w-1.5 h-1.5 bg-secondary/40 rounded-full animate-pulse"></div>
          </div>
        </div>
      </CardHeader>

      <CardContent className="relative p-4 sm:p-6">
        <div className="overflow-x-auto custom-scrollbar">
          <div className="w-full">
            {/* Enhanced Level Headers - Responsive Layout */}
            <div className="space-y-6 md:space-y-8">
              {/* Mobile: Stack plan cards vertically */}
              <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 md:hidden">
                {mainLevels.map((pkg, index) => {
                  const isCurrentLevel = pkg.level === currentLevel;
                  const isHigherLevel = pkg.numericLevel > currentLevelNumeric;

                  return (
                    <div
                      key={pkg.level}
                      className={`relative p-4 rounded-2xl border-2 transition-all duration-500 hover:scale-105 ${
                        isCurrentLevel 
                          ? `border-primary/50 bg-gradient-to-br ${levelGradients[pkg.level as MainLevelType]}/10` 
                          : 'border-muted/30 hover:border-primary/30'
                      } backdrop-blur-sm`}
                      style={{ animationDelay: `${index * 100}ms` }}
                    >
                      <div className="text-center space-y-3">
                        {/* Enhanced Icon */}
                        <div className="relative">
                          <div
                            className={`mx-auto flex justify-center text-white p-3 rounded-2xl bg-gradient-to-br ${levelGradients[pkg.level as MainLevelType]} shadow-lg transform transition-transform hover:scale-110`}
                          >
                            {levelIcons[pkg.level as MainLevelType]}
                          </div>
                          {(pkg.level === 'GOLD' || pkg.level === 'PLATINUM') && (
                            <div className="absolute -top-1 -right-1 w-4 h-4 bg-yellow-400 rounded-full animate-bounce">
                              <Star className="h-2.5 w-2.5 text-yellow-800 m-0.5" />
                            </div>
                          )}
                        </div>

                        {/* Package Info */}
                        <div className="space-y-2">
                          <h3 className="font-bold text-base text-foreground">
                            {pkg.name}
                          </h3>
                          <div className="flex items-center justify-center gap-1">
                            <span className="text-xl font-bold text-foreground">
                              ${pkg.price}
                            </span>
                            <span className="text-sm text-muted-foreground">
                              /mo
                            </span>
                          </div>
                          <p className="text-sm text-muted-foreground">
                            {pkg.rankingLimit} stocks access
                          </p>
                        </div>

                        {/* Status Badge */}
                        {isCurrentLevel && (
                          <Badge className={`text-xs bg-gradient-to-r ${levelGradients[pkg.level as MainLevelType]} text-white border-0 animate-pulse`}>
                            Current Plan
                          </Badge>
                        )}

                        {/* Upgrade Button */}
                        {isHigherLevel && (
                          <Button
                            onClick={() => router.push(`/payment?package=${pkg.id}`)}
                            size="sm"
                            className={`w-full text-xs bg-gradient-to-r ${levelGradients[pkg.level as MainLevelType]} hover:opacity-90 text-white border-0 shadow-lg hover:shadow-xl transition-all transform hover:scale-105 flex items-center gap-1`}
                          >
                            Upgrade Now
                            <ArrowRight className="h-3 w-3" />
                          </Button>
                        )}
                      </div>
                    </div>
                  );
                })}
              </div>
              
              {/* Desktop: Horizontal layout with features label */}
              <div className="hidden md:grid md:grid-cols-5 gap-4 items-end">
                <div className="text-sm font-semibold text-muted-foreground min-w-[220px] flex items-center gap-2">
                  <div className="w-3 h-3 bg-primary/20 rounded-full"></div>
                  Features
                </div>
                {mainLevels.map((pkg, index) => {
                  const isCurrentLevel = pkg.level === currentLevel;
                  const isHigherLevel = pkg.numericLevel > currentLevelNumeric;

                  return (
                    <div
                      key={pkg.level}
                      className={`relative p-3 sm:p-4 rounded-2xl border-2 transition-all duration-500 hover:scale-105 ${
                        isCurrentLevel 
                          ? `border-primary/50 bg-gradient-to-br ${levelGradients[pkg.level as MainLevelType]}/10` 
                          : 'border-muted/30 hover:border-primary/30'
                      } backdrop-blur-sm`}
                      style={{ animationDelay: `${index * 100}ms` }}
                    >
                      <div className="text-center space-y-3">
                        {/* Enhanced Icon */}
                        <div className="relative">
                          <div
                            className={`mx-auto flex justify-center text-white p-3 sm:p-4 rounded-2xl bg-gradient-to-br ${levelGradients[pkg.level as MainLevelType]} shadow-lg transform transition-transform hover:scale-110`}
                          >
                            {levelIcons[pkg.level as MainLevelType]}
                          </div>
                          {(pkg.level === 'GOLD' || pkg.level === 'PLATINUM') && (
                            <div className="absolute -top-1 -right-1 w-4 h-4 bg-yellow-400 rounded-full animate-bounce">
                              <Star className="h-2.5 w-2.5 text-yellow-800 m-0.5" />
                            </div>
                          )}
                        </div>

                        {/* Package Info */}
                        <div className="space-y-2">
                          <h3 className="font-bold text-sm sm:text-base text-foreground">
                            {pkg.name}
                          </h3>
                          <div className="flex items-center justify-center gap-1">
                            <span className="text-lg sm:text-2xl font-bold text-foreground">
                              ${pkg.price}
                            </span>
                            <span className="text-xs sm:text-sm text-muted-foreground">
                              /mo
                            </span>
                          </div>
                          <p className="text-xs sm:text-sm text-muted-foreground px-2">
                            {pkg.rankingLimit} stocks access
                          </p>
                        </div>

                        {/* Status Badge */}
                        {isCurrentLevel && (
                          <Badge className={`text-xs bg-gradient-to-r ${levelGradients[pkg.level as MainLevelType]} text-white border-0 animate-pulse`}>
                            Current Plan
                          </Badge>
                        )}

                        {/* Upgrade Button */}
                        {isHigherLevel && (
                          <Button
                            onClick={() => router.push(`/payment?package=${pkg.id}`)}
                            size="sm"
                            className={`w-full text-xs bg-gradient-to-r ${levelGradients[pkg.level as MainLevelType]} hover:opacity-90 text-white border-0 shadow-lg hover:shadow-xl transition-all transform hover:scale-105 flex items-center gap-1`}
                          >
                            <span className="hidden sm:inline">Upgrade Now</span>
                            <span className="sm:hidden">Upgrade</span>
                            <ArrowRight className="h-3 w-3" />
                          </Button>
                        )}
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>

            {/* Enhanced Tab Buttons */}
            <div className="mb-6">
              <div className="flex items-center bg-muted p-1 rounded-md w-fit mx-auto">
                <button
                  onClick={() => setActiveTab('overview')}
                  className={`flex items-center gap-2 px-3 py-1.5 text-sm font-medium rounded-sm transition-all ${
                    activeTab === 'overview'
                      ? 'bg-background text-foreground shadow-sm'
                      : 'text-muted-foreground hover:text-foreground'
                  }`}
                >
                  <Sparkles className="h-4 w-4" />
                  <span className="hidden sm:inline">Quick Overview</span>
                  <span className="sm:hidden">Overview</span>
                </button>
                <button
                  onClick={() => setActiveTab('detailed')}
                  className={`flex items-center gap-2 px-3 py-1.5 text-sm font-medium rounded-sm transition-all ${
                    activeTab === 'detailed'
                      ? 'bg-background text-foreground shadow-sm'
                      : 'text-muted-foreground hover:text-foreground'
                  }`}
                >
                  <Info className="h-4 w-4" />
                  <span className="hidden sm:inline">Detailed Features</span>
                  <span className="sm:hidden">Detailed</span>
                </button>
              </div>
            </div>

            {/* Tab Content */}
            {activeTab === 'overview' && (
              <div className="space-y-4">
                {/* Quick comparison grid */}
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                  {mainLevels.map((pkg) => {
                    const isCurrentLevel = pkg.level === currentLevel;
                    const isRecommended = pkg.level === recommendedLevel;
                    
                    return (
                      <div
                        key={pkg.level}
                        className={`relative p-4 rounded-xl border-2 transition-all duration-300 hover:scale-105 ${
                          isCurrentLevel 
                            ? `border-primary/50 bg-gradient-to-br ${levelGradients[pkg.level as MainLevelType]}/10`
                            : isRecommended
                            ? 'border-primary/30 bg-primary/5'
                            : 'border-muted/30 hover:border-primary/20'
                        }`}
                      >
                        {isRecommended && !isCurrentLevel && (
                          <div className="absolute -top-2 left-1/2 transform -translate-x-1/2">
                            <Badge className="bg-primary text-white text-xs px-2 py-1">
                              Recommended
                            </Badge>
                          </div>
                        )}
                        
                        <div className="text-center space-y-3">
                          <div className={`mx-auto w-12 h-12 rounded-full bg-gradient-to-br ${levelGradients[pkg.level as MainLevelType]} flex items-center justify-center text-white`}>
                            {levelIcons[pkg.level as MainLevelType]}
                          </div>
                          
                          <div>
                            <h4 className="font-bold text-sm">{pkg.name}</h4>
                            <div className="flex items-center justify-center gap-1 mt-1">
                              <span className="text-lg font-bold">${pkg.price}</span>
                              <span className="text-xs text-muted-foreground">/mo</span>
                            </div>
                          </div>
                          
                          <div className="space-y-2 text-xs">
                            <div className="flex items-center justify-center gap-1">
                              <Database className="h-3 w-3 text-muted-foreground" />
                              <span>{pkg.rankingLimit} stocks</span>
                            </div>
                            
                            {/* Key features preview */}
                            <div className="space-y-1">
                              {Object.entries(benefitCategories).slice(0, 2).map(([categoryName, category]) => {
                                const availableFeatures = category.benefits.filter(benefit => 
                                  getPackageBenefitStatus(pkg.level, benefit)
                                ).length;
                                const totalFeatures = category.benefits.length;
                                
                                return (
                                  <div key={categoryName} className="flex items-center justify-between">
                                    <span className="text-muted-foreground">{categoryName}</span>
                                    <span className="font-medium">{availableFeatures}/{totalFeatures}</span>
                                  </div>
                                );
                              })}
                            </div>
                          </div>
                          
                          {isCurrentLevel ? (
                            <Badge className={`w-full text-xs bg-gradient-to-r ${levelGradients[pkg.level as MainLevelType]} text-white border-0`}>
                              Current Plan
                            </Badge>
                          ) : pkg.numericLevel > currentLevelNumeric ? (
                            <Button
                              onClick={() => router.push(`/payment?package=${pkg.id}`)}
                              size="sm"
                              className={`w-full text-xs bg-gradient-to-r ${levelGradients[pkg.level as MainLevelType]} hover:opacity-90 text-white border-0`}
                            >
                              Upgrade
                            </Button>
                          ) : (
                            <div className="text-xs text-muted-foreground">Lower tier</div>
                          )}
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            )}

            {activeTab === 'detailed' && (
              <TooltipProvider>
                <div className="space-y-6">
                  {/* Categorized benefits */}
                  {Object.entries(benefitCategories).map(([categoryName, category]) => (
                    <div key={categoryName} className="space-y-4">
                      <button
                        onClick={() => toggleCategory(categoryName)}
                        className={`w-full p-4 rounded-lg ${category.bgColor} hover:bg-opacity-80 transition-colors`}
                      >
                        <div className="flex items-center justify-between w-full">
                          <div className="flex items-center gap-3">
                            <div className={`p-2 rounded-lg bg-white/50 dark:bg-gray-800/50 ${category.color}`}>
                              {category.icon}
                            </div>
                            <h3 className="font-semibold text-lg">{categoryName}</h3>
                            <Badge variant="outline" className="text-xs">
                              {category.benefits.length} features
                            </Badge>
                          </div>
                          {expandedCategories.includes(categoryName) ? (
                            <ChevronUp className="h-5 w-5" />
                          ) : (
                            <ChevronDown className="h-5 w-5" />
                          )}
                        </div>
                      </button>
                      
                      {expandedCategories.includes(categoryName) && (
                        <div className="pt-4 space-y-3">
                          {category.benefits.map((benefit, benefitIndex) => (
                            <div
                              key={benefitIndex}
                              className="p-3 rounded-lg hover:bg-primary/5 transition-all duration-300 group border border-muted/20"
                            >
                              {/* Mobile Layout */}
                              <div className="block md:hidden space-y-3">
                                <div className="flex items-start gap-2">
                                  <Tooltip>
                                    <TooltipTrigger asChild>
                                      <div className="flex items-center gap-2 cursor-help">
                                        <span className="font-medium text-sm">{benefit.name}</span>
                                        <Info className="h-3 w-3 text-muted-foreground" />
                                      </div>
                                    </TooltipTrigger>
                                    <TooltipContent side="top" className="max-w-xs">
                                      <p className="text-sm">{benefit.description}</p>
                                    </TooltipContent>
                                  </Tooltip>
                                </div>
                                <p className="text-xs text-muted-foreground">
                                  {benefit.description}
                                </p>
                                
                                {/* Mobile plan availability grid */}
                                <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
                                  {mainLevels.map((pkg) => {
                                    const hasFeature = getPackageBenefitStatus(pkg.level, benefit);
                                    
                                    return (
                                      <div key={pkg.level} className="text-center space-y-1">
                                        <div className="text-xs font-medium text-muted-foreground">{pkg.name}</div>
                                        <div className="flex justify-center">
                                          {hasFeature ? (
                                            <div
                                              className={`flex items-center justify-center w-8 h-8 rounded-full bg-gradient-to-br ${levelGradients[pkg.level as MainLevelType]} shadow-lg`}
                                            >
                                              <Check className="h-3 w-3 text-white" />
                                            </div>
                                          ) : (
                                            <div className="flex items-center justify-center w-8 h-8 rounded-full bg-gray-200 dark:bg-gray-700">
                                              <X className="h-3 w-3 text-gray-400" />
                                            </div>
                                          )}
                                        </div>
                                      </div>
                                    );
                                  })}
                                </div>
                              </div>
                              
                              {/* Desktop Layout */}
                              <div className="hidden md:grid md:grid-cols-5 gap-3 sm:gap-4 items-center">
                                <div className="col-span-1 space-y-1">
                                  <div className="flex items-start gap-2">
                                    <Tooltip>
                                      <TooltipTrigger asChild>
                                        <div className="flex items-center gap-2 cursor-help">
                                          <span className="font-medium text-sm">{benefit.name}</span>
                                          <Info className="h-3 w-3 text-muted-foreground" />
                                        </div>
                                      </TooltipTrigger>
                                      <TooltipContent side="top" className="max-w-xs">
                                        <p className="text-sm">{benefit.description}</p>
                                      </TooltipContent>
                                    </Tooltip>
                                  </div>
                                  <p className="text-xs text-muted-foreground line-clamp-2">
                                    {benefit.description}
                                  </p>
                                </div>
                                
                                {mainLevels.map((pkg) => {
                                  const hasFeature = getPackageBenefitStatus(pkg.level, benefit);

                                  return (
                                    <div
                                      key={pkg.level}
                                      className="flex items-center justify-center"
                                    >
                                      {hasFeature ? (
                                        <Tooltip>
                                          <TooltipTrigger asChild>
                                            <div
                                              className={`relative flex items-center justify-center w-10 h-10 sm:w-12 sm:h-12 rounded-full bg-gradient-to-br ${levelGradients[pkg.level as MainLevelType]} shadow-lg transform transition-all hover:scale-110 group-hover:shadow-xl cursor-pointer`}
                                            >
                                              <Check className="h-4 w-4 sm:h-5 sm:w-5 text-white" />
                                              <div className="absolute inset-0 rounded-full bg-white/20 animate-ping opacity-0 group-hover:opacity-100"></div>
                                            </div>
                                          </TooltipTrigger>
                                          <TooltipContent>
                                            <p>Available in {pkg.name}</p>
                                          </TooltipContent>
                                        </Tooltip>
                                      ) : (
                                        <Tooltip>
                                          <TooltipTrigger asChild>
                                            <div className="flex items-center justify-center w-10 h-10 sm:w-12 sm:h-12 rounded-full bg-gray-200 dark:bg-gray-700 transition-all group-hover:bg-gray-300 dark:group-hover:bg-gray-600 cursor-pointer">
                                              <X className="h-4 w-4 sm:h-5 sm:w-5 text-gray-400" />
                                            </div>
                                          </TooltipTrigger>
                                          <TooltipContent>
                                            <p>Not available in {pkg.name}</p>
                                          </TooltipContent>
                                        </Tooltip>
                                      )}
                                    </div>
                                  );
                                })}
                              </div>
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              </TooltipProvider>
            )}

            {/* Enhanced Call to Action */}
            {currentLevelNumeric < 3 && (
              <div className="mt-8 sm:mt-12 relative">
                <div className="absolute inset-0 bg-gradient-to-r from-primary/10 via-secondary/10 to-primary/10 rounded-2xl blur-xl"></div>
                <div className="relative p-6 sm:p-8 rounded-2xl bg-gradient-to-r from-primary/5 via-background/80 to-secondary/5 border border-primary/20 backdrop-blur-sm">
                  <div className="text-center space-y-4 sm:space-y-6">
                    <div className="space-y-2">
                      <h4 className="text-xl sm:text-2xl font-bold text-foreground flex items-center justify-center gap-2">
                        <Crown className="h-6 w-6 text-primary animate-bounce" />
                        Ready to Level Up?
                      </h4>
                      <p className="text-sm sm:text-base text-muted-foreground max-w-md mx-auto">
                        Unlock premium features, higher limits, and exclusive benefits with our advanced plans
                      </p>
                    </div>
                    
                    <div className="flex flex-col sm:flex-row gap-3 sm:gap-4 justify-center items-center">
                      <Button
                        onClick={() => router.push('/payment')}
                        size="lg"
                        className="bg-gradient-to-r from-primary to-secondary hover:from-primary/90 hover:to-secondary/90 text-white border-0 shadow-xl hover:shadow-2xl transition-all transform hover:scale-105 w-full sm:w-auto flex items-center gap-2"
                      >
                        <Crown className="h-5 w-5" />
                        Explore Upgrade Options
                        <ArrowRight className="h-5 w-5" />
                      </Button>
                      
                      <div className="flex items-center gap-2 text-xs sm:text-sm text-muted-foreground">
                        <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
                        <span>30-day money-back guarantee</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
