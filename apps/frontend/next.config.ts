import type { NextConfig } from 'next';

const STUB_PATH = '../../shared/stubs/empty.ts';
const DEFAULT_MEDIA_ORIGIN = 'http://localhost:9100';
interface RemotePattern {
  protocol: 'http' | 'https';
  hostname: string;
  port?: string;
}

function buildImageRemotePatterns(): RemotePattern[] {
  const staticPatterns: RemotePattern[] = [
    { protocol: 'https', hostname: '**.public.blob.vercel-storage.com' },
    { protocol: 'https', hostname: 'images.unsplash.com' },
  ];
  const origins = [
    process.env.MINIO_ENDPOINT,
    process.env.MINIO_PUBLIC_URL,
    process.env.NEXT_PUBLIC_CDN_URL,
    DEFAULT_MEDIA_ORIGIN,
    'https://cdn.epsx.io',
    'https://dev-cdn.epsx.io',
    'https://staging-cdn.epsx.io',
  ];

  const patterns = new Map<string, RemotePattern>();
  for (const origin of origins) {
    if (origin === undefined || origin.trim() === '') { continue; }
    try {
      const url = new URL(origin);
      if (url.protocol !== 'http:' && url.protocol !== 'https:') { continue; }
      const pattern: RemotePattern = {
        protocol: url.protocol === 'https:' ? 'https' : 'http',
        hostname: url.hostname,
      };
      if (url.port !== '') { pattern.port = url.port; }
      patterns.set(`${pattern.protocol}:${pattern.hostname}:${pattern.port ?? ''}`, pattern);
    } catch {
      // Ignore malformed optional env values; build should still use safe defaults.
    }
  }
  return [...patterns.values(), ...staticPatterns];
}

const nextConfig: NextConfig = {
  // Keep standalone output for legacy Docker and local container workflows.
  output: 'standalone',

  experimental: {
    serverActions: {
      bodySizeLimit: '80mb',
    },
    // Tree-shake large packages — only bundle exports that are actually imported
    optimizePackageImports: [
      'wagmi',
      'viem',
      '@rainbow-me/rainbowkit',
      '@tanstack/react-query',
      'lucide-react',
      'sonner',
      '@walletconnect/core',
      '@wagmi/core',
      '@wagmi/connectors',
    ],
  },

  images: {
    remotePatterns: buildImageRemotePatterns(),
  },

  async rewrites() {
    const minioEndpoint =
      process.env.MINIO_ENDPOINT ??
      process.env.MINIO_PUBLIC_URL ??
      process.env.NEXT_PUBLIC_CDN_URL ??
      DEFAULT_MEDIA_ORIGIN;
    return [
      {
        source: '/media-proxy/:path*',
        destination: `${minioEndpoint}/:path*`,
      },
      {
        source: '/news-img/:path*',
        destination: `${minioEndpoint}/news/:path*`,
      },
    ];
  },

  // Skip static generation for error pages that fail with useContext issues
  // This is a known issue with Next.js 16 + React 19 + complex provider trees
  staticPageGenerationTimeout: 60,

  // Transpile shared packages only
  transpilePackages: ['@/shared'],

  // Silence Next.js 16 Turbopack warning (using webpack config for dev)
  turbopack: {
    resolveAlias: {
      'thread-stream': STUB_PATH,
      'pino-pretty': STUB_PATH,
      'pino-elasticsearch': STUB_PATH,
      'tap': STUB_PATH,
      'tape': STUB_PATH,
      'why-is-node-running': STUB_PATH,
      'desm': STUB_PATH,
      'fastbench': STUB_PATH,
      'react-native': STUB_PATH,
      'react-native-device-info': STUB_PATH,
      'react-native-keychain': STUB_PATH,
    },
  },
  // Improve HMR WebSocket reliability and fix SSR issues
  webpack: (config: any, { dev, isServer, webpack }: { dev: boolean; isServer: boolean; webpack: any }) => {

    if (dev && !isServer) {
      config.watchOptions = {
        poll: 1000,
        aggregateTimeout: 300,
      };
    }

    // Split vendor chunks for better caching on mobile
    if (!dev && !isServer) {
      config.optimization.splitChunks = {
        ...config.optimization.splitChunks,
        cacheGroups: {
          ...config.optimization.splitChunks?.cacheGroups,
          web3: {
            name: 'vendor-web3',
            test: /[\\/]node_modules[\\/](wagmi|viem|@wagmi|@rainbow-me|@walletconnect|@coinbase|@metamask)[\\/]/,
            chunks: 'all',
            priority: 30,
          },
          tanstack: {
            name: 'vendor-tanstack',
            test: /[\\/]node_modules[\\/](@tanstack)[\\/]/,
            chunks: 'all',
            priority: 25,
          },
        },
      };
    }

    // Web3/Wagmi build fix: Explicitly stub out problematic modules
    const path = require('node:path');
    const stubPath = path.join(process.cwd(), '../../shared/stubs/empty.ts');

    // Ensure shared components can resolve modules from the app's node_modules
    const appNodeModules = path.resolve(process.cwd(), 'node_modules');
    config.resolve.modules = [
      appNodeModules,
      ...(config.resolve.modules ?? ['node_modules']),
    ];

    config.resolve.alias = {
      ...config.resolve.alias,
      'thread-stream': stubPath,
      tap: stubPath,
      tape: stubPath,
      'why-is-node-running': stubPath,
      'pino-pretty': stubPath,
      'zod/mini': require.resolve('zod'),
      'zod/v4/core': require.resolve('zod'),
    };
    // Standard Node.js polyfills for the browser (Client Component bundle)
    if (!isServer) {
      config.resolve.fallback = {
        ...config.resolve.fallback,
        fs: false,
        net: false,
        tls: false,
        crypto: false,
        stream: false,
        url: false,
        zlib: false,
        http: false,
        https: false,
        os: false,
        path: false,
        worker_threads: false,
        lokijs: false,
        encoding: false,
        'pino-pretty': false,
      };
    }

    // Fix specific module resolution issues for both Server and Client
    config.plugins.push(
      new webpack.IgnorePlugin({
        resourceRegExp: /^(tap|tape|desm|fastbench|pino-elasticsearch|why-is-node-running|thread-stream\/test)$/,
      })
    );

    return config;
  },
};

export default nextConfig;
