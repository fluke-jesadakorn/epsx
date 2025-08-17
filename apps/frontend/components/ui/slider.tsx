"use client"

import * as React from "react"

import { cn } from "@/lib/utils"

// Simplified slider component as fallback
// Provides basic functionality without @radix-ui/react-slider dependency

interface SliderProps extends React.HTMLAttributes<HTMLDivElement> {
  value?: number[]
  onValueChange?: (value: number[]) => void
  min?: number
  max?: number
  step?: number
  disabled?: boolean
}

const Slider = React.forwardRef<HTMLDivElement, SliderProps>(
  ({ className, value = [0], onValueChange, min = 0, max = 100, step = 1, disabled = false, ...props }, ref) => {
    const [internalValue, setInternalValue] = React.useState(value)

    const handleChange = (event: React.ChangeEvent<HTMLInputElement>) => {
      const newValue = [Number(event.target.value)]
      setInternalValue(newValue)
      onValueChange?.(newValue)
    }

    React.useEffect(() => {
      setInternalValue(value)
    }, [value])

    return (
      <div
        ref={ref}
        className={cn("relative flex w-full touch-none select-none items-center", className)}
        {...props}
      >
        <input
          type="range"
          min={min}
          max={max}
          step={step}
          value={internalValue[0]}
          onChange={handleChange}
          disabled={disabled}
          className={cn(
            "h-2 w-full cursor-pointer appearance-none rounded-lg bg-gray-200 accent-primary",
            "disabled:cursor-not-allowed disabled:opacity-50",
            "[&::-webkit-slider-thumb]:h-5 [&::-webkit-slider-thumb]:w-5 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-primary [&::-webkit-slider-thumb]:shadow-lg",
            "[&::-moz-range-thumb]:h-5 [&::-moz-range-thumb]:w-5 [&::-moz-range-thumb]:border-0 [&::-moz-range-thumb]:rounded-full [&::-moz-range-thumb]:bg-primary [&::-moz-range-thumb]:shadow-lg"
          )}
        />
      </div>
    )
  }
)
Slider.displayName = "Slider"

export { Slider }