"use client";

import { PaymentForm } from "@/components/features/payment/PaymentForm";

export default function PaymentSettings() {
  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold">Payment Settings</h2>
      <div className="max-w-2xl">
        <PaymentForm />
      </div>
    </div>
  );
}
