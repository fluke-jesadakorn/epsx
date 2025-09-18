'use client';

import { ReactNode } from 'react';
import { cn } from '@/lib/utils';

interface ProfessionalCardProps {
  variant?: 'default' | 'analytics' | 'premium' | 'glass';
  children: ReactNode;
  className?: string;
  hover?: boolean;
  accent?: 'left' | 'top' | 'bottom' | 'right' | 'none';
  size?: 'sm' | 'md' | 'lg';
}

export function ProfessionalCard({
  variant = 'default',
  children,
  className = '',
  hover = true,
  accent = 'left',
  size = 'md'
}: ProfessionalCardProps) {
  const variants = {
    default: {
      bg: 'bg-white',
      border: 'border-gray-200',
      accent: 'bg-gradient-to-r from-blue-500 to-blue-600',
      shadow: 'shadow-sm hover:shadow-md',
      text: 'text-gray-900'
    },
    analytics: {
      bg: 'bg-gradient-to-br from-blue-50 to-indigo-50',
      border: 'border-blue-200',
      accent: 'bg-gradient-to-r from-indigo-500 to-purple-600',
      shadow: 'shadow-sm hover:shadow-lg',
      text: 'text-gray-900'
    },
    premium: {
      bg: 'bg-gradient-to-br from-purple-50 to-pink-50',
      border: 'border-purple-200',
      accent: 'bg-gradient-to-r from-purple-500 to-pink-600',
      shadow: 'shadow-sm hover:shadow-lg',
      text: 'text-gray-900'
    },
    glass: {
      bg: 'bg-white/80 backdrop-blur-lg',
      border: 'border-gray-200/50',
      accent: 'bg-gradient-to-r from-blue-500/80 to-indigo-600/80',
      shadow: 'shadow-lg hover:shadow-xl',
      text: 'text-gray-900'
    }
  };

  const sizes = {
    sm: 'p-4',
    md: 'p-6',
    lg: 'p-8'
  };

  const style = variants[variant];

  const accentPositions = {
    left: 'left-0 top-0 bottom-0 w-1',
    right: 'right-0 top-0 bottom-0 w-1',
    top: 'top-0 left-0 right-0 h-1',
    bottom: 'bottom-0 left-0 right-0 h-1',
    none: 'hidden'
  };

  return (
    <div
      className={cn(
        'relative border rounded-lg transition-all duration-300 ease-out',
        style.bg,
        style.border,
        style.text,
        style.shadow,
        sizes[size],
        hover && 'hover:-translate-y-1',
        className
      )}
    >
      {/* Accent Bar */}
      {accent !== 'none' && (
        <div
          className={cn(
            'absolute rounded-sm',
            accentPositions[accent],
            style.accent
          )}
        />
      )}

      {/* Content */}
      <div className="relative z-10">
        {children}
      </div>
    </div>
  );
}

// Specialized Professional Cards
export function AnalyticsCard({ ...props }: Omit<ProfessionalCardProps, 'variant'>) {
  return <ProfessionalCard variant="analytics" {...props} />;
}

export function PremiumCard({ ...props }: Omit<ProfessionalCardProps, 'variant'>) {
  return <ProfessionalCard variant="premium" {...props} />;
}

export function GlassCard({ ...props }: Omit<ProfessionalCardProps, 'variant'>) {
  return <ProfessionalCard variant="glass" {...props} />;
}

// Professional Stats Card
interface ProfessionalStatsCardProps {
  variant?: 'default' | 'analytics' | 'premium';
  icon?: ReactNode;
  value?: string | number;
  label?: string;
  trend?: 'up' | 'down' | 'neutral';
  percentage?: string;
  className?: string;
}

export function ProfessionalStatsCard({
  variant = 'default',
  icon,
  value,
  label,
  trend,
  percentage,
  className
}: ProfessionalStatsCardProps) {
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
    <ProfessionalCard variant={variant} className={cn('min-w-48', className)}>
      <div className="flex items-start justify-between">
        <div className="flex-1">
          {icon && (
            <div className="text-2xl mb-3 text-blue-500">
              {icon}
            </div>
          )}
          
          <div className="text-2xl font-bold mb-1 text-gray-900">
            {value}
          </div>
          
          {label && (
            <div className="text-sm text-gray-600 font-medium">
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
    </ProfessionalCard>
  );
}

// Professional List Card
interface ProfessionalListItem {
  icon?: ReactNode;
  title: string;
  subtitle?: string;
  action?: ReactNode;
  href?: string;
}

interface ProfessionalListCardProps {
  variant?: 'default' | 'analytics' | 'premium';
  items: ProfessionalListItem[];
  title?: string;
  className?: string;
}

export function ProfessionalListCard({ 
  variant = 'default', 
  items, 
  title,
  className 
}: ProfessionalListCardProps) {
  return (
    <ProfessionalCard variant={variant} className={className}>
      {title && (
        <h3 className="font-semibold text-lg mb-4 text-gray-900">{title}</h3>
      )}
      
      <div className="space-y-3">
        {items.map((item, index) => {
          const Component = item.href ? 'a' : 'div';
          return (
            <Component
              key={index}
              href={item.href}
              className={cn(
                'flex items-center justify-between py-3 border-b border-gray-100 last:border-b-0 transition-colors',
                item.href && 'hover:bg-gray-50 cursor-pointer rounded-md px-2 -mx-2'
              )}
            >
              <div className="flex items-center space-x-3">
                {item.icon && (
                  <div className="text-lg text-blue-500">{item.icon}</div>
                )}
                <div>
                  <div className="font-medium text-gray-900">{item.title}</div>
                  {item.subtitle && (
                    <div className="text-sm text-gray-600">{item.subtitle}</div>
                  )}
                </div>
              </div>
              
              {item.action && (
                <div className="text-sm text-gray-500">{item.action}</div>
              )}
            </Component>
          );
        })}
      </div>
    </ProfessionalCard>
  );
}

// Professional Feature Card
interface ProfessionalFeatureCardProps {
  variant?: 'default' | 'analytics' | 'premium';
  icon?: ReactNode;
  title: string;
  description: string;
  action?: ReactNode;
  className?: string;
}

export function ProfessionalFeatureCard({
  variant = 'default',
  icon,
  title,
  description,
  action,
  className
}: ProfessionalFeatureCardProps) {
  return (
    <ProfessionalCard variant={variant} className={cn('text-center', className)}>
      {icon && (
        <div className="text-3xl mb-4 text-blue-500 flex justify-center">
          {icon}
        </div>
      )}
      
      <h3 className="font-semibold text-lg mb-2 text-gray-900">{title}</h3>
      <p className="text-gray-600 mb-4">{description}</p>
      
      {action && (
        <div className="mt-4">
          {action}
        </div>
      )}
    </ProfessionalCard>
  );
}