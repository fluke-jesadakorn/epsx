"use client";

import React, { Suspense } from "react";
import Link from "next/link";
import { ThemeProvider } from "./ThemeProvider";
import { AuthProvider } from "@/context/auth-context";
import Navbar from "../Navbar";
import { Toaster } from "sonner";
import { useFirebaseAnalytics } from "@/hooks/useFirebaseAnalytics";
import LoadingForm from "../common/LoadingForm";

interface ClientLayoutProps {
  children: React.ReactNode;
}

const ClientLayout: React.FC<ClientLayoutProps> = ({ children }) => {
  const { AnalyticsWrapper } = useFirebaseAnalytics();
  return (
    <ThemeProvider>
      <AuthProvider>
        <Suspense fallback={<LoadingForm>Loading...</LoadingForm>}>
        <AnalyticsWrapper />
        <div className="min-h-screen flex flex-col">
          <header>
            <Navbar />
          </header>
          <main className="flex-grow">{children}</main>
          <footer className="py-6 px-4 border-t">
            <div className="max-w-7xl mx-auto flex flex-col items-center gap-4">
              <nav className="flex gap-6">
                <Link 
                  href="/terms" 
                  className="text-sm text-gray-600 hover:text-gray-900 transition-colors"
                >
                  Terms of Service
                </Link>
                <Link 
                  href="/privacy" 
                  className="text-sm text-gray-600 hover:text-gray-900 transition-colors"
                >
                  Privacy Policy
                </Link>
              </nav>
              <p className="text-sm text-gray-600">
                © {new Date().getFullYear()} EPSx. All rights reserved.
              </p>
            </div>
          </footer>
        </div>
        <Toaster richColors />
        </Suspense>
      </AuthProvider>
    </ThemeProvider>
  );
};

export default ClientLayout;
