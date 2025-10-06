/**
 * Separator UI Component
 */

import { cn } from '@/lib/utils'

interface SeparatorProps {
  className?: string
  orientation?: 'horizontal' | 'vertical'
}

/**
 *
 * @param root0
 * @param root0.className
 * @param root0.orientation
 */
export function Separator({ className, orientation = 'horizontal' }: SeparatorProps) {
  return (
    <div
      className={cn(
        'shrink-0 bg-border',
        orientation === 'horizontal' ? 'h-[1px] w-full' : 'h-full w-[1px]',
        className
      )}
    />
  )
}