export interface AssetInfo {
  currency: string
  chain: string
  decimals: number
  depositThreshold: number
  addressFormat: string
}

export interface CreatePaymentRequest {
  currency: string
  amount: string
  payment_method: 'on_line' | 'on_chain'
  product_name: string
  notify_url?: string
}

export interface CreatePaymentResponse {
  request_id: string
  order_no: string
  currency: string
  order_amount: string
  status: number
  payment_method: string
  receive_address?: string
  checkout_url?: string
}
