'use client';

import { useState } from 'react';
import { Card, CardContent } from '@/components/ui/card';
import PackageSelect from './PackageSelect';
import PaymentDetails from './PaymentDetails';
import StatusMonitor from './StatusMonitor';
import type { PaymentFlowState, Package, PaymentMethod, OrderDetails } from './types';

export default function PaymentFlow() {
  const [state, setState] = useState<PaymentFlowState>({
    currentStep: 'SELECT',
    selectedPackage: null,
    paymentMethod: null,
    orderDetails: null,
  });

  const handlePackageSelect = (selectedPackage: Package, method: PaymentMethod) => {
    setState(prev => ({
      ...prev,
      currentStep: 'DETAILS',
      selectedPackage,
      paymentMethod: method,
    }));
  };

  const handlePaymentSubmit = async (orderDetails: OrderDetails) => {
    setState(prev => ({
      ...prev,
      currentStep: 'MONITOR',
      orderDetails,
    }));
  };

  const handleStatusChange = (status: OrderDetails['status']) => {
    if (state.orderDetails) {
      setState(prev => ({
        ...prev,
        orderDetails: {
          ...prev.orderDetails!,
          status,
        },
      }));
    }
  };

  return (
    <Card className="w-full max-w-4xl mx-auto">
      <CardContent className="p-6">
        {state.currentStep === 'SELECT' && (
          <PackageSelect
            onSelect={handlePackageSelect}
          />
        )}

        {state.currentStep === 'DETAILS' && state.selectedPackage && state.paymentMethod && (
          <PaymentDetails
            package={state.selectedPackage}
            method={state.paymentMethod}
            onSubmit={handlePaymentSubmit}
          />
        )}

        {state.currentStep === 'MONITOR' && state.orderDetails && (
          <StatusMonitor
            orderId={state.orderDetails.id}
            status={state.orderDetails.status}
            onStatusChange={handleStatusChange}
          />
        )}
      </CardContent>
    </Card>
  );
}
