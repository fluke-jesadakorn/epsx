import { cn } from "@/lib/utils";

interface SkeletonProps {
  className?: string;
  count?: number;
}

export const Skeleton = ({ className, count = 1 }: SkeletonProps) => {
  return (
    <div className="w-full space-y-4">
      {Array.from({ length: count }).map((_, i) => (
        <div
          key={i}
          className={cn(
            "h-4 bg-gray-200 rounded-md animate-pulse dark:bg-gray-700",
            className
          )}
        />
      ))}
    </div>
  );
};

export const SkeletonLoader = () => {
  return (
    <div className="w-full max-w-2xl mx-auto p-4 space-y-4">
      <Skeleton className="w-3/4 h-8" />
      <Skeleton count={3} />
    </div>
  );
};
