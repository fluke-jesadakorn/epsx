/**
 * User List Skeleton Component
 * Loading placeholder for the enhanced user list
 */

import { adminCardVariants, cn } from '@/design-system'

export function UserListSkeleton() {
  return (
    <div className="space-y-6">
      {/* Search and Filters Skeleton */}
      <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <div className="flex items-center justify-between mb-4">
          <div className="h-6 w-32 bg-muted rounded animate-pulse" />
          <div className="h-9 w-24 bg-muted rounded animate-pulse" />
        </div>
        
        <div className="space-y-4">
          {/* Search bar skeleton */}
          <div className="h-10 w-full bg-muted rounded-lg animate-pulse" />
          
          {/* Filter controls skeleton */}
          <div className="flex items-center gap-4">
            <div className="h-8 w-20 bg-muted rounded animate-pulse" />
            <div className="h-8 w-24 bg-muted rounded animate-pulse" />
            <div className="h-8 w-20 bg-muted rounded animate-pulse" />
            <div className="h-8 w-32 bg-muted rounded animate-pulse" />
          </div>
        </div>
      </div>

      {/* User Cards Skeleton */}
      <div className="space-y-4">
        {Array.from({ length: 5 }).map((_, index) => (
          <div key={index} className={cn(adminCardVariants({ variant: 'pancake' }))}>
            <div className="flex items-start justify-between">
              <div className="flex items-start gap-4 flex-1">
                {/* Avatar skeleton */}
                <div className="w-12 h-12 bg-muted rounded-full animate-pulse" />
                
                {/* User info skeleton */}
                <div className="flex-1 space-y-2">
                  <div className="h-5 w-48 bg-muted rounded animate-pulse" />
                  <div className="h-4 w-64 bg-muted rounded animate-pulse" />
                  <div className="flex items-center gap-4">
                    <div className="h-3 w-24 bg-muted rounded animate-pulse" />
                    <div className="h-3 w-28 bg-muted rounded animate-pulse" />
                  </div>
                </div>
              </div>

              {/* Stats skeleton */}
              <div className="flex items-center gap-6">
                {Array.from({ length: 3 }).map((_, statIndex) => (
                  <div key={statIndex} className="text-center space-y-1">
                    <div className="h-4 w-6 bg-muted rounded animate-pulse mx-auto" />
                    <div className="h-3 w-12 bg-muted rounded animate-pulse" />
                  </div>
                ))}
                <div className="h-8 w-8 bg-muted rounded animate-pulse" />
              </div>
            </div>

            {/* Additional info skeleton */}
            <div className="mt-4 pt-4 border-t border-muted">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-4">
                  <div className="h-4 w-20 bg-muted rounded animate-pulse" />
                  <div className="h-4 w-16 bg-muted rounded animate-pulse" />
                </div>
                <div className="flex items-center gap-4">
                  <div className="h-4 w-8 bg-muted rounded animate-pulse" />
                  <div className="h-4 w-12 bg-muted rounded animate-pulse" />
                </div>
                <div className="flex items-center gap-4">
                  <div className="h-4 w-16 bg-muted rounded animate-pulse" />
                  <div className="h-4 w-10 bg-muted rounded animate-pulse" />
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Pagination skeleton */}
      <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <div className="flex items-center justify-between">
          <div className="h-4 w-48 bg-muted rounded animate-pulse" />
          <div className="flex items-center gap-1">
            {Array.from({ length: 7 }).map((_, index) => (
              <div key={index} className="h-8 w-8 bg-muted rounded animate-pulse" />
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}