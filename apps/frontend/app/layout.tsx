import { AuthProvider } from "@/context/auth-context";
import { LoadingProvider } from "@/context/loading-context";

import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "EPSX",
  description: "Your financial companion",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>
        <AuthProvider>
          <LoadingProvider>
            {children}
          </LoadingProvider>
        </AuthProvider>
      </body>
    </html>
  );
}
