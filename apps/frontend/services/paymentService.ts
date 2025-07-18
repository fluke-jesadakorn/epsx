// Stub payment service
export class PaymentService {
  static async checkPremiumAccess() {
    return { hasAccess: true };
  }
  
  static async getPaymentStatus() {
    return { status: 'active' };
  }
}

export const paymentService = {
  checkPremiumAccess: PaymentService.checkPremiumAccess,
  getPaymentStatus: PaymentService.getPaymentStatus
};
