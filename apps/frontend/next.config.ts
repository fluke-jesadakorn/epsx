import type { NextConfig } from 'next';
import dotenv from 'dotenv';
import path from 'path';

// Load environment variables from root .env file
dotenv.config({ path: path.resolve(__dirname, '../../.env') });

const nextConfig: NextConfig = {
  // TEMPORARILY DISABLED: output: 'standalone',
  // outputFileTracingRoot: process.env.NODE_ENV === 'production' ? '/app' : process.cwd(),
  eslint: {
    ignoreDuringBuilds: true,
  },
  experimental: {
    // Fix WebSocket connection issues in Next.js 15
    webpackBuildWorker: true,
  },
  // Explicitly expose NEXT_PUBLIC_ variables
  env: {
    NEXT_PUBLIC_BACKEND_URL: process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080',
    NEXT_PUBLIC_APP_URL: process.env.NEXT_PUBLIC_APP_URL || 'http://localhost:3000',
    NEXT_PUBLIC_ADMIN_URL: process.env.NEXT_PUBLIC_ADMIN_URL || 'http://localhost:3001',
    NEXT_PUBLIC_OAUTH_CLIENT_ID: process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID || 'epsx-frontend',
    NEXT_PUBLIC_BLOCKCHAIN_NETWORK: process.env.NEXT_PUBLIC_BLOCKCHAIN_NETWORK || 'testnet',
    NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID: process.env.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID || 'epsx-web3-frontend',
    NEXT_PUBLIC_CHAIN_ID: process.env.NEXT_PUBLIC_CHAIN_ID || '97',
    NEXT_PUBLIC_PAYMENT_MAINNET_ADDRESS: process.env.NEXT_PUBLIC_PAYMENT_MAINNET_ADDRESS || '0x7877e415a13532d9E43Df7Fd2CC256f93a39ced7',
    NEXT_PUBLIC_PAYMENT_TESTNET_ADDRESS: process.env.NEXT_PUBLIC_PAYMENT_TESTNET_ADDRESS || '0x7877e415a13532d9E43Df7Fd2CC256f93a39ced7',
  },
  // Improve HMR WebSocket reliability and fix SSR issues
  webpack: (config, { dev, isServer }) => {
    if (dev && !isServer) {
      config.watchOptions = {
        poll: 1000,
        aggregateTimeout: 300,
      };
    }
    
    // Fix browser-only APIs during SSR
    if (isServer) {
      config.resolve.fallback = {
        ...config.resolve.fallback,
        'indexeddb-js': false,
        'fake-indexeddb': false,
      };
    }

    // Fix RainbowKit and wagmi chunking issues
    config.optimization = {
      ...config.optimization,
      splitChunks: {
        ...config.optimization.splitChunks,
        cacheGroups: {
          ...config.optimization.splitChunks?.cacheGroups,
          rainbowkit: {
            test: /[\\/]node_modules[\\/]@rainbow-me[\\/]rainbowkit[\\/]/,
            name: 'rainbowkit',
            chunks: 'all',
            priority: 10,
          },
          wagmi: {
            test: /[\\/]node_modules[\\/]wagmi[\\/]/,
            name: 'wagmi',
            chunks: 'all',
            priority: 10,
          },
        },
      },
    };
    
    return config;
  },
};

export default nextConfig;