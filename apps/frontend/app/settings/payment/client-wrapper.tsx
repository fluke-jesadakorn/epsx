"use client";

import dynamic from "next/dynamic";
import { Suspense } from "react";
import LoadingForm from "@/components/common/LoadingForm";

const PaymentClient = dynamic(() => import("./payment-client"), {
  loading: () => <LoadingForm>Loading...</LoadingForm>,
  ssr: false,
});

export default function ClientWrapper() {
  return (
    <Suspense fallback={<LoadingForm>Loading...</LoadingForm>}>
      <PaymentClient />
    </Suspense>
  );
}
