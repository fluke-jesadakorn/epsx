'use client';

import { ArrowRight, Calendar, CheckCircle2, Clock, TrendingUp, Zap } from 'lucide-react';
import { useMemo } from 'react';
import { TierBadge } from '@/shared/components/plans/tier-badge';
import { cn } from '@/shared/utils/cn';

interface PlanComparisonCardProps {
  currentPlan: {
    name: string;
    tier_level: number;
    expires_at: string | null;
    days_remaining: number;
    status: 'active' | 'expiring_soon' | 'grace_period' | 'expired' | 'no_plan';
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
  } | null;
}

export function PlanComparisonCard({ currentPlan, newPlan, upgradePreview }: PlanComparisonCardProps) {
  const actionType = useMemo(() => {
    if (!currentPlan || currentPlan.status === 'no_plan') return 'new';
    if (newPlan.tier_level > currentPlan.tier_level) return 'upgrade';
    if (newPlan.tier_level < currentPlan.tier_level) return 'downgrade';
    return 'extend';
  }, [currentPlan, newPlan]);

  const durationDays = newPlan.duration_days ?? 30;
  const totalDays = useMemo(() => {
    if (actionType === 'upgrade' && upgradePreview) {
      return durationDays + (upgradePreview.bonus_days ?? 0);
    }
    if (actionType === 'extend' && currentPlan?.days_remaining) {
      return currentPlan.days_remaining + durationDays;
    }
    return durationDays;
  }, [actionType, durationDays, currentPlan, upgradePreview]);

  const expiryDate = useMemo(() => {
    if (upgradePreview?.new_expiry_date) {
      return new Date(upgradePreview.new_expiry_date);
    }
    const date = new Date();
    date.setDate(date.getDate() + totalDays);
    return date;
  }, [upgradePreview, totalDays]);

  const actionColors = {
    new: { bg: 'bg-blue-50', border: 'border-blue-200', text: 'text-blue-700', icon: 'text-blue-500' },
    upgrade: { bg: 'bg-emerald-50', border: 'border-emerald-200', text: 'text-emerald-700', icon: 'text-emerald-500' },
    extend: { bg: 'bg-purple-50', border: 'border-purple-200', text: 'text-purple-700', icon: 'text-purple-500' },
    downgrade: { bg: 'bg-amber-50', border: 'border-amber-200', text: 'text-amber-700', icon: 'text-amber-500' },
  };

  const colors = actionColors[actionType];

  return (
    <div className={cn('rounded-lg border-2 p-6', colors.border, colors.bg)}>
      {/* Header */}
      <div className="mb-6 flex items-start justify-between">
        <div>
          <h3 className="text-lg font-semibold text-gray-900">
            {actionType === 'new' && 'New Subscription'}
            {actionType === 'upgrade' && '🚀 Upgrade Plan'}
            {actionType === 'extend' && '⏰ Extend Current Plan'}
            {actionType === 'downgrade' && '⬇️ Downgrade Plan'}
          </h3>
          <p className="mt-1 text-sm text-gray-600">
            Review the comparison before completing payment
          </p>
        </div>
        <div className={cn('rounded-full p-2', colors.bg)}>
          {actionType === 'upgrade' && <TrendingUp className={cn('h-6 w-6', colors.icon)} />}
          {actionType === 'extend' && <Clock className={cn('h-6 w-6', colors.icon)} />}
          {actionType === 'new' && <Zap className={cn('h-6 w-6', colors.icon)} />}
          {actionType === 'downgrade' && <Calendar className={cn('h-6 w-6', colors.icon)} />}
        </div>
      </div>

      {/* Comparison Grid */}
      <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
        {/* Current Plan (if exists) */}
        {currentPlan && currentPlan.status !== 'no_plan' && (
          <div className="space-y-3">
            <div className="flex items-center gap-2">
              <h4 className="text-sm font-medium text-gray-500 uppercase">Current Plan</h4>
            </div>
            <div className="rounded-lg border border-gray-200 bg-white p-4 space-y-3">
              <div className="flex items-center justify-between">
                <span className="font-semibold text-gray-900">{currentPlan.name}</span>
                <TierBadge tierLevel={currentPlan.tier_level} size="sm" showIcon />
              </div>

              {currentPlan.expires_at && (
                <div className="space-y-2">
                  <div className="flex items-center gap-2 text-sm">
                    <Clock className="h-4 w-4 text-gray-400" />
                    <span className="text-gray-600">
                      {currentPlan.days_remaining > 0 ? (
                        <>
                          <span className="font-medium text-gray-900">{currentPlan.days_remaining}</span> days remaining
                        </>
                      ) : (
                        <span className="font-medium text-red-600">Expired</span>
                      )}
                    </span>
                  </div>
                  <div className="flex items-center gap-2 text-sm">
                    <Calendar className="h-4 w-4 text-gray-400" />
                    <span className="text-gray-600">
                      Expires: <span className="font-medium text-gray-900">
                        {new Date(currentPlan.expires_at).toLocaleDateString('en-US', {
                          month: 'short',
                          day: 'numeric',
                          year: 'numeric',
                        })}
                      </span>
                    </span>
                  </div>
                </div>
              )}

              {/* Status Badge */}
              <div>
                <span
                  className={cn(
                    'inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-medium',
                    currentPlan.status === 'active' && 'bg-emerald-100 text-emerald-700',
                    currentPlan.status === 'expiring_soon' && 'bg-amber-100 text-amber-700',
                    currentPlan.status === 'grace_period' && 'bg-orange-100 text-orange-700',
                    currentPlan.status === 'expired' && 'bg-red-100 text-red-700'
                  )}
                >
                  {currentPlan.status === 'active' && '✓ Active'}
                  {currentPlan.status === 'expiring_soon' && '⚠️ Expiring Soon'}
                  {currentPlan.status === 'grace_period' && '⏰ Grace Period'}
                  {currentPlan.status === 'expired' && '✗ Expired'}
                </span>
              </div>
            </div>
          </div>
        )}

        {/* Arrow Separator (desktop only) */}
        {currentPlan && currentPlan.status !== 'no_plan' && (
          <div className="hidden md:flex items-center justify-center -mx-3">
            <ArrowRight className={cn('h-8 w-8', colors.icon)} />
          </div>
        )}

        {/* New Plan */}
        <div className={cn('space-y-3', currentPlan && currentPlan.status !== 'no_plan' ? '' : 'md:col-span-2')}>
          <div className="flex items-center gap-2">
            <h4 className="text-sm font-medium text-gray-500 uppercase">
              {actionType === 'extend' ? 'Extended Plan' : 'New Plan'}
            </h4>
          </div>
          <div className={cn('rounded-lg border-2 p-4 space-y-3', colors.border, 'bg-white')}>
            <div className="flex items-center justify-between">
              <span className="font-semibold text-gray-900">{newPlan.name}</span>
              <TierBadge tierLevel={newPlan.tier_level} size="sm" showIcon />
            </div>

            {/* Duration & Expiry */}
            <div className="space-y-2">
              <div className="flex items-center gap-2 text-sm">
                <Calendar className={cn('h-4 w-4', colors.icon)} />
                <span className="text-gray-600">
                  Duration: <span className="font-medium text-gray-900">{totalDays} days</span>
                  {upgradePreview?.bonus_days ? (
                    <span className={cn('ml-1 text-xs', colors.text)}>
                      (+{upgradePreview.bonus_days} bonus)
                    </span>
                  ) : null}
                </span>
              </div>
              <div className="flex items-center gap-2 text-sm">
                <Clock className={cn('h-4 w-4', colors.icon)} />
                <span className="text-gray-600">
                  Valid until: <span className="font-medium text-gray-900">
                    {expiryDate.toLocaleDateString('en-US', {
                      month: 'short',
                      day: 'numeric',
                      year: 'numeric',
                    })}
                  </span>
                </span>
              </div>
            </div>

            {/* Price */}
            <div className={cn('rounded-lg p-3', colors.bg)}>
              <div className="flex items-baseline justify-between">
                <span className="text-sm text-gray-600">Payment Amount</span>
                <div className="flex items-baseline gap-1">
                  <span className="text-2xl font-bold text-gray-900">${newPlan.price}</span>
                  <span className="text-sm text-gray-500">USDT</span>
                </div>
              </div>
              {upgradePreview?.credit_amount && upgradePreview.credit_amount > 0 ? (
                <div className="mt-2 text-xs text-gray-600">
                  Includes ${upgradePreview.credit_amount.toFixed(2)} credit from current plan
                </div>
              ) : null}
            </div>

            {/* Features Preview (first 3) */}
            {newPlan.features && newPlan.features.length > 0 && (
              <div className="space-y-1.5 pt-2 border-t border-gray-100">
                {newPlan.features.slice(0, 3).map((feature, idx) => (
                  <div key={idx} className="flex items-start gap-2 text-sm">
                    <CheckCircle2 className={cn('h-4 w-4 mt-0.5 flex-shrink-0', colors.icon)} />
                    <span className="text-gray-700">{feature}</span>
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

      {/* Summary Banner */}
      {actionType === 'upgrade' && upgradePreview && (
        <div className="mt-6 rounded-lg bg-emerald-50 border border-emerald-200 p-4">
          <div className="flex items-start gap-3">
            <div className="rounded-full bg-emerald-100 p-2">
              <TrendingUp className="h-5 w-5 text-emerald-600" />
            </div>
            <div className="flex-1">
              <h4 className="font-medium text-emerald-900">Upgrade Bonus</h4>
              <p className="mt-1 text-sm text-emerald-700">
                You'll receive <span className="font-semibold">{upgradePreview.bonus_days} bonus days</span> from
                your remaining ${upgradePreview.credit_amount.toFixed(2)} credit, extending your new plan to{' '}
                <span className="font-semibold">{totalDays} total days</span>.
              </p>
            </div>
          </div>
        </div>
      )}

      {actionType === 'extend' && currentPlan && (
        <div className="mt-6 rounded-lg bg-purple-50 border border-purple-200 p-4">
          <div className="flex items-start gap-3">
            <div className="rounded-full bg-purple-100 p-2">
              <Clock className="h-5 w-5 text-purple-600" />
            </div>
            <div className="flex-1">
              <h4 className="font-medium text-purple-900">Plan Extension</h4>
              <p className="mt-1 text-sm text-purple-700">
                Your current plan will be extended by <span className="font-semibold">{durationDays} days</span>,
                giving you a total of <span className="font-semibold">{totalDays} days</span> until{' '}
                {expiryDate.toLocaleDateString('en-US', { month: 'long', day: 'numeric', year: 'numeric' })}.
              </p>
            </div>
          </div>
        </div>
      )}

      {actionType === 'downgrade' && (
        <div className="mt-6 rounded-lg bg-amber-50 border border-amber-200 p-4">
          <div className="flex items-start gap-3">
            <div className="rounded-full bg-amber-100 p-2">
              <Calendar className="h-5 w-5 text-amber-600" />
            </div>
            <div className="flex-1">
              <h4 className="font-medium text-amber-900">Downgrade Notice</h4>
              <p className="mt-1 text-sm text-amber-700">
                Your current plan will remain active until it expires. The new plan will activate on{' '}
                {currentPlan?.expires_at ? (
                  new Date(currentPlan.expires_at).toLocaleDateString('en-US', {
                    month: 'long',
                    day: 'numeric',
                    year: 'numeric',
                  })
                ) : (
                  'expiration'
                )}.
              </p>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
