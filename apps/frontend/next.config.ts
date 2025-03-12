// @ts-check

import { composePlugins, withNx } from "@nx/next";
import type { NextConfig } from "next";
import { join } from "path";

const nextConfig: NextConfig = {
  reactStrictMode: true,
  experimental: {
    serverActions: {
      allowedOrigins: ["*"],
    },
    optimizePackageImports: ["@radix-ui/react-*"],
  },
  outputFileTracingRoot: join(__dirname, "../../"),
  poweredByHeader: false,
  generateEtags: true,
  nx: {
    svgr: false,
  },
  output: "standalone",
};

const plugins = [withNx];

export default composePlugins(...plugins)(nextConfig);
