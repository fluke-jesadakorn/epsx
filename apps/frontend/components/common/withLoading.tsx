import { Skeleton } from "@/components/ui/skeleton";

interface WithLoadingProps {
  loading: boolean;
  children: React.ReactNode;
  className?: string;
}

export function WithLoading({ loading, children, className = "" }: WithLoadingProps) {
  if (loading) {
    return (
      <div className={`min-h-[100px] space-y-4 ${className}`}>
        <Skeleton className="h-8 w-3/4" />
        <Skeleton className="h-4 w-1/2" />
        <Skeleton className="h-4 w-2/3" />
      </div>
    );
  }

  return <>{children}</>;
}
