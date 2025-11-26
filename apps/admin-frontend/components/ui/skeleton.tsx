import { cn } from "@/lib/utils"

/**
 *
 * @param root0
 * @param root0.className
 */
function Skeleton({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn("rounded-md bg-muted", className)}
      {...props}
    />
  )
}

export { Skeleton }