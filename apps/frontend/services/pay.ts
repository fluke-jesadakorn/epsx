import { createPaymentService } from './payment.service';

// Create payment service instance
const paymentService = createPaymentService();

// Export the status function for compatibility with existing imports
export const status = paymentService.getPaymentStatus;

// Export other functions that might be needed
export const recordPayment = paymentService.recordPayment;
export const confirmPayment = paymentService.confirmPayment;
export const getTxHistory = paymentService.getTxHistory;
export const getTxHistoryForNewUser = paymentService.getTxHistoryForNewUser;

// Export types
export type { PaymentStatus, PaymentTx } from './payment.service';