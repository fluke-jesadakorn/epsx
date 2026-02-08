/**
 * PolicyCard Component
 * Unified card for displaying Access Policies (both Plans and Groups)
 * Adapts visuals based on policy type
 */
'use client';

import { Clock, Edit, Eye, MoreHorizontal, Shield, Trash2, TrendingUp, Users } from 'lucide-react';
import Link from 'next/link';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { cn } from '@/lib/utils';
import {
  type AccessPolicy,
  POLICY_TYPE_CONFIG,
  getPolicyEditUrl,
  getPolicyMembersUrl,
  isSubscriptionPolicy,
} from './types';

interface PolicyCardProps {
  policy: AccessPolicy;
  isSelected?: boolean;
  onSelect?: (selected: boolean) => void;
  onDelete?: () => void;
  className?: string;
}

// Generate a deterministic gradient based on policy name
function getAvatarGradient(name: string): string {
  let hash = 0;
  for (let i = 0; i < name.length; i++) {
    hash = name.charCodeAt(i) + ((hash << 5) - hash);
  }
  const hue1 = Math.abs(hash % 360);
  const hue2 = (hue1 + 40) % 360;
  return `linear-gradient(135deg, hsl(${hue1}, 70%, 60%) 0%, hsl(${hue2}, 80%, 50%) 100%)`;
}

// Get initials from policy name
function getInitials(name: string): string {
  return name
    .split(' ')
    .map(word => word[0])
    .join('')
    .substring(0, 2)
    .toUpperCase();
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
        // Base card styles with premium glassmorphism
        'card-insight-enhanced group relative overflow-hidden',
        'text-card-foreground',
        // Border handled by card-insight-enhanced
        // Smooth transitions
        'transition-all duration-300 ease-out',
        // Hover effects
        'hover:shadow-xl hover:shadow-primary/5',
        'hover:border-border',
        'hover:scale-[1.01] hover:-translate-y-0.5',
        // Selected state
        isSelected && 'ring-2 ring-primary ring-offset-2 dark:ring-offset-background border-primary/50',
        className
      )}
    >
      {/* Type-based gradient overlay on hover */}
      <div className={cn(
        'absolute inset-0 bg-gradient-to-br opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none',
        policy.type === 'subscription' && 'from-blue-500/5 via-indigo-500/5 to-blue-500/5',
        policy.type === 'manual' && 'from-amber-500/5 via-orange-500/5 to-amber-500/5',
        policy.type === 'web3_asset' && 'from-purple-500/5 via-pink-500/5 to-purple-500/5',
        policy.type === 'dao' && 'from-emerald-500/5 via-teal-500/5 to-emerald-500/5',
        policy.type === 'system' && 'from-gray-500/5 via-slate-500/5 to-gray-500/5',
      )} />

      {/* Shine effect on hover */}
      <div className="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none overflow-hidden">
        <div className="absolute -inset-full top-0 w-1/2 h-full bg-gradient-to-r from-transparent via-white/10 to-transparent skew-x-12 group-hover:animate-[shimmer_2s_ease-in-out_infinite]" />
      </div>

      <div className="relative flex flex-col p-5 gap-4">
        {/* Header: Checkbox + Avatar + Name + Type Badge */}
        <div className="flex items-center gap-4">
          {/* Checkbox */}
          {onSelect && (
            <div className="flex items-center">
              <input
                type="checkbox"
                checked={isSelected}
                onChange={(e: React.ChangeEvent<HTMLInputElement>) => onSelect(e.target.checked)}
                className="h-4 w-4 rounded-md border-border text-primary focus:ring-primary focus:ring-offset-0 transition-colors cursor-pointer bg-card"
              />
            </div>
          )}

          {/* Premium Avatar with Type Icon */}
          <div className="relative group/avatar">
            <div
              className={cn(
                'relative flex h-12 w-12 shrink-0 items-center justify-center rounded-xl text-sm font-bold shadow-lg',
                'text-white transition-all duration-300',
                'group-hover/avatar:scale-105 group-hover/avatar:shadow-xl',
              )}
              style={{ background: getAvatarGradient(policy.name) }}
            >
              {getInitials(policy.name)}

              {/* System/Protected indicator */}
              {policy.isSystemGroup && (
                <div className={cn(
                  'absolute -bottom-0.5 -right-0.5 h-3 w-3 rounded-full border-2 border-white dark:border-gray-900',
                  'bg-purple-500',
                )}>
                  <span className={cn(
                    'absolute inset-0 rounded-full bg-purple-500',
                    'animate-ping opacity-40'
                  )} />
                </div>
              )}
            </div>
          </div>

          {/* Name & Description */}
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
            {policy.description && (
              <p className="text-xs text-muted-foreground line-clamp-1 mt-0.5">
                {policy.description}
              </p>
            )}
          </div>

          {/* Status Badges */}
          <div className="hidden sm:flex items-center gap-2">
            {/* Type Badge */}
            <Badge className={cn(
              'text-xs px-2 py-0.5 font-medium border rounded-full',
              typeConfig.badgeClass
            )}>
              <span className="mr-1">{typeConfig.icon}</span>
              {typeConfig.label}
            </Badge>

            {/* System Badge */}
            {policy.isSystemGroup && (
              <Badge className="text-xs px-2 py-0.5 font-medium border rounded-full bg-purple-50 text-purple-700 border-purple-200 dark:bg-purple-900/30 dark:text-purple-400 dark:border-purple-800">
                <Shield className="h-3 w-3 mr-1" />
                Protected
              </Badge>
            )}

            {/* Active/Inactive Badge */}
            <Badge className={cn(
              'text-xs px-3 py-1 font-semibold border rounded-full',
              'transition-all duration-200 hover:scale-105',
              policy.isActive
                ? 'bg-emerald-50 text-emerald-700 border-emerald-200 dark:bg-emerald-900/30 dark:text-emerald-400 dark:border-emerald-800'
                : 'bg-gray-50 text-gray-500 border-gray-200 dark:bg-gray-900/30 dark:text-gray-400 dark:border-gray-700'
            )}>
              {policy.isActive ? 'Active' : 'Inactive'}
            </Badge>
          </div>

          {/* Actions Menu */}
          <div className="flex items-center gap-1.5">
            <Link href={editUrl}>
              <Button
                variant="ghost"
                size="sm"
                className={cn(
                  'hidden sm:flex h-9 px-4 gap-2 text-sm font-medium rounded-xl',
                  'bg-gradient-to-r from-blue-50 to-indigo-50 text-blue-700',
                  'hover:from-blue-100 hover:to-indigo-100 hover:text-blue-800',
                  'dark:from-blue-900/30 dark:to-indigo-900/30 dark:text-blue-400',
                  'dark:hover:from-blue-900/50 dark:hover:to-indigo-900/50',
                  'transition-all duration-200 hover:scale-105 hover:shadow-md'
                )}
              >
                <Eye className="h-4 w-4" />
                View
              </Button>
            </Link>

            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-9 w-9 p-0 rounded-xl hover:bg-muted transition-all duration-200 hover:scale-110"
                >
                  <MoreHorizontal className="h-4 w-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" className="w-48 rounded-xl p-1">
                <Link href={editUrl}>
                  <DropdownMenuItem className="rounded-lg">
                    <Edit className="h-4 w-4 mr-2" />
                    <span className="text-sm">Edit Policy</span>
                  </DropdownMenuItem>
                </Link>
                <Link href={membersUrl}>
                  <DropdownMenuItem className="rounded-lg">
                    <Users className="h-4 w-4 mr-2" />
                    <span className="text-sm">{isSubscription ? 'View Subscribers' : 'Manage Members'}</span>
                  </DropdownMenuItem>
                </Link>
                <DropdownMenuSeparator />
                {!policy.isSystemGroup && onDelete && (
                  <DropdownMenuItem
                    onClick={onDelete}
                    className="text-red-700 dark:text-red-400 rounded-lg"
                  >
                    <Trash2 className="h-4 w-4 mr-2" />
                    <span className="text-sm">Delete Policy</span>
                  </DropdownMenuItem>
                )}
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </div>

        {/* Key Metrics Grid - 2 cols for readability */}
        <div className="grid grid-cols-2 gap-3">
          {/* Type (all) */}
          <div className="flex flex-col p-3 rounded-xl bg-muted/40 border border-border">
            <span className="text-[10px] uppercase text-muted-foreground font-medium tracking-wide mb-1">Type</span>
            <div className="flex items-center gap-1.5">
              <span className="text-base">{typeConfig.icon}</span>
              <span className="text-sm font-semibold text-foreground">{typeConfig.label}</span>
            </div>
          </div>

          {/* Members/Subscribers (all) */}
          <div className="flex flex-col p-3 rounded-xl bg-muted/40 border border-border">
            <span className="text-[10px] uppercase text-muted-foreground font-medium tracking-wide mb-1">
              {isSubscription ? 'Subs' : 'Users'}
            </span>
            <div className="flex items-center gap-1.5">
              <Users className="h-4 w-4 text-blue-500" />
              <span className="text-sm font-semibold text-foreground">
                {policy.memberCount}
              </span>
            </div>
          </div>

          {/* Subscription: Price & Revenue */}
          {isSubscription && (
            <>
              <div className="flex flex-col p-3 rounded-xl bg-blue-500/10 border border-blue-500/20">
                <span className="text-[10px] uppercase text-muted-foreground font-medium tracking-wide mb-1">Price</span>
                <span className="text-sm font-semibold text-foreground">
                  {policy.pricing?.amount === 0 ? (
                    <span className="text-emerald-600 dark:text-emerald-400">Free</span>
                  ) : (
                    <>
                      ${policy.pricing?.amount?.toFixed(2)}
                      <span className="text-muted-foreground font-normal text-xs ml-1">
                        {policy.pricing?.currency}
                      </span>
                    </>
                  )}
                </span>
              </div>

              <div className="flex flex-col p-3 rounded-xl bg-emerald-500/10 border border-emerald-500/20">
                <span className="text-[10px] uppercase text-muted-foreground font-medium tracking-wide mb-1">Revenue (30d)</span>
                <div className="flex items-center gap-1.5">
                  <TrendingUp className="h-4 w-4 text-emerald-500" />
                  <span className="text-sm font-semibold text-emerald-600 dark:text-emerald-400">
                    ${(policy.revenue ?? 0).toFixed(2)}
                  </span>
                </div>
              </div>
            </>
          )}

          {/* Group: Permissions & Priority/Expiry */}
          {!isSubscription && (
            <>
              <div className="flex flex-col p-3 rounded-xl bg-muted/40 border border-border">
                <span className="text-[10px] uppercase text-muted-foreground font-medium tracking-wide mb-1">Perms</span>
                <span className="text-sm font-semibold text-foreground">
                  {policy.permissions.length} <span className="text-muted-foreground font-normal text-xs">active</span>
                </span>
              </div>

              <div className="flex flex-col p-3 rounded-xl bg-muted/40 border border-border">
                <span className="text-[10px] uppercase text-muted-foreground font-medium tracking-wide mb-1">
                  {policy.expiryDays ? 'Expiry' : 'Priority'}
                </span>
                <div className="flex items-center gap-1.5">
                  {policy.expiryDays ? (
                    <>
                      <Clock className="h-4 w-4 text-amber-500" />
                      <span className="text-sm font-semibold text-foreground">
                        {policy.expiryDays === -1 ? 'Permanent' : `${policy.expiryDays}d`}
                      </span>
                    </>
                  ) : (
                    <span className="text-sm font-semibold text-foreground">
                      Level {policy.priorityLevel ?? 0}
                    </span>
                  )}
                </div>
              </div>
            </>
          )}
        </div>

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
