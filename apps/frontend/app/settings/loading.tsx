import { Skeleton } from "@/components/ui/skeleton";

export default function SettingsLoading() {
  return (
    <div className="container mx-auto p-4 space-y-8">
      {/* Settings header */}
      <div className="flex justify-between items-center">
        <Skeleton className="h-8 w-[200px]" />
        <Skeleton className="h-10 w-[120px] rounded-full" />
      </div>

      {/* Settings sections */}
      <div className="grid gap-8">
        {/* Profile section */}
        <div className="space-y-6 p-6 border rounded-lg">
          <Skeleton className="h-6 w-[150px]" />
          <div className="flex items-center gap-6">
            <Skeleton className="h-24 w-24 rounded-full" />
            <div className="space-y-4 flex-1">
              <div className="space-y-2">
                <Skeleton className="h-4 w-[200px]" />
                <Skeleton className="h-10 w-full" />
              </div>
              <div className="space-y-2">
                <Skeleton className="h-4 w-[150px]" />
                <Skeleton className="h-10 w-full" />
              </div>
            </div>
          </div>
        </div>

        {/* Preferences section */}
        <div className="space-y-6 p-6 border rounded-lg">
          <Skeleton className="h-6 w-[180px]" />
          <div className="space-y-4">
            {[...Array(4)].map((_, i) => (
              <div key={i} className="flex justify-between items-center">
                <div className="space-y-1">
                  <Skeleton className="h-5 w-[160px]" />
                  <Skeleton className="h-4 w-[240px]" />
                </div>
                <Skeleton className="h-6 w-12 rounded-full" />
              </div>
            ))}
          </div>
        </div>

        {/* API Keys section */}
        <div className="space-y-6 p-6 border rounded-lg">
          <Skeleton className="h-6 w-[120px]" />
          <div className="space-y-4">
            <div className="flex justify-between items-center">
              <Skeleton className="h-10 w-[300px]" />
              <Skeleton className="h-10 w-[100px]" />
            </div>
            <Skeleton className="h-4 w-full" />
          </div>
        </div>
      </div>
    </div>
  );
}
