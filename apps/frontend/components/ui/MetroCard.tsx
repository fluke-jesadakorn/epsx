'use client';

import { motion } from 'framer-motion';
import { ReactNode } from 'react';

interface MetroCardProps {
  variant?: 'pancake' | 'admin' | 'analytics';
  children: ReactNode;
  className?: string;
  hover?: boolean;
  accent?: 'left' | 'top' | 'bottom' | 'right';
  glassmorphism?: boolean;
}

export function MetroCard({
  variant = 'pancake',
  children,
  className = '',
  hover = true,
  accent = 'left',
  glassmorphism = false
}: MetroCardProps) {
  const variants = {
    pancake: {
      bg: glassmorphism ? 'bg-white/10' : 'bg-white',
      border: 'border-orange-500',
      accent: 'bg-gradient-to-r from-orange-400 to-yellow-500',
      shadow: 'shadow-orange-200',
      text: glassmorphism ? 'text-white' : 'text-gray-800'
    },
    admin: {
      bg: glassmorphism ? 'bg-slate-800/20' : 'bg-slate-800',
      border: 'border-blue-500',
      accent: 'bg-gradient-to-r from-blue-600 to-indigo-700',
      shadow: 'shadow-blue-200',
      text: 'text-white'
    },
    analytics: {
      bg: glassmorphism ? 'bg-gray-800/10' : 'bg-white',
      border: 'border-gray-500',
      accent: 'bg-gradient-to-r from-slate-600 to-gray-700',
      shadow: 'shadow-gray-200',
      text: glassmorphism ? 'text-white' : 'text-gray-800'
    }
  };

  const style = variants[variant];

  const accentPositions = {
    left: 'left-0 top-0 bottom-0 w-1',
    right: 'right-0 top-0 bottom-0 w-1',
    top: 'top-0 left-0 right-0 h-1',
    bottom: 'bottom-0 left-0 right-0 h-1'
  };

  return (
    <motion.div
      className={`
        relative 
        ${style.bg} 
        ${style.text}
        ${glassmorphism ? 'backdrop-blur-xl' : ''} 
        p-6 
        shadow-lg 
        ${style.shadow}
        overflow-hidden
        ${className}
      `}
      whileHover={hover ? { y: -4, scale: 1.01 } : {}}
      transition={{ duration: 0.3, ease: 'easeOut' }}
    >
      {/* Accent Bar */}
      <motion.div
        className={`absolute ${accentPositions[accent]} ${style.accent} z-10`}
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.2, duration: 0.5 }}
      />

      {/* Windows Phone Live Tile Effect */}
      <motion.div
        className="absolute inset-0 opacity-5"
        animate={{
          background: [
            'linear-gradient(45deg, transparent 30%, rgba(255,255,255,0.1) 50%, transparent 70%)',
            'linear-gradient(135deg, transparent 40%, rgba(255,255,255,0.1) 60%, transparent 80%)',
            'linear-gradient(45deg, transparent 30%, rgba(255,255,255,0.1) 50%, transparent 70%)'
          ]
        }}
        transition={{ duration: 4, repeat: Infinity }}
      />

      {/* Content */}
      <div className="relative z-5">
        {children}
      </div>
    </motion.div>
  );
}

// Specialized Metro Cards
export function PancakeCard({ glassmorphism = false, ...props }: Omit<MetroCardProps, 'variant'>) {
  return <MetroCard variant="pancake" glassmorphism={glassmorphism} {...props} />;
}

export function AdminCard({ glassmorphism = true, ...props }: Omit<MetroCardProps, 'variant'>) {
  return <MetroCard variant="admin" glassmorphism={glassmorphism} {...props} />;
}

export function AnalyticsCard({ glassmorphism = false, ...props }: Omit<MetroCardProps, 'variant'>) {
  return <MetroCard variant="analytics" glassmorphism={glassmorphism} {...props} />;
}

// Metro Stats Card
interface MetroStatsCardProps {
  variant?: 'pancake' | 'admin' | 'analytics';
  icon?: string;
  value?: string | number;
  label?: string;
  trend?: 'up' | 'down' | 'neutral';
  percentage?: string;
}

export function MetroStatsCard({
  variant = 'pancake',
  icon,
  value,
  label,
  trend,
  percentage
}: MetroStatsCardProps) {
  const trendIcons = {
    up: '📈',
    down: '📉',
    neutral: '➡'
  };

  const trendColors = {
    up: 'text-green-500',
    down: 'text-red-500',
    neutral: 'text-gray-500'
  };

  return (
    <MetroCard variant={variant} className="min-w-48">
      <div className="flex items-start justify-between">
        <div className="flex-1">
          {icon && (
            <motion.div 
              className="text-2xl mb-2"
              animate={{ scale: [1, 1.1, 1] }}
              transition={{ duration: 2, repeat: Infinity }}
            >
              {icon}
            </motion.div>
          )}
          
          <motion.div 
            className="text-3xl font-bold mb-1"
            initial={{ scale: 0 }}
            animate={{ scale: 1 }}
            transition={{ delay: 0.3, type: 'spring' }}
          >
            {value}
          </motion.div>
          
          {label && (
            <div className="text-sm opacity-70 font-medium">
              {label}
            </div>
          )}
        </div>

        {trend && percentage && (
          <motion.div 
            className={`flex items-center space-x-1 ${trendColors[trend]}`}
            initial={{ x: 20, opacity: 0 }}
            animate={{ x: 0, opacity: 1 }}
            transition={{ delay: 0.5 }}
          >
            <span>{trendIcons[trend]}</span>
            <span className="text-sm font-medium">{percentage}</span>
          </motion.div>
        )}
      </div>
    </MetroCard>
  );
}

// Metro List Card
interface MetroListItem {
  icon?: string;
  title: string;
  subtitle?: string;
  action?: string;
}

interface MetroListCardProps {
  variant?: 'pancake' | 'admin' | 'analytics';
  items: MetroListItem[];
  title?: string;
}

export function MetroListCard({ variant = 'pancake', items, title }: MetroListCardProps) {
  return (
    <MetroCard variant={variant}>
      {title && (
        <h3 className="font-bold text-lg mb-4 opacity-90">{title}</h3>
      )}
      
      <div className="space-y-3">
        {items.map((item, index) => (
          <motion.div
            key={index}
            className="flex items-center justify-between py-2 border-b border-gray-200/20 last:border-b-0"
            initial={{ x: -20, opacity: 0 }}
            animate={{ x: 0, opacity: 1 }}
            transition={{ delay: index * 0.1 }}
          >
            <div className="flex items-center space-x-3">
              {item.icon && (
                <span className="text-xl">{item.icon}</span>
              )}
              <div>
                <div className="font-medium">{item.title}</div>
                {item.subtitle && (
                  <div className="text-sm opacity-70">{item.subtitle}</div>
                )}
              </div>
            </div>
            
            {item.action && (
              <span className="text-sm opacity-70">{item.action}</span>
            )}
          </motion.div>
        ))}
      </div>
    </MetroCard>
  );
}