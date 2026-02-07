import type { UnifiedApiClient, ApiResponse } from '../utils/api-client'

export interface Promotion {
  id: number
  name: string
  code: string
  description?: string
  discountType: 'percentage' | 'fixed'
  discountValue: string
  maxDiscountAmount?: string
  minPurchaseAmount?: string
  usageLimit?: number
  currentUsage: number
  isActive: boolean
  startDate: string
  endDate?: string
  applicablePlans: string[]
  createdAt: string
  updatedAt: string
  totalRevenue: string
  conversionRate: number
}

export interface CreatePromotionRequest {
  name: string
  code: string
  description?: string
  discountType: 'percentage' | 'fixed'
  discountValue: number
  maxDiscountAmount?: number
  minPurchaseAmount?: number
  usageLimit?: number
  isActive: boolean
  startDate: string
  endDate?: string
  applicablePlans: string[]
}

export interface UpdatePromotionRequest {
  name?: string
  description?: string
  isActive?: boolean
  discountValue?: number
  usageLimit?: number
}

export interface ListPromotionsParams {
  limit?: number
  offset?: number
  isActive?: boolean
}

export interface PromotionsResponse {
  promotions: Promotion[]
  total_count: number
  has_more: boolean
}

export function createPromotionsClient(apiClient: UnifiedApiClient) {
  return {
    async getPromotions(params?: ListPromotionsParams): Promise<ApiResponse<PromotionsResponse>> {
      const query = new URLSearchParams()
      if (params?.limit) {query.append('limit', params.limit.toString())}
      if (params?.offset) {query.append('offset', params.offset.toString())}
      if (params?.isActive !== undefined) {query.append('is_active', params.isActive.toString())}

      return apiClient.get(`/admin/promotions${query.toString() ? `?${query}` : ''}`)
    },

    async getPromotion(id: number): Promise<ApiResponse<Promotion>> {
      return apiClient.get(`/admin/promotions/${id}`)
    },

    async createPromotion(data: CreatePromotionRequest): Promise<ApiResponse<Promotion>> {
      return apiClient.post('/admin/promotions', data)
    },

    async updatePromotion(id: number, data: UpdatePromotionRequest): Promise<ApiResponse<Promotion>> {
      return apiClient.put(`/admin/promotions/${id}`, data)
    },

    async deletePromotion(id: number): Promise<ApiResponse<void>> {
      return apiClient.delete(`/admin/promotions/${id}`)
    },
  }
}

export function isApiSuccess<T>(response: ApiResponse<T>): response is ApiResponse<T> & { success: true; data: T } {
  return response.success === true
}
