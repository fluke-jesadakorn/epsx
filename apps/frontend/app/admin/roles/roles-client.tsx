"use client";

import dynamic from "next/dynamic";
import { Suspense } from "react";

const UserRoleManager = dynamic(
  () => import("@/components/admin/UserRoleManager").then(mod => ({ default: mod.UserRoleManager })),
  {
    loading: () => <div>Loading...</div>,
    ssr: false
  }
);

export default function RolesClient() {
  return (
    <div className="container mx-auto py-8">
      <Suspense fallback={<div>Loading...</div>}>
        <UserRoleManager />
      </Suspense>
    </div>
  );
}
