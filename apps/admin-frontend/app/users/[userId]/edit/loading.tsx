/**
 * Loading page for user edit
 * Shows loading skeleton with PancakeSwap theme
 */

import { PancakeCard } from '@/components/ui/PancakeCard'
import { Skeleton } from '@/components/ui/skeleton'

export default function UserEditLoading() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 via-indigo-50 to-purple-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      {/* Background Decorations */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-blue-400/20 to-indigo-500/20 rounded-full blur-xl animate-pulse"></div>
        <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-purple-400/20 to-pink-500/20 rounded-full blur-lg animate-pulse animation-delay-1000"></div>
        <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-indigo-400/15 to-blue-500/15 rounded-full blur-xl animate-pulse animation-delay-2000"></div>
      </div>
      
      <div className="relative z-10 max-w-6xl mx-auto space-y-8">
        {/* Header Skeleton */}
        <div className="text-center space-y-4">
          <Skeleton className="h-12 w-96 mx-auto rounded-2xl" />
          <Skeleton className="h-6 w-80 mx-auto rounded-xl" />
        </div>
        
        {/* Breadcrumb Skeleton */}
        <PancakeCard className="p-4">
          <div className="flex items-center gap-2">
            <Skeleton className="h-4 w-12 rounded" />
            <span className="text-gray-400">/</span>
            <Skeleton className="h-4 w-24 rounded" />
            <span className="text-gray-400">/</span>
            <Skeleton className="h-4 w-8 rounded" />
          </div>
        </PancakeCard>
        
        {/* Progress Steps Skeleton */}
        <PancakeCard className="p-6">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Skeleton className="w-8 h-8 rounded-full" />
              <Skeleton className="h-4 w-24 rounded" />
            </div>
            <Skeleton className="h-0.5 flex-1 mx-4 rounded" />
            <div className="flex items-center gap-2">
              <Skeleton className="w-8 h-8 rounded-full" />
              <Skeleton className="h-4 w-32 rounded" />
            </div>
            <Skeleton className="h-0.5 flex-1 mx-4 rounded" />
            <div className="flex items-center gap-2">
              <Skeleton className="w-8 h-8 rounded-full" />
              <Skeleton className="h-4 w-28 rounded" />
            </div>
          </div>
        </PancakeCard>

        {/* Form Skeleton */}
        <PancakeCard className="p-6">
          <div className="space-y-6">
            {/* Form Header */}
            <div className="flex items-center gap-2 mb-6">
              <Skeleton className="w-6 h-6 rounded" />
              <Skeleton className="h-6 w-32 rounded" />
            </div>

            {/* Form Fields */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              <div className="space-y-2">
                <Skeleton className="h-4 w-20 rounded" />
                <Skeleton className="h-12 w-full rounded-2xl" />
              </div>
              <div className="space-y-2">
                <Skeleton className="h-4 w-24 rounded" />
                <Skeleton className="h-12 w-full rounded-2xl" />
              </div>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              <div className="space-y-2">
                <Skeleton className="h-4 w-20 rounded" />
                <Skeleton className="h-12 w-full rounded-2xl" />
              </div>
              <div className="space-y-2">
                <Skeleton className="h-4 w-18 rounded" />
                <Skeleton className="h-12 w-full rounded-2xl" />
              </div>
            </div>

            {/* Action Buttons */}
            <div className="flex justify-between pt-6 border-t border-gray-200 dark:border-gray-700">
              <Skeleton className="h-10 w-20 rounded-2xl" />
              <div className="flex gap-3">
                <Skeleton className="h-10 w-16 rounded-2xl" />
                <Skeleton className="h-10 w-24 rounded-2xl" />
              </div>
            </div>
          </div>
        </PancakeCard>
      </div>
    </div>
  )
}