import { ReactNode } from "react";
import { Skeleton } from "@/components/ui/skeleton";

interface LoadingFormProps {
  children?: ReactNode;
}

function LoadingFormComponent({ children }: LoadingFormProps) {
  return (
    <div className="w-full max-w-md mx-auto p-6 space-y-6 bg-white rounded-lg shadow-md">
      {children && <div className="text-center mb-4">{children}</div>}
      <div className="space-y-2">
        <Skeleton className="h-4 w-3/4" />
        <Skeleton className="h-10 w-full" />
      </div>
      <div className="space-y-2">
        <Skeleton className="h-4 w-3/4" />
        <Skeleton className="h-10 w-full" />
      </div>
      <div className="space-y-2">
        <Skeleton className="h-4 w-3/4" />
        <Skeleton className="h-10 w-full" />
      </div>
      <Skeleton className="h-10 w-full" />
    </div>
  );
}

// Export both as default and named export
export default LoadingFormComponent;
export { LoadingFormComponent as LoadingForm };
