'use client';

import { motion } from 'framer-motion';
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
    <motion.button
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
        transition-all duration-300
        ${disabled || loading ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'}
      `}
      whileHover={disabled || loading ? {} : { scale: 1.02, y: -2 }}
      whileTap={disabled || loading ? {} : { scale: 0.98 }}
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3 }}
    >
      {/* Windows Phone Live Tile Effect */}
      {metro && (
        <motion.div
          className="absolute inset-0 opacity-20"
          animate={{
            background: [
              'linear-gradient(45deg, transparent 30%, rgba(255,255,255,0.1) 50%, transparent 70%)',
              'linear-gradient(135deg, transparent 40%, rgba(255,255,255,0.1) 60%, transparent 80%)',
              'linear-gradient(45deg, transparent 30%, rgba(255,255,255,0.1) 50%, transparent 70%)'
            ]
          }}
          transition={{ duration: 3, repeat: Infinity }}
        />
      )}

      {/* Button shine effect */}
      <motion.div
        className="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-300"
        style={{
          background: 'linear-gradient(90deg, transparent, rgba(255,255,255,0.2), transparent)',
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

      {/* Content */}
      <div className="relative z-10 flex items-center justify-center space-x-2">
        {loading ? (
          <motion.div
            className="w-5 h-5 border-2 border-white border-t-transparent rounded-full"
            animate={{ rotate: 360 }}
            transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}
          />
        ) : (
          <>
            {icon && <span className="text-lg">{icon}</span>}
            <span>{children}</span>
          </>
        )}
      </div>

      {/* Metro accent line */}
      {metro && (
        <motion.div
          className="absolute bottom-0 left-0 h-1 bg-white/30"
          initial={{ width: '0%' }}
          animate={{ width: '100%' }}
          transition={{ duration: 0.5, delay: 0.2 }}
        />
      )}
    </motion.button>
  );
}

// Icon Button variant
interface PancakeIconButtonProps {
  variant?: 'pancake' | 'admin' | 'analytics';
  icon: string;
  size?: 'sm' | 'md' | 'lg';
  onClick?: () => void;
  badge?: string | number;
  disabled?: boolean;
}

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
    <motion.button
      onClick={onClick}
      disabled={disabled}
      className={`
        ${sizeClasses[size]}
        ${variants[variant]}
        text-white
        relative
        overflow-hidden
        shadow-lg
        ${disabled ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'}
      `}
      whileHover={disabled ? {} : { scale: 1.05, rotate: 5 }}
      whileTap={disabled ? {} : { scale: 0.95 }}
    >
      {/* Badge */}
      {badge && (
        <motion.div
          initial={{ scale: 0 }}
          animate={{ scale: 1 }}
          className="absolute -top-1 -right-1 bg-red-500 text-white rounded-full w-5 h-5 flex items-center justify-center text-xs font-bold z-10"
        >
          {badge}
        </motion.div>
      )}

      {/* Icon */}
      <span className="relative z-5">{icon}</span>

      {/* Pulse effect */}
      <motion.div
        className="absolute inset-0 bg-white"
        animate={{
          scale: [0, 1],
          opacity: [0.5, 0]
        }}
        transition={{
          duration: 1.5,
          repeat: Infinity,
          repeatDelay: 2
        }}
      />
    </motion.button>
  );
}

// Floating Action Button
interface PancakeFABProps {
  variant?: 'pancake' | 'admin' | 'analytics';
  icon: string;
  onClick?: () => void;
  position?: 'bottom-right' | 'bottom-left' | 'top-right' | 'top-left';
}

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
    <motion.button
      onClick={onClick}
      className={`
        fixed ${positions[position]}
        w-16 h-16
        ${variants[variant]}
        text-white text-2xl
        shadow-2xl
        z-50
        overflow-hidden
      `}
      whileHover={{ scale: 1.1, rotate: 15 }}
      whileTap={{ scale: 0.9 }}
      initial={{ scale: 0, rotate: -180 }}
      animate={{ scale: 1, rotate: 0 }}
      transition={{ type: 'spring', duration: 0.5 }}
    >
      <motion.div
        animate={{ rotate: [0, 360] }}
        transition={{ duration: 8, repeat: Infinity, ease: 'linear' }}
      >
        {icon}
      </motion.div>

      {/* Ripple effect */}
      <motion.div
        className="absolute inset-0 bg-white rounded-full"
        animate={{
          scale: [0, 2],
          opacity: [0.3, 0]
        }}
        transition={{
          duration: 2,
          repeat: Infinity,
          repeatDelay: 1
        }}
      />
    </motion.button>
  );
}