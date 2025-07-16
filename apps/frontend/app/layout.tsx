import { AuthProvider } from "@/context/auth-context-improved";
import { LoadingProvider } from "@/context/loading-context";
import { ThemeProvider } from "next-themes";
import NavbarComponent from "@/components/features/navigation/Navbar";
import { EmailVerificationBanner } from "@/components/auth/EmailVerificationBanner";

import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "EPSX",
  description: "Your data analytics companion",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body>
        <ThemeProvider attribute="class" defaultTheme="system" enableSystem disableTransitionOnChange>
          <AuthProvider>
            <LoadingProvider>
              <div>
                <header>
                  <NavbarComponent />
                  <EmailVerificationBanner />
                </header>
                <main>
                  {children}
                </main>
              </div>
            </LoadingProvider>
          </AuthProvider>
        </ThemeProvider>
      </body>
    </html>
  );
}
