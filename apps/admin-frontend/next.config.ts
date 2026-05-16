import type { NextConfig } from 'next';
import path from 'node:path';

const SHARED_STUB = '../../shared/stubs/empty.ts';
const ZOD_ALIAS = 'zod';
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
        source: '/minio-img/:path*',
        destination: `${minioEndpoint}/:path*`,
      },
      {
        source: '/news-img/:path*',
        destination: `${minioEndpoint}/news/:path*`,
      },
    ];
  },

  // Enabled Turbopack for development
  turbopack: {
    resolveAlias: {
      'thread-stream': SHARED_STUB,
      'thread-stream/test': SHARED_STUB,
      'pino-pretty': SHARED_STUB,
      'pino-elasticsearch': SHARED_STUB,
      'tap': SHARED_STUB,
      'tape': SHARED_STUB,
      'why-is-node-running': SHARED_STUB,
      'desm': SHARED_STUB,
      'fastbench': SHARED_STUB,
      'react-native': SHARED_STUB,
      'react-native-device-info': SHARED_STUB,
      'react-native-keychain': SHARED_STUB,
      'zod/mini': ZOD_ALIAS,
      'zod/v4/core': ZOD_ALIAS,
    },
  },

  // Transpile shared packages and web3 libraries containing ESM exports
  transpilePackages: ['@/shared', '@rainbow-me/rainbowkit', 'wagmi', 'viem'],

  // Improve HMR WebSocket reliability and fix module resolution
  webpack: (config: any, { dev, isServer, webpack }: any) => {
    // Only apply heavy Webpack custom logic if not using Turbopack
    // Note: Next.js 15+ will automatically prefer Turbopack if --turbo is used.

    const isDev = Boolean(dev);
    const isServerSide = Boolean(isServer);

    if (isDev && !isServerSide) {
      config.watchOptions = {
        poll: 1000,
        aggregateTimeout: 300,
      };
    }

    // Web3/Wagmi build fix: Explicitly stub out problematic modules
    const stubPath = path.join(process.cwd(), SHARED_STUB);

    // Ensure shared components can resolve modules from the app's node_modules
    const appNodeModules = path.resolve(process.cwd(), 'node_modules');
    config.resolve.modules = [
      appNodeModules,
      ...((config.resolve.modules as string[] | undefined) ?? ['node_modules']),
    ];

    config.resolve.alias = {
      ...(config.resolve.alias as Record<string, string>),
      'thread-stream': stubPath,
      'thread-stream/test': stubPath,
      tap: stubPath,
      tape: stubPath,
      'why-is-node-running': stubPath,
      'pino-pretty': stubPath,
      'pino-elasticsearch': stubPath,
      desm: stubPath,
      fastbench: stubPath,
      'zod/mini': require.resolve('zod'),
      'zod/v4/core': require.resolve('zod'),
    };
    // Standard Node.js polyfills for the browser (Client Component bundle)
    if (!isServerSide) {
      config.resolve.fallback = {
        ...(config.resolve.fallback as Record<string, boolean | string>),
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
        // Web3 specific ignores
        'pino-pretty': false,
        lokijs: false,
        encoding: false,
      };
    }

    // Fix specific module resolution issues for both Server and Client
    config.plugins.push(
      new webpack.IgnorePlugin({
        resourceRegExp: /^(tap|tape|desm|fastbench|pino-elasticsearch|why-is-node-running|thread-stream(\/test)?|react-native(-device-info|-keychain)?|@react-native)/,
      })
    );

    return config;
  },
};

export default nextConfig;
