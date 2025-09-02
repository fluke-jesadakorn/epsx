'use client';

import { motion } from 'framer-motion';

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
      {/* Main Loader - Pancake Stack Animation */}
      <div className="relative">
        {/* Bottom Pancake */}
        <motion.div
          className={`${sizeClasses[size]} bg-gradient-to-br ${style.primary} rounded-full relative z-10`}
          animate={{
            scale: [1, 1.1, 1],
            rotate: [0, 180, 360]
          }}
          transition={{
            duration: 2,
            repeat: Infinity,
            ease: 'easeInOut'
          }}
        />
        
        {/* Middle Pancake */}
        <motion.div
          className={`${sizeClasses[size]} bg-gradient-to-br ${style.secondary} rounded-full absolute top-0 left-0 z-20`}
          animate={{
            scale: [1.1, 1, 1.1],
            rotate: [360, 180, 0]
          }}
          transition={{
            duration: 2,
            repeat: Infinity,
            ease: 'easeInOut',
            delay: 0.3
          }}
        />

        {/* Top Pancake with Icon */}
        <motion.div
          className={`${sizeClasses[size]} bg-gradient-to-br ${style.primary} rounded-full absolute top-0 left-0 z-30 flex items-center justify-center text-white`}
          animate={{
            scale: [1, 1.2, 1],
            rotate: [0, -180, -360]
          }}
          transition={{
            duration: 2,
            repeat: Infinity,
            ease: 'easeInOut',
            delay: 0.6
          }}
        >
          <span className="text-lg">{style.icon}</span>
        </motion.div>

        {/* Syrup Drip Effect */}
        <motion.div
          className={`absolute -bottom-2 left-1/2 transform -translate-x-1/2 w-1 ${style.accent} rounded-full`}
          animate={{
            height: [0, 16, 0],
            opacity: [0, 1, 0]
          }}
          transition={{
            duration: 1.5,
            repeat: Infinity,
            ease: 'easeInOut',
            delay: 1
          }}
        />
      </div>

      {/* Windows Phone Progress Dots */}
      <div className="flex space-x-2">
        {Array.from({ length: 3 }).map((_, i) => (
          <motion.div
            key={i}
            className={`w-2 h-2 ${style.accent} rounded-none`}
            animate={{
              opacity: [0.3, 1, 0.3],
              scale: [1, 1.2, 1]
            }}
            transition={{
              duration: 1,
              repeat: Infinity,
              delay: i * 0.2
            }}
          />
        ))}
      </div>

      {/* Loading Message */}
      {message && (
        <motion.p
          className="text-sm font-medium text-gray-600"
          animate={{
            opacity: [0.7, 1, 0.7]
          }}
          transition={{
            duration: 2,
            repeat: Infinity
          }}
        >
          {message}
        </motion.p>
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
    <div className="w-full h-1 bg-gray-200 overflow-hidden">
      <motion.div
        className={`h-full ${variants[variant]}`}
        initial={{ width: '0%' }}
        animate={{ width: `${progress}%` }}
        transition={{ duration: animated ? 0.5 : 0 }}
      />
      
      {animated && (
        <motion.div
          className={`h-full ${variants[variant]} absolute top-0 opacity-50`}
          animate={{
            x: ['-100%', '100%']
          }}
          transition={{
            duration: 2,
            repeat: Infinity,
            ease: 'linear'
          }}
        />
      )}
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
    <motion.div
      className={`${sizeClasses[size]} bg-gradient-to-br ${variants[variant]} rounded-full flex items-center justify-center text-white`}
      animate={{
        rotateY: [0, 180, 360],
        scale: [1, 1.1, 1]
      }}
      transition={{
        duration: 1.5,
        repeat: Infinity,
        ease: 'easeInOut'
      }}
    >
      🥞
    </motion.div>
  );
}