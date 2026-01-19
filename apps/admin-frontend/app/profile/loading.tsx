import { Loader2 } from 'lucide-react';

/**
 *
 */
export default function AdminProfileLoading() {
  return (
    <div className="min-h-screen bg-background">
      <div className="container mx-auto px-6 py-8">
        <div className="max-w-6xl mx-auto">
          {/* Header Skeleton */}
          <div className="mb-8">
            <div className="h-8 w-48 bg-slate-200 dark:bg-slate-700 rounded mb-2" />
            <div className="h-5 w-96 bg-slate-100 dark:bg-slate-800 rounded" />
          </div>

          {/* Content Skeleton */}
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            {/* Sidebar */}
            <div className="lg:col-span-1">
              <div className="bg-white dark:bg-slate-800 rounded-2xl border border-yellow-200 dark:border-slate-700 p-6">
                <div className="space-y-4">
                  {/* Avatar */}
                  <div className="mx-auto w-20 h-20 bg-slate-200 dark:bg-slate-700 rounded-full" />
                  {/* Profile info */}
                  <div className="space-y-2">
                    <div className="h-6 bg-slate-200 dark:bg-slate-700 rounded mx-auto w-32" />
                    <div className="h-4 bg-slate-100 dark:bg-slate-800 rounded mx-auto w-40" />
                  </div>
                  {/* Stats */}
                  <div className="space-y-2 pt-4">
                    <div className="h-4 bg-slate-100 dark:bg-slate-800 rounded" />
                    <div className="h-4 bg-slate-100 dark:bg-slate-800 rounded" />
                    <div className="h-4 bg-slate-100 dark:bg-slate-800 rounded" />
                  </div>
                </div>
              </div>
            </div>

            {/* Main Content */}
            <div className="lg:col-span-2">
              <div className="bg-white dark:bg-slate-800 rounded-2xl border border-yellow-200 dark:border-slate-700 p-6">
                <div className="flex items-center justify-center py-12">
                  <Loader2 className="h-8 w-8 animate-spin text-orange-500" />
                  <span className="ml-2 text-slate-600 dark:text-slate-400">
                    Loading admin profile...
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}