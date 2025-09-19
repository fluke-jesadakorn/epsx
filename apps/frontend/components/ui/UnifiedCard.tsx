'use client';

import { ReactNode } from 'react';
import { cn } from '@/lib/utils';

type CardVariant = 'default' | 'pancake' | 'admin' | 'analytics' | 'premium' | 'glass';
type CardSize = 'sm' | 'md' | 'lg';
type CardPadding = 'none' | 'sm' | 'md' | 'lg';
type AccentPosition = 'left' | 'top' | 'bottom' | 'right' | 'none';

interface UnifiedCardProps {
  children: ReactNode;
  variant?: CardVariant;
  size?: CardSize;
  padding?: CardPadding;
  hover?: boolean;
  accent?: AccentPosition;
  glassmorphism?: boolean;
  className?: string;
}

interface UnifiedCardSectionProps {
  children: ReactNode;
  className?: string;
}

export function UnifiedCard({
  children,
  variant = 'default',
  size = 'md',
  padding = 'md',
  hover = true,
  accent = 'none',
  glassmorphism = false,
  className
}: UnifiedCardProps) {
  const variants = {
    default: {
      bg: glassmorphism ? 'bg-white/80' : 'bg-white',
      border: 'border-gray-200',
      accent: 'bg-gradient-to-r from-blue-500 to-blue-600',
      shadow: 'shadow-sm',
      text: 'text-gray-900'
    },
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
      bg: glassmorphism ? 'bg-blue-50/80' : 'bg-gradient-to-br from-blue-50 to-indigo-50',
      border: 'border-blue-200',
      accent: 'bg-gradient-to-r from-indigo-500 to-purple-600',
      shadow: 'shadow-sm',
      text: 'text-gray-900'
    },
    premium: {
      bg: glassmorphism ? 'bg-purple-50/80' : 'bg-gradient-to-br from-purple-50 to-pink-50',
      border: 'border-purple-200',
      accent: 'bg-gradient-to-r from-purple-500 to-pink-600',
      shadow: 'shadow-sm',
      text: 'text-gray-900'
    },
    glass: {
      bg: 'bg-white/80',
      border: 'border-gray-200/50',
      accent: 'bg-gradient-to-r from-blue-500/80 to-indigo-600/80',
      shadow: 'shadow-lg',
      text: 'text-gray-900'
    }
  };

  const sizes = {
    sm: 'max-w-sm',
    md: 'max-w-md',
    lg: 'max-w-lg'
  };

  const paddings = {
    none: '',
    sm: 'p-4',
    md: 'p-6',
    lg: 'p-8'
  };

  const accentPositions = {
    left: 'left-0 top-0 bottom-0 w-1',
    right: 'right-0 top-0 bottom-0 w-1',
    top: 'top-0 left-0 right-0 h-1',
    bottom: 'bottom-0 left-0 right-0 h-1',
    none: 'hidden'
  };

  const style = variants[variant];

  return (
    <div
      className={cn(
        'relative border rounded-lg overflow-hidden',
        style.bg,
        style.border,
        style.text,
        style.shadow,
        sizes[size],
        paddings[padding],
        glassmorphism && 'backdrop-blur-xl',
        hover && 'hover:shadow-xl transition-all duration-200',
        className
      )}
    >
      {/* Accent Bar */}
      {accent !== 'none' && (
        <div
          className={cn(
            'absolute z-10 rounded-sm',
            accentPositions[accent],
            style.accent
          )}
        />
      )}

      {/* Content */}
      <div className="relative z-5">
        {children}
      </div>
    </div>
  );
}

export function UnifiedCardHeader({ children, className }: UnifiedCardSectionProps) {
  return (
    <div className={cn('pb-4 border-b border-gray-200', className)}>
      {children}
    </div>
  );
}

export function UnifiedCardContent({ children, className }: UnifiedCardSectionProps) {
  return (
    <div className={cn('py-4', className)}>
      {children}
    </div>
  );
}

export function UnifiedCardFooter({ children, className }: UnifiedCardSectionProps) {
  return (
    <div className={cn('pt-4 border-t border-gray-200', className)}>
      {children}
    </div>
  );
}

// Specialized Cards (Legacy compatibility)
export function PancakeCard({ glassmorphism = false, ...props }: Omit<UnifiedCardProps, 'variant'>) {
  return <UnifiedCard variant="pancake" glassmorphism={glassmorphism} {...props} />;
}

export function AdminCard({ glassmorphism = true, ...props }: Omit<UnifiedCardProps, 'variant'>) {
  return <UnifiedCard variant="admin" glassmorphism={glassmorphism} {...props} />;
}

export function AnalyticsCard({ glassmorphism = false, ...props }: Omit<UnifiedCardProps, 'variant'>) {
  return <UnifiedCard variant="analytics" glassmorphism={glassmorphism} {...props} />;
}

export function PremiumCard({ ...props }: Omit<UnifiedCardProps, 'variant'>) {
  return <UnifiedCard variant="premium" {...props} />;
}

export function GlassCard({ ...props }: Omit<UnifiedCardProps, 'variant'>) {
  return <UnifiedCard variant="glass" {...props} />;
}

// Stats Card
interface UnifiedStatsCardProps {
  variant?: CardVariant;
  icon?: ReactNode;
  value?: string | number;
  label?: string;
  trend?: 'up' | 'down' | 'neutral';
  percentage?: string;
  className?: string;
}

export function UnifiedStatsCard({
  variant = 'default',
  icon,
  value,
  label,
  trend,
  percentage,
  className
}: UnifiedStatsCardProps) {
  const trendIcons = {
    up: '↗',
    down: '↘',
    neutral: '→'
  };

  const trendColors = {
    up: 'text-green-600',
    down: 'text-red-600',
    neutral: 'text-gray-500'
  };

  return (
    <UnifiedCard variant={variant} className={cn('min-w-48', className)}>
      <div className="flex items-start justify-between">
        <div className="flex-1">
          {icon && (
            <div className="text-2xl mb-3">
              {icon}
            </div>
          )}
          
          <div className="text-2xl font-bold mb-1">
            {value}
          </div>
          
          {label && (
            <div className="text-sm opacity-70 font-medium">
              {label}
            </div>
          )}
        </div>

        {trend && percentage && (
          <div className={cn(
            'flex items-center space-x-1 text-sm font-medium',
            trendColors[trend]
          )}>
            <span className="text-base">{trendIcons[trend]}</span>
            <span>{percentage}</span>
          </div>
        )}
      </div>
    </UnifiedCard>
  );
}

// List Card
interface UnifiedListItem {
  icon?: ReactNode;
  title: string;
  subtitle?: string;
  action?: ReactNode;
  href?: string;
}

interface UnifiedListCardProps {
  variant?: CardVariant;
  items: UnifiedListItem[];
  title?: string;
  className?: string;
}

export function UnifiedListCard({ 
  variant = 'default', 
  items, 
  title,
  className 
}: UnifiedListCardProps) {
  return (
    <UnifiedCard variant={variant} className={className}>
      {title && (
        <h3 className="font-semibold text-lg mb-4 opacity-90">{title}</h3>
      )}
      
      <div className="space-y-3">
        {items.map((item, index) => {
          const Component = item.href ? 'a' : 'div';
          return (
            <Component
              key={index}
              href={item.href}
              className={cn(
                'flex items-center justify-between py-3 border-b border-gray-100 last:border-b-0',
                item.href && 'hover:bg-gray-50 cursor-pointer rounded-md px-2 -mx-2'
              )}
            >
              <div className="flex items-center space-x-3">
                {item.icon && (
                  <div className="text-lg">{item.icon}</div>
                )}
                <div>
                  <div className="font-medium">{item.title}</div>
                  {item.subtitle && (
                    <div className="text-sm opacity-70">{item.subtitle}</div>
                  )}
                </div>
              </div>
              
              {item.action && (
                <div className="text-sm opacity-70">{item.action}</div>
              )}
            </Component>
          );
        })}
      </div>
    </UnifiedCard>
  );
}

// Feature Card
interface UnifiedFeatureCardProps {
  variant?: CardVariant;
  icon?: ReactNode;
  title: string;
  description: string;
  action?: ReactNode;
  className?: string;
}

export function UnifiedFeatureCard({
  variant = 'default',
  icon,
  title,
  description,
  action,
  className
}: UnifiedFeatureCardProps) {
  return (
    <UnifiedCard variant={variant} className={cn('text-center', className)}>
      {icon && (
        <div className="text-3xl mb-4 flex justify-center">
          {icon}
        </div>
      )}
      
      <h3 className="font-semibold text-lg mb-2">{title}</h3>
      <p className="opacity-70 mb-4">{description}</p>
      
      {action && (
        <div className="mt-4">
          {action}
        </div>
      )}
    </UnifiedCard>
  );
}

// Legacy component aliases for backward compatibility
export const MetroCard = UnifiedCard;
export const EPSXCard = UnifiedCard;
export const ProfessionalCard = UnifiedCard;
export const MetroStatsCard = UnifiedStatsCard;
export const ProfessionalStatsCard = UnifiedStatsCard;
export const MetroListCard = UnifiedListCard;
export const ProfessionalListCard = UnifiedListCard;
export const ProfessionalFeatureCard = UnifiedFeatureCard;
export const EPSXCardHeader = UnifiedCardHeader;
export const EPSXCardContent = UnifiedCardContent;
export const EPSXCardFooter = UnifiedCardFooter;