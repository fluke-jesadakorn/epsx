
export interface Plan {
    id: number | string
    name: string
    description?: string
    plan_type: string
    current_price: number | string
    base_price?: number | string
    currency: string
    is_active?: boolean
    features: string[] | { text: string; included: boolean }[]
    permissions?: string[]
    is_highlighted?: boolean
    is_promoted?: boolean
    tier_level: number
}

export interface PricingCardData {
    id: number | string
    title: string
    price: string
    originalPrice?: string
    features: { text: string; included: boolean }[]
    highlight?: boolean
    buttonText: string
    promotions?: string[]
    badges?: string[]
    savings?: string
    // Extended properties for payment flow logic
    tier_level: number
    plan_type?: string
    description?: string
    is_current_plan?: boolean
}
