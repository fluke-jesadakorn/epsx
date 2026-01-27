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

interface AnalyticsIconProps {
  name: string;
  className?: string;
  size?: number;
}

/**
 *
 * @param root0
 * @param root0.name
 * @param root0.className
 * @param root0.size
 */
export function AnalyticsIcon({ name, className = '', size = 24 }: AnalyticsIconProps) {
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

// Analytics Stats Card with frontend design system
interface AnalyticsStatsCardProps {
  title: string;
  value: string | number;
  subtitle?: string;
  iconName: string;
  trend?: 'up' | 'down' | 'neutral';
  trendValue?: string;
  onClick?: () => void;
  isLoading?: boolean;
  error?: boolean;
  statusColor?: 'green' | 'yellow' | 'red' | 'blue' | 'purple';
  rank?: number;
}

/**
 *
 * @param root0
 * @param root0.title
 * @param root0.value
 * @param root0.subtitle
 * @param root0.iconName
 * @param root0.trend
 * @param root0.trendValue
 * @param root0.onClick
 * @param root0.isLoading
 * @param root0.error
 * @param root0.statusColor
 * @param root0.rank
 */
export function AnalyticsStatsCard({
  title,
  value,
  subtitle,
  iconName,
  trend = 'neutral',
  trendValue,
  onClick,
  isLoading = false,
  error = false,
  statusColor = 'blue',
  rank
}: AnalyticsStatsCardProps) {
  const statusColors = {
    green: 'bg-green-400 text-purple-800',
    yellow: 'bg-yellow-400 text-purple-800',
    red: 'bg-red-400 text-white',
    blue: 'bg-blue-400 text-white',
    purple: 'bg-purple-400 text-white'
  };

  const trendColors = {
    up: 'text-green-700 dark:text-green-300 bg-green-100 dark:bg-green-900/50',
    down: 'text-red-700 dark:text-red-300 bg-red-100 dark:bg-red-900/50',
    neutral: 'text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-900/50'
  };

  if (error) {
    return (
      <div className="w-full max-w-sm mx-auto bg-gradient-to-br from-red-600 via-red-700 to-red-800 rounded-3xl shadow-2xl border-2 border-gray-400/30 overflow-hidden p-6">
        <div className="flex items-center justify-center h-24">
          <div className="text-center">
            <AnalyticsIcon name="error" className="text-white mb-2" size={32} />
            <p className="text-sm text-white">Error loading data</p>
          </div>
        </div>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="w-full max-w-sm mx-auto bg-gradient-to-br from-purple-600 via-purple-700 to-purple-800 rounded-3xl shadow-2xl border-2 border-gray-400/30 overflow-hidden p-6">
        <div className="animate-pulse space-y-3">
          <div className="w-10 h-10 bg-purple-300 rounded-lg"></div>
          <div className="w-20 h-4 bg-purple-300 rounded"></div>
          <div className="w-24 h-8 bg-purple-300 rounded"></div>
          <div className="w-32 h-3 bg-purple-300 rounded"></div>
        </div>
      </div>
    );
  }

  // Calculate days left for next action (mock for now)
  const daysLeft = Math.floor(Math.random() * 90) + 1; // 1-90 days
  const progressPercentage = Math.max(10, Math.min(90, (90 - daysLeft) / 90 * 100));

  const isActive = statusColor === 'green';
  const isInactive = statusColor === 'red';

  return (
    <div
      className={`w-full max-w-sm mx-auto bg-gradient-to-br from-indigo-600 via-purple-700 to-blue-800 rounded-3xl shadow-2xl shadow-indigo-500/30 border-2 border-indigo-400/30 overflow-hidden touch-manipulation transition-all duration-300 hover:shadow-3xl hover:shadow-indigo-500/50 hover:scale-[1.02] p-6 ${onClick ? 'cursor-pointer' : ''}`}
      onClick={onClick}
    >
      {/* Header Section */}
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-4">
          {rank && (
            <div className={`w-12 h-12 rounded-full flex items-center justify-center ${
              isActive 
                ? 'bg-green-400 text-purple-800' 
                : isInactive
                ? 'bg-red-400 text-white'
                : 'bg-blue-400 text-white'
            }`}>
              <span className="font-bold text-lg">{rank}</span>
            </div>
          )}
          <h3 className="font-bold text-3xl text-white">{title}</h3>
        </div>
        {trendValue && (
          <div className={`px-4 py-2 rounded-full font-bold text-sm flex items-center gap-2 transition-colors ${
            isActive 
              ? 'bg-green-400 text-purple-800 hover:bg-green-300' 
              : isInactive
              ? 'bg-red-400 text-white hover:bg-red-300'
              : 'bg-blue-400 text-white hover:bg-blue-300'
          }`}>
            <AnalyticsIcon name={trend} size={16} />
            <span>{trendValue}</span>
          </div>
        )}
      </div>

      {/* Status Button */}
      <div className="mb-6 flex justify-center">
        <button className={`px-8 py-3 rounded-full font-bold text-lg transition-colors ${
          isActive 
            ? 'bg-green-400 text-purple-800 hover:bg-green-300' 
            : isInactive
            ? 'bg-red-400 text-white hover:bg-red-300'
            : 'bg-blue-400 text-white hover:bg-blue-300'
        }`}>
          ● {statusColor.toUpperCase()}
        </button>
      </div>

      {/* Next Action Progress */}
      <div className="mb-6">
        <div className="flex items-center justify-between mb-3">
          <span className="text-white font-medium">Status</span>
          <span className="text-white font-medium">{daysLeft}d left</span>
        </div>
        <div className="w-full bg-purple-500/50 rounded-full h-3">
          <div 
            className="bg-green-400 h-3 rounded-full transition-all duration-1000"
            style={{ width: `${progressPercentage}%` }}
          ></div>
        </div>
      </div>

      {/* Value and Subtitle Section */}
      <div className="grid grid-cols-2 gap-4">
        {/* Main Value */}
        <div className={`rounded-2xl p-4 text-center ${
          trend === 'up' 
            ? 'bg-green-400 text-purple-800' 
            : trend === 'down'
            ? 'bg-red-500 text-white'
            : 'bg-blue-400 text-white'
        }`}>
          <div className="font-bold text-sm mb-1">Value</div>
          <div className="font-bold text-xl">
            {typeof value === 'number' ? value.toLocaleString() : value}
          </div>
        </div>
        
        {/* Subtitle */}
        <div className="text-center flex flex-col justify-center">
          <div className="text-white/80 font-medium text-sm mb-1">Info</div>
          <div className="text-white font-bold text-lg">
            {subtitle || 'Active'}
          </div>
        </div>
      </div>
    </div>
  );
}

// Large Summary Card matching frontend design
interface AnalyticsSummaryCardProps {
  title: string;
  value: string | number;
  subtitle: string;
  className?: string;
}

/**
 *
 * @param root0
 * @param root0.title
 * @param root0.value
 * @param root0.subtitle
 * @param root0.className
 */
export function AnalyticsSummaryCard({
  title,
  value,
  subtitle,
  className = ''
}: AnalyticsSummaryCardProps) {
  return (
    <div className={`bg-gradient-to-br from-indigo-600 via-purple-600 to-blue-700 border-2 border-indigo-400/50 shadow-xl shadow-indigo-500/30 hover:shadow-2xl hover:shadow-indigo-500/50 hover:scale-105 transition-all duration-300 rounded-2xl p-6 text-center ${className}`}>
      <div className="text-7xl font-black text-white mb-3 drop-shadow-sm">
        {typeof value === 'number' ? value.toLocaleString() : value}
      </div>
      <div className="text-2xl font-bold text-white/90 mb-2">{title}</div>
      <div className="text-base text-white/70 font-medium">{subtitle}</div>
    </div>
  );
}