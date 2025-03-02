"use client";

import React, { useEffect, Suspense } from "react";
import { usePathname, useSearchParams } from "next/navigation";
import { analytics } from "../lib/firebase-client";
import { logEvent } from "firebase/analytics";

const AnalyticsTracker = () => {
  const pathname = usePathname();
  const searchParams = useSearchParams();

  useEffect(() => {
    if (!analytics) return;

    // Construct full URL with search params
    const url = searchParams?.toString()
      ? `${pathname}?${searchParams.toString()}`
      : pathname;

    // Log page_view event
    logEvent(analytics, "page_view", {
      page_path: pathname,
      page_url: url,
      page_title: document.title,
    });
  }, [pathname, searchParams]);

  return null;
};

type FirebaseAnalyticsHook = {
  AnalyticsWrapper: React.ComponentType;
  logEvent: (eventName: string, eventParams?: Record<string, any>) => void;
};

export const useFirebaseAnalytics = (): FirebaseAnalyticsHook => {
  const AnalyticsWrapper = () => (
    <Suspense fallback={null}>
      <AnalyticsTracker />
    </Suspense>
  );

  return {
    AnalyticsWrapper,
    logEvent: (eventName: string, eventParams?: Record<string, any>) => {
      if (!analytics) return;
      logEvent(analytics, eventName, eventParams);
    },
  };
};
