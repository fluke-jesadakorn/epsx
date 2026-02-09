
import type { Promotion } from '@/shared/api/promotions';

export interface DisplayPromotion extends Omit<Promotion, 'discountValue' | 'maxDiscountAmount' | 'minPurchaseAmount' | 'totalRevenue'> {
    discountValue: number;
    maxDiscountAmount: number | null;
    minPurchaseAmount: number;
    totalRevenue: number;
}
