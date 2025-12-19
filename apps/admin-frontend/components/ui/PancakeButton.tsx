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
      bg: 'bg-gradient-to-r from-orange-500 to-yellow-500',
      hover: 'hover:from-orange-600 hover:to-yellow-600',
      text: 'text-white',
      shadow: 'shadow-orange-300',
      accent: 'border-orange-400'
    },
    admin: {
      bg: 'bg-gradient-to-r from-blue-600 to-indigo-700',
      hover: 'hover:from-blue-700 hover:to-indigo-800',
      text: 'text-white',
      shadow: 'shadow-blue-300',
      accent: 'border-blue-500'
    },
    analytics: {
      bg: 'bg-gradient-to-r from-slate-600 to-gray-700',
      hover: 'hover:from-slate-700 hover:to-gray-800',
      text: 'text-white',
      shadow: 'shadow-gray-300',
      accent: 'border-slate-500'
    },
    ghost: {
      bg: 'bg-transparent',
      hover: 'hover:bg-gray-100',
      text: 'text-gray-700',
      shadow: '',
      accent: 'border-gray-300'
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
        ${metro ? '' : 'rounded-xl'}
        ${style.shadow && 'shadow-lg'}
        border-2 ${style.accent}
        relative
        overflow-hidden
        ${disabled || loading ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'}
      `}
    >
      {/* Windows Phone Static Metro Effect */}
      {metro && (
        <div
          className="absolute inset-0 opacity-20"
          style={{
            background: 'linear-gradient(45deg, transparent 30%, rgba(255,255,255,0.1) 50%, transparent 70%)'
          }}
        />
      )}

      {/* Button static shine effect */}
      <div
        className="absolute inset-0 opacity-0 hover:opacity-100"
        style={{
          background: 'linear-gradient(90deg, transparent, rgba(255,255,255,0.2), transparent)',
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
        <div className="absolute bottom-0 left-0 w-full h-1 bg-white/30" />
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
    pancake: 'bg-gradient-to-br from-orange-500 to-yellow-500',
    admin: 'bg-gradient-to-br from-blue-600 to-indigo-700',
    analytics: 'bg-gradient-to-br from-slate-600 to-gray-700'
  };

  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className={`
        ${sizeClasses[size]}
        ${variants[variant]}
        text-white
        relative
        overflow-hidden
        shadow-lg
        hover:shadow-xl
        ${disabled ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'}
      `}
    >
      {/* Badge */}
      {badge && (
        <div className="absolute -top-1 -right-1 bg-red-500 text-white rounded-full w-5 h-5 flex items-center justify-center text-xs font-bold z-10">
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
    pancake: 'bg-gradient-to-br from-orange-500 to-yellow-500',
    admin: 'bg-gradient-to-br from-blue-600 to-indigo-700',
    analytics: 'bg-gradient-to-br from-slate-600 to-gray-700'
  };

  return (
    <button
      onClick={onClick}
      className={`
        fixed ${positions[position]}
        w-16 h-16
        ${variants[variant]}
        text-white text-2xl
        shadow-2xl
        hover:shadow-3xl
        z-50
        overflow-hidden
      `}
    >
      <div>{icon}</div>
    </button>
  );
}