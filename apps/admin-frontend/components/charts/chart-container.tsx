/**
 * Chart Container Component
 * Windows Phone + PancakeSwap styled container for all chart components
 */

import React from 'react'

import { adminCardVariants } from '@/design-system'
import { cn } from '@/lib/utils'

interface ChartContainerProps {
  title: string
  subtitle?: string
  children: React.ReactNode
  className?: string
  variant?: 'default' | 'pancake' | 'glass'
  size?: 'default' | 'sm' | 'lg'
  showLiveDot?: boolean
}

/**
 *
 * @param root0
 * @param root0.title
 * @param root0.subtitle
 * @param root0.children
 * @param root0.className
 * @param root0.variant
 * @param root0.size
 * @param root0.showLiveDot
 */
export function ChartContainer({
  title,
  subtitle,
  children,
  className,
  variant = 'pancake',
  size = 'default',
  showLiveDot = false
}: ChartContainerProps) {
  return (
    <div className={cn(
      adminCardVariants({
        variant,
        hover: 'glow',
        size
      }),
      'relative overflow-hidden',
      className
    )}>
      {/* Windows Phone accent bar */}
      <div className="absolute left-0 top-0 bottom-0 w-1 bg-yellow-400/80" />
      
      {/* Live status indicator */}
      {showLiveDot && (
        <div className="absolute top-3 right-3 w-2 h-2 bg-green-400/80 rounded-full animate-pulse-subtle" />
      )}
      
      <div className="relative z-10 p-6">
        {/* Header */}
        <div className="mb-6">
          <h3 className="text-lg font-light text-foreground uppercase tracking-wide mb-1">
            {title}
          </h3>
          {subtitle !== undefined && subtitle !== '' && (
            <p className="text-sm text-muted-foreground font-light">
              {subtitle}
            </p>
          )}
        </div>
        
        {/* Chart content */}
        <div className="relative">
          {children}
        </div>
      </div>
      
      {/* Windows Phone corner accent */}
      <div className="absolute bottom-2 right-2 w-1.5 h-1.5 bg-yellow-400/60 rounded-full" />
    </div>
  )
}