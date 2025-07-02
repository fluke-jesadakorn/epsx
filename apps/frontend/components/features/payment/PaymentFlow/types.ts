export type PaymentMethod = 'on_line' | 'on_chain';
export type OrderStatus = 'pending' | 'processing' | 'completed' | 'failed';
export type PaymentStep = 'SELECT' | 'DETAILS' | 'MONITOR';

export interface Package {
  id: string;
  name: string;
  price: number;
  currency: string;
  description: string;
  features: string[];
}

export interface PaymentDetails {
  amount: number;
  currency: string;
  method: PaymentMethod;
  packageId: string;
}

export interface OrderDetails {
  id: string;
  status: OrderStatus;
  amount: number;
  currency: string;
  paymentMethod: PaymentMethod;
  receiveAddress?: string;
  checkoutUrl?: string;
  createdAt: Date;
}

export interface PaymentFlowState {
  currentStep: PaymentStep;
  selectedPackage: Package | null;
  paymentMethod: PaymentMethod | null;
  orderDetails: OrderDetails | null;
}
