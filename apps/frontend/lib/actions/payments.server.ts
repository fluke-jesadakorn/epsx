'use server';

// Re-export from consolidated server actions package
export {
  createPayment,
  getPaymentStatus,
  validatePayment,
  getPaymentPlans,
  getPaymentPlan,
  getUserSubscription,
  cancelSubscription,
  updateSubscription,
  getPaymentHistory,
  downloadInvoice,
  applyPromoCode,
  createLegacyPayment,
  initQRPayment
} from '@epsx/server-actions/actions/payments';

// Re-export enhanced versions for direct use
export {
  enhancedCreatePayment,
  enhancedGetPaymentStatus,
  enhancedValidatePayment,
  enhancedGetPaymentPlans,
  enhancedGetPaymentPlan,
  enhancedGetUserSubscription,
  enhancedCancelSubscription,
  enhancedUpdateSubscription,
  enhancedGetPaymentHistory,
  enhancedDownloadInvoice,
  enhancedApplyPromoCode
} from '@epsx/server-actions/actions/payments';

// Legacy compatibility functions if needed
import { createLegacyPayment, initQRPayment } from '@epsx/server-actions/actions/payments';

export async function createMusePayPaymentAction(formData: FormData) {
  const amount = formData.get('amount') as string;
  const currency = formData.get('currency') as string;
  const description = formData.get('description') as string;
  const orderNo = formData.get('orderNo') as string;

  return await createLegacyPayment({
    amount: Number(amount),
    currency,
    description,
    orderNo
  });
}

export async function initQRPaymentAction(formData: FormData) {
  const amount = formData.get('amount') as string;
  const currency = formData.get('currency') as string;
  const orderNo = formData.get('orderNo') as string;

  return await initQRPayment({
    amount: Number(amount),
    currency,
    orderNo
  });
}