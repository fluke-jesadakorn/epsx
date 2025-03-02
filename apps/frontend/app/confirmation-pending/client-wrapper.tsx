"use client";

import dynamic from "next/dynamic";
import { Suspense } from "react";

const ConfirmationClient = dynamic(
  () => import("./confirmation-client"),
  {
    loading: () => <div>Loading...</div>,
    ssr: false
  }
);

export default function ClientWrapper() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <ConfirmationClient />
    </Suspense>
  );
}
