
'use client'

import { getPublicPlansAction } from '@/app/actions/plans'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { usePlanAccess } from '@/hooks/use-plan-access'
import { createCreditsApi } from '@/shared/api/credits'
import { PricingCard } from '@/shared/components/plans/pricing-card'
import type { Plan, PlanGroup, PricingCardData } from '@/shared/types/plans'
import { createFrontendApiClient } from '@/shared/utils/api-client'
import { fmtAmt } from '@/shared/utils/formatting/currency'
import { AlertCircle, Building2, Code2, Star, User as UserIcon } from 'lucide-react'
import { useRouter, useSearchParams } from 'next/navigation'
import type { ReactNode } from 'react'
import { useEffect, useState } from 'react'

interface UserData {
  [key: string]: unknown;
}

interface PlanSelectionProps {
  currentUser?: UserData;
  className?: string;
}

export function PlanSelection({ currentUser: _currentUser, className }: PlanSelectionProps) {
  const [pricingCards, setPricingCards] = useState<PricingCardData[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [affiliateCode, setAffiliateCode] = useState<string | null>(null)
  const [affiliateInfo, _setAffiliateInfo] = useState<Record<string, unknown> | null>(null)
  const [creditBalance, setCreditBalance] = useState<number>(0)

  const router = useRouter()
  const searchParams = useSearchParams()
  const { planAccess } = usePlanAccess()

  const PLAN_GROUPS: PlanGroup[] = ['personal', 'enterprise', 'api', 'custom']

  const GROUP_CONFIG: Record<string, { label: string; desc: string; icon: ReactNode }> = {
    personal: { label: 'Personal Plans', desc: 'For individual traders and analysts', icon: <UserIcon className="h-6 w-6" /> },
    enterprise: { label: 'Enterprise Plans', desc: 'For teams and organizations', icon: <Building2 className="h-6 w-6" /> },
    api: { label: 'API Plans', desc: 'For developers and integrations', icon: <Code2 className="h-6 w-6" /> },
    custom: { label: 'Custom Plans', desc: 'Tailored solutions for partners and enterprises', icon: <Star className="h-6 w-6" /> },
  }

  // Transform backend plan to pricing card format
  const transformToPricingCard = (plan: Plan): PricingCardData => {
    const basePrice = typeof plan.current_price === 'string'
      ? parseFloat(plan.current_price)
      : plan.current_price

    const hasPromo = plan.promotion_active === true &&
      plan.effective_price !== undefined &&
      plan.effective_price < basePrice

    const displayPrice = hasPromo ? (plan.effective_price ?? basePrice) : basePrice
    const isFree = displayPrice === 0

    const discount = plan.promotion_discount ?? 0
    const savedAmt = hasPromo ? basePrice - displayPrice : 0

    return {
      id: plan.id,
      title: plan.name,
      price: isFree ? 'Free' : `$${fmtAmt(displayPrice)} ${plan.currency || 'USD'}`,
      originalPrice: hasPromo ? `$${fmtAmt(basePrice)} ${plan.currency || 'USD'}` : undefined,
      features: Array.isArray(plan.features)
        ? plan.features.map(f => typeof f === 'string' ? { text: f, included: true } : f)
        : [],
      highlight: plan.is_highlighted ?? plan.is_promoted ?? false,
      buttonText: isFree ? 'Start Free' : 'Get Started',
      promotions: hasPromo ? [`${Math.round(discount)}% OFF`] : [],
      badges: [],
      savings: hasPromo ? `Save $${fmt(savedAmt)}` : undefined,
      promotion_ends_at: hasPromo ? plan.promotion_ends_at : undefined,
      tier_level: plan.tier_level,
      plan_type: plan.plan_type,
      description: plan.description,
      plan_group: plan.plan_group,
    }
  }

  // Extract affiliate code from URL parameters
  useEffect(() => {
    const refCode = searchParams.get('ref') ?? searchParams.get('affiliate') ?? searchParams.get('aff')
    if ((refCode?.length ?? 0) > 0) {
      setAffiliateCode(refCode)
      // Store in cookie for persistence
      document.cookie = `affiliate_code=${encodeURIComponent(refCode ?? '')}; path=/; max-age=2592000; SameSite=lax`
    } else {
      // Check cookies for existing affiliate code
      const cookies = document.cookie.split(';').reduce<Record<string, string>>((acc, cookie) => {
        const [key, value] = cookie.trim().split('=')
        if (key && value) { acc[key] = value }
        return acc
      }, {})

      const storedCode = cookies.affiliate_code ?? localStorage.getItem('affiliateCode')
      if ((storedCode?.length ?? 0) > 0) {
        setAffiliateCode(decodeURIComponent(storedCode ?? ''))
      }
    }
  }, [searchParams])

  // Fetch credit balance
  useEffect(() => {
    const fetchCreditBalance = async () => {
      try {
        const apiClient = createFrontendApiClient()
        const creditsApi = createCreditsApi(apiClient)
        const res = await creditsApi.getBalance()

        if (res.success && res.data) {
          setCreditBalance(Number(res.data.available_balance));
        }
      } catch (_err) {
        // Silent failure - no credits available
        setCreditBalance(0)
      }
    }

    void fetchCreditBalance()
  }, [])

  // Fetch plans from backend
  useEffect(() => {
    const fetchPlans = async () => {
      try {
        setLoading(true)
        const response = await getPublicPlansAction({
          affiliate_code: affiliateCode ?? undefined
        })

        if (response.success && response.data && Array.isArray(response.data)) {
          const cards = response.data
            .filter((plan: Plan) => (plan.is_active ?? false))
            .sort((a: Plan, b: Plan) => (a.display_order ?? 0) - (b.display_order ?? 0))
            .map((plan: Plan) => transformToPricingCard(plan))

          setPricingCards(cards)
        } else {
          throw new Error(response.error?.message ?? 'Invalid API response')
        }
      } catch (_err) {
        // Error logged silently
        setError('Failed to load plans. Please try again later.')
        setPricingCards([])
      } finally {
        setLoading(false)
      }
    }

    void fetchPlans()
  }, [affiliateCode])

  // Handle plan selection - redirect to payment
  const handlePlanClick = (card: PricingCardData) => {
    const planId = card.id.toString()
    const paymentUrl = (affiliateCode?.length ?? 0) > 0
      ? `/payment?planId=${planId}&ref=${affiliateCode ?? ''}`
      : `/payment?planId=${planId}`

    router.push(paymentUrl)
  }

  if ((error?.length ?? 0) > 0) {
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
      {(affiliateCode?.length ?? 0) > 0 && affiliateInfo && (
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

      {/* Pricing Cards - Grouped by plan_group */}
      {loading ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {Array.from({ length: 3 }).map((_, i) => (
            <div key={`plan-skeleton-${String(i)}`} className="animate-pulse bg-gray-200 dark:bg-gray-800 rounded-2xl h-[500px]" />
          ))}
        </div>
      ) : pricingCards.length === 0 ? (
        <div className="text-center py-12">
          <h3 className="text-xl font-semibold text-gray-600 dark:text-gray-400 mb-2">
            No plans available
          </h3>
          <p className="text-gray-500">Check back later for subscription plans.</p>
        </div>
      ) : (
        <div className="space-y-16 px-4 py-8">
          {PLAN_GROUPS.map((group) => {
            const groupCards = pricingCards.filter((c) => (c.plan_group ?? 'personal') === group)
            if (groupCards.length === 0) {return null}
            const cfg = GROUP_CONFIG[group]
            return (
              <section key={group}>
                <div className="flex items-center gap-3 mb-8">
                  <div className="p-2 rounded-xl bg-gradient-to-br from-emerald-500/20 to-blue-500/20 text-emerald-600 dark:text-emerald-400">
                    {cfg.icon}
                  </div>
                  <div>
                    <h2 className="text-2xl font-bold text-gray-900 dark:text-white">{cfg.label}</h2>
                    <p className="text-sm text-gray-500 dark:text-gray-400">{cfg.desc}</p>
                  </div>
                </div>
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                  {groupCards.map((card) => {
                    let actionType: 'extend' | 'upgrade' | 'downgrade' | 'select' | 'locked' = 'select'
                    let buttonTextOverride: string | undefined
                    let isSelected = false

                    if (planAccess) {
                      if (planAccess.plan_name === card.title) {
                        actionType = 'extend'
                        buttonTextOverride = 'Extend Plan'
                        isSelected = true
                      } else if ((card.tier_level ?? 0) > planAccess.tier_level) {
                        actionType = 'upgrade'
                      } else if ((card.tier_level ?? 0) < planAccess.tier_level) {
                        actionType = 'locked'
                      }
                    }

                    // Combine proration credit + wallet credit for upgrade cards
                    const prorationCredit = parseFloat(planAccess?.proration_credit ?? '0') || 0
                    const totalCredit = actionType === 'upgrade' ? creditBalance + prorationCredit : undefined

                    const finalCard = (buttonTextOverride?.length ?? 0) > 0
                      ? { ...card, buttonText: buttonTextOverride ?? '' }
                      : card

                    return (
                      <PricingCard
                        key={card.id}
                        card={finalCard}
                        onSelect={handlePlanClick}
                        affiliateInfo={affiliateInfo}
                        affiliateCode={affiliateCode}
                        actionType={actionType}
                        isSelected={isSelected}
                        isDisabled={actionType === 'locked'}
                        creditBalance={totalCredit}
                      />
                    )
                  })}
                </div>
              </section>
            )
          })}

        </div>
      )}
    </div>
  )
}

export default PlanSelection