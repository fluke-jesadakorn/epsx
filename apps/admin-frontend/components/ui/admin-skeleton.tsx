/**
 * AdminSkeleton - Unified loading skeleton component
 * Provides consistent loading states across admin dashboard
 * Follows zero animation policy - no transitions or animations
 */

import { cn } from '@/lib/utils'

interface SkeletonProps {
  className?: string
  variant?: 'text' | 'circle' | 'rect' | 'card'
  width?: string
  height?: string
  style?: React.CSSProperties
}

/**
 * Base skeleton element
 * @param root0
 * @param root0.className
 * @param root0.variant
 * @param root0.width
 * @param root0.height
 * @param root0.style
 */
export function Skeleton({ className, variant = 'rect', width, height, style }: SkeletonProps) {
  const variantStyles = {
    text: 'h-4 rounded',
    circle: 'rounded-full',
    rect: 'rounded-lg',
    card: 'rounded-2xl'
  }

  return (
    <div
      className={cn(
        'bg-gray-200 dark:bg-gray-700',
        variantStyles[variant],
        className
      )}
      style={{ width, height, ...style }}
      aria-busy="true"
      aria-live="polite"
    />
  )
}

/**
 * Table skeleton loader
 * @param root0
 * @param root0.rows
 * @param root0.columns
 */
export function TableSkeleton({ rows = 5, columns = 6 }: { rows?: number; columns?: number }) {
  return (
    <div className="w-full space-y-4">
      {/* Header */}
      <div className="flex gap-4">
        {Array.from({ length: columns }).map((_, i) => (
          <Skeleton key={`header-${i}`} className="h-8 flex-1" />
        ))}
      </div>

      {/* Rows */}
      {Array.from({ length: rows }).map((_, rowIdx) => (
        <div key={`row-${rowIdx}`} className="flex gap-4">
          {Array.from({ length: columns }).map((_, colIdx) => (
            <Skeleton key={`cell-${rowIdx}-${colIdx}`} className="h-12 flex-1" />
          ))}
        </div>
      ))}
    </div>
  )
}

/**
 * Card skeleton loader
 * @param root0
 * @param root0.count
 */
export function CardSkeleton({ count = 1 }: { count?: number }) {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
      {Array.from({ length: count }).map((_, i) => (
        <div key={i} className="bg-white dark:bg-gray-800 rounded-2xl p-6 space-y-4">
          <Skeleton variant="circle" width="48px" height="48px" />
          <Skeleton variant="text" width="60%" />
          <Skeleton variant="text" width="40%" />
          <Skeleton variant="rect" className="h-24" />
        </div>
      ))}
    </div>
  )
}

/**
 * Stats card skeleton
 * @param root0
 * @param root0.count
 */
export function StatsCardSkeleton({ count = 4 }: { count?: number }) {
  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
      {Array.from({ length: count }).map((_, i) => (
        <div key={i} className="bg-white/80 dark:bg-gray-800/80 rounded-2xl p-6 space-y-3">
          <div className="flex justify-between">
            <Skeleton variant="circle" width="40px" height="40px" />
            <Skeleton variant="text" width="30%" />
          </div>
          <Skeleton variant="text" width="50%" className="h-8" />
          <Skeleton variant="text" width="70%" />
        </div>
      ))}
    </div>
  )
}

/**
 * List skeleton loader
 * @param root0
 * @param root0.items
 */
export function ListSkeleton({ items = 5 }: { items?: number }) {
  return (
    <div className="space-y-3">
      {Array.from({ length: items }).map((_, i) => (
        <div key={i} className="flex items-center gap-3 p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
          <Skeleton variant="circle" width="32px" height="32px" />
          <div className="flex-1 space-y-2">
            <Skeleton variant="text" width="40%" />
            <Skeleton variant="text" width="60%" />
          </div>
          <Skeleton variant="text" width="80px" />
        </div>
      ))}
    </div>
  )
}

/**
 * Dashboard skeleton - Full page skeleton
 */
export function DashboardSkeleton() {
  return (
    <div className="min-h-screen p-6 space-y-8">
      {/* Header */}
      <div className="space-y-4">
        <Skeleton variant="text" width="300px" className="h-10" />
        <Skeleton variant="text" width="200px" />
      </div>

      {/* Stats Cards */}
      <StatsCardSkeleton count={4} />

      {/* Main Content */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2">
          <div className="bg-white dark:bg-gray-800 rounded-2xl p-6">
            <Skeleton variant="text" width="200px" className="h-6 mb-4" />
            <ListSkeleton items={5} />
          </div>
        </div>
        <div className="space-y-6">
          <div className="bg-white dark:bg-gray-800 rounded-2xl p-6">
            <Skeleton variant="text" width="150px" className="h-6 mb-4" />
            <div className="space-y-3">
              {Array.from({ length: 3 }).map((_, i) => (
                <div key={i} className="flex justify-between">
                  <Skeleton variant="text" width="100px" />
                  <Skeleton variant="text" width="60px" />
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

/**
 * Chart skeleton loader
 */
export function ChartSkeleton() {
  return (
    <div className="w-full h-64 flex items-end justify-between space-x-2 p-4">
      {Array.from({ length: 12 }).map((_, i) => {
        const height = Math.random() * 80 + 20 // Random height between 20-100%
        return (
          <Skeleton
            key={i}
            className="flex-1"
            style={{ height: `${height}%` }}
          />
        )
      })}
    </div>
  )
}
