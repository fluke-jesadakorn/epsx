"use client";

import Link from "next/link";
import { useState, useEffect } from "react";

import { getCookieConsent, setCookieConsent  } from "@/app/actions/cookieActions";

import type {ConsentStatus} from "@/app/actions/cookieActions";

export default function CookieBanner() {
  const [showBanner, setShowBanner] = useState(false);

  useEffect(() => {
    const checkConsent = async () => {
      const consent = await getCookieConsent();
      setShowBanner(consent === null);
    };
    checkConsent();
  }, []);

  const handleConsent = async (status: ConsentStatus) => {
    if (status) {
      await setCookieConsent(status);
      setShowBanner(false);
    }
  };

  if (!showBanner) return null;

  return (
    <div className="fixed bottom-0 left-0 right-0 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 z-50 border-t">
      <div className="container flex flex-col sm:flex-row items-center gap-4 py-4 px-6 md:px-8">
        <div className="text-sm flex-1">
          We use cookies to enhance your browsing experience and analyze our traffic. 
          By clicking &quot;Accept&quot;, you consent to our use of cookies. Learn more in our{" "}
          <Link href="/privacy" className="font-medium underline underline-offset-4 hover:text-primary">
            Privacy Policy
          </Link>
          .
        </div>
        <div className="flex gap-3">
          <button
            onClick={() => handleConsent("rejected")}
            className="shrink-0 px-4 py-2 text-sm font-semibold border border-gray-300 rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-500"
          >
            Reject
          </button>
          <button
            onClick={() => handleConsent("accepted")}
            className="shrink-0 px-4 py-2 text-sm font-semibold bg-primary text-primary-foreground rounded-md hover:bg-primary/90 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary"
          >
            Accept
          </button>
        </div>
      </div>
    </div>
  );
}
