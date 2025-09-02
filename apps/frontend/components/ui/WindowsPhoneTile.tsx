'use client';

import { motion } from 'framer-motion';
import { ReactNode } from 'react';

interface WindowsPhoneTileProps {
  size?: 'small' | 'medium' | 'large' | 'wide';
  variant?: 'pancake' | 'admin' | 'analytics';
  children?: ReactNode;
  icon?: string;
  badge?: string | number;
  onClick?: () => void;
  animate?: boolean;
  delay?: number;
}

export function WindowsPhoneTile({
  size = 'medium',
  variant = 'pancake',
  children,
  icon,
  badge,
  onClick,
  animate = true,
  delay = 0
}: WindowsPhoneTileProps) {
  const sizeClasses = {
    small: 'w-16 h-16',
    medium: 'w-32 h-32',
    large: 'w-32 h-64',
    wide: 'w-64 h-32'
  };

  const variantStyles = {
    pancake: {
      bg: 'bg-gradient-to-br from-orange-400 via-yellow-500 to-orange-600',
      accent: 'bg-orange-600',
      text: 'text-white'
    },
    admin: {
      bg: 'bg-gradient-to-br from-blue-600 via-indigo-600 to-blue-800',
      accent: 'bg-blue-800',
      text: 'text-white'
    },
    analytics: {
      bg: 'bg-gradient-to-br from-slate-600 via-gray-700 to-slate-800',
      accent: 'bg-slate-800',
      text: 'text-white'
    }
  };

  const style = variantStyles[variant];

  return (
    <motion.div
      initial={animate ? { scale: 0, opacity: 0 } : {}}
      animate={animate ? { scale: 1, opacity: 1 } : {}}
      transition={{ duration: 0.3, delay }}
      whileHover={{ scale: 1.02, y: -2 }}
      whileTap={{ scale: 0.98 }}
      className={`
        ${sizeClasses[size]}
        ${style.bg}
        ${style.text}
        relative overflow-hidden cursor-pointer
        shadow-lg hover:shadow-xl transition-shadow duration-300
        group
      `}
      onClick={onClick}
    >
      {/* Live Tile Animation Background */}
      <motion.div
        className="absolute inset-0 opacity-20"
        animate={{
          background: [
            'linear-gradient(45deg, transparent 30%, rgba(255,255,255,0.1) 50%, transparent 70%)',
            'linear-gradient(45deg, transparent 40%, rgba(255,255,255,0.1) 60%, transparent 80%)',
            'linear-gradient(45deg, transparent 30%, rgba(255,255,255,0.1) 50%, transparent 70%)'
          ]
        }}
        transition={{ duration: 3, repeat: Infinity }}
      />

      {/* Badge */}
      {badge && (
        <motion.div
          initial={{ scale: 0 }}
          animate={{ scale: 1 }}
          className={`absolute -top-2 -right-2 ${style.accent} text-white rounded-full w-6 h-6 flex items-center justify-center text-xs font-bold z-10`}
        >
          {badge}
        </motion.div>
      )}

      {/* Content */}
      <div className="relative z-5 h-full flex flex-col p-3">
        {/* Icon */}
        {icon && (
          <motion.div
            className="text-2xl mb-2"
            animate={animate ? { rotate: [0, 5, 0, -5, 0] } : {}}
            transition={{ duration: 4, repeat: Infinity, delay: delay + 1 }}
          >
            {icon}
          </motion.div>
        )}

        {/* Children Content */}
        <div className="flex-1 flex flex-col justify-center">
          {children}
        </div>

        {/* Bottom accent line */}
        <motion.div
          className={`absolute bottom-0 left-0 h-1 ${style.accent}`}
          initial={{ width: '0%' }}
          animate={{ width: '100%' }}
          transition={{ duration: 1, delay: delay + 0.5 }}
        />
      </div>

      {/* Hover effect */}
      <motion.div
        className="absolute inset-0 bg-white opacity-0 group-hover:opacity-10 transition-opacity duration-300"
        initial={false}
      />
    </motion.div>
  );
}

// Metro Dashboard Grid Component
interface MetroDashboardProps {
  children: ReactNode;
  className?: string;
}

export function MetroDashboard({ children, className = '' }: MetroDashboardProps) {
  return (
    <div className={`grid gap-4 p-4 ${className}`} style={{ gridTemplateColumns: 'repeat(auto-fit, minmax(128px, 1fr))' }}>
      {children}
    </div>
  );
}

// Pancake-themed tiles
export function PancakeTile({ size = 'medium', ...props }: Omit<WindowsPhoneTileProps, 'variant'>) {
  return <WindowsPhoneTile variant="pancake" size={size} {...props} />;
}

export function AdminTile({ size = 'medium', ...props }: Omit<WindowsPhoneTileProps, 'variant'>) {
  return <WindowsPhoneTile variant="admin" size={size} {...props} />;
}

export function AnalyticsTile({ size = 'medium', ...props }: Omit<WindowsPhoneTileProps, 'variant'>) {
  return <WindowsPhoneTile variant="analytics" size={size} {...props} />;
}