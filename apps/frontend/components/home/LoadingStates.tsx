import { Card, CardContent, CardHeader, Skeleton } from "@epsx/ui";

export const TableRowsSkeleton = () => (
  <>
    {[...Array(5)].map((_, i) => (
      <tr key={i} className="hover:bg-transparent">
        <td className="p-4">
          <Skeleton className="h-4 w-8" />
        </td>
        <td className="p-4">
          <Skeleton className="h-4 w-16" />
        </td>
        <td className="p-4">
          <Skeleton className="h-4 w-48" />
        </td>
        <td className="p-4">
          <Skeleton className="h-4 w-20" />
        </td>
        <td className="p-4">
          <Skeleton className="h-4 w-12" />
        </td>
        <td className="p-4">
          <Skeleton className="h-4 w-24" />
        </td>
        <td className="p-4">
          <Skeleton className="h-4 w-32" />
        </td>
      </tr>
    ))}
  </>
);

export const EpsCardsSkeleton = () => (
  <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
    {[...Array(3)].map((_, i) => (
      <Card key={i} className="overflow-hidden">
        <CardHeader className="pb-3">
          <Skeleton className="h-6 w-16" />
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Skeleton className="h-6 w-3/4" />
            <Skeleton className="h-4 w-1/4" />
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Skeleton className="h-4 w-20" />
              <Skeleton className="h-5 w-16" />
            </div>
            <div className="space-y-2">
              <Skeleton className="h-4 w-16" />
              <Skeleton className="h-5 w-20" />
            </div>
          </div>
          <Skeleton className="h-4 w-32" />
        </CardContent>
      </Card>
    ))}
  </div>
);

export const HeroSkeleton = () => (
  <div className="text-center space-y-8 max-w-3xl mx-auto px-4">
    <div className="space-y-4">
      <div className="flex flex-col items-center gap-4">
        <Skeleton className="h-12 w-3/4" />
        <Skeleton className="h-8 w-2/3" />
      </div>
    </div>
    <div className="flex justify-center gap-4">
      <Skeleton className="h-12 w-40" />
      <Skeleton className="h-12 w-40" />
    </div>
    <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
      {[...Array(3)].map((_, i) => (
        <div key={i} className="p-6 border rounded-2xl">
          <Skeleton className="h-8 w-20 mb-2" />
          <Skeleton className="h-5 w-32" />
        </div>
      ))}
    </div>
  </div>
);
