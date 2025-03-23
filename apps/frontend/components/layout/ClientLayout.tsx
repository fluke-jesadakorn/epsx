"use client";

import React, { Suspense } from "react";
import Link from "next/link";
import { ThemeProvider } from "./ThemeProvider";
import { Toaster } from "sonner";
import { useFirebaseAnalytics } from "@/hooks/useFirebaseAnalytics";
import DefaultLoadingForm from "../common/LoadingForm";
import CookieBanner from "../common/CookieBanner";

interface ClientLayoutProps {
  children: React.ReactNode;
}

const ClientLayout: React.FC<ClientLayoutProps> = ({ children }) => {
  const { AnalyticsWrapper } = useFirebaseAnalytics();
  return (
    <ThemeProvider>
      <Suspense fallback={<DefaultLoadingForm>Loading...</DefaultLoadingForm>}>
        <AnalyticsWrapper />
        <div className="min-h-screen flex flex-col">
          <main className="flex-grow">{children}</main>
          <footer className="py-6 px-4 border-t">
            <div className="max-w-7xl mx-auto flex flex-col items-center gap-4">
              <nav className="flex gap-6">
                <Link
                  href="/terms"
                  className="text-sm text-primary no-underline hover:no-underline"
                >
                  Terms of Service
                </Link>
                <Link
                  href="/privacy"
                  className="text-sm text-primary no-underline hover:no-underline"
                >
                  Privacy Policy
                </Link>
              </nav>
              <p className="text-sm text-primary">
                © {new Date().getFullYear()} EPSx. All rights reserved.
              </p>
            </div>
          </footer>
        </div>
        <Toaster richColors />
        <CookieBanner />
      </Suspense>
    </ThemeProvider>
  );
};

export default ClientLayout;
