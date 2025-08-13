import React from 'react';
import { Card, CardContent } from '@epsx/ui';

interface SimpleCardSkeletonProps {
  index: number;
}

/**
 * Simple loading skeleton for analytics cards
 */
export function SimpleCardSkeleton({ index }: SimpleCardSkeletonProps): React.JSX.Element {
  return (
    <Card className="w-full transition-all duration-200 hover:shadow-2xl border-0 bg-gradient-to-br from-blue-50/50 via-purple-50/50 to-pink-50/50 dark:from-[#232946]/50 dark:via-[#1a1a2e]/50 dark:to-[#0f1021]/50 rounded-3xl shadow-lg relative animate-pulse">
      {/* Rank Badge */}
      <div className="absolute top-4 left-4 z-30 w-12 h-12 flex items-center justify-center">
        <div className="w-10 h-10 bg-gradient-to-br from-orange-200/70 via-yellow-100/60 to-amber-100/50 dark:from-orange-900/40 dark:via-yellow-900/30 dark:to-amber-900/30 rounded-full flex items-center justify-center">
          <span className="text-sm font-bold text-slate-600 dark:text-slate-400">
            #{index + 1}
          </span>
        </div>
      </div>

      {/* Floating decorative elements */}
      <div className="absolute top-2 right-2 text-2xl opacity-20">
        🥞
      </div>
      <div className="absolute bottom-2 left-2 text-lg opacity-15">
        💰
      </div>

      <CardContent className="p-6 pt-16">
        {/* Loading animation */}
        <div className="space-y-6">
          <div className="flex items-center justify-center py-8">
            <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-orange-500"></div>
          </div>
          
          <div className="text-center space-y-2">
            <div className="h-6 bg-gradient-to-r from-orange-300/20 to-yellow-300/20 rounded-lg animate-pulse"></div>
            <div className="h-4 bg-gradient-to-r from-blue-300/20 to-purple-300/20 rounded-lg animate-pulse"></div>
          </div>
          
          <div className="grid grid-cols-3 gap-3">
            {[...Array(3)].map((_, i) => (
              <div key={i} className="h-16 bg-gradient-to-br from-slate-100/50 to-slate-200/50 dark:from-slate-700/30 dark:to-slate-800/30 rounded-xl animate-pulse"></div>
            ))}
          </div>
          
          <div className="h-12 bg-gradient-to-r from-orange-200/30 to-yellow-200/30 rounded-xl animate-pulse"></div>
          
          <div className="space-y-2">
            {[...Array(4)].map((_, i) => (
              <div key={i} className="h-8 bg-gradient-to-r from-slate-100/30 to-slate-200/30 dark:from-slate-700/20 dark:to-slate-800/20 rounded-lg animate-pulse"></div>
            ))}
          </div>
        </div>
        
        <div className="text-center mt-4">
          <p className="text-sm text-slate-500 dark:text-slate-400">
            Loading sweet data... 🍯
          </p>
        </div>
      </CardContent>
    </Card>
  );
}

export default SimpleCardSkeleton;
