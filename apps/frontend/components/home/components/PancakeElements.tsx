'use client';

import React from 'react';

interface FloatingElementsProps {
  className?: string;
}

/**
 * PancakeSwap-inspired floating decorative elements
 */
export function FloatingElements({ className = '' }: FloatingElementsProps): React.JSX.Element {
  return (
    <div className={`absolute inset-0 pointer-events-none overflow-hidden ${className}`}>
      {/* Primary floating elements */}
      <div className="absolute top-20 left-10 w-16 h-16 bg-gradient-to-br from-orange-400/20 to-yellow-400/20 rounded-full animate-float" />
      <div className="absolute top-40 right-20 w-12 h-12 bg-gradient-to-br from-amber-400/20 to-orange-400/20 rounded-full animate-bounce-gentle" />
      <div className="absolute bottom-40 left-20 w-8 h-8 bg-gradient-to-br from-yellow-400/20 to-amber-400/20 rounded-full animate-pulse" />
      <div className="absolute bottom-20 right-10 w-20 h-20 bg-gradient-to-br from-orange-400/20 to-yellow-400/20 rounded-full animate-float-reverse" />
      
      {/* Secondary floating elements */}
      <div className="absolute top-1/3 left-1/3 w-6 h-6 bg-gradient-to-br from-orange-300/15 to-yellow-300/15 rounded-full animate-pulse-gentle" />
      <div className="absolute top-2/3 right-1/3 w-10 h-10 bg-gradient-to-br from-amber-300/15 to-orange-300/15 rounded-full animate-float-gentle" />
      
      {/* Floating emojis */}
      <div className="absolute top-1/4 left-1/4 text-4xl opacity-10 animate-spin-slow">
        🥞
      </div>
      <div className="absolute top-3/4 right-1/4 text-3xl opacity-15 animate-bounce-gentle">
        📈
      </div>
      <div className="absolute bottom-1/4 left-1/3 text-2xl opacity-20 animate-float">
        💰
      </div>
      <div className="absolute top-1/2 right-10 text-3xl opacity-15 animate-pulse">
        ⚡
      </div>
      <div className="absolute bottom-1/3 right-1/3 text-2xl opacity-10 animate-bounce-gentle">
        🚀
      </div>
      <div className="absolute top-10 right-1/2 text-2xl opacity-12 animate-wiggle">
        🍯
      </div>
      <div className="absolute bottom-10 left-1/2 text-2xl opacity-18 animate-float-reverse">
        ✨
      </div>

      {/* Additional glow effects */}
      <div className="absolute top-1/2 left-1/2 w-32 h-32 bg-gradient-to-r from-orange-300/5 via-yellow-300/10 to-amber-300/5 rounded-full blur-3xl animate-pulse-slow" />
    </div>
  );
}

interface PancakeButtonProps {
  children: React.ReactNode;
  variant?: 'primary' | 'secondary' | 'success';
  size?: 'sm' | 'md' | 'lg';
  onClick?: () => void;
  className?: string;
  disabled?: boolean;
}

/**
 * PancakeSwap-style button component
 */
export function PancakeButton({
  children,
  variant = 'primary',
  size = 'md',
  onClick,
  className = '',
  disabled = false,
}: PancakeButtonProps): React.JSX.Element {
  const variants = {
    primary: 'bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 text-white',
    secondary: 'bg-gradient-to-r from-blue-500 to-cyan-500 hover:from-blue-600 hover:to-cyan-600 text-white',
    success: 'bg-gradient-to-r from-green-500 to-emerald-500 hover:from-green-600 hover:to-emerald-600 text-white',
  };

  const sizes = {
    sm: 'px-4 py-2 text-sm',
    md: 'px-6 py-3 text-base',
    lg: 'px-8 py-4 text-lg',
  };

  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className={`
        ${variants[variant]}
        ${sizes[size]}
        font-bold rounded-2xl shadow-lg 
        transition-all duration-300 
        hover:scale-105 hover:shadow-xl
        active:scale-95
        focus:outline-none focus:ring-4 focus:ring-orange-500/20
        disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:scale-100
        relative overflow-hidden
        ${className}
      `}
    >
      <span className="relative z-10 flex items-center justify-center gap-2">
        {children}
      </span>
      
      {/* Shine effect */}
      <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/20 to-transparent -skew-x-12 -translate-x-full hover:translate-x-full transition-transform duration-700" />
    </button>
  );
}

interface GlowCardProps {
  children: React.ReactNode;
  glowColor?: 'orange' | 'yellow' | 'green' | 'blue';
  className?: string;
}

/**
 * PancakeSwap-style card with glow effect
 */
export function GlowCard({
  children,
  glowColor = 'orange',
  className = '',
}: GlowCardProps): React.JSX.Element {
  const glowColors = {
    orange: 'shadow-orange-500/20 hover:shadow-orange-500/40',
    yellow: 'shadow-yellow-500/20 hover:shadow-yellow-500/40',
    green: 'shadow-green-500/20 hover:shadow-green-500/40',
    blue: 'shadow-blue-500/20 hover:shadow-blue-500/40',
  };

  return (
    <div className={`
      bg-white/90 dark:bg-slate-900/90 
      backdrop-blur-sm 
      rounded-2xl 
      border border-white/20 dark:border-slate-700/50
      transition-all duration-300
      hover:scale-[1.02]
      shadow-xl ${glowColors[glowColor]}
      relative overflow-hidden
      ${className}
    `}>
      {/* Gradient border effect */}
      <div className="absolute inset-0 rounded-2xl bg-gradient-to-r from-orange-500/20 via-yellow-500/20 to-amber-500/20 opacity-0 hover:opacity-100 transition-opacity duration-300 blur-sm" />
      
      <div className="relative z-10">
        {children}
      </div>
    </div>
  );
}

interface EmojiBadgeProps {
  emoji: string;
  label: string;
  count?: number;
  variant?: 'default' | 'success' | 'warning' | 'error';
}

/**
 * PancakeSwap-style emoji badge
 */
export function EmojiBadge({
  emoji,
  label,
  count,
  variant = 'default',
}: EmojiBadgeProps): React.JSX.Element {
  const variants = {
    default: 'bg-orange-50 text-orange-700 border-orange-200 dark:bg-orange-900/20 dark:text-orange-300 dark:border-orange-700/30',
    success: 'bg-green-50 text-green-700 border-green-200 dark:bg-green-900/20 dark:text-green-300 dark:border-green-700/30',
    warning: 'bg-yellow-50 text-yellow-700 border-yellow-200 dark:bg-yellow-900/20 dark:text-yellow-300 dark:border-yellow-700/30',
    error: 'bg-red-50 text-red-700 border-red-200 dark:bg-red-900/20 dark:text-red-300 dark:border-red-700/30',
  };

  return (
    <div className={`
      inline-flex items-center gap-2 px-3 py-1.5 
      rounded-full border
      font-medium text-sm
      transition-all duration-200
      hover:scale-105
      ${variants[variant]}
    `}>
      <span className="text-lg animate-bounce-gentle">{emoji}</span>
      <span>{label}</span>
      {count !== undefined && (
        <span className="bg-current/20 text-current px-2 py-0.5 rounded-full text-xs font-bold">
          {count}
        </span>
      )}
    </div>
  );
}
