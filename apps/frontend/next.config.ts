import type { NextConfig } from "next";
import type { Configuration } from 'webpack';

const nextConfig: NextConfig = {
  reactStrictMode: true,
  images: {
    unoptimized: true,
  },
  experimental: {
    // Enable server components for better performance
    serverActions: {
      allowedOrigins: ["*"],
    },
  },
};

export default nextConfig;
