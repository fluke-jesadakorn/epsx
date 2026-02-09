import { cn } from '@/shared/utils/cn';
import { cva, type VariantProps } from 'class-variance-authority';
import type { HTMLAttributes } from 'react';

const badgeVariants = cva(
  "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-all focus:outline-none focus:ring-2 focus:ring-purple-500/50 focus:ring-offset-2",
  {
    variants: {
      variant: {
        default: "border-transparent bg-gradient-to-r from-purple-500 to-orange-500 text-white shadow-lg shadow-purple-500/20 hover:shadow-xl hover:shadow-purple-500/30",
        secondary: "border-transparent bg-gradient-to-r from-purple-500/20 to-orange-500/20 text-purple-400 border-purple-500/20 backdrop-blur-sm hover:bg-purple-500/30",
        destructive: "border-transparent bg-gradient-to-r from-red-500 to-red-600 text-white shadow-lg shadow-red-500/20 hover:shadow-xl hover:shadow-red-500/30",
        outline: "text-foreground border-white/20 bg-white/5 backdrop-blur-sm hover:bg-white/10",
        success: "border-transparent bg-emerald-500/10 text-emerald-400 border-emerald-500/20 hover:bg-emerald-500/20 backdrop-blur-sm",
        active: "border-transparent bg-emerald-500/10 text-emerald-400 border-emerald-500/20 backdrop-blur-sm",
        info: "border-transparent bg-blue-500/10 text-blue-400 border-blue-500/20 hover:bg-blue-500/20 backdrop-blur-sm",
        warning: "border-transparent bg-amber-500/10 text-amber-400 border-amber-500/20 hover:bg-amber-500/20 backdrop-blur-sm",
        pending: "border-transparent bg-slate-500/10 text-slate-400 border-slate-500/20 hover:bg-slate-500/20 backdrop-blur-sm",
        pancake: "border-transparent bg-gradient-to-r from-purple-500 to-orange-500 text-white shadow-lg shadow-purple-500/20 hover:shadow-xl hover:shadow-purple-500/30",
        glass: "border-white/20 bg-white/5 backdrop-blur-xl text-foreground hover:bg-white/10",
      },
      size: {
        default: "px-2.5 py-0.5 text-xs",
        sm: "px-2 py-0.5 text-[10px]",
        lg: "px-3 py-1 text-sm",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
);

export interface BadgeProps
  extends HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof badgeVariants> {}

function Badge({ className, variant, size, ...props }: BadgeProps) {
  return (
    <div className={cn(badgeVariants({ variant, size }), className)} {...props} />
  );
}

export { Badge, badgeVariants };
