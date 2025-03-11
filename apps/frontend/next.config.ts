import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  reactStrictMode: true,
  experimental: {
    // Enable server components for better performance
    serverActions: {
      allowedOrigins: ["*"],
    },
  },
  poweredByHeader: false,
  generateEtags: true,
};

export default nextConfig;
