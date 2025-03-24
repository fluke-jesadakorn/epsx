"use client";

import React, { Suspense } from "react";
import Link from "next/link";
import { ThemeProvider } from "./ThemeProvider";
import { Toaster } from "sonner";
import { useFirebaseAnalytics } from "@/hooks/useFirebaseAnalytics";
import { SkeletonLoader } from "@/components/common/Skeleton";
import CookieBanner from "../common/CookieBanner";
import { LoadingProvider } from "@/context/loading-context";

interface ClientLayoutProps {
  children: React.ReactNode;
}

const ClientLayout: React.FC<ClientLayoutProps> = ({ children }) => {
  const { AnalyticsWrapper } = useFirebaseAnalytics();
  return (
    <ThemeProvider>
      <LoadingProvider>
        <Suspense fallback={<SkeletonLoader />}>
        <AnalyticsWrapper />
        <div className="min-h-screen flex flex-col">
          <main className="flex-grow">{children}</main>
          <footer className="py-8 px-4 border-t bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
            <div className="max-w-7xl mx-auto grid gap-8 md:grid-cols-2 lg:grid-cols-4">
              <div className="space-y-3">
                <h3 className="text-lg font-semibold">EPSx</h3>
                <p className="text-sm text-muted-foreground">
                  Tech Insights and Data Management
                </p>
                <div className="flex space-x-4">
                  <a href="https://twitter.com" className="text-muted-foreground hover:text-primary" target="_blank" rel="noopener noreferrer">
                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M22 4s-.7 2.1-2 3.4c1.6 10-9.4 17.3-18 11.6 2.2.1 4.4-.6 6-2C3 15.5.5 9.6 3 5c2.2 2.6 5.6 4.1 9 4-.9-4.2 4-6.6 7-3.8 1.1 0 3-1.2 3-1.2z"/></svg>
                  </a>
                  <a href="https://linkedin.com" className="text-muted-foreground hover:text-primary" target="_blank" rel="noopener noreferrer">
                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M16 8a6 6 0 0 1 6 6v7h-4v-7a2 2 0 0 0-2-2 2 2 0 0 0-2 2v7h-4v-7a6 6 0 0 1 6-6z"/><rect width="4" height="12" x="2" y="9"/><circle cx="4" cy="4" r="2"/></svg>
                  </a>
                </div>
              </div>
              
              <div className="space-y-3">
                <h3 className="text-lg font-semibold">Services</h3>
                <nav className="flex flex-col space-y-2">
                  <Link href="/research" className="text-sm text-muted-foreground hover:text-primary transition-colors">Research</Link>
                  <Link href="/trading" className="text-sm text-muted-foreground hover:text-primary transition-colors">Automation</Link>
                  <Link href="/services" className="text-sm text-muted-foreground hover:text-primary transition-colors">All Services</Link>
                </nav>
              </div>

              <div className="space-y-3">
                <h3 className="text-lg font-semibold">Company</h3>
                <nav className="flex flex-col space-y-2">
                  <Link href="/about" className="text-sm text-muted-foreground hover:text-primary transition-colors">About</Link>
                  <Link href="/privacy" className="text-sm text-muted-foreground hover:text-primary transition-colors">Privacy Policy</Link>
                  <Link href="/terms" className="text-sm text-muted-foreground hover:text-primary transition-colors">Terms of Service</Link>
                </nav>
              </div>

              <div className="space-y-3">
                <h3 className="text-lg font-semibold">Contact</h3>
                <nav className="flex flex-col space-y-2">
                  <a href="mailto:info@epsx.com" className="text-sm text-muted-foreground hover:text-primary transition-colors">info@epsx.com</a>
                </nav>
              </div>
            </div>
            <div className="max-w-7xl mx-auto mt-8 pt-8 border-t">
              <p className="text-sm text-center text-muted-foreground">
                © {new Date().getFullYear()} EPSx. All rights reserved.
              </p>
            </div>
          </footer>
        </div>
        <Toaster richColors />
        <CookieBanner />
        </Suspense>
      </LoadingProvider>
    </ThemeProvider>
  );
};

export default ClientLayout;
