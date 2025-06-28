export enum UserLevel {
  Basic = 'Basic',
  Premium = 'Premium',
  VIP = 'VIP'
}

export interface PaymentStatus {
  lastPaymentDate: Date
  expirationDate: Date
  paymentMethod: 'USDT'
  transactionId: string
  amount: number
}

export interface USDTDetails {
  network: 'ERC20' | 'TRC20' | 'BEP20' | 'Arbitrum' | 'TON'
  walletAddress: string
  qrCodePath?: string
  tag?: string
  paymentStatus: PaymentStatus
  userLevel: UserLevel
}
