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
  network: 'ERC20' | 'TRC20'
  walletAddress: string
  paymentStatus: PaymentStatus
  userLevel: UserLevel
}
