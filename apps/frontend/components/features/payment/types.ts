// Raw API response interface (backend data)
export interface ApiPaymentPlan {
  id: number | string;
  name: string;
  plan_type: string;
  base_price?: number;
  current_price: number | string;
  effective_price?: number;
  currency: string;
  features: string[] | string;
  affiliate_commission_rate?: number;
  display_order?: number;
  is_active: boolean;
  is_highlighted?: boolean;
  created_at: string;
  updated_at: string;
  promotional_badge?: string;
  promotional_message?: string;
  discount_type?: string;
  discount_value?: number;
  max_discount_amount?: number;
  promotion_active?: boolean;
  promotion_status?: string;
  promotion_discount?: number;
  promotion_ends_at?: string;
}

// UI-enhanced payment package interface
export interface PaymentPackage
  extends Omit<ApiPaymentPlan, 'features' | 'current_price' | 'id'> {
  id: number | string;
  original_plan_id: number | string;
  features: string[];
  current_price: number;
  base_price: number;
  icon?: string;
  description?: string;
  popular?: boolean;
}

export type PaymentStep = 'package' | 'payment' | 'confirmation';

export interface OneClickPaymentProps {
  className?: string;
  preselectedPackage?: string;
}
