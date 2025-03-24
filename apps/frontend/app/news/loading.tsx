import { Skeleton } from "@/components/ui/skeleton";

export default function NewsLoading() {
  return (
    <div className="container mx-auto p-4 space-y-8">
      {/* News filters */}
      <div className="flex flex-wrap gap-4 items-center">
        <Skeleton className="h-10 w-[150px]" />
        <Skeleton className="h-10 w-[120px]" />
        <Skeleton className="h-10 w-[100px]" />
      </div>

      {/* News articles */}
      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
        {[...Array(9)].map((_, i) => (
          <div key={i} className="border rounded-lg overflow-hidden">
            <Skeleton className="w-full h-48" />
            <div className="p-4 space-y-4">
              <div className="space-y-2">
                <Skeleton className="h-6 w-3/4" />
                <Skeleton className="h-4 w-1/4" />
              </div>
              <div className="space-y-2">
                <Skeleton className="h-4 w-full" />
                <Skeleton className="h-4 w-full" />
                <Skeleton className="h-4 w-2/3" />
              </div>
              <div className="flex justify-between items-center">
                <Skeleton className="h-4 w-[100px]" />
                <Skeleton className="h-8 w-[80px]" />
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Pagination */}
      <div className="flex justify-center gap-2 mt-8">
        {[...Array(5)].map((_, i) => (
          <Skeleton key={i} className="h-10 w-10 rounded-full" />
        ))}
      </div>
    </div>
  );
}
