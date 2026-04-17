export type DeploymentEnvironment = 'development' | 'staging' | 'production';
export type BlockchainNetwork = 'testnet' | 'mainnet';

export interface DeploymentConfig {
  frontendUrl: string;
  adminUrl: string;
  backendUrl: string;
  blockchainNetwork: BlockchainNetwork;
  chainId: '97' | '56';
}

const LOCAL_DEV_HOST_PATTERNS = [
  /^localhost$/i,
  /^127\.0\.0\.1$/i,
  /^100\./,
  /^192\.168\./,
  /^10\./,
  /^172\./,
] as const;

const DEVELOPMENT_HOSTS = new Set([
  'dev.epsx.io',
  'dev-admin.epsx.io',
  'dev-api.epsx.io',
]);

const STAGING_HOSTS = new Set([
  'staging.epsx.io',
  'staging-admin.epsx.io',
  'staging-api.epsx.io',
]);

const PRODUCTION_HOSTS = new Set([
  'epsx.io',
  'www.epsx.io',
  'admin.epsx.io',
  'api.epsx.io',
]);

const ENVIRONMENT_ALIASES: Record<string, DeploymentEnvironment> = {
  dev: 'development',
  development: 'development',
  preview: 'development',
  stage: 'staging',
  staging: 'staging',
  prod: 'production',
  production: 'production',
  main: 'production',
  master: 'production',
};

export const DEPLOYMENT_MAP: Record<DeploymentEnvironment, DeploymentConfig> = {
  development: {
    frontendUrl: 'https://dev.epsx.io',
    adminUrl: 'https://dev-admin.epsx.io',
    backendUrl: 'https://dev-api.epsx.io',
    blockchainNetwork: 'testnet',
    chainId: '97',
  },
  staging: {
    frontendUrl: 'https://staging.epsx.io',
    adminUrl: 'https://staging-admin.epsx.io',
    backendUrl: 'https://staging-api.epsx.io',
    blockchainNetwork: 'testnet',
    chainId: '97',
  },
  production: {
    frontendUrl: 'https://epsx.io',
    adminUrl: 'https://admin.epsx.io',
    backendUrl: 'https://api.epsx.io',
    blockchainNetwork: 'mainnet',
    chainId: '56',
  },
};

export function isLocalDevelopmentHostname(hostname: string): boolean {
  return LOCAL_DEV_HOST_PATTERNS.some(pattern => pattern.test(hostname));
}

export function normalizeEnvironmentName(value?: string | null): DeploymentEnvironment | undefined {
  if (!value) { return undefined; }
  return ENVIRONMENT_ALIASES[value.trim().toLowerCase()];
}

function getProcessEnv(key: string): string | undefined {
  if (typeof process === 'undefined') { return undefined; }
  return process.env[key];
}

function extractHostname(value?: string | null): string | undefined {
  if (!value) { return undefined; }

  const trimmed = value.trim();
  if (trimmed === '') { return undefined; }

  try {
    const normalized = trimmed.includes('://') ? trimmed : `https://${trimmed}`;
    return new URL(normalized).hostname.toLowerCase();
  } catch {
    return undefined;
  }
}

export function inferEnvironmentFromHostname(hostname?: string | null): DeploymentEnvironment | undefined {
  const normalizedHostname = hostname?.toLowerCase();
  if (!normalizedHostname) { return undefined; }

  if (isLocalDevelopmentHostname(normalizedHostname)) {
    return 'development';
  }
  if (DEVELOPMENT_HOSTS.has(normalizedHostname)) {
    return 'development';
  }
  if (STAGING_HOSTS.has(normalizedHostname)) {
    return 'staging';
  }
  if (PRODUCTION_HOSTS.has(normalizedHostname)) {
    return 'production';
  }

  return undefined;
}

export function inferEnvironmentFromBranch(branch?: string | null): DeploymentEnvironment | undefined {
  if (!branch) { return undefined; }

  const normalizedBranch = branch.trim().toLowerCase();
  if (normalizedBranch === '') { return undefined; }

  if (normalizedBranch === 'staging' || normalizedBranch.startsWith('staging/')) {
    return 'staging';
  }
  if (normalizedBranch === 'development' || normalizedBranch === 'develop' || normalizedBranch.startsWith('development/') || normalizedBranch.startsWith('develop/')) {
    return 'development';
  }
  if (normalizedBranch === 'production' || normalizedBranch === 'main' || normalizedBranch === 'master' || normalizedBranch.startsWith('production/')) {
    return 'production';
  }

  return undefined;
}

function inferEnvironmentFromConfiguredUrls(): DeploymentEnvironment | undefined {
  const urlCandidates = [
    getProcessEnv('NEXT_PUBLIC_APP_URL'),
    getProcessEnv('NEXT_PUBLIC_ADMIN_URL'),
    getProcessEnv('NEXT_PUBLIC_BACKEND_URL'),
    getProcessEnv('FRONTEND_URL'),
    getProcessEnv('ADMIN_FRONTEND_URL'),
    getProcessEnv('BACKEND_URL'),
    getProcessEnv('VERCEL_URL'),
    getProcessEnv('VERCEL_PROJECT_PRODUCTION_URL'),
  ];

  for (const candidate of urlCandidates) {
    const inferred = inferEnvironmentFromHostname(extractHostname(candidate));
    if (inferred) {
      return inferred;
    }
  }

  return undefined;
}

export function resolveDeploymentEnvironment(): DeploymentEnvironment {
  const explicitEnvironment = [
    getProcessEnv('DEPLOYMENT_ENV'),
    getProcessEnv('NEXT_PUBLIC_DEPLOYMENT_ENV'),
    getProcessEnv('ENV'),
    getProcessEnv('APP_ENV'),
    getProcessEnv('EPSX_ENV'),
    getProcessEnv('RUST_ENV'),
  ]
    .map(normalizeEnvironmentName)
    .find((value): value is DeploymentEnvironment => value !== undefined);

  if (explicitEnvironment) {
    return explicitEnvironment;
  }

  const branchEnvironment = [
    getProcessEnv('VERCEL_GIT_COMMIT_REF'),
    getProcessEnv('CI_COMMIT_BRANCH'),
    getProcessEnv('GIT_BRANCH'),
  ]
    .map(inferEnvironmentFromBranch)
    .find((value): value is DeploymentEnvironment => value !== undefined);

  if (branchEnvironment) {
    return branchEnvironment;
  }

  const configuredEnvironment = inferEnvironmentFromConfiguredUrls();
  if (configuredEnvironment) {
    return configuredEnvironment;
  }

  if (typeof window !== 'undefined') {
    const browserEnvironment = inferEnvironmentFromHostname(window.location.hostname);
    if (browserEnvironment) {
      return browserEnvironment;
    }
  }

  if (getProcessEnv('NODE_ENV') === 'development') {
    return 'development';
  }

  return 'production';
}

export function getDeploymentConfig(environment = resolveDeploymentEnvironment()): DeploymentConfig {
  return DEPLOYMENT_MAP[environment];
}

export function getDefaultBackendUrlForEnvironment(environment = resolveDeploymentEnvironment()): string {
  return getDeploymentConfig(environment).backendUrl;
}

export function getDefaultFrontendUrlForEnvironment(environment = resolveDeploymentEnvironment()): string {
  return getDeploymentConfig(environment).frontendUrl;
}

export function getDefaultAdminUrlForEnvironment(environment = resolveDeploymentEnvironment()): string {
  return getDeploymentConfig(environment).adminUrl;
}

export function getDefaultBlockchainNetworkForEnvironment(environment = resolveDeploymentEnvironment()): BlockchainNetwork {
  return getDeploymentConfig(environment).blockchainNetwork;
}

export function getDefaultChainIdForEnvironment(environment = resolveDeploymentEnvironment()): '97' | '56' {
  return getDeploymentConfig(environment).chainId;
}
