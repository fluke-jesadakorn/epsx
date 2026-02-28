'use client';

import { TierBadge } from '@/shared/components/plans/tier-badge';
import { cn } from '@/shared/utils/cn';
import { fmtAmt } from '@/shared/utils/formatting/currency';
import {
  Calendar,
  CheckCircle2,
  ChevronDown,
  Clock,
  Equal,
  Plus,
  TrendingDown,
  TrendingUp,
  XCircle,
  Zap,
} from 'lucide-react';
import { useMemo } from 'react';

interface PlanComparisonCardProps {
  currentPlan: {
    name: string;
    tier_level: number;
    expires_at: string | null;
    days_remaining: number;
    status: 'active' | 'expiring_soon' | 'grace_period' | 'expired' | 'no_plan';
    features?: string[];
    price?: number;
  } | null;
  newPlan: {
    id: string;
    name: string;
    tier_level: number;
    price: number;
    duration_days?: number;
    features: string[];
  };
  upgradePreview?: {
    credit_amount: number;
    bonus_days: number;
    new_expiry_date: string;
    payment_required?: number;
  } | null;
}

type ActionType = 'new' | 'upgrade' | 'extend' | 'downgrade';

const ACTION_CONFIG = {
  new: {
    title: 'New Subscription',
    Icon: Zap,
    gradient: 'from-cyan-500 to-blue-600',
    accent: 'text-cyan-400',
    accentBg: 'bg-cyan-500/10',
    border: 'border-cyan-500/30',
    ring: 'ring-cyan-500/20',
    glow: 'shadow-cyan-500/10',
    noticeBg: 'bg-gradient-to-r from-cyan-500/10 to-blue-500/10',
    noticeBorder: 'border-cyan-500/20',
    noticeIcon: 'bg-cyan-500/20 text-cyan-400',
  },
  upgrade: {
    title: 'Upgrade Plan',
    Icon: TrendingUp,
    gradient: 'from-emerald-500 to-cyan-500',
    accent: 'text-emerald-400',
    accentBg: 'bg-emerald-500/10',
    border: 'border-emerald-500/30',
    ring: 'ring-emerald-500/20',
    glow: 'shadow-emerald-500/10',
    noticeBg: 'bg-gradient-to-r from-emerald-500/10 to-cyan-500/10',
    noticeBorder: 'border-emerald-500/20',
    noticeIcon: 'bg-emerald-500/20 text-emerald-400',
  },
  extend: {
    title: 'Extend Plan',
    Icon: Clock,
    gradient: 'from-violet-500 to-purple-600',
    accent: 'text-violet-400',
    accentBg: 'bg-violet-500/10',
    border: 'border-violet-500/30',
    ring: 'ring-violet-500/20',
    glow: 'shadow-violet-500/10',
    noticeBg: 'bg-gradient-to-r from-violet-500/10 to-purple-500/10',
    noticeBorder: 'border-violet-500/20',
    noticeIcon: 'bg-violet-500/20 text-violet-400',
  },
  downgrade: {
    title: 'Downgrade Plan',
    Icon: TrendingDown,
    gradient: 'from-amber-500 to-orange-500',
    accent: 'text-amber-400',
    accentBg: 'bg-amber-500/10',
    border: 'border-amber-500/30',
    ring: 'ring-amber-500/20',
    glow: 'shadow-amber-500/10',
    noticeBg: 'bg-gradient-to-r from-amber-500/10 to-orange-500/10',
    noticeBorder: 'border-amber-500/20',
    noticeIcon: 'bg-amber-500/20 text-amber-400',
  },
} as const;

const STATUS_STYLES = {
  active: 'bg-emerald-500/15 text-emerald-400 ring-1 ring-emerald-500/30',
  expiring_soon: 'bg-amber-500/15 text-amber-400 ring-1 ring-amber-500/30',
  grace_period: 'bg-orange-500/15 text-orange-400 ring-1 ring-orange-500/30',
  expired: 'bg-red-500/15 text-red-400 ring-1 ring-red-500/30',
} as const;

const STATUS_LABELS = {
  active: 'Active',
  expiring_soon: 'Expiring Soon',
  grace_period: 'Grace Period',
  expired: 'Expired',
} as const;

function fmtDate(d: string | Date) {
  const date = typeof d === 'string' ? new Date(d) : d;
  return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
}

/* ── Feature Comparison ─────────────────────────────────────── */
function FeatureTable({
  features,
  cfg,
}: {
  features: Array<{ text: string; inCurrent: boolean; inNew: boolean }>;
  cfg: (typeof ACTION_CONFIG)[ActionType];
}) {
  if (features.length === 0) { return null; }

  return (
    <div className="mt-5">
      <h4 className="text-xs font-semibold uppercase tracking-wider text-gray-400 mb-3">
        Feature Comparison
      </h4>

      {/* Desktop */}
      <div className="hidden sm:block rounded-xl overflow-hidden border border-white/[0.06]">
        <table className="w-full">
          <thead>
            <tr className="bg-white/[0.03]">
              <th className="px-4 py-2.5 text-left text-[11px] font-medium text-gray-500 uppercase tracking-wider">
                Feature
              </th>
              <th className="px-4 py-2.5 text-center text-[11px] font-medium text-gray-500 uppercase tracking-wider">
                Current
              </th>
              <th className="px-4 py-2.5 text-center text-[11px] font-medium text-gray-500 uppercase tracking-wider">
                New
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-white/[0.04]">
            {features.map((f, i) => (
              <tr key={i} className="hover:bg-white/[0.02] transition-colors">
                <td className="px-4 py-2.5 text-sm text-gray-300">{f.text}</td>
                <td className="px-4 py-2.5 text-center">
                  {f.inCurrent ? (
                    <CheckCircle2 className="h-4 w-4 text-emerald-400 inline-block" />
                  ) : (
                    <XCircle className="h-4 w-4 text-gray-600 inline-block" />
                  )}
                </td>
                <td className="px-4 py-2.5 text-center">
                  {f.inNew ? (
                    <CheckCircle2 className={cn('h-4 w-4 inline-block', cfg.accent)} />
                  ) : (
                    <XCircle className="h-4 w-4 text-gray-600 inline-block" />
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Mobile */}
      <div className="sm:hidden space-y-2">
        {features.map((f, i) => (
          <div key={i} className="rounded-lg bg-white/[0.03] border border-white/[0.06] p-3">
            <p className="text-sm font-medium text-gray-200 mb-2">{f.text}</p>
            <div className="flex gap-4 text-xs">
              <span className="flex items-center gap-1.5 text-gray-400">
                Current:
                {f.inCurrent ? (
                  <CheckCircle2 className="h-3.5 w-3.5 text-emerald-400" />
                ) : (
                  <XCircle className="h-3.5 w-3.5 text-gray-600" />
                )}
              </span>
              <span className="flex items-center gap-1.5 text-gray-400">
                New:
                {f.inNew ? (
                  <CheckCircle2 className={cn('h-3.5 w-3.5', cfg.accent)} />
                ) : (
                  <XCircle className="h-3.5 w-3.5 text-gray-600" />
                )}
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

/* ── Extend Day Breakdown ───────────────────────────────────── */
function DayBreakdown({
  remaining,
  added,
  total,
}: {
  remaining: number;
  added: number;
  total: number;
}) {
  return (
    <div className="flex flex-wrap items-center gap-2 text-sm">
      <div className="flex items-center gap-1.5 rounded-lg bg-violet-500/10 border border-violet-500/20 px-3 py-1.5">
        <Clock className="h-3.5 w-3.5 text-violet-400" />
        <span className="font-semibold text-violet-300">{remaining}d</span>
        <span className="text-[10px] text-violet-400/70 uppercase">left</span>
      </div>
      <Plus className="h-3.5 w-3.5 text-violet-400/60" />
      <div className="flex items-center gap-1.5 rounded-lg bg-violet-500/10 border border-violet-500/20 px-3 py-1.5">
        <Zap className="h-3.5 w-3.5 text-violet-400" />
        <span className="font-semibold text-violet-300">{added}d</span>
        <span className="text-[10px] text-violet-400/70 uppercase">new</span>
      </div>
      <Equal className="h-3.5 w-3.5 text-violet-400/60" />
      <div className="flex items-center gap-1.5 rounded-lg bg-violet-500/15 border-2 border-violet-500/30 px-3 py-1.5">
        <Calendar className="h-3.5 w-3.5 text-violet-300" />
        <span className="font-bold text-violet-200">{total}d</span>
        <span className="text-[10px] text-violet-300/80 uppercase">total</span>
      </div>
    </div>
  );
}

/* ── Main Component ─────────────────────────────────────────── */
export function PlanComparisonCard({ currentPlan, newPlan, upgradePreview }: PlanComparisonCardProps) {
  const actionType: ActionType = useMemo(() => {
    if (!currentPlan || currentPlan.status === 'no_plan') { return 'new'; }
    if (newPlan.tier_level > currentPlan.tier_level) { return 'upgrade'; }
    if (newPlan.tier_level < currentPlan.tier_level) { return 'downgrade'; }
    return 'extend';
  }, [currentPlan, newPlan]);

  const cfg = ACTION_CONFIG[actionType];
  const durationDays = newPlan.duration_days ?? 30;

  const totalDays = useMemo(() => {
    if (actionType === 'upgrade' && upgradePreview) { return durationDays + (upgradePreview.bonus_days ?? 0); }
    if (actionType === 'extend' && currentPlan?.days_remaining) { return currentPlan.days_remaining + durationDays; }
    return durationDays;
  }, [actionType, durationDays, currentPlan, upgradePreview]);

  const expiryDate = useMemo(() => {
    if (upgradePreview?.new_expiry_date) { return new Date(upgradePreview.new_expiry_date); }
    const d = new Date();
    d.setDate(d.getDate() + totalDays);
    return d;
  }, [upgradePreview, totalDays]);

  const allFeatures = useMemo(() => {
    if (actionType === 'extend' || actionType === 'new' || !currentPlan?.features) { return []; }
    const cur = new Set(currentPlan.features);
    const nxt = new Set(newPlan.features);
    return [...new Set([...cur, ...nxt])].map(t => ({ text: t, inCurrent: cur.has(t), inNew: nxt.has(t) }));
  }, [actionType, currentPlan?.features, newPlan.features]);

  const hasCurrent = Boolean(currentPlan && currentPlan.status !== 'no_plan');
  const { Icon } = cfg;

  return (
    <div className={cn(
      'relative rounded-2xl border border-white/[0.08] bg-gradient-to-b from-white/[0.04] to-transparent',
      'shadow-2xl', cfg.glow,
      'overflow-hidden',
    )}>
      {/* Top accent bar */}
      <div className={cn('h-[3px] bg-gradient-to-r', cfg.gradient)} />

      <div className="p-5 sm:p-6">
        {/* ── Header ─────────────────────────────────────── */}
        <div className="flex items-start justify-between mb-6">
          <div>
            <h3 className="text-lg font-bold text-white flex items-center gap-2">
              {cfg.title}
            </h3>
            <p className="mt-0.5 text-sm text-gray-400">
              Review the comparison before completing payment
            </p>
          </div>
          <div className={cn('rounded-xl p-2.5', cfg.accentBg, 'ring-1', cfg.ring)}>
            <Icon className={cn('h-5 w-5', cfg.accent)} />
          </div>
        </div>

        {/* ── Plan Cards (Vertical Stack) ────────────────── */}
        <div className="space-y-0">
          {/* Current Plan */}
          {hasCurrent && currentPlan && (
            <>
              <div>
                <p className="text-[11px] font-semibold uppercase tracking-wider text-gray-500 mb-2">
                  Current Plan
                </p>
                <div className="rounded-xl border border-white/[0.08] bg-white/[0.03] p-4 space-y-3">
                  <div className="flex items-center justify-between">
                    <span className="text-base font-semibold text-white">{currentPlan.name}</span>
                    <TierBadge tierLevel={currentPlan.tier_level} size="sm" showIcon />
                  </div>

                  {currentPlan.expires_at && (
                    <div className="flex flex-wrap gap-x-4 gap-y-1 text-sm">
                      <span className="flex items-center gap-1.5 text-gray-400">
                        <Clock className="h-3.5 w-3.5 text-gray-500" />
                        {currentPlan.days_remaining > 0 ? (
                          <>
                            <span className="font-medium text-white">{currentPlan.days_remaining}</span> days left
                          </>
                        ) : (
                          <span className="font-medium text-red-400">Expired</span>
                        )}
                      </span>
                      <span className="flex items-center gap-1.5 text-gray-400">
                        <Calendar className="h-3.5 w-3.5 text-gray-500" />
                        Expires {fmtDate(currentPlan.expires_at)}
                      </span>
                    </div>
                  )}

                  <span className={cn(
                    'inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-medium',
                    STATUS_STYLES[currentPlan.status === 'no_plan' ? 'expired' : currentPlan.status],
                  )}>
                    {currentPlan.status === 'active' && <CheckCircle2 className="h-3 w-3" />}
                    {STATUS_LABELS[currentPlan.status === 'no_plan' ? 'expired' : currentPlan.status]}
                  </span>
                </div>
              </div>

              {/* ── Arrow Down ─────────────────────────────── */}
              <div className="flex flex-col items-center py-3">
                <div className="flex flex-col items-center gap-0.5">
                  <div className={cn('h-4 w-px bg-gradient-to-b', cfg.gradient, 'opacity-40')} />
                  <div className={cn(
                    'rounded-full p-1.5',
                    cfg.accentBg, 'ring-1', cfg.ring,
                  )}>
                    <ChevronDown className={cn('h-4 w-4', cfg.accent, 'animate-bounce')} style={{ animationDuration: '2s' }} />
                  </div>
                  <div className={cn('h-4 w-px bg-gradient-to-b', cfg.gradient, 'opacity-40')} />
                </div>
              </div>
            </>
          )}

          {/* New Plan */}
          <div>
            <p className="text-[11px] font-semibold uppercase tracking-wider text-gray-500 mb-2">
              {actionType === 'extend' ? 'Extended Plan' : 'New Plan'}
            </p>
            <div className={cn(
              'rounded-xl border-2 p-4 space-y-3',
              cfg.border,
              'bg-gradient-to-br from-white/[0.04] to-transparent',
            )}>
              <div className="flex items-center justify-between">
                <span className="text-base font-semibold text-white">{newPlan.name}</span>
                <TierBadge tierLevel={newPlan.tier_level} size="sm" showIcon />
              </div>

              {/* Duration & Expiry */}
              <div className="flex flex-wrap gap-x-4 gap-y-1 text-sm">
                <span className="flex items-center gap-1.5 text-gray-400">
                  <Calendar className={cn('h-3.5 w-3.5', cfg.accent)} />
                  Duration: <span className="font-medium text-white ml-0.5">{totalDays} days</span>
                  {upgradePreview?.bonus_days ? (
                    <span className={cn('text-xs', cfg.accent)}>(+{upgradePreview.bonus_days} bonus)</span>
                  ) : null}
                </span>
                <span className="flex items-center gap-1.5 text-gray-400">
                  <Clock className={cn('h-3.5 w-3.5', cfg.accent)} />
                  Valid until: <span className="font-medium text-white ml-0.5">{fmtDate(expiryDate)}</span>
                </span>
              </div>

              {/* Price Breakdown */}
              <div className={cn('rounded-lg p-3', cfg.accentBg, 'ring-1', cfg.ring)}>
                {actionType === 'upgrade' && upgradePreview && (upgradePreview.payment_required === 0 || upgradePreview.payment_required === undefined) ? (
                  <div className="space-y-2">
                    <div className="flex justify-between text-sm">
                      <span className="text-gray-400">Days Converted</span>
                      <span className="font-medium text-emerald-400">
                        {currentPlan?.days_remaining ?? 0}d → {totalDays}d
                      </span>
                    </div>
                    <div className="border-t border-white/[0.08] pt-2 flex justify-between items-baseline">
                      <span className="text-sm font-medium text-gray-300">Amount Due</span>
                      <div className="flex items-baseline gap-1">
                        <span className="text-2xl font-bold text-emerald-400">$0</span>
                        <span className="text-xs text-emerald-500 uppercase">FREE</span>
                      </div>
                    </div>
                  </div>
                ) : actionType === 'downgrade' && currentPlan?.price ? (
                  <div className="space-y-2">
                    <div className="flex justify-between text-sm">
                      <span className="text-gray-400">Current Plan</span>
                      <span className="font-medium text-white">${fmtAmt(Number(currentPlan.price))}/mo</span>
                    </div>
                    <div className="flex justify-between text-sm">
                      <span className="text-gray-400">New Plan</span>
                      <span className="font-medium text-white">${fmtAmt(Number(newPlan.price))}/mo</span>
                    </div>
                    <div className="border-t border-white/[0.08] pt-2 flex justify-between items-baseline">
                      <span className="text-sm font-medium text-emerald-400">Monthly Savings</span>
                      <span className="text-xl font-bold text-emerald-400">
                        ${fmtAmt(Number(currentPlan.price) - Number(newPlan.price))}
                      </span>
                    </div>
                  </div>
                ) : (
                  <div className="flex justify-between items-baseline">
                    <span className="text-sm text-gray-400">Payment Amount</span>
                    <div className="flex items-baseline gap-1">
                      <span className="text-2xl font-bold text-white">${fmtAmt(newPlan.price)}</span>
                      <span className="text-xs text-gray-500 uppercase">USDT</span>
                    </div>
                  </div>
                )}
              </div>

              {/* Features Preview (new/extend only) */}
              {(actionType === 'new' || actionType === 'extend') && newPlan.features.length > 0 && (
                <div className="space-y-1.5 pt-2 border-t border-white/[0.06]">
                  {newPlan.features.slice(0, 3).map((f, i) => (
                    <div key={i} className="flex items-start gap-2 text-sm">
                      <CheckCircle2 className={cn('h-4 w-4 mt-0.5 shrink-0', cfg.accent)} />
                      <span className="text-gray-300">{f}</span>
                    </div>
                  ))}
                  {newPlan.features.length > 3 && (
                    <p className="text-xs text-gray-500 pl-6">
                      +{newPlan.features.length - 3} more features
                    </p>
                  )}
                </div>
              )}
            </div>
          </div>
        </div>

        {/* ── Feature Comparison Table (upgrade/downgrade) ── */}
        {(actionType === 'upgrade' || actionType === 'downgrade') && (
          <FeatureTable features={allFeatures} cfg={cfg} />
        )}

        {/* ── Notice Banners ─────────────────────────────── */}
        {actionType === 'upgrade' && upgradePreview && (
          <div className={cn('mt-5 rounded-xl p-4 border', cfg.noticeBg, cfg.noticeBorder)}>
            <div className="flex items-start gap-3">
              <div className={cn('rounded-lg p-2 shrink-0', cfg.noticeIcon)}>
                <TrendingUp className="h-4 w-4" />
              </div>
              <div>
                <h4 className="text-sm font-semibold text-white">Free Upgrade — Day Conversion</h4>
                <p className="mt-1 text-sm text-gray-400 leading-relaxed">
                  Your remaining{' '}
                  <span className="font-semibold text-white">{currentPlan?.days_remaining ?? 0} days</span>{' '}
                  will be converted to{' '}
                  <span className="font-semibold text-emerald-400">{totalDays} days</span>{' '}
                  on {newPlan.name}. No payment required.
                </p>
              </div>
            </div>
          </div>
        )}

        {actionType === 'extend' && currentPlan && (
          <div className={cn('mt-5 rounded-xl p-4 border', cfg.noticeBg, cfg.noticeBorder)}>
            <div className="flex items-start gap-3">
              <div className={cn('rounded-lg p-2 shrink-0', cfg.noticeIcon)}>
                <Clock className="h-4 w-4" />
              </div>
              <div className="space-y-3">
                <h4 className="text-sm font-semibold text-white">Plan Extension</h4>
                <DayBreakdown
                  remaining={currentPlan.days_remaining}
                  added={durationDays}
                  total={totalDays}
                />
                <p className="text-sm text-gray-400">
                  Valid until{' '}
                  <span className="font-semibold text-white">
                    {expiryDate.toLocaleDateString('en-US', { month: 'long', day: 'numeric', year: 'numeric' })}
                  </span>
                </p>
              </div>
            </div>
          </div>
        )}

        {actionType === 'downgrade' && (
          <div className={cn('mt-5 rounded-xl p-4 border', cfg.noticeBg, cfg.noticeBorder)}>
            <div className="flex items-start gap-3">
              <div className={cn('rounded-lg p-2 shrink-0', cfg.noticeIcon)}>
                <Calendar className="h-4 w-4" />
              </div>
              <div>
                <h4 className="text-sm font-semibold text-white">Downgrade Not Available</h4>
                <p className="mt-1 text-sm text-gray-400 leading-relaxed">
                  Plan downgrades are not supported. You can only upgrade to a higher-tier plan.
                </p>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
