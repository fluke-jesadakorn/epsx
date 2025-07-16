import React from 'react';
import {
  GRADIENTS,
  SPACING,
  TYPOGRAPHY,
  ANIMATIONS,
} from '../constants/styles';

/**
 * Loading state component for financial data
 */
export function FinancialDataLoading(): React.JSX.Element {
  return (
    <div
      className={`
      w-full min-h-screen flex items-center justify-center 
      bg-gradient-to-br ${GRADIENTS.background}
      relative overflow-hidden
    `}
    >
      {/* PancakeSwap-style floating elements */}
      <div className="absolute inset-0 pointer-events-none">
        <div className="absolute top-20 left-4 sm:left-10 w-12 sm:w-16 h-12 sm:h-16 bg-gradient-to-br from-orange-400/20 to-yellow-400/20 rounded-full animate-float" />
        <div className="absolute top-32 right-8 sm:right-20 w-8 sm:w-12 h-8 sm:h-12 bg-gradient-to-br from-amber-400/20 to-orange-400/20 rounded-full animate-bounce-gentle" />
        <div className="absolute bottom-40 left-8 sm:left-20 w-6 sm:w-8 h-6 sm:h-8 bg-gradient-to-br from-yellow-400/20 to-amber-400/20 rounded-full animate-pulse" />

        {/* Floating emojis */}
        <div className="absolute top-1/4 left-1/4 text-3xl sm:text-4xl opacity-10 animate-spin-slow">
          🥞
        </div>
        <div className="absolute bottom-1/4 right-1/4 text-2xl sm:text-3xl opacity-15 animate-bounce-gentle">
          💰
        </div>
      </div>

      <div className="text-center space-y-4 relative z-10 px-4 sm:px-0">
        <div className="w-12 sm:w-16 h-12 sm:h-16 mx-auto rounded-full bg-gradient-to-br from-orange-500 to-yellow-600 flex items-center justify-center animate-pulse">
          <span className="text-white text-xl sm:text-2xl">🥞</span>
        </div>
        <h3
          className={`${TYPOGRAPHY.subtitle} font-semibold text-slate-600 dark:text-slate-400 flex items-center justify-center gap-2`}
        >
          <span className="animate-bounce">📊</span>
          Loading Sweet Data...
          <span className="animate-pulse">✨</span>
        </h3>
        <p className="text-sm sm:text-base text-slate-500 dark:text-slate-500">
          🚀 Please wait while we fetch the most delicious financial rankings 🍯
        </p>
      </div>
    </div>
  );
}

/**
 * Header section component
 */
export function FinancialDataHeader(): React.JSX.Element {
  return (
    <div className="relative overflow-hidden">
      <div className="absolute inset-0 bg-gradient-to-r from-orange-600/5 via-yellow-600/5 to-amber-600/5 dark:from-orange-500/10 dark:via-yellow-500/10 dark:to-amber-500/10" />

      {/* Floating background elements with PancakeSwap colors */}
      <div className="absolute top-6 sm:top-10 left-4 sm:left-10 w-16 sm:w-20 h-16 sm:h-20 bg-orange-400/10 rounded-full blur-xl animate-pulse"></div>
      <div
        className="absolute top-12 sm:top-20 right-8 sm:right-20 w-24 sm:w-32 h-24 sm:h-32 bg-yellow-400/10 rounded-full blur-xl animate-pulse"
        style={{ animationDelay: '1s' }}
      ></div>
      <div
        className="absolute bottom-6 sm:bottom-10 left-1/4 sm:left-1/3 w-20 sm:w-24 h-20 sm:h-24 bg-amber-400/10 rounded-full blur-xl animate-pulse"
        style={{ animationDelay: '2s' }}
      ></div>

      {/* Floating emojis */}
      <div className="absolute top-4 sm:top-8 right-4 sm:right-8 text-2xl sm:text-3xl opacity-20 animate-bounce-gentle">
        🥞
      </div>
      <div className="absolute top-8 sm:top-16 left-8 sm:left-16 text-xl sm:text-2xl opacity-15 animate-float">
        💰
      </div>
      <div className="absolute bottom-4 sm:bottom-8 right-1/4 text-xl sm:text-2xl opacity-10 animate-pulse">
        🚀
      </div>

      <div
        className={`relative ${SPACING.mobileContainer} pt-6 sm:pt-8 pb-8 sm:pb-10 md:pb-12`}
      >
        <div className="max-w-7xl mx-auto">
          <div className="text-center space-y-4 sm:space-y-6">
            <h1
              className={`
              ${TYPOGRAPHY.hero} 
              bg-gradient-to-r ${GRADIENTS.primary} 
              bg-clip-text text-transparent leading-tight 
              ${ANIMATIONS.fadeIn} duration-1000
              flex flex-col sm:flex-row items-center justify-center gap-2 sm:gap-4
            `}
            >
              <span className="text-4xl sm:text-5xl md:text-6xl animate-bounce-gentle">
                🍯
              </span>
              <span className="text-center">Financial Rankings</span>
              <span className="text-4xl sm:text-5xl md:text-6xl animate-float">
                🚀
              </span>
            </h1>

            <p
              className={`
                ${TYPOGRAPHY.subtitle} 
                text-slate-600 dark:text-slate-400 
                max-w-xl sm:max-w-2xl md:max-w-3xl mx-auto leading-relaxed 
                ${ANIMATIONS.fadeIn} duration-1000 px-4 sm:px-0
              `}
              style={{ animationDelay: '200ms' }}
            >
              🥞 Discover the sweetest performing stocks with comprehensive
              quarterly analysis, real-time data, and intelligent growth metrics
              ✨
              <span className="block mt-2 text-orange-600 dark:text-orange-400 font-semibold">
                Make your portfolio as sweet as pancakes! 🎯
              </span>
            </p>

            <div
              className={`
                flex flex-wrap items-center justify-center gap-3 sm:gap-4 md:gap-6 lg:gap-8 mt-6 sm:mt-8 
                ${ANIMATIONS.fadeIn} duration-1000 px-4 sm:px-0
              `}
              style={{ animationDelay: '400ms' }}
            >
              <StatusBadge
                label="🔥 Growth Trending"
                color="emerald"
                delay="0s"
              />
              <StatusBadge label="⚡ Live Data" color="orange" delay="0.5s" />
              <StatusBadge
                label="📊 Multi-Quarter Analysis"
                color="yellow"
                delay="1s"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

interface StatusBadgeProps {
  label: string;
  color: 'emerald' | 'blue' | 'purple' | 'orange' | 'yellow';
  delay: string;
}

/**
 * Status badge component for header
 */
function StatusBadge({
  label,
  color,
  delay,
}: StatusBadgeProps): React.JSX.Element {
  const colorClasses = {
    emerald: {
      bg: 'bg-emerald-50 dark:bg-emerald-900/20 hover:bg-emerald-100 dark:hover:bg-emerald-900/30',
      dot: 'bg-emerald-400',
      text: 'text-emerald-700 dark:text-emerald-300',
    },
    blue: {
      bg: 'bg-blue-50 dark:bg-blue-900/20 hover:bg-blue-100 dark:hover:bg-blue-900/30',
      dot: 'bg-blue-400',
      text: 'text-blue-700 dark:text-blue-300',
    },
    purple: {
      bg: 'bg-purple-50 dark:bg-purple-900/20 hover:bg-purple-100 dark:hover:bg-purple-900/30',
      dot: 'bg-purple-400',
      text: 'text-purple-700 dark:text-purple-300',
    },
    orange: {
      bg: 'bg-orange-50 dark:bg-orange-900/20 hover:bg-orange-100 dark:hover:bg-orange-900/30',
      dot: 'bg-orange-400',
      text: 'text-orange-700 dark:text-orange-300',
    },
    yellow: {
      bg: 'bg-yellow-50 dark:bg-yellow-900/20 hover:bg-yellow-100 dark:hover:bg-yellow-900/30',
      dot: 'bg-yellow-400',
      text: 'text-yellow-700 dark:text-yellow-300',
    },
  };

  const colors = colorClasses[color];

  return (
    <div
      className={`
      flex items-center gap-2 sm:gap-3 px-3 sm:px-4 py-2 rounded-full 
      ${colors.bg} 
      transition-colors duration-200
    `}
    >
      <div
        className={`w-3 sm:w-4 h-3 sm:h-4 rounded-full ${colors.dot} animate-pulse`}
        style={{ animationDelay: delay }}
      />
      <span className={`text-xs sm:text-sm font-semibold ${colors.text}`}>
        {label}
      </span>
    </div>
  );
}
