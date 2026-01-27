'use client';

import {
  Activity,
  AlertCircle,
  ArrowDown,
  ArrowRight,
  ArrowUp,
  BarChart3,
  Bell,
  CheckCircle,
  ChevronRight,
  Copy,
  CreditCard,
  MoreHorizontal,
  Settings,
  Shield,
  ShieldCheck,
  TrendingUp,
  Users,
  XCircle,
  Zap
} from 'lucide-react';

import { cn } from '@/design-system';

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

// Analytics Stats Card with slate navy dark theme
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
  className?: string;
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
  statusColor = 'purple',
  rank,
  className = ''
}: AnalyticsStatsCardProps) {
  const statusColors = {
    green: 'bg-gradient-to-br from-emerald-500 to-emerald-600 text-white shadow-lg shadow-emerald-500/20',
    yellow: 'bg-gradient-to-br from-amber-500 to-amber-600 text-white shadow-lg shadow-amber-500/20',
    red: 'bg-gradient-to-br from-red-500 to-red-600 text-white shadow-lg shadow-red-500/20',
    blue: 'bg-gradient-to-br from-blue-500 to-blue-600 text-white shadow-lg shadow-blue-500/20',
    purple: 'bg-gradient-to-br from-purple-500 to-orange-500 text-white shadow-lg shadow-purple-500/20'
  };

  const trendColors = {
    up: 'text-[#31d0aa] bg-[#31d0aa]/10 border border-[#31d0aa]/20',
    down: 'text-[#ed4b9e] bg-[#ed4b9e]/10 border border-[#ed4b9e]/20',
    neutral: 'text-slate-400 bg-white/5 border border-white/10'
  };

  if (error) {
    return (
      <div className="w-full max-w-sm mx-auto bg-white/5 backdrop-blur-xl rounded-2xl shadow-lg border border-red-500/30 overflow-hidden p-6">
        <div className="flex items-center justify-center h-24">
          <div className="text-center">
            <AnalyticsIcon name="error" className="text-red-400 mb-2" size={32} />
            <p className="text-sm text-slate-300">Error loading data</p>
          </div>
        </div>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="w-full max-w-sm mx-auto bg-white/5 backdrop-blur-xl rounded-2xl shadow-lg border border-white/10 overflow-hidden p-6">
        <div className="animate-pulse space-y-3">
          <div className="w-10 h-10 bg-white/10 rounded-lg"></div>
          <div className="w-20 h-4 bg-white/10 rounded"></div>
          <div className="w-24 h-8 bg-white/10 rounded"></div>
          <div className="w-32 h-3 bg-white/10 rounded"></div>
        </div>
      </div>
    );
  }

  // Calculate days left for next action (mock for now)
  const daysLeft = Math.floor(Math.random() * 90) + 1;
  const progressPercentage = Math.max(10, Math.min(90, (90 - daysLeft) / 90 * 100));

  const isActive = statusColor === 'green';
  const isInactive = statusColor === 'red';

  return (
    <div
      className={cn(
        "group relative bg-slate-900/40 backdrop-blur-2xl border border-white/5 rounded-[32px] shadow-xl overflow-hidden transition-all duration-300 hover:border-[#1fc7d4]/30 active:scale-[0.99] p-8",
        className,
        onClick && 'cursor-pointer'
      )}
      onClick={onClick}
    >
      <div className="absolute -right-6 -top-6 w-24 h-24 bg-[#1fc7d4]/5 rounded-full blur-3xl group-hover:bg-[#1fc7d4]/10 transition-colors" />
      {/* Header Section */}
      <div className="flex items-center justify-between mb-8">
        <div className="flex items-center gap-4">
          <div className="p-3 rounded-2xl bg-white/5 border border-white/5 text-[#1fc7d4]">
            <AnalyticsIcon name={iconName} size={24} />
          </div>
          <div>
            <h3 className="text-sm font-bold text-muted-foreground uppercase tracking-[0.2em]">{title}</h3>
            {rank && <div className="text-[10px] font-black text-[#7645d9]/60 uppercase">Rank #{rank}</div>}
          </div>
        </div>
        {trendValue && (
          <div className={cn("px-3 py-1.5 rounded-xl font-bold text-xs flex items-center gap-1.5 transition-all", trendColors[trend])}>
            <AnalyticsIcon name={trend} size={14} />
            <span>{trendValue}</span>
          </div>
        )}
      </div>

      {/* Status Badge */}
      <div className="mb-6 flex justify-center">
        <span className={`px-4 py-2 rounded-full font-medium text-sm transition-colors backdrop-blur-sm ${statusColors[statusColor]}`}>
          ● {statusColor.toUpperCase()}
        </span>
      </div>

      {/* Progress Bar */}
      <div className="mb-6">
        <div className="flex items-center justify-between mb-3">
          <span className="text-slate-200 font-medium text-sm">Status</span>
          <span className="text-slate-200 font-medium text-sm">{daysLeft}d left</span>
        </div>
        <div className="w-full bg-white/10 rounded-full h-2 backdrop-blur-sm">
          <div
            className="bg-gradient-to-r from-purple-500 to-orange-500 h-2 rounded-full transition-all duration-1000 shadow-lg shadow-purple-500/20"
            style={{ width: `${progressPercentage}%` }}
          ></div>
        </div>
      </div>

      {/* Value and Subtitle Section */}
      <div className="grid grid-cols-2 gap-4">
        {/* Main Value */}
        <div className={`rounded-xl p-4 text-center backdrop-blur-sm ${statusColors[statusColor]}`}>
          <div className="font-medium text-xs mb-1 opacity-80">Value</div>
          <div className="font-bold text-lg">
            {typeof value === 'number' ? value.toLocaleString() : value}
          </div>
        </div>

        {/* Subtitle */}
        <div className="text-center flex flex-col justify-center">
          <div className="text-slate-400 font-medium text-sm mb-1">Info</div>
          <div className="bg-gradient-to-r from-purple-400 to-orange-400 bg-clip-text text-transparent font-semibold text-base">
            {subtitle || 'Active'}
          </div>
        </div>
      </div>
    </div>
  );
}

// Summary Card matching slate navy dark theme
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
    <div className={cn(
      "group relative bg-slate-900/40 backdrop-blur-2xl border border-white/5 rounded-[32px] p-8 shadow-xl transition-all duration-300 hover:border-[#1fc7d4]/30 overflow-hidden",
      className
    )}>
      <div className="absolute -right-6 -top-6 w-24 h-24 bg-[#7645d9]/5 rounded-full blur-3xl group-hover:bg-[#7645d9]/10 transition-colors" />
      <div className="relative z-10 flex flex-col items-center text-center">
        <div className="text-xs font-black text-muted-foreground uppercase tracking-[0.2em] mb-4">{title}</div>
        <div className="text-4xl sm:text-5xl font-black bg-gradient-to-r from-[#1fc7d4] to-[#7645d9] bg-clip-text text-transparent tracking-tighter mb-2">
          {typeof value === 'number' ? value.toLocaleString() : value}
        </div>
        <div className="text-sm font-bold text-muted-foreground/60">{subtitle}</div>
      </div>
    </div>
  );
}

// User/Entity Card with enhanced UX
interface AnalyticsUserCardProps {
  avatarLabel?: string;
  address: string;
  plan?: string;
  group?: string;
  permissionsCount?: number;
  onViewDetails?: () => void;
  className?: string;
}

/**
 * High-fidelity user card with glassmorphism and rich details
 */
export function AnalyticsUserCard({
  avatarLabel = 'BB',
  address,
  plan = 'Free',
  group = 'User',
  permissionsCount = 0,
  onViewDetails,
  className = ''
}: AnalyticsUserCardProps) {
  const handleCopy = (e: React.MouseEvent) => {
    e.stopPropagation();
    navigator.clipboard.writeText(address);
    // Could add toast here
  };

  return (
    <div
      onClick={onViewDetails}
      className={cn(
        "group relative w-full overflow-hidden rounded-[24px] border border-white/5 bg-[#0f172a]/60 p-1 backdrop-blur-xl transition-all duration-300 hover:border-[#7645d9]/30 hover:shadow-2xl hover:shadow-[#7645d9]/10",
        className,
        onViewDetails && "cursor-pointer"
      )}
    >
      {/* Background Gradients */}
      <div className="absolute -left-16 -top-16 h-32 w-32 rounded-full bg-[#1fc7d4]/10 blur-[50px] transition-all duration-500 group-hover:bg-[#1fc7d4]/20" />
      <div className="absolute -right-16 -bottom-16 h-32 w-32 rounded-full bg-[#7645d9]/10 blur-[50px] transition-all duration-500 group-hover:bg-[#7645d9]/20" />

      <div className="relative flex flex-col gap-6 rounded-[20px] bg-white/[0.02] p-4 sm:p-6 lg:flex-row lg:items-center lg:justify-between lg:gap-8">

        {/* Left Section: Identity */}
        <div className="flex items-center gap-4 sm:gap-5">
          {/* Avatar with Glow */}
          <div className="relative shrink-0">
            <div className="absolute inset-0 rounded-2xl bg-gradient-to-br from-[#1fc7d4] to-[#7645d9] blur-md opacity-40 group-hover:opacity-60 transition-opacity" />
            <div className="relative flex h-14 w-14 sm:h-16 sm:w-16 items-center justify-center rounded-2xl bg-gradient-to-br from-[#1fc7d4] to-[#7645d9] text-lg sm:text-xl font-black text-white shadow-inner shadow-white/20">
              {avatarLabel && avatarLabel.substring(0, 2).toUpperCase()}
            </div>
            <div className="absolute -bottom-1 -right-1 rounded-full border-[3px] border-[#0f172a] bg-emerald-500 h-4 w-4 sm:h-5 sm:w-5" />
          </div>

          {/* Address & Id */}
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2 mb-1">
              <span className="font-mono text-base sm:text-lg font-bold text-slate-100/90 tracking-tight text-ellipsis overflow-hidden whitespace-nowrap">
                {address && address.length > 12 ? `${address.substring(0, 6)}...${address.substring(address.length - 6)}` : address}
              </span>
              <button
                onClick={handleCopy}
                className="shrink-0 rounded-lg p-1.5 text-slate-400 hover:bg-white/10 hover:text-white transition-colors"
                title="Copy Address"
              >
                <Copy size={14} />
              </button>
            </div>
            <div className="flex flex-wrap items-center gap-x-2 gap-y-1 text-xs font-medium text-slate-400/80">
              <span className="uppercase tracking-wider">Account ID</span>
              <span className="bg-white/5 px-1.5 py-0.5 rounded text-slate-300 font-mono">
                #{Math.floor(Math.random() * 10000).toString().padStart(4, '0')}
              </span>
            </div>
          </div>
        </div>

        {/* Middle Section: Stats & Badges */}
        <div className="grid grid-cols-2 gap-4 sm:flex sm:flex-wrap sm:items-center sm:gap-6 lg:gap-8">
          {/* Plan */}
          <div className="flex flex-col gap-1.5">
            <span className="text-[10px] font-bold uppercase tracking-wider text-slate-500">Plan</span>
            <div className={cn(
              "flex items-center gap-2 rounded-lg border px-3 py-1.5 text-sm font-bold backdrop-blur-md w-full sm:w-auto justify-center sm:justify-start",
              plan === 'Premium'
                ? "border-[#7645d9]/30 bg-[#7645d9]/10 text-[#7645d9]"
                : "border-slate-700 bg-slate-800/50 text-slate-300"
            )}>
              <CreditCard size={14} />
              {plan}
            </div>
          </div>

          {/* Group */}
          <div className="flex flex-col gap-1.5">
            <span className="text-[10px] font-bold uppercase tracking-wider text-slate-500">Group</span>
            <div className="flex items-center gap-2 text-sm font-medium text-slate-200 bg-white/5 sm:bg-transparent rounded-lg sm:rounded-none px-3 py-1.5 sm:p-0 justify-center sm:justify-start border border-white/5 sm:border-none">
              <AnalyticsIcon name="users" size={16} className="text-[#1fc7d4]" />
              {group}
            </div>
          </div>

          {/* Perms */}
          <div className="col-span-2 sm:col-span-1 flex flex-col gap-1.5">
            <span className="text-[10px] font-bold uppercase tracking-wider text-slate-500">Access</span>
            <div className="flex items-center justify-center sm:justify-start gap-2 rounded-lg bg-[#7645d9]/20 px-3 py-1.5 text-xs font-bold text-[#b8add2] shadow-[0_0_15px_-3px_rgba(118,69,217,0.3)]">
              <ShieldCheck size={14} />
              {permissionsCount} PERMS
            </div>
          </div>
        </div>

        {/* Right Section: Actions */}
        <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-3 mt-2 lg:mt-0 lg:border-l lg:border-white/5 lg:pl-8">
          <button
            onClick={(e) => {
              e.stopPropagation();
              onViewDetails?.();
            }}
            className="group/btn flex items-center justify-center gap-2 rounded-xl bg-white/5 px-5 py-2.5 text-sm font-bold text-white transition-all hover:bg-white/10 hover:shadow-lg hover:shadow-purple-500/10 active:scale-95 border border-white/5 hover:border-white/10 w-full sm:w-auto"
          >
            <span>View Details</span>
            <ChevronRight size={16} className="text-slate-400 transition-transform group-hover/btn:translate-x-1" />
          </button>

          <button
            className="flex h-10 w-full sm:w-10 items-center justify-center rounded-xl border border-transparent text-slate-400 transition-all hover:bg-white/5 hover:text-white hover:border-white/5 active:scale-95 bg-white/[0.02] sm:bg-transparent"
            aria-label="More actions"
          >
            <MoreHorizontal size={20} />
          </button>
        </div>
      </div>
    </div>
  );
}
