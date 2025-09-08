'use client';

import { motion } from 'framer-motion';
import { ReactNode } from 'react';
import { cn } from '@/lib/utils';

interface PancakeCardProps {
  children: ReactNode;
  className?: string;
  variant?: 'default' | 'stats' | 'feature' | 'user' | 'analytics' | 'settings';
  hover?: boolean;
  glow?: boolean;
  animated?: boolean;
  clickable?: boolean;
  onClick?: () => void;
}

export function PancakeCard({
  children,
  className,
  variant = 'default',
  hover = true,
  glow = false,
  animated = true,
  clickable = false,
  onClick
}: PancakeCardProps) {
  const variants = {
    default: {
      bg: 'bg-gradient-to-br from-orange-50 via-yellow-50 to-orange-100 dark:from-orange-950/50 dark:via-yellow-950/50 dark:to-orange-900/50',
      border: 'border border-orange-200/50 dark:border-orange-800/50',
      shadow: 'shadow-lg shadow-orange-200/30 dark:shadow-orange-900/30',
      hoverShadow: 'hover:shadow-2xl hover:shadow-orange-300/40 dark:hover:shadow-orange-800/40'
    },
    stats: {
      bg: 'bg-gradient-to-br from-orange-100 via-yellow-100 to-orange-200 dark:from-orange-900/70 dark:via-yellow-900/70 dark:to-orange-800/70',
      border: 'border-2 border-orange-300/60 dark:border-orange-700/60',
      shadow: 'shadow-xl shadow-orange-300/40 dark:shadow-orange-900/40',
      hoverShadow: 'hover:shadow-2xl hover:shadow-orange-400/50 dark:hover:shadow-orange-700/50'
    },
    feature: {
      bg: 'bg-gradient-to-br from-yellow-50 via-orange-50 to-yellow-100 dark:from-yellow-950/50 dark:via-orange-950/50 dark:to-yellow-900/50',
      border: 'border-2 border-gradient-to-r border-orange-300 border-yellow-300 dark:border-orange-800 dark:border-yellow-800',
      shadow: 'shadow-lg shadow-yellow-300/30 dark:shadow-yellow-900/30',
      hoverShadow: 'hover:shadow-2xl hover:shadow-yellow-400/40 dark:hover:shadow-yellow-800/40'
    },
    user: {
      bg: 'bg-gradient-to-br from-orange-100/80 via-yellow-100/80 to-orange-200/80 dark:from-orange-900/60 dark:via-yellow-900/60 dark:to-orange-800/60',
      border: 'border-l-4 border-orange-400 border-r border-b border-t border-orange-200/60 dark:border-orange-700/60',
      shadow: 'shadow-lg shadow-orange-200/40 dark:shadow-orange-900/40',
      hoverShadow: 'hover:shadow-xl hover:shadow-orange-300/50 dark:hover:shadow-orange-800/50'
    },
    analytics: {
      bg: 'bg-gradient-to-br from-orange-50/90 via-yellow-50/90 to-orange-100/90 dark:from-orange-950/40 dark:via-yellow-950/40 dark:to-orange-900/40',
      border: 'border border-orange-200/70 dark:border-orange-800/70',
      shadow: 'shadow-lg shadow-orange-200/25 dark:shadow-orange-900/25',
      hoverShadow: 'hover:shadow-xl hover:shadow-orange-300/35 dark:hover:shadow-orange-800/35'
    },
    settings: {
      bg: 'bg-gradient-to-br from-yellow-100 via-orange-100 to-yellow-200 dark:from-yellow-900/50 dark:via-orange-900/50 dark:to-yellow-800/50',
      border: 'border-2 border-yellow-300/70 dark:border-yellow-700/70',
      shadow: 'shadow-lg shadow-yellow-200/40 dark:shadow-yellow-900/40',
      hoverShadow: 'hover:shadow-xl hover:shadow-yellow-300/50 dark:hover:shadow-yellow-800/50'
    }
  };

  const style = variants[variant];

  return (
    <motion.div
      onClick={clickable ? onClick : undefined}
      className={cn(
        'relative overflow-hidden backdrop-blur-sm',
        style.bg,
        style.border,
        style.shadow,
        hover && style.hoverShadow,
        hover && 'hover:scale-[1.02] transition-all duration-300',
        clickable && 'cursor-pointer',
        glow && 'hover:shadow-orange-400/60 dark:hover:shadow-orange-600/60',
        'rounded-xl p-6',
        className
      )}
      initial={animated ? { opacity: 0, y: 20, scale: 0.95 } : false}
      animate={animated ? { opacity: 1, y: 0, scale: 1 } : false}
      transition={animated ? { duration: 0.4, ease: "easeOut" } : false}
      whileHover={hover ? { 
        y: -2,
        transition: { duration: 0.2 }
      } : false}
    >
      {/* PancakeSwap Shine Effect */}
      <motion.div
        className="absolute inset-0 opacity-0 hover:opacity-100 transition-opacity duration-500"
        style={{
          background: 'linear-gradient(45deg, transparent 30%, rgba(255,193,7,0.1) 50%, transparent 70%)',
        }}
        animate={{
          x: ['-100%', '100%']
        }}
        transition={{
          duration: 2,
          repeat: Infinity,
          repeatDelay: 3
        }}
      />

      {/* PancakeSwap Corner Accent */}
      <div className="absolute top-0 right-0 w-16 h-16 bg-gradient-to-br from-orange-400/20 to-yellow-400/20 dark:from-orange-600/30 dark:to-yellow-600/30 rounded-bl-3xl" />
      
      {/* Content */}
      <div className="relative z-10">
        {children}
      </div>

      {/* Bottom Gradient Line */}
      <motion.div
        className="absolute bottom-0 left-0 right-0 h-1 bg-gradient-to-r from-orange-400 via-yellow-400 to-orange-400 opacity-60"
        initial={{ scaleX: 0 }}
        animate={{ scaleX: 1 }}
        transition={{ duration: 0.8, delay: 0.2 }}
      />
    </motion.div>
  );
}

// Stats Card Component
interface PancakeStatsCardProps {
  title: string;
  value: string | number;
  subtitle?: string;
  icon?: string;
  trend?: 'up' | 'down' | 'neutral';
  trendValue?: string;
  onClick?: () => void;
  isLoading?: boolean;
  error?: boolean;
}

export function PancakeStatsCard({
  title,
  value,
  subtitle,
  icon,
  trend = 'neutral',
  trendValue,
  onClick,
  isLoading = false,
  error = false
}: PancakeStatsCardProps) {
  const trendColors = {
    up: 'text-green-500 dark:text-green-400',
    down: 'text-red-500 dark:text-red-400',
    neutral: 'text-gray-500 dark:text-gray-400'
  };

  const trendIcons = {
    up: '↗️',
    down: '↘️',
    neutral: '➡️'
  };

  if (error) {
    return (
      <PancakeCard variant="stats" className="border-red-200 dark:border-red-800">
        <div className="flex items-center justify-center h-24">
          <div className="text-center">
            <div className="text-2xl mb-1">⚠️</div>
            <p className="text-sm text-red-600 dark:text-red-400">Error loading data</p>
          </div>
        </div>
      </PancakeCard>
    );
  }

  if (isLoading) {
    return (
      <PancakeCard variant="stats">
        <div className="animate-pulse">
          <div className="w-8 h-8 bg-orange-300 dark:bg-orange-700 rounded mb-3"></div>
          <div className="w-16 h-4 bg-orange-300 dark:bg-orange-700 rounded mb-2"></div>
          <div className="w-20 h-8 bg-orange-300 dark:bg-orange-700 rounded mb-2"></div>
          <div className="w-24 h-3 bg-orange-300 dark:bg-orange-700 rounded"></div>
        </div>
      </PancakeCard>
    );
  }

  return (
    <PancakeCard variant="stats" clickable={!!onClick} onClick={onClick} glow>
      <div className="flex items-start justify-between">
        <div className="flex-1">
          {icon && (
            <div className="text-3xl mb-2 filter drop-shadow-sm">
              {icon}
            </div>
          )}
          <h3 className="text-sm font-semibold text-orange-700 dark:text-orange-300 uppercase tracking-wide mb-1">
            {title}
          </h3>
          <div className="text-3xl font-bold bg-gradient-to-r from-orange-600 to-yellow-600 bg-clip-text text-transparent mb-1">
            {typeof value === 'number' ? value.toLocaleString() : value}
          </div>
          {subtitle && (
            <p className="text-sm text-orange-600/80 dark:text-orange-400/80">
              {subtitle}
            </p>
          )}
        </div>
        
        {trendValue && (
          <div className={cn('flex items-center gap-1 text-sm font-medium', trendColors[trend])}>
            <span>{trendIcons[trend]}</span>
            <span>{trendValue}</span>
          </div>
        )}
      </div>
    </PancakeCard>
  );
}

// Feature Card Component
interface PancakeFeatureCardProps {
  title: string;
  description: string;
  icon?: string;
  badge?: string;
  onClick?: () => void;
  children?: ReactNode;
}

export function PancakeFeatureCard({
  title,
  description,
  icon,
  badge,
  onClick,
  children
}: PancakeFeatureCardProps) {
  return (
    <PancakeCard variant="feature" clickable={!!onClick} onClick={onClick} hover>
      <div className="space-y-4">
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-3">
            {icon && (
              <div className="w-12 h-12 bg-gradient-to-br from-orange-400 to-yellow-400 dark:from-orange-600 dark:to-yellow-600 rounded-lg flex items-center justify-center text-white text-xl font-bold shadow-lg">
                {icon}
              </div>
            )}
            <div>
              <h3 className="text-lg font-bold bg-gradient-to-r from-orange-600 to-yellow-600 bg-clip-text text-transparent">
                {title}
              </h3>
              <p className="text-sm text-orange-600/80 dark:text-orange-400/80 mt-1">
                {description}
              </p>
            </div>
          </div>
          
          {badge && (
            <span className="px-3 py-1 bg-gradient-to-r from-orange-500 to-yellow-500 text-white text-xs font-bold rounded-full shadow-sm">
              {badge}
            </span>
          )}
        </div>
        
        {children && (
          <div className="pt-2 border-t border-orange-200/50 dark:border-orange-800/50">
            {children}
          </div>
        )}
      </div>
    </PancakeCard>
  );
}

// User Card Component
interface PancakeUserCardProps {
  name: string;
  email: string;
  role?: string;
  avatar?: string;
  status: 'active' | 'inactive' | 'pending';
  permissions?: number;
  onClick?: () => void;
  actions?: ReactNode;
}

export function PancakeUserCard({
  name,
  email,
  role,
  avatar,
  status,
  permissions,
  onClick,
  actions
}: PancakeUserCardProps) {
  const statusColors = {
    active: 'bg-green-500',
    inactive: 'bg-gray-500',
    pending: 'bg-yellow-500'
  };

  const statusLabels = {
    active: 'Online',
    inactive: 'Offline',
    pending: 'Pending'
  };

  return (
    <PancakeCard variant="user" clickable={!!onClick} onClick={onClick}>
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <div className="relative">
            {avatar ? (
              <img src={avatar} alt={name} className="w-12 h-12 rounded-full object-cover" />
            ) : (
              <div className="w-12 h-12 bg-gradient-to-br from-orange-400 to-yellow-400 rounded-full flex items-center justify-center text-white font-bold text-lg">
                {name.charAt(0).toUpperCase()}
              </div>
            )}
            <div className={cn('absolute -bottom-1 -right-1 w-4 h-4 rounded-full border-2 border-white', statusColors[status])} />
          </div>
          
          <div className="flex-1">
            <h3 className="font-semibold text-gray-900 dark:text-white">{name}</h3>
            <p className="text-sm text-orange-600/80 dark:text-orange-400/80">{email}</p>
            <div className="flex items-center gap-2 mt-1">
              {role && (
                <span className="px-2 py-1 bg-orange-100 dark:bg-orange-900/50 text-orange-700 dark:text-orange-300 text-xs rounded-full">
                  {role}
                </span>
              )}
              <span className="text-xs text-gray-500">
                {statusLabels[status]}
              </span>
            </div>
          </div>
        </div>
        
        <div className="flex items-center gap-2">
          {permissions && (
            <div className="text-center">
              <div className="text-lg font-bold text-orange-600 dark:text-orange-400">{permissions}</div>
              <div className="text-xs text-gray-500">permissions</div>
            </div>
          )}
          {actions}
        </div>
      </div>
    </PancakeCard>
  );
}