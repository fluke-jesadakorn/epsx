'use client';

import { ReactNode } from 'react';

interface PancakeButtonProps {
  variant?: 'pancake' | 'admin' | 'analytics' | 'ghost';
  size?: 'sm' | 'md' | 'lg' | 'xl';
  children: ReactNode;
  onClick?: () => void;
  disabled?: boolean;
  loading?: boolean;
  icon?: string;
  fullWidth?: boolean;
  metro?: boolean;
}

/**
 *
 * @param root0
 * @param root0.variant
 * @param root0.size
 * @param root0.children
 * @param root0.onClick
 * @param root0.disabled
 * @param root0.loading
 * @param root0.icon
 * @param root0.fullWidth
 * @param root0.metro
 */
export function PancakeButton({
  variant = 'pancake',
  size = 'md',
  children,
  onClick,
  disabled = false,
  loading = false,
  icon,
  fullWidth = false,
  metro = true
}: PancakeButtonProps) {
  const sizeClasses = {
    sm: 'px-4 py-2 text-sm',
    md: 'px-6 py-3 text-base',
    lg: 'px-8 py-4 text-lg',
    xl: 'px-10 py-5 text-xl'
  };

  const variants = {
    pancake: {
      bg: 'bg-primary',
      hover: 'hover:opacity-90',
      text: 'text-primary-foreground',
      shadow: 'shadow-primary/20',
      accent: 'border-primary/20'
    },
    admin: {
      bg: 'bg-primary',
      hover: 'hover:opacity-90',
      text: 'text-primary-foreground',
      shadow: 'shadow-primary/20',
      accent: 'border-primary/20'
    },
    analytics: {
      bg: 'bg-secondary',
      hover: 'hover:opacity-90',
      text: 'text-secondary-foreground',
      shadow: 'shadow-secondary/20',
      accent: 'border-secondary/20'
    },
    ghost: {
      bg: 'bg-transparent',
      hover: 'hover:bg-accent hover:text-accent-foreground',
      text: 'text-foreground',
      shadow: '',
      accent: 'border-input'
    }
  };

  const style = variants[variant];

  return (
    <button
      onClick={onClick}
      disabled={disabled || loading}
      className={`
        ${fullWidth ? 'w-full' : ''}
        ${sizeClasses[size]}
        ${style.bg}
        ${style.hover}
        ${style.text}
        font-bold
        ${metro ? 'rounded-none' : 'rounded-xl'}
        ${style.shadow && 'shadow-lg'}
        border ${style.accent}
        relative
        overflow-hidden
        transition-all
        duration-200
        ${disabled || loading ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'}
      `}
    >
      {/* Windows Phone Static Metro Effect */}
      {metro && (
        <div
          className="absolute inset-0 opacity-10"
          style={{
            background: 'linear-gradient(45deg, transparent 30%, rgba(255,255,255,0.2) 50%, transparent 70%)'
          }}
        />
      )}

      {/* Button static shine effect */}
      <div
        className="absolute inset-0 opacity-0 hover:opacity-100 transition-opacity"
        style={{
          background: 'linear-gradient(90deg, transparent, rgba(255,255,255,0.1), transparent)',
        }}
      />

      {/* Content */}
      <div className="relative z-10 flex items-center justify-center space-x-2">
        {loading ? (
          <div className="flex items-center gap-2">
            <svg className="h-4 w-4 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
              <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            <span className="text-sm">Loading...</span>
          </div>
        ) : (
          <>
            {icon && <span className="text-lg">{icon}</span>}
            <span>{children}</span>
          </>
        )}
      </div>

      {/* Metro accent line */}
      {metro && (
        <div className="absolute bottom-0 left-0 w-full h-0.5 bg-white/20" />
      )}
    </button>
  );
}

interface PancakeIconButtonProps {
  variant?: 'pancake' | 'admin' | 'analytics';
  icon: string;
  size?: 'sm' | 'md' | 'lg';
  onClick?: () => void;
  badge?: string | number;
  disabled?: boolean;
}

/**
 *
 * @param root0
 * @param root0.variant
 * @param root0.icon
 * @param root0.size
 * @param root0.onClick
 * @param root0.badge
 * @param root0.disabled
 */
export function PancakeIconButton({
  variant = 'pancake',
  icon,
  size = 'md',
  onClick,
  badge,
  disabled = false
}: PancakeIconButtonProps) {
  const sizeClasses = {
    sm: 'w-8 h-8 text-sm',
    md: 'w-12 h-12 text-lg',
    lg: 'w-16 h-16 text-xl'
  };

  const variants = {
    pancake: 'bg-primary',
    admin: 'bg-primary',
    analytics: 'bg-secondary'
  };

  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className={`
        ${sizeClasses[size]}
        ${variants[variant]}
        text-primary-foreground
        relative
        overflow-hidden
        shadow-lg
        shadow-primary/20
        hover:opacity-90
        transition-all
        ${disabled ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'}
      `}
    >
      {/* Badge */}
      {badge && (
        <div className="absolute -top-1 -right-1 bg-destructive text-destructive-foreground rounded-full w-5 h-5 flex items-center justify-center text-xs font-bold z-10">
          {badge}
        </div>
      )}

      {/* Icon */}
      <span className="relative z-5">{icon}</span>
    </button>
  );
}

// Floating Action Button
interface PancakeFABProps {
  variant?: 'pancake' | 'admin' | 'analytics';
  icon: string;
  onClick?: () => void;
  position?: 'bottom-right' | 'bottom-left' | 'top-right' | 'top-left';
}

/**
 *
 * @param root0
 * @param root0.variant
 * @param root0.icon
 * @param root0.onClick
 * @param root0.position
 */
export function PancakeFAB({
  variant = 'pancake',
  icon,
  onClick,
  position = 'bottom-right'
}: PancakeFABProps) {
  const positions = {
    'bottom-right': 'bottom-6 right-6',
    'bottom-left': 'bottom-6 left-6',
    'top-right': 'top-6 right-6',
    'top-left': 'top-6 left-6'
  };

  const variants = {
    pancake: 'bg-primary',
    admin: 'bg-primary',
    analytics: 'bg-secondary'
  };

  return (
    <button
      onClick={onClick}
      className={`
        fixed ${positions[position]}
        w-16 h-16
        ${variants[variant]}
        text-primary-foreground text-2xl
        shadow-2xl
        shadow-primary/30
        hover:opacity-90
        transition-all
        z-50
        overflow-hidden
      `}
    >
      <div className="flex items-center justify-center h-full">{icon}</div>
    </button>
  );
}