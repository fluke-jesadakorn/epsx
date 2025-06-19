"use client";

import React, { useEffect, Suspense } from "react";
import { usePathname, useSearchParams } from "next/navigation";
import { analytics } from "@/lib/firebase";
import { logEvent } from "firebase/analytics";

const AnalyticsTracker = () => {
  const pathname = usePathname();
  const searchParams = useSearchParams();

  useEffect(() => {
    const initAnalytics = async () => {
      if (!analytics) return;
      
      const analyticsInstance = await analytics;
      
      // Construct full URL with search params
      const url = searchParams?.toString()
        ? `${pathname}?${searchParams.toString()}`
        : pathname;

      // Log page_view event
      logEvent(analyticsInstance, "page_view", {
        page_path: pathname,
        page_url: url,
        page_title: document.title,
      });
    };

    initAnalytics();
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
    logEvent: async (eventName: string, eventParams?: Record<string, any>) => {
      if (!analytics) return;
      const analyticsInstance = await analytics;
      logEvent(analyticsInstance, eventName, eventParams);
    },
  };
};
