export interface PlanFormData {
    name: string
    description: string
    current_price: number
    is_active: boolean
    api_calls_limit: number
    ranking_offset: number
    analytics_queries: number
    premium_features: boolean
    export_limit: number
    promo_enabled: boolean
    promo_type: 'percentage' | 'fixed'
    promo_value: number
    promo_price: number
    promo_start: string
    promo_end: string
}

export interface PlanFormProps {
    formData: PlanFormData
    setFormData: (data: PlanFormData) => void
}
