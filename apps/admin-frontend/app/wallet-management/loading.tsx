import { Skeleton } from '@/components/ui/skeleton';

const STAT_COLORS = [
  'from-[#1fc7d4]/20',
  'from-[#31d0aa]/20',
  'from-[#ed4b9e]/20',
  'from-[#7645d9]/20',
  'from-[#ffb237]/20',
];

export default function WalletManagementLoading() {
  return (
    <div className="p-3 sm:p-6 lg:p-8">
      <div className="max-w-7xl mx-auto space-y-6">
        {/* Header */}
        <div className="mb-6 sm:mb-8">
          <div className="flex items-center gap-3 mb-2">
            <Skeleton className="w-10 h-10 sm:w-12 sm:h-12 rounded-xl" />
            <Skeleton className="h-10 sm:h-12 w-64 sm:w-80 rounded-2xl" />
          </div>
          <Skeleton className="h-4 sm:h-5 w-80 sm:w-96 mt-3 rounded-full" />
        </div>

        {/* Stats — 5 cards matching DashboardSection */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-4 mb-6">
          {STAT_COLORS.map((color) => (
            <div
              key={color}
              className={`relative overflow-hidden rounded-2xl p-0.5 bg-gradient-to-br ${color} via-transparent to-transparent`}
            >
              <div className="relative bg-card/90 rounded-2xl p-5 space-y-4">
                <div className="flex items-start justify-between">
                  <Skeleton className="w-9 h-9 rounded-lg" />
                  <Skeleton className="h-5 w-10 rounded-full" />
                </div>
                <div className="space-y-1.5">
                  <Skeleton className="h-7 w-16 rounded-lg" />
                  <Skeleton className="h-3 w-24 rounded-full" />
                </div>
              </div>
            </div>
          ))}
        </div>

        {/* Content area */}
        <div className="rounded-2xl border border-border/20 bg-card shadow-xl overflow-hidden">
          <div className="h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" />
          <div className="p-6 space-y-3">
            {Array.from({ length: 8 }, (_, i) => `skel-${i}`).map((key) => (
              <Skeleton key={key} className="h-14 w-full rounded-xl" />
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
