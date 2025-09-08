'use client';

import { 
  Users, 
  Shield, 
  BarChart3, 
  Settings, 
  Bell, 
  TrendingUp, 
  Activity, 
  Zap,
  CheckCircle,
  AlertCircle,
  XCircle,
  ArrowUp,
  ArrowDown,
  ArrowRight
} from 'lucide-react';

interface AdminIconProps {
  name: string;
  className?: string;
  size?: number;
}

export function AdminIcon({ name, className = '', size = 24 }: AdminIconProps) {
  const icons = {
    users: Users,
    permissions: Shield,
    analytics: BarChart3,
    system: Settings,
    notifications: Bell,
    eps: TrendingUp,
    realtime: Activity,
    actions: Zap,
    success: CheckCircle,
    warning: AlertCircle,
    error: XCircle,
    up: ArrowUp,
    down: ArrowDown,
    neutral: ArrowRight
  };

  const IconComponent = icons[name as keyof typeof icons] || Users;

  return (
    <div className={`inline-flex items-center justify-center ${className}`}>
      <IconComponent size={size} />
    </div>
  );
}

// Enhanced Stats Card with consistent icons
interface EnhancedStatsCardProps {
  title: string;
  value: string | number;
  subtitle?: string;
  iconName: string;
  trend?: 'up' | 'down' | 'neutral';
  trendValue?: string;
  onClick?: () => void;
  isLoading?: boolean;
  error?: boolean;
  statusColor?: 'green' | 'yellow' | 'red' | 'blue';
}

export function EnhancedStatsCard({
  title,
  value,
  subtitle,
  iconName,
  trend = 'neutral',
  trendValue,
  onClick,
  isLoading = false,
  error = false,
  statusColor = 'blue'
}: EnhancedStatsCardProps) {
  const statusColors = {
    green: 'bg-green-100 text-green-800 dark:bg-green-900/50 dark:text-green-200 border border-green-300 dark:border-green-700',
    yellow: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/50 dark:text-yellow-200 border border-yellow-300 dark:border-yellow-700',
    red: 'bg-red-100 text-red-800 dark:bg-red-900/50 dark:text-red-200 border border-red-300 dark:border-red-700',
    blue: 'bg-blue-100 text-blue-800 dark:bg-blue-900/50 dark:text-blue-200 border border-blue-300 dark:border-blue-700'
  };

  const trendColors = {
    up: 'text-green-700 dark:text-green-300 bg-green-100 dark:bg-green-900/50',
    down: 'text-red-700 dark:text-red-300 bg-red-100 dark:bg-red-900/50',
    neutral: 'text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-900/50'
  };

  if (error) {
    return (
      <div className="relative overflow-hidden backdrop-blur-sm bg-gradient-to-br from-red-50 via-red-50 to-red-100 dark:from-red-950/50 dark:via-red-950/50 dark:to-red-900/50 border-2 border-red-200/60 dark:border-red-800/60 shadow-xl shadow-red-200/40 dark:shadow-red-900/40 rounded-xl p-6">
        <div className="flex items-center justify-center h-24">
          <div className="text-center">
            <AdminIcon name="error" className="text-red-500 dark:text-red-400 mb-2" size={32} />
            <p className="text-sm text-red-600 dark:text-red-400">Error loading data</p>
          </div>
        </div>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="relative overflow-hidden backdrop-blur-sm bg-gradient-to-br from-orange-100 via-yellow-100 to-orange-200 dark:from-orange-900/70 dark:via-yellow-900/70 dark:to-orange-800/70 border-2 border-orange-300/60 dark:border-orange-700/60 shadow-xl shadow-orange-300/40 dark:shadow-orange-900/40 rounded-xl p-6">
        <div className="animate-pulse space-y-3">
          <div className="w-10 h-10 bg-orange-300 dark:bg-orange-700 rounded-lg"></div>
          <div className="w-20 h-4 bg-orange-300 dark:bg-orange-700 rounded"></div>
          <div className="w-24 h-8 bg-orange-300 dark:bg-orange-700 rounded"></div>
          <div className="w-32 h-3 bg-orange-300 dark:bg-orange-700 rounded"></div>
        </div>
      </div>
    );
  }

  return (
    <div 
      className={`relative overflow-hidden backdrop-blur-sm bg-gradient-to-br from-white via-orange-50 to-yellow-50 dark:from-gray-900 dark:via-orange-950 dark:to-yellow-950 border-2 border-orange-300 dark:border-orange-700 shadow-xl shadow-orange-300/30 dark:shadow-orange-900/50 hover:shadow-2xl hover:shadow-orange-400/40 dark:hover:shadow-orange-700/60 hover:scale-[1.02] transition-all duration-300 rounded-xl p-6 group ${onClick ? 'cursor-pointer' : ''}`}
      onClick={onClick}
    >
      {/* Shine Effect */}
      <div className="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-500 bg-gradient-to-r from-transparent via-white/10 to-transparent -skew-x-12 transform translate-x-[-100%] group-hover:translate-x-[100%] transition-transform duration-1000"></div>
      
      {/* Corner Accent */}
      <div className="absolute top-0 right-0 w-16 h-16 bg-gradient-to-br from-orange-400/20 to-yellow-400/20 dark:from-orange-600/30 dark:to-yellow-600/30 rounded-bl-3xl"></div>
      
      {/* Content */}
      <div className="relative z-10 flex items-start justify-between">
        <div className="flex-1">
          {/* Icon */}
          <div className={`w-12 h-12 rounded-xl ${statusColors[statusColor]} flex items-center justify-center mb-4 shadow-sm`}>
            <AdminIcon name={iconName} size={24} />
          </div>
          
          {/* Title */}
          <h3 className="text-sm font-bold text-gray-700 dark:text-gray-300 uppercase tracking-wide mb-2">
            {title}
          </h3>
          
          {/* Value */}
          <div className="text-4xl font-black text-gray-900 dark:text-white mb-2 leading-none drop-shadow-sm">
            {typeof value === 'number' ? value.toLocaleString() : value}
          </div>
          
          {/* Subtitle */}
          {subtitle && (
            <p className="text-sm text-gray-600 dark:text-gray-400 font-medium">
              {subtitle}
            </p>
          )}
        </div>
        
        {/* Trend */}
        {trendValue && (
          <div className={`flex items-center gap-1 text-sm font-bold ${trendColors[trend]} px-3 py-1 rounded-full shadow-sm border`}>
            <AdminIcon name={trend} size={16} />
            <span>{trendValue}</span>
          </div>
        )}
      </div>

      {/* Bottom Gradient Line */}
      <div className="absolute bottom-0 left-0 right-0 h-1 bg-gradient-to-r from-orange-400 via-yellow-400 to-orange-400 opacity-60 group-hover:opacity-100 transition-opacity duration-300"></div>
    </div>
  );
}