/**
 * FRONTEND BADGE COMPONENT
 * Migrated to use unified shared UI component
 */

import { Badge as SharedBadge, badgeVariants, type BadgeProps as SharedBadgeProps } from "../../../../shared/components/ui/badge"

export interface BadgeProps extends SharedBadgeProps {}

function Badge({ className, variant, ...props }: BadgeProps) {
  return (
    <SharedBadge
      className={className}
      variant={variant}
      {...props}
    />
  )
}

export { Badge, badgeVariants }
