import { Skeleton } from "@/components/ui/skeleton";

export default function ResearchLoading() {
  return (
    <div className="container mx-auto p-4 space-y-8">
      {/* Research filters and controls */}
      <div className="flex flex-wrap justify-between items-center gap-4">
        <div className="flex gap-4">
          <Skeleton className="h-10 w-[200px]" />
          <Skeleton className="h-10 w-[150px]" />
        </div>
        <Skeleton className="h-10 w-[120px]" />
      </div>

      {/* Research content */}
      <div className="grid lg:grid-cols-3 gap-8">
        {/* Main analysis panel */}
        <div className="lg:col-span-2 space-y-6">
          {/* Chart */}
          <div className="border rounded-lg p-4">
            <div className="space-y-4">
              <div className="flex justify-between items-center">
                <Skeleton className="h-6 w-[180px]" />
                <Skeleton className="h-8 w-[120px]" />
              </div>
              <Skeleton className="h-[400px] w-full" />
            </div>
          </div>

          {/* Data tables */}
          <div className="border rounded-lg p-4 space-y-4">
            <Skeleton className="h-6 w-[150px]" />
            <div className="grid gap-4">
              {[...Array(5)].map((_, i) => (
                <div key={i} className="grid grid-cols-4 gap-4">
                  <Skeleton className="h-8 w-full" />
                  <Skeleton className="h-8 w-full" />
                  <Skeleton className="h-8 w-full" />
                  <Skeleton className="h-8 w-full" />
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Sidebar panels */}
        <div className="space-y-6">
          {/* Summary panel */}
          <div className="border rounded-lg p-4 space-y-4">
            <Skeleton className="h-6 w-[120px]" />
            {[...Array(4)].map((_, i) => (
              <div key={i} className="flex justify-between items-center">
                <Skeleton className="h-4 w-[100px]" />
                <Skeleton className="h-4 w-[80px]" />
              </div>
            ))}
          </div>

          {/* Recent activity */}
          <div className="border rounded-lg p-4 space-y-4">
            <Skeleton className="h-6 w-[160px]" />
            {[...Array(3)].map((_, i) => (
              <div key={i} className="space-y-2">
                <Skeleton className="h-5 w-full" />
                <Skeleton className="h-4 w-2/3" />
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
