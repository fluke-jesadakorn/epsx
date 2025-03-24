import { Skeleton } from "@/components/ui/skeleton";

export default function TradingLoading() {
  return (
    <div className="container mx-auto p-4 space-y-8">
      {/* Chart area loading */}
      <div className="w-full aspect-[2/1] rounded-lg">
        <Skeleton className="w-full h-full" />
      </div>

      {/* Trading controls loading */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="space-y-4">
          <Skeleton className="h-10 w-full" />
          <Skeleton className="h-10 w-full" />
          <div className="flex gap-4">
            <Skeleton className="h-12 w-1/2" />
            <Skeleton className="h-12 w-1/2" />
          </div>
        </div>
        
        <div className="md:col-span-2 space-y-4">
          <div className="flex justify-between items-center">
            <Skeleton className="h-6 w-[120px]" />
            <Skeleton className="h-6 w-[100px]" />
          </div>
          <div className="grid grid-cols-2 gap-4">
            {[...Array(4)].map((_, i) => (
              <div key={i} className="border rounded-lg p-4 space-y-2">
                <Skeleton className="h-4 w-[100px]" />
                <Skeleton className="h-6 w-[80px]" />
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
