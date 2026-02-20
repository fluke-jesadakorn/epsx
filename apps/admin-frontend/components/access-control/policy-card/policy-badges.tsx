import { Shield } from 'lucide-react';

import { type AccessPolicy, POLICY_TYPE_CONFIG } from '@/components/access-control/types';
import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils';

export function PolicyBadges({ policy }: { policy: AccessPolicy }) {
    const typeConfig = POLICY_TYPE_CONFIG[policy.type];

    return (
        <div className="hidden sm:flex items-center gap-2">
            {/* Type Badge */}
            <Badge
                className={cn(
                    'text-xs px-2 py-0.5 font-medium border rounded-full',
                    typeConfig.badgeClass
                )}
            >
                <span className="mr-1">{typeConfig.icon}</span>
                {typeConfig.label}
            </Badge>

            {/* System Badge */}
            {policy.isSystemGroup === true && (
                <Badge className="text-xs px-2 py-0.5 font-medium border rounded-full bg-purple-50 text-purple-700 border-purple-200 dark:bg-purple-900/30 dark:text-purple-400 dark:border-purple-800">
                    <Shield className="h-3 w-3 mr-1" />
                    Protected
                </Badge>
            )}

            {/* Active/Inactive Badge */}
            <Badge
                className={cn(
                    'text-xs px-3 py-1 font-semibold border rounded-full',
                    'transition-all duration-200 hover:scale-105',
                    policy.isActive
                        ? 'bg-emerald-50 text-emerald-700 border-emerald-200 dark:bg-emerald-900/30 dark:text-emerald-400 dark:border-emerald-800'
                        : 'bg-gray-50 text-gray-500 border-gray-200 dark:bg-slate-900/30 dark:text-gray-400 dark:border-gray-700'
                )}
            >
                {policy.isActive ? 'Active' : 'Inactive'}
            </Badge>
        </div>
    );
}
