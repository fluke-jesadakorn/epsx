import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  reactStrictMode: true,
  images: {
    unoptimized: true,
  },
  output: 'standalone',
  // Enable experimental features for better optimization
  experimental: {
    // Enable server components for better performance
    serverActions: {
      allowedOrigins: ['*'],
    },
  },
};

export default nextConfig;
