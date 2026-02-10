/**
 * Windows Phone + PancakeSwap Breadcrumb Component
 * Enhanced navigation with pivot-style interactions and yellow highlights
 */

'use client'

import { ChevronRight, Home } from 'lucide-react'
import Link from 'next/link'
import React from 'react'

import { cn } from '@/lib/utils'

export interface BreadcrumbItem {
  label: string
  href?: string
  icon?: React.ComponentType<{ className?: string }>
  isActive?: boolean
}

interface BreadcrumbProps {
  items?: BreadcrumbItem[]
  className?: string
  showHome?: boolean
  variant?: 'default' | 'pivot' | 'minimal'
}

/**
 *
 * @param root0
 * @param root0.items
 * @param root0.className
 * @param root0.showHome
 * @param root0.variant
 */
export function Breadcrumb({
  items = [],
  className,
  showHome = true,
  variant = 'pivot'
}: BreadcrumbProps) {
  const allItems = showHome 
    ? [{ label: 'Home', href: '/users', icon: Home }, ...items]
    : items

  if (variant === 'pivot') {
    return (
      <nav className={cn("mb-6", className)} aria-label="breadcrumb">
        {/* Windows Phone pivot-style breadcrumb */}
        <div className="flex items-center overflow-x-auto pb-2">
          <div className="flex items-center gap-1 min-w-max">
            {allItems.map((item, index) => {
              const isLast = index === allItems.length - 1
              const Icon = item.icon
              
              return (
                <div key={index} className="flex items-center">
                  {item.href && !isLast ? (
                    <Link
                      href={item.href}
                      className={cn(
                        "group flex items-center gap-2 px-4 py-3 rounded-lg transition-all duration-300",
                        "text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100",
                        "hover:bg-yellow-50 dark:hover:bg-yellow-900/10",
                        "border-2 border-transparent hover:border-yellow-400/30",
                        "font-light tracking-wide"
                      )}
                    >
                      {Icon && (
                        <Icon className="h-4 w-4 group-hover:text-yellow-500 transition-colors" />
                      )}
                      <span className="group-hover:font-normal transition-all">
                        {item.label}
                      </span>
                      
                      {/* Windows Phone accent line on hover */}
                      <div className="absolute bottom-0 left-4 right-4 h-0.5 bg-yellow-400 scale-x-0 group-hover:scale-x-100 transition-transform duration-300" />
                    </Link>
                  ) : (
                    <div
                      className={cn(
                        "flex items-center gap-2 px-4 py-3 rounded-lg",
                        "bg-gradient-to-r from-yellow-400 to-orange-500 text-gray-900 dark:text-gray-900",
                        "font-medium tracking-wide shadow-lg",
                        "relative overflow-hidden"
                      )}
                    >
                      {/* PancakeSwap corner accent */}
                      <div className="absolute top-0 right-0 w-3 h-3 bg-gradient-to-bl from-white/30 to-transparent" />
                      
                      {Icon && <Icon className="h-4 w-4" />}
                      <span className="relative z-10">{item.label}</span>
                      
                      {/* Windows Phone accent dot */}
                      <div className="absolute bottom-1 right-1 w-1 h-1 bg-white/60 rounded-full" />
                    </div>
                  )}
                  
                  {!isLast && (
                    <div className="flex items-center mx-2">
                      <ChevronRight className="h-4 w-4 text-yellow-500" />
                    </div>
                  )}
                </div>
              )
            })}
          </div>
        </div>
        
        {/* Windows Phone underline accent */}
        <div className="w-full h-px bg-gradient-to-r from-transparent via-yellow-400/20 to-transparent" />
      </nav>
    )
  }

  if (variant === 'minimal') {
    return (
      <nav className={cn("flex items-center gap-1 text-sm text-gray-500 dark:text-gray-400 mb-4", className)}>
        {allItems.map((item, index) => {
          const isLast = index === allItems.length - 1
          const Icon = item.icon
          
          return (
            <React.Fragment key={index}>
              {item.href && !isLast ? (
                <Link
                  href={item.href}
                  className="hover:text-yellow-500 dark:hover:text-yellow-400 flex items-center gap-1 transition-colors"
                >
                  {Icon && <Icon className="h-3 w-3" />}
                  {item.label}
                </Link>
              ) : (
                <span className="text-gray-900 dark:text-gray-100 font-medium flex items-center gap-1">
                  {Icon && <Icon className="h-3 w-3" />}
                  {item.label}
                </span>
              )}
              
              {!isLast && <ChevronRight className="h-3 w-3 text-gray-400" />}
            </React.Fragment>
          )
        })}
      </nav>
    )
  }

  // Default variant
  return (
    <nav className={cn("flex items-center gap-2 text-sm mb-4", className)} aria-label="breadcrumb">
      {allItems.map((item, index) => {
        const isLast = index === allItems.length - 1
        const Icon = item.icon
        
        return (
          <div key={index} className="flex items-center gap-2">
            {item.href && !isLast ? (
              <Link
                href={item.href}
                className={cn(
                  "flex items-center gap-1 px-2 py-1 rounded-md transition-all",
                  "text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100",
                  "hover:bg-yellow-50 dark:hover:bg-yellow-900/10"
                )}
              >
                {Icon && <Icon className="h-4 w-4" />}
                {item.label}
              </Link>
            ) : (
              <span className="text-gray-900 dark:text-gray-100 font-medium flex items-center gap-1">
                {Icon && <Icon className="h-4 w-4" />}
                {item.label}
              </span>
            )}
            
            {!isLast && <ChevronRight className="h-4 w-4 text-gray-400" />}
          </div>
        )
      })}
    </nav>
  )
}