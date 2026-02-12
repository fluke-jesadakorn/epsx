'use client';

import { ArrowRight, Calendar, CheckCircle2, Clock, TrendingUp, Zap, XCircle, Plus, Equal, DollarSign, TrendingDown } from 'lucide-react';
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

// Feature Comparison Table Component
function FeatureComparisonTable({
  features,
  actionType,
  colors,
}: {
  features: Array<{ text: string; inCurrent: boolean; inNew: boolean }>;
  actionType: string;
  colors: { text: string; icon: string };
}) {
  if (features.length === 0) return null;

  return (
    <div className="mt-6">
      <h4 className="text-sm font-semibold text-gray-900 dark:text-white mb-3">
        Feature Comparison
      </h4>

      {/* Desktop Table */}
      <div className="hidden md:block overflow-hidden rounded-lg border border-gray-200 dark:border-gray-700">
        <table className="w-full">
          <thead className="bg-gray-50 dark:bg-gray-800/50">
            <tr>
              <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                Feature
              </th>
              <th className="px-4 py-3 text-center text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                Current
              </th>
              <th className="px-4 py-3 text-center text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                New
              </th>
            </tr>
          </thead>
          <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
            {features.map((feature, idx) => (
              <tr key={idx}>
                <td className="px-4 py-3 text-sm text-gray-900 dark:text-white">
                  {feature.text}
                </td>
                <td className="px-4 py-3 text-center">
                  {feature.inCurrent ? (
                    <CheckCircle2 className="h-5 w-5 text-green-500 dark:text-green-400 inline-block" />
                  ) : (
                    <XCircle className="h-5 w-5 text-gray-300 dark:text-gray-600 inline-block" />
                  )}
                </td>
                <td className="px-4 py-3 text-center">
                  {feature.inNew ? (
                    <CheckCircle2 className={cn('h-5 w-5 inline-block', colors.icon)} />
                  ) : (
                    <XCircle className="h-5 w-5 text-gray-300 dark:text-gray-600 inline-block" />
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Mobile Stacked Cards */}
      <div className="md:hidden space-y-3">
        {features.map((feature, idx) => (
          <div
            key={idx}
            className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-3"
          >
            <p className="text-sm font-medium text-gray-900 dark:text-white mb-2">
              {feature.text}
            </p>
            <div className="flex gap-4">
              <div className="flex items-center gap-2">
                <span className="text-xs text-gray-500 dark:text-gray-400">Current:</span>
                {feature.inCurrent ? (
                  <CheckCircle2 className="h-4 w-4 text-green-500 dark:text-green-400" />
                ) : (
                  <XCircle className="h-4 w-4 text-gray-300 dark:text-gray-600" />
                )}
              </div>
              <div className="flex items-center gap-2">
                <span className="text-xs text-gray-500 dark:text-gray-400">New:</span>
                {feature.inNew ? (
                  <CheckCircle2 className={cn('h-4 w-4', colors.icon)} />
                ) : (
                  <XCircle className="h-4 w-4 text-gray-300 dark:text-gray-600" />
                )}
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
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
    new: { bg: 'bg-blue-50 dark:bg-blue-900/20', border: 'border-blue-200 dark:border-blue-700', text: 'text-blue-700 dark:text-blue-400', icon: 'text-blue-500 dark:text-blue-400' },
    upgrade: { bg: 'bg-emerald-50 dark:bg-emerald-900/20', border: 'border-emerald-200 dark:border-emerald-700', text: 'text-emerald-700 dark:text-emerald-400', icon: 'text-emerald-500 dark:text-emerald-400' },
    extend: { bg: 'bg-purple-50 dark:bg-purple-900/20', border: 'border-purple-200 dark:border-purple-700', text: 'text-purple-700 dark:text-purple-400', icon: 'text-purple-500 dark:text-purple-400' },
    downgrade: { bg: 'bg-amber-50 dark:bg-amber-900/20', border: 'border-amber-200 dark:border-amber-700', text: 'text-amber-700 dark:text-amber-400', icon: 'text-amber-500 dark:text-amber-400' },
  };

  const allFeatures = useMemo(() => {
    if (actionType === 'extend' || actionType === 'new' || !currentPlan?.features) return [];

    const currentTexts = new Set(currentPlan.features);
    const newTexts = new Set(newPlan.features);
    const all = new Set([...currentTexts, ...newTexts]);

    return [...all].map(text => ({
      text,
      inCurrent: currentTexts.has(text),
      inNew: newTexts.has(text),
    }));
  }, [actionType, currentPlan?.features, newPlan.features]);

  const colors = actionColors[actionType];

  return (
    <div className={cn('rounded-lg border-2 p-6', colors.border, colors.bg)}>
      {/* Header */}
      <div className="mb-6 flex items-start justify-between">
        <div>
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
            {actionType === 'new' && 'New Subscription'}
            {actionType === 'upgrade' && '🚀 Upgrade Plan'}
            {actionType === 'extend' && '⏰ Extend Current Plan'}
            {actionType === 'downgrade' && '⬇️ Downgrade Plan'}
          </h3>
          <p className="mt-1 text-sm text-gray-600 dark:text-gray-400">
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
              <h4 className="text-sm font-medium text-gray-500 dark:text-gray-400 uppercase">Current Plan</h4>
            </div>
            <div className="rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 p-4 space-y-3">
              <div className="flex items-center justify-between">
                <span className="font-semibold text-gray-900 dark:text-white">{currentPlan.name}</span>
                <TierBadge tierLevel={currentPlan.tier_level} size="sm" showIcon />
              </div>

              {currentPlan.expires_at && (
                <div className="space-y-2">
                  <div className="flex items-center gap-2 text-sm">
                    <Clock className="h-4 w-4 text-gray-400 dark:text-gray-500" />
                    <span className="text-gray-600 dark:text-gray-400">
                      {currentPlan.days_remaining > 0 ? (
                        <>
                          <span className="font-medium text-gray-900 dark:text-white">{currentPlan.days_remaining}</span> days remaining
                        </>
                      ) : (
                        <span className="font-medium text-red-600 dark:text-red-400">Expired</span>
                      )}
                    </span>
                  </div>
                  <div className="flex items-center gap-2 text-sm">
                    <Calendar className="h-4 w-4 text-gray-400 dark:text-gray-500" />
                    <span className="text-gray-600 dark:text-gray-400">
                      Expires: <span className="font-medium text-gray-900 dark:text-white">
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
                    currentPlan.status === 'active' && 'bg-emerald-100 dark:bg-emerald-900/30 text-emerald-700 dark:text-emerald-400',
                    currentPlan.status === 'expiring_soon' && 'bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-400',
                    currentPlan.status === 'grace_period' && 'bg-orange-100 dark:bg-orange-900/30 text-orange-700 dark:text-orange-400',
                    currentPlan.status === 'expired' && 'bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-400'
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
            <h4 className="text-sm font-medium text-gray-500 dark:text-gray-400 uppercase">
              {actionType === 'extend' ? 'Extended Plan' : 'New Plan'}
            </h4>
          </div>
          <div className={cn('rounded-lg border-2 p-4 space-y-3', colors.border, 'bg-white dark:bg-gray-800')}>
            <div className="flex items-center justify-between">
              <span className="font-semibold text-gray-900 dark:text-white">{newPlan.name}</span>
              <TierBadge tierLevel={newPlan.tier_level} size="sm" showIcon />
            </div>

            {/* Duration & Expiry */}
            <div className="space-y-2">
              <div className="flex items-center gap-2 text-sm">
                <Calendar className={cn('h-4 w-4', colors.icon)} />
                <span className="text-gray-600 dark:text-gray-400">
                  Duration: <span className="font-medium text-gray-900 dark:text-white">{totalDays} days</span>
                  {upgradePreview?.bonus_days ? (
                    <span className={cn('ml-1 text-xs', colors.text)}>
                      (+{upgradePreview.bonus_days} bonus)
                    </span>
                  ) : null}
                </span>
              </div>
              <div className="flex items-center gap-2 text-sm">
                <Clock className={cn('h-4 w-4', colors.icon)} />
                <span className="text-gray-600 dark:text-gray-400">
                  Valid until: <span className="font-medium text-gray-900 dark:text-white">
                    {expiryDate.toLocaleDateString('en-US', {
                      month: 'short',
                      day: 'numeric',
                      year: 'numeric',
                    })}
                  </span>
                </span>
              </div>
            </div>

            {/* Price with Breakdown */}
            <div className={cn('rounded-lg p-3', colors.bg)}>
              {actionType === 'upgrade' && upgradePreview?.credit_amount && upgradePreview.credit_amount > 0 ? (
                <div className="space-y-2">
                  <div className="flex items-baseline justify-between text-sm">
                    <span className="text-gray-600 dark:text-gray-400">Plan Price</span>
                    <span className="font-medium text-gray-900 dark:text-white">${newPlan.price.toFixed(2)}</span>
                  </div>
                  <div className="flex items-baseline justify-between text-sm">
                    <span className="text-emerald-600 dark:text-emerald-400">Credit Applied</span>
                    <span className="font-medium text-emerald-600 dark:text-emerald-400">-${upgradePreview.credit_amount.toFixed(2)}</span>
                  </div>
                  <div className="border-t border-gray-200 dark:border-gray-700 pt-2 flex items-baseline justify-between">
                    <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Amount Due</span>
                    <div className="flex items-baseline gap-1">
                      <span className="text-2xl font-bold text-gray-900 dark:text-white">
                        ${(upgradePreview.payment_required ?? (newPlan.price - upgradePreview.credit_amount)).toFixed(2)}
                      </span>
                      <span className="text-sm text-gray-500 dark:text-gray-400">USDT</span>
                    </div>
                  </div>
                </div>
              ) : actionType === 'downgrade' && currentPlan?.price ? (
                <div className="space-y-2">
                  <div className="flex items-baseline justify-between text-sm">
                    <span className="text-gray-600 dark:text-gray-400">Current Plan</span>
                    <span className="font-medium text-gray-900 dark:text-white">${currentPlan.price.toFixed(2)}/mo</span>
                  </div>
                  <div className="flex items-baseline justify-between text-sm">
                    <span className="text-gray-600 dark:text-gray-400">New Plan</span>
                    <span className="font-medium text-gray-900 dark:text-white">${newPlan.price.toFixed(2)}/mo</span>
                  </div>
                  <div className="border-t border-gray-200 dark:border-gray-700 pt-2 flex items-baseline justify-between">
                    <span className="text-sm font-medium text-emerald-600 dark:text-emerald-400">Monthly Savings</span>
                    <span className="text-xl font-bold text-emerald-600 dark:text-emerald-400">
                      ${(currentPlan.price - newPlan.price).toFixed(2)}
                    </span>
                  </div>
                </div>
              ) : (
                <div className="flex items-baseline justify-between">
                  <span className="text-sm text-gray-600 dark:text-gray-400">Payment Amount</span>
                  <div className="flex items-baseline gap-1">
                    <span className="text-2xl font-bold text-gray-900 dark:text-white">${newPlan.price.toFixed(2)}</span>
                    <span className="text-sm text-gray-500 dark:text-gray-400">USDT</span>
                  </div>
                </div>
              )}
            </div>

            {/* Features Preview (first 3) - only for new/extend */}
            {(actionType === 'new' || actionType === 'extend') && newPlan.features && newPlan.features.length > 0 && (
              <div className="space-y-1.5 pt-2 border-t border-gray-100 dark:border-gray-700">
                {newPlan.features.slice(0, 3).map((feature, idx) => (
                  <div key={idx} className="flex items-start gap-2 text-sm">
                    <CheckCircle2 className={cn('h-4 w-4 mt-0.5 flex-shrink-0', colors.icon)} />
                    <span className="text-gray-700 dark:text-gray-300">{feature}</span>
                  </div>
                ))}
                {newPlan.features.length > 3 && (
                  <p className="text-xs text-gray-500 dark:text-gray-400 pl-6">
                    +{newPlan.features.length - 3} more features
                  </p>
                )}
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Feature Comparison Table - for upgrade/downgrade */}
      {(actionType === 'upgrade' || actionType === 'downgrade') && (
        <FeatureComparisonTable features={allFeatures} actionType={actionType} colors={colors} />
      )}

      {/* Summary Banner */}
      {actionType === 'upgrade' && upgradePreview && (
        <div className="mt-6 rounded-lg bg-emerald-50 dark:bg-emerald-900/20 border border-emerald-200 dark:border-emerald-800 p-4">
          <div className="flex items-start gap-3">
            <div className="rounded-full bg-emerald-100 dark:bg-emerald-800 p-2">
              <TrendingUp className="h-5 w-5 text-emerald-600 dark:text-emerald-400" />
            </div>
            <div className="flex-1">
              <h4 className="font-medium text-emerald-900 dark:text-emerald-100">Upgrade Bonus</h4>
              <p className="mt-1 text-sm text-emerald-700 dark:text-emerald-300">
                You'll receive <span className="font-semibold">{upgradePreview.bonus_days} bonus days</span> from
                your remaining ${upgradePreview.credit_amount.toFixed(2)} credit, extending your new plan to{' '}
                <span className="font-semibold">{totalDays} total days</span>.
              </p>
            </div>
          </div>
        </div>
      )}

      {actionType === 'extend' && currentPlan && (
        <div className="mt-6 rounded-lg bg-purple-50 dark:bg-purple-900/20 border border-purple-200 dark:border-purple-800 p-4">
          <div className="flex items-start gap-3">
            <div className="rounded-full bg-purple-100 dark:bg-purple-800 p-2">
              <Clock className="h-5 w-5 text-purple-600 dark:text-purple-400" />
            </div>
            <div className="flex-1">
              <h4 className="font-medium text-purple-900 dark:text-purple-100 mb-3">Plan Extension</h4>

              {/* Visual Day Breakdown */}
              <div className="flex items-center gap-2 text-sm mb-2">
                <div className="flex items-center gap-2 bg-white dark:bg-gray-800 rounded-lg px-3 py-2 border border-purple-200 dark:border-purple-700">
                  <Clock className="h-4 w-4 text-purple-600 dark:text-purple-400" />
                  <span className="font-semibold text-purple-900 dark:text-purple-100">{currentPlan.days_remaining} days</span>
                  <span className="text-xs text-gray-500 dark:text-gray-400">remaining</span>
                </div>

                <Plus className="h-4 w-4 text-purple-600 dark:text-purple-400" />

                <div className="flex items-center gap-2 bg-white dark:bg-gray-800 rounded-lg px-3 py-2 border border-purple-200 dark:border-purple-700">
                  <Zap className="h-4 w-4 text-purple-600 dark:text-purple-400" />
                  <span className="font-semibold text-purple-900 dark:text-purple-100">{durationDays} days</span>
                  <span className="text-xs text-gray-500 dark:text-gray-400">new</span>
                </div>

                <Equal className="h-4 w-4 text-purple-600 dark:text-purple-400" />

                <div className="flex items-center gap-2 bg-purple-100 dark:bg-purple-800 rounded-lg px-3 py-2 border-2 border-purple-300 dark:border-purple-600">
                  <Calendar className="h-4 w-4 text-purple-700 dark:text-purple-300" />
                  <span className="font-bold text-purple-900 dark:text-purple-100">{totalDays} days</span>
                  <span className="text-xs text-purple-700 dark:text-purple-300">total</span>
                </div>
              </div>

              <p className="mt-2 text-sm text-purple-700 dark:text-purple-300">
                Valid until{' '}
                <span className="font-semibold">
                  {expiryDate.toLocaleDateString('en-US', { month: 'long', day: 'numeric', year: 'numeric' })}
                </span>
              </p>
            </div>
          </div>
        </div>
      )}

      {actionType === 'downgrade' && (
        <div className="mt-6 rounded-lg bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 p-4">
          <div className="flex items-start gap-3">
            <div className="rounded-full bg-amber-100 dark:bg-amber-800 p-2">
              <Calendar className="h-5 w-5 text-amber-600 dark:text-amber-400" />
            </div>
            <div className="flex-1">
              <h4 className="font-medium text-amber-900 dark:text-amber-100">Downgrade Notice</h4>
              <p className="mt-1 text-sm text-amber-700 dark:text-amber-300">
                Your current plan will remain active until it expires. The new plan will activate on{' '}
                {currentPlan?.expires_at ? (
                  <span className="font-semibold">
                    {new Date(currentPlan.expires_at).toLocaleDateString('en-US', {
                      month: 'long',
                      day: 'numeric',
                      year: 'numeric',
                    })}
                  </span>
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
