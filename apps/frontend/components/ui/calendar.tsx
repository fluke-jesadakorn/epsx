"use client"

import * as React from "react"
import { ChevronLeft, ChevronRight } from "lucide-react"
import { cn } from "@/lib/utils"

export interface CalendarProps extends Omit<React.HTMLAttributes<HTMLDivElement>, 'onSelect'> {
  showOutsideDays?: boolean;
  selected?: Date;
  onSelect?: (date: Date | undefined) => void;
}

function Calendar({
  className,
  selected,
  onSelect,
  ...props
}: CalendarProps) {
  return (
    <div
      className={cn("p-3 border rounded-md", className)}
      {...props}
    >
      <div className="text-center">
        <p className="text-sm text-gray-500">Calendar component</p>
        <p className="text-xs text-gray-400">react-day-picker not installed</p>
        {selected && (
          <p className="text-xs mt-2">Selected: {selected.toLocaleDateString()}</p>
        )}
      </div>
    </div>
  )
}
Calendar.displayName = "Calendar"

export { Calendar }