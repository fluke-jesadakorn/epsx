'use client'

import { getPublicPlansAction } from '@/app/actions/plans'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { usePlanAccess } from '@/hooks/use-plan-access'
import { PricingCard } from '@/shared/components/plans/pricing-card'
import type { Plan, PricingCardData } from '@/shared/types/plans'
import { AlertCircle, Star } from 'lucide-react'
import { useRouter, useSearchParams } from 'next/navigation'
import { useEffect, useState } from 'react'

interface PlanSelectionProps {
  currentUser?: any
  className?: string
}

export function PlanSelection({ currentUser, className }: PlanSelectionProps) {
  const [pricingCards, setPricingCards] = useState<PricingCardData[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [affiliateCode, setAffiliateCode] = useState<string | null>(null)
  const [affiliateInfo, setAffiliateInfo] = useState<any>(null)

  const router = useRouter()
  const searchParams = useSearchParams()
  const { planAccess } = usePlanAccess()

  // Extract affiliate code from URL parameters
  useEffect(() => {
    const refCode = searchParams.get('ref') || searchParams.get('affiliate') || searchParams.get('aff')
    if (refCode) {
      setAffiliateCode(refCode)
      // Store in cookie for persistence
      document.cookie = `affiliate_code=${encodeURIComponent(refCode)}; path=/; max-age=2592000; SameSite=lax`
    } else {
      // Check cookies for existing affiliate code
      const cookies = document.cookie.split(';').reduce<Record<string, string>>((acc, cookie) => {
        const [key, value] = cookie.trim().split('=')
        if (key && value) {acc[key] = value}
        return acc
      }, {})

      const storedCode = cookies.affiliate_code || localStorage.getItem('affiliateCode')
      if (storedCode) {
        setAffiliateCode(decodeURIComponent(storedCode))
      }
    }
  }, [searchParams])

  // Fetch plans from backend
  useEffect(() => {
    const fetchPlans = async () => {
      try {
        setLoading(true)
        const response = await getPublicPlansAction({
          affiliate_code: affiliateCode || undefined
        } as any)

        if (response.success && response.data && Array.isArray(response.data)) {
          const cards = response.data
            .filter((plan: Plan) => plan.is_active)
            .sort((a: Plan, b: Plan) => (a.display_order || 0) - (b.display_order || 0))
            .map((plan: Plan) => transformToPricingCard(plan))

          setPricingCards(cards)
        } else {
          throw new Error(response.error?.message || 'Invalid API response')
        }
      } catch (err) {
        console.error('[PlanSelection] Error fetching plans:', err)
        setError('Failed to load plans. Please try again later.')
        setPricingCards([])
      } finally {
        setLoading(false)
      }
    }

    fetchPlans()
  }, [affiliateCode])

  // Transform backend plan to pricing card format
  const transformToPricingCard = (plan: Plan): PricingCardData => {
    const price = typeof plan.current_price === 'string'
      ? parseFloat(plan.current_price)
      : plan.current_price
    const isFree = price === 0

    return {
      id: plan.id,
      title: plan.name,
      price: isFree ? 'Free' : `$${price.toFixed(2)} ${plan.currency || 'USD'}`,
      features: Array.isArray(plan.features)
        ? plan.features.map(f => typeof f === 'string' ? { text: f, included: true } : f)
        : [],
      highlight: plan.is_highlighted || plan.is_promoted || false,
      buttonText: isFree ? 'Start Free' : 'Get Started',
      promotions: [],
      badges: [],
      tier_level: plan.tier_level,
      plan_type: plan.plan_type,
      description: plan.description
    }
  }

  // Handle plan selection - redirect to payment
  const handlePlanClick = (card: PricingCardData) => {
    // Direct navigation to dynamic route
    const planId = card.id.toString()
    let paymentUrl = ''

    if (affiliateCode) {
      paymentUrl = `/payment/plan/${planId}?ref=${affiliateCode}`
    } else {
      paymentUrl = `/payment/plan/${planId}`
    }

    router.push(paymentUrl)
  }

  if (error) {
    return (
      <Alert variant="destructive">
        <AlertCircle className="h-4 w-4" />
        <AlertDescription>{error}</AlertDescription>
      </Alert>
    )
  }

  return (
    <div className={className}>
      {/* Affiliate attribution banner */}
      {affiliateCode && affiliateInfo && (
        <div className="mb-8">
          <div className="bg-gradient-to-r from-purple-500 to-pink-500 text-white p-4 rounded-2xl text-center shadow-xl">
            <div className="flex items-center justify-center gap-2 text-lg font-semibold">
              <Star className="h-5 w-5" />
              <span>You're eligible for {affiliateInfo.commission_rate}% affiliate rewards!</span>
              <Star className="h-5 w-5" />
            </div>
            <p className="text-sm mt-1 opacity-90">
              Referred by partner: {affiliateCode} • Special pricing applied
            </p>
          </div>
        </div>
      )}

      {/* Pricing Cards Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8 md:gap-12 lg:gap-16 px-4 py-8">
        {loading
          ? Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="animate-pulse bg-gray-200 dark:bg-gray-800 rounded-2xl h-[500px]" />
          ))
          : pricingCards.map((card) => {
            // Determine action type based on current plan
            let actionType: 'extend' | 'upgrade' | 'downgrade' | 'select' | 'locked' = 'select';
            let buttonTextOverride = undefined;
            let isSelected = false;

            if (planAccess) {
              if (planAccess.plan_name === card.title) {
                actionType = 'extend';
                buttonTextOverride = 'Extend Plan';
                isSelected = true;
              } else if ((card.tier_level || 0) > planAccess.tier_level) {
                actionType = 'upgrade';
              } else if ((card.tier_level || 0) < planAccess.tier_level) {
                actionType = 'downgrade';
                buttonTextOverride = 'Switch Plan'; // Will queue after expiry
              }
            }

            // Override button text if needed
            const finalCard = buttonTextOverride
              ? { ...card, buttonText: buttonTextOverride }
              : card;

            return (
              <PricingCard
                key={card.id}
                card={finalCard}
                onSelect={handlePlanClick}
                affiliateInfo={affiliateInfo}
                affiliateCode={affiliateCode}
                actionType={actionType}
                isSelected={isSelected}
              />
            )
          })
        }
      </div>

      {!loading && pricingCards.length === 0 && (
        <div className="text-center py-12">
          <div className="text-6xl mb-4">🔍</div>
          <h3 className="text-xl font-semibold text-gray-600 dark:text-gray-400 mb-2">
            No plans available
          </h3>
          <p className="text-gray-500">Check back later for subscription plans.</p>
        </div>
      )}
    </div>
  )
}

export default PlanSelection