import { Skeleton } from '@/components/ui/skeleton';

export default function RankingLoading() {
  return (
    <div className="container mx-auto p-4 space-y-8">
      <div className="flex items-center justify-between">
        <Skeleton className="h-8 w-[200px]" />
        <Skeleton className="h-10 w-[150px]" />
      </div>

      <div className="space-y-4">
        <div className="grid gap-4">
          {[...Array(10)].map((_, i) => (
            <div key={i} className="border rounded-lg p-4 space-y-2">
              <div className="flex items-center justify-between">
                <Skeleton className="h-6 w-[150px]" />
                <Skeleton className="h-6 w-[100px]" />
              </div>
              <Skeleton className="h-4 w-full" />
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
