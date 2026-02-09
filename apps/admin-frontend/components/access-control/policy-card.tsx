/**
 * PolicyCard Component
 * Unified card for displaying Access Policies (both Plans and Groups)
 * Adapts visuals based on policy type
 */
'use client';

import { cn } from '@/lib/utils';

import {
  type AccessPolicy,
  getPolicyEditUrl,
  getPolicyMembersUrl,
  isSubscriptionPolicy,
  POLICY_TYPE_CONFIG,
} from './types';

import { PolicyActions } from './policy-card/policy-actions';
import { PolicyAvatar } from './policy-card/policy-avatar';
import { PolicyBadges } from './policy-card/policy-badges';
import { PolicyMetrics } from './policy-card/policy-metrics';

interface PolicyCardProps {
  policy: AccessPolicy;
  isSelected?: boolean;
  onSelect?: (selected: boolean) => void;
  onDelete?: () => void;
  className?: string;
}

/**
 * PolicyCard displays a unified view for both subscription plans and permission groups
 */
export function PolicyCard({
  policy,
  isSelected = false,
  onSelect,
  onDelete,
  className,
}: PolicyCardProps) {
  const typeConfig = POLICY_TYPE_CONFIG[policy.type];
  const isSubscription = isSubscriptionPolicy(policy);
  const editUrl = getPolicyEditUrl(policy);
  const membersUrl = getPolicyMembersUrl(policy);

  return (
    <div
      className={cn(
        'card-insight-enhanced group relative overflow-hidden text-card-foreground',
        'transition-all duration-300 ease-out',
        'hover:shadow-xl hover:shadow-primary/5 hover:border-border',
        'hover:scale-[1.01] hover:-translate-y-0.5',
        isSelected &&
        'ring-2 ring-primary ring-offset-2 dark:ring-offset-background border-primary/50',
        className
      )}
    >
      {/* Type-based gradient overlay on hover */}
      <div
        className={cn(
          'absolute inset-0 bg-gradient-to-br opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none',
          policy.type === 'subscription' &&
          'from-blue-500/5 via-indigo-500/5 to-blue-500/5',
          policy.type === 'manual' &&
          'from-amber-500/5 via-orange-500/5 to-amber-500/5',
          policy.type === 'web3_asset' &&
          'from-purple-500/5 via-pink-500/5 to-purple-500/5',
          policy.type === 'dao' &&
          'from-emerald-500/5 via-teal-500/5 to-emerald-500/5',
          policy.type === 'system' &&
          'from-gray-500/5 via-slate-500/5 to-gray-500/5'
        )}
      />

      <div className="relative flex flex-col p-5 gap-4">
        {/* Header: Checkbox + Avatar + Name + Type Badge */}
        <div className="flex items-center gap-4">
          {onSelect && (
            <div className="flex items-center">
              <input
                type="checkbox"
                checked={isSelected}
                onChange={(e) => onSelect(e.target.checked)}
                className="h-4 w-4 rounded-md border-border text-primary focus:ring-primary focus:ring-offset-0 transition-colors cursor-pointer bg-card"
              />
            </div>
          )}

          <PolicyAvatar name={policy.name} isSystemGroup={policy.isSystemGroup} />

          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2">
              <span className="font-semibold text-foreground truncate">
                {policy.name}
              </span>
              {isSubscription && policy.tierLevel !== undefined && (
                <span className="text-xs font-mono text-muted-foreground bg-muted px-1.5 py-0.5 rounded">
                  T{policy.tierLevel}
                </span>
              )}
            </div>
            {policy.description.length > 0 && (
              <p className="text-xs text-muted-foreground line-clamp-1 mt-0.5">
                {policy.description}
              </p>
            )}
          </div>

          <PolicyBadges policy={policy} />

          <PolicyActions
            editUrl={editUrl}
            membersUrl={membersUrl}
            isSubscription={isSubscription}
            isSystemGroup={policy.isSystemGroup}
            onDelete={onDelete}
          />
        </div>

        <PolicyMetrics
          policy={policy}
          isSubscription={isSubscription}
          typeIcon={typeConfig.icon}
          typeLabel={typeConfig.label}
        />

        {/* Permissions Preview */}
        {policy.permissions.length > 0 && (
          <div className="flex flex-wrap gap-1.5">
            {policy.permissions.slice(0, 4).map((perm) => (
              <span
                key={perm}
                className="text-xs px-2 py-1 bg-muted text-muted-foreground rounded-lg"
              >
                {perm.split(':').pop()}
              </span>
            ))}
            {policy.permissions.length > 4 && (
              <span className="text-xs px-2 py-1 bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded-lg font-medium">
                +{policy.permissions.length - 4} more
              </span>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
