'use client';

import { useRouter } from 'next/navigation';
import { useEffect, useState } from 'react';

import { PlanManagement } from '@/components/plans/PlanManagement';
import { PromotionManagement } from '@/components/promotions/PromotionManagement';
import { useSharedAuth } from '@/shared/components/auth/Provider';

function PlansHubSkeleton() {
  return (
    <div className="min-h-screen p-6">
      <div className="max-w-7xl mx-auto">
        {/* Hero section skeleton */}
        <div className="text-center mb-12">
          <div className="h-16 bg-primary/20 rounded-2xl w-96 mx-auto mb-4 scale-105 shadow-sm"></div>
          <div className="h-6 bg-muted rounded-full w-64 mx-auto "></div>
        </div>

        {/* Action cards skeleton */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
          {Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="bg-card rounded-3xl p-8 border border-border/50">
              <div className="h-12 w-12 bg-muted rounded-2xl mb-6"></div>
              <div className="h-8 bg-muted rounded-xl mb-4 w-3/4"></div>
              <div className="h-5 bg-muted/60 rounded-lg mb-6 w-full"></div>
              <div className="h-12 bg-primary/20 rounded-2xl"></div>
            </div>
          ))}
        </div>

        {/* Stats grid skeleton */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          {Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className="bg-card rounded-3xl p-6 border border-border/50 ">
              <div className="h-6 bg-primary/10 rounded-lg mb-4 w-1/2"></div>
              <div className="h-12 bg-muted rounded-xl mb-2 w-3/4"></div>
              <div className="h-4 bg-muted/60 rounded-lg w-1/3"></div>
            </div>
          ))}
        </div>

        {/* Plans table skeleton */}
        <div className="bg-card rounded-3xl border border-border/30 overflow-hidden">
          <div className="p-8">
            <div className="h-8 bg-primary/10 rounded-xl mb-6 w-1/3 "></div>
            <div className="space-y-4">
              {Array.from({ length: 8 }).map((_, i) => (
                <div key={i} className="flex items-center justify-between p-4 bg-muted/20 rounded-2xl ">
                  <div className="flex items-center gap-4 flex-1">
                    <div className="h-10 w-10 bg-primary/10 rounded-2xl"></div>
                    <div className="space-y-2 flex-1">
                      <div className="h-5 bg-muted rounded-lg w-1/3"></div>
                      <div className="h-4 bg-muted/60 rounded-lg w-1/2"></div>
                    </div>
                  </div>
                  <div className="h-8 w-24 bg-primary/10 rounded-full"></div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

/**
 *
 */
export default function AdminPlansPage() {
  const { user, isAuthenticated, isLoading } = useSharedAuth()
  const router = useRouter()
  const [activeTab, setActiveTab] = useState<'plans' | 'promotions'>('plans');

  // Redirect to auth if not authenticated
  useEffect(() => {
    if (!isLoading && !isAuthenticated) {
      router.push('/auth')
    }
  }, [isAuthenticated, isLoading, router])

  // Show loading state while checking authentication
  if (isLoading) {
    return <PlansHubSkeleton />
  }

  // Show loading state if not authenticated (will redirect)
  if (!isAuthenticated) {
    return <PlansHubSkeleton />
  }

  return (
    <div className="min-h-screen p-6">
      <div className="max-w-7xl mx-auto mb-8">
        {/* Tab Navigation */}
        <div className="p-1 rounded-2xl bg-muted/30 border border-border/50 backdrop-blur-sm">
          <div className="grid grid-cols-2 gap-2">
            <button
              onClick={() => setActiveTab('plans')}
              className={`px-6 py-4 rounded-xl font-bold text-base transition-all duration-300 ${activeTab === 'plans'
                ? 'bg-primary text-primary-foreground shadow-lg shadow-primary/20 scale-[1.02]'
                : 'text-muted-foreground hover:bg-muted hover:text-foreground'
                }`}
            >
              💳 Plans Management
            </button>
            <button
              onClick={() => setActiveTab('promotions')}
              className={`px-6 py-4 rounded-xl font-bold text-base transition-all duration-300 ${activeTab === 'promotions'
                ? 'bg-secondary text-secondary-foreground shadow-lg shadow-secondary/20 scale-[1.02]'
                : 'text-muted-foreground hover:bg-muted hover:text-foreground'
                }`}
            >
              🎁 Promotions & Deals
            </button>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto">
        {activeTab === 'plans' ? (
          <PlanManagement />
        ) : (
          <PromotionManagement />
        )}
      </div>
    </div>
  )
}