import "@ant-design/v5-patch-for-react-19";
import { Geist, Geist_Mono } from "next/font/google";
import AntdRegistryProvider from "@/components/layout/AntdRegistry";
import type { Metadata } from "next";
import EmotionRegistry from "@/components/EmotionRegistry";
import { AuthProvider } from "@/contexts/AuthContext";
import "./globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
  display: "swap",
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
  display: "swap",
});

export const metadata: Metadata = {
  title: "ESPx",
  description: "ESPx - Stock Insights and Portfolio Management",
};

export const viewport = {
  width: "device-width",
  initialScale: 1,
  maximumScale: 1,
  userScalable: false,
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      lang="en"
      suppressHydrationWarning
      className={`${geistSans.className} ${geistMono.className}`}
    >
      <body style={{ minHeight: "100vh" }}>
        {/* TODO: Consider adding a loading skeleton for initial render */}
        <AuthProvider>
          <AntdRegistryProvider>
            <EmotionRegistry>{children}</EmotionRegistry>
          </AntdRegistryProvider>
        </AuthProvider>
      </body>
    </html>
  );
}
