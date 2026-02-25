import { Skeleton } from '@/components/ui/skeleton';

export default function AnalyticsLoading() {
  return (
    <div className="relative min-h-screen overflow-hidden">
      {/* Same decorative background as main page */}
      <div className="fixed inset-0 z-0">
        {/* Main gradient background */}
        <div className="absolute inset-0 bg-gradient-to-br from-blue-50 via-indigo-50 to-purple-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900" />

        {/* Floating gradient orbs */}
        <div className="absolute -top-40 -left-40 h-96 w-96 rounded-full bg-gradient-to-br from-blue-400/30 to-indigo-400/30 blur-3xl" />
        <div className="absolute top-20 -right-32 h-80 w-80 rounded-full bg-gradient-to-br from-purple-400/25 to-pink-400/25 blur-3xl" />
        <div className="absolute bottom-20 left-20 h-72 w-72 rounded-full bg-gradient-to-br from-cyan-400/20 to-teal-400/20 blur-3xl" />
        <div className="absolute top-1/2 right-1/4 h-64 w-64 rounded-full bg-gradient-to-br from-indigo-400/15 to-blue-400/15 blur-3xl" />
      </div>

      {/* Main content */}
      <div className="relative z-10">
        <div className="container mx-auto max-w-7xl px-4 py-8">
          {/* Header skeleton */}
          <div className="mb-12">
            <div className="p-8 text-center">
              <div className="relative">
                <Skeleton className="h-12 w-80 mx-auto mb-4" />
                <Skeleton className="h-6 w-96 mx-auto" />
                
                {/* Decorative dots skeleton */}
                <div className="mt-6 flex items-center justify-center gap-4">
                  <div className="h-2 w-2 rounded-full bg-blue-400/50" />
                  <div className="h-3 w-3 rounded-full bg-purple-400/50" />
                  <div className="h-2 w-2 rounded-full bg-indigo-400/50" />
                </div>
              </div>
            </div>
          </div>

          {/* Dashboard content skeleton */}
          <div className="relative">
            <div className="space-y-8">
              {/* Filter/Controls skeleton */}
              <div className="flex items-center justify-between flex-wrap gap-4">
                <Skeleton className="h-10 w-[200px]" />
                <div className="flex gap-2">
                  <Skeleton className="h-10 w-[120px]" />
                  <Skeleton className="h-10 w-[100px]" />
                </div>
              </div>

              {/* Cards grid skeleton */}
              <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
                {[...Array(6)].map((_, i) => (
                  <div key={`card-skeleton-${String(i)}`} className="relative overflow-hidden rounded-2xl border border-blue-200/50 bg-white/60 p-6 dark:border-blue-700/50 dark:bg-card/60">
                    <div className="space-y-4">
                      <div className="flex items-center justify-between">
                        <Skeleton className="h-6 w-[100px]" />
                        <Skeleton className="h-5 w-[60px]" />
                      </div>
                      <Skeleton className="h-8 w-[150px]" />
                      <Skeleton className="h-4 w-full" />
                      <Skeleton className="h-4 w-3/4" />
                      <div className="flex items-center justify-between pt-2">
                        <Skeleton className="h-4 w-[80px]" />
                        <Skeleton className="h-6 w-[60px]" />
                      </div>
                    </div>
                  </div>
                ))}
              </div>

              {/* Additional loading indicators */}
              <div className="grid gap-4 md:grid-cols-2">
                {[...Array(4)].map((_, i) => (
                  <div key={`additional-skeleton-${String(i)}`} className="border rounded-xl p-4 space-y-3 bg-gray-100 dark:bg-white/40 dark:bg-card/40">
                    <div className="flex items-center justify-between">
                      <Skeleton className="h-5 w-[120px]" />
                      <Skeleton className="h-5 w-[80px]" />
                    </div>
                    <Skeleton className="h-4 w-full" />
                    <Skeleton className="h-4 w-2/3" />
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
