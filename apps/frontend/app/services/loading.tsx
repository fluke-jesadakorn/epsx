import { Skeleton } from "@/components/ui/skeleton";

export default function ServicesLoading() {
  return (
    <div className="container mx-auto p-4 space-y-12">
      {/* Header */}
      <div className="text-center space-y-4">
        <Skeleton className="h-10 w-[300px] mx-auto" />
        <Skeleton className="h-4 w-[500px] mx-auto" />
      </div>

      {/* Service categories */}
      <div className="grid gap-8 md:grid-cols-2 lg:grid-cols-3">
        {[...Array(6)].map((_, i) => (
          <div key={i} className="border rounded-xl p-6 space-y-4">
            {/* Service icon */}
            <div className="flex justify-center">
              <Skeleton className="h-16 w-16 rounded-full" />
            </div>

            {/* Service info */}
            <div className="text-center space-y-3">
              <Skeleton className="h-6 w-[180px] mx-auto" />
              <div className="space-y-2">
                <Skeleton className="h-4 w-full" />
                <Skeleton className="h-4 w-[90%] mx-auto" />
                <Skeleton className="h-4 w-[80%] mx-auto" />
              </div>
            </div>

            {/* Price and action */}
            <div className="flex justify-between items-center pt-4">
              <Skeleton className="h-8 w-[100px]" />
              <Skeleton className="h-10 w-[120px] rounded-full" />
            </div>
          </div>
        ))}
      </div>

      {/* Featured service */}
      <div className="rounded-2xl border p-8">
        <div className="flex flex-col md:flex-row gap-8">
          <Skeleton className="w-full md:w-1/2 h-[300px] rounded-lg" />
          <div className="w-full md:w-1/2 space-y-6">
            <Skeleton className="h-8 w-3/4" />
            <div className="space-y-3">
              <Skeleton className="h-4 w-full" />
              <Skeleton className="h-4 w-full" />
              <Skeleton className="h-4 w-2/3" />
            </div>
            <div className="flex items-center gap-4">
              <Skeleton className="h-12 w-[150px] rounded-full" />
              <Skeleton className="h-12 w-[120px] rounded-full" />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
