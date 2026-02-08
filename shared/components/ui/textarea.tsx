"use client"

import { cva, type VariantProps } from "class-variance-authority"
import * as React from "react"

import { cn } from "@shared/utils"

const textareaVariants = cva(
    "flex w-full min-h-[80px] resize-y rounded-md border text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50",
    {
        variants: {
            variant: {
                default: "border-input bg-background",
                wp: "border-2 border-secondary/30 bg-card text-foreground focus-visible:border-secondary focus-visible:ring-2 focus-visible:ring-secondary/20 hover:border-secondary/50 shadow-sm hover:shadow-md rounded-none",
                pancake: "border-2 border-primary/30 bg-gradient-to-r from-background to-primary/5 text-foreground focus-visible:border-primary focus-visible:ring-2 focus-visible:ring-primary/20 hover:border-primary/50 shadow-lg hover:shadow-xl rounded-xl",
                ghost: "border-0 border-b-2 border-muted-foreground/30 bg-transparent rounded-none text-foreground focus-visible:border-primary focus-visible:ring-0 hover:border-muted-foreground/50 px-0",
                tile: "border-0 bg-gradient-to-br from-card to-muted rounded-xl text-foreground focus-visible:ring-2 focus-visible:ring-primary shadow-lg hover:shadow-xl",
                outlined: "border-2 border-primary bg-transparent text-foreground focus-visible:bg-primary/5 focus-visible:border-primary focus-visible:ring-2 focus-visible:ring-primary/30 hover:border-primary/80",
                admin: "border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus-visible:border-primary focus-visible:ring-2 focus-visible:ring-primary/20 hover:border-gray-400 dark:hover:border-gray-500 rounded-lg"
            },
            size: {
                default: "min-h-[80px] px-3 py-2",
                sm: "min-h-[60px] px-3 py-2 text-sm rounded-md",
                lg: "min-h-[120px] px-6 py-4 text-base rounded-xl",
                xl: "min-h-[160px] px-8 py-5 text-lg rounded-2xl",
                tile: "min-h-[140px] px-6 py-5 text-lg rounded-2xl", // Windows Phone live tile size
            },
            state: {
                default: "",
                error: "border-destructive focus-visible:border-destructive focus-visible:ring-destructive/20 text-destructive-foreground",
                success: "border-success focus-visible:border-success focus-visible:ring-success/20",
                warning: "border-warning focus-visible:border-warning focus-visible:ring-warning/20",
            }
        },
        defaultVariants: {
            variant: "default",
            size: "default",
            state: "default",
        },
    }
)

export interface TextareaProps
    extends React.TextareaHTMLAttributes<HTMLTextAreaElement>,
    VariantProps<typeof textareaVariants> { }

const Textarea = React.forwardRef<HTMLTextAreaElement, TextareaProps>(
    ({ className, variant, size, state, ...props }, ref) => {
        return (
            <div className="relative w-full">
                <textarea
                    className={cn(textareaVariants({ variant, size, state, className }))}
                    ref={ref}
                    {...props}
                />

                {/* Windows Phone accent dot for certain variants */}
                {(variant === "pancake" || variant === "tile") && (
                    <div className="absolute bottom-2 right-2 w-1 h-1 bg-primary/60 rounded-full pointer-events-none" />
                )}

                {/* PancakeSwap corner accent */}
                {variant === "pancake" && (
                    <div className="absolute top-0 right-0 w-3 h-3 bg-gradient-to-bl from-primary/30 to-transparent rounded-tr-lg pointer-events-none" />
                )}
            </div>
        )
    }
)
Textarea.displayName = "Textarea"

export { Textarea, textareaVariants }
