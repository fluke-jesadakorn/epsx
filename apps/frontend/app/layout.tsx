import { AuthProvider } from "@/context/auth-context";
import { LoadingProvider } from "@/context/loading-context";
import { ThemeProvider } from "next-themes";
import NavbarComponent from "@/components/features/navigation/Navbar";

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
