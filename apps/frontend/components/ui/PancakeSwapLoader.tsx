'use client';

interface PancakeSwapLoaderProps {
  variant?: 'pancake' | 'admin' | 'analytics';
  size?: 'sm' | 'md' | 'lg';
  message?: string;
}

export function PancakeSwapLoader({ 
  variant = 'pancake', 
  size = 'md',
  message 
}: PancakeSwapLoaderProps) {
  const sizeClasses = {
    sm: 'w-8 h-8',
    md: 'w-12 h-12', 
    lg: 'w-16 h-16'
  };

  const variants = {
    pancake: {
      primary: 'from-orange-400 to-yellow-500',
      secondary: 'from-yellow-500 to-orange-600',
      accent: 'bg-orange-500',
      icon: '🥞'
    },
    admin: {
      primary: 'from-blue-600 to-indigo-700',
      secondary: 'from-indigo-700 to-blue-800',
      accent: 'bg-blue-600',
      icon: '⚡'
    },
    analytics: {
      primary: 'from-slate-600 to-gray-700',
      secondary: 'from-gray-700 to-slate-800',
      accent: 'bg-slate-600',
      icon: '📊'
    }
  };

  const style = variants[variant];

  return (
    <div className="flex flex-col items-center justify-center space-y-4">
      {/* Main Loader - Static Stack */}
      <div className="relative">
        {/* Bottom Pancake */}
        <div className={`${sizeClasses[size]} bg-gradient-to-br ${style.primary} rounded-full relative z-10`} />
        
        {/* Middle Pancake */}
        <div className={`${sizeClasses[size]} bg-gradient-to-br ${style.secondary} rounded-full absolute top-0 left-0 z-20`} />

        {/* Top Pancake with Icon */}
        <div className={`${sizeClasses[size]} bg-gradient-to-br ${style.primary} rounded-full absolute top-0 left-0 z-30 flex items-center justify-center text-white`}>
          <span className="text-lg">{style.icon}</span>
        </div>
      </div>

      {/* Progress Dots */}
      <div className="flex space-x-2">
        {Array.from({ length: 3 }).map((_, i) => (
          <div
            key={i}
            className={`w-2 h-2 ${style.accent} rounded-none`}
          />
        ))}
      </div>

      {/* Loading Message */}
      {message && (
        <p className="text-sm font-medium text-gray-600">
          {message}
        </p>
      )}
    </div>
  );
}

// Metro-style Progress Bar
interface MetroProgressBarProps {
  progress?: number;
  variant?: 'pancake' | 'admin' | 'analytics';
  animated?: boolean;
}

export function MetroProgressBar({ 
  progress = 0, 
  variant = 'pancake',
  animated = true 
}: MetroProgressBarProps) {
  const variants = {
    pancake: 'bg-gradient-to-r from-orange-400 to-yellow-500',
    admin: 'bg-gradient-to-r from-blue-600 to-indigo-700',
    analytics: 'bg-gradient-to-r from-slate-600 to-gray-700'
  };

  return (
    <div className="w-full h-1 bg-gray-200 overflow-hidden relative">
      <div
        className={`h-full transition-all duration-500 ${variants[variant]}`}
        style={{ width: `${Math.min(Math.max(progress, 0), 100)}%` }}
      />
    </div>
  );
}

// Pancake Flip Animation
export function PancakeFlip({ variant = 'pancake', size = 'md' }: Pick<PancakeSwapLoaderProps, 'variant' | 'size'>) {
  const sizeClasses = {
    sm: 'w-6 h-6',
    md: 'w-10 h-10',
    lg: 'w-14 h-14'
  };

  const variants = {
    pancake: 'from-orange-400 to-yellow-500',
    admin: 'from-blue-600 to-indigo-700', 
    analytics: 'from-slate-600 to-gray-700'
  };

  return (
    <div className={`${sizeClasses[size]} bg-gradient-to-br ${variants[variant]} rounded-full flex items-center justify-center text-white`}>
      🥞
    </div>
  );
}