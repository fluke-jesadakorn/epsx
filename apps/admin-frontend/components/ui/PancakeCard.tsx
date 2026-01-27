/**
 * ADMIN PANCAKE CARD
 * Legacy wrapper for shared Card component
 */

import * as React from "react"

import { CardContent, CardDescription, CardFooter, CardHeader, CardTitle, Card as SharedCard } from "@/shared/components/ui/card"

import { cn } from "@/lib/utils"

const PancakeCard = React.forwardRef<HTMLDivElement, React.ComponentProps<typeof SharedCard>>(
  ({ className, ...props }, ref) => (
    <SharedCard
      ref={ref}
      className={cn("glass rounded-2xl border border-white/10 bg-white/5 backdrop-blur-xl shadow-xl hover:shadow-2xl hover-lift", className)}
      {...props}
    />
  )
)
PancakeCard.displayName = "PancakeCard"

// Stats Card Wrapper
const PancakeStatsCard = React.forwardRef<HTMLDivElement, any>(
  ({ className, title, value, trend, icon: Icon, ...props }, ref) => (
    <PancakeCard ref={ref} className={cn("overflow-hidden", className)} {...props}>
      <CardContent className="p-6">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm font-medium text-muted-foreground">{title}</p>
            <h3 className="text-2xl font-bold mt-2 bg-gradient-to-r from-purple-400 to-orange-400 bg-clip-text text-transparent">{value}</h3>
          </div>
          {Icon && <div className="p-3 bg-gradient-to-br from-purple-500/20 to-orange-500/20 rounded-xl bg-clip-text text-purple-400 border border-purple-500/20"><Icon className="w-6 h-6" /></div>}
        </div>
        {trend && (
          <div className="mt-4 flex items-center text-sm">
            <span className={cn("font-medium px-2 py-1 rounded-full", trend > 0 ? "text-success bg-emerald-500/10" : "text-destructive bg-red-500/10")}>
              {trend > 0 ? "+" : ""}{trend}%
            </span>
            <span className="text-muted-foreground ml-2">from last month</span>
          </div>
        )}
      </CardContent>
    </PancakeCard>
  )
)
PancakeStatsCard.displayName = "PancakeStatsCard"

// Feature Card
const PancakeFeatureCard = React.forwardRef<HTMLDivElement, any>(
  ({ className, title, description, icon: Icon, action, ...props }, ref) => (
    <PancakeCard ref={ref} className={cn("hover:border-purple-500/30 transition-all", className)} {...props}>
      <CardHeader>
        {Icon && <div className="mb-4 text-purple-400 bg-gradient-to-br from-purple-500/20 to-orange-500/20 p-3 rounded-xl border border-purple-500/20 inline-block"><Icon className="w-8 h-8" /></div>}
        <CardTitle className="text-foreground">{title}</CardTitle>
        <CardDescription>{description}</CardDescription>
      </CardHeader>
      {action && <CardFooter>{action}</CardFooter>}
    </PancakeCard>
  )
)
PancakeFeatureCard.displayName = "PancakeFeatureCard"

export { PancakeCard, PancakeFeatureCard, PancakeStatsCard }
