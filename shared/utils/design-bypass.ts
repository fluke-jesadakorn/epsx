import type { UserInfoResponse } from '../auth/client';
import type { EPSXJWTPayload } from '../auth/jwt';
import type { User } from '../types/auth';

const TRUTHY_VALUES = new Set(['1', 'true', 'yes', 'on']);

export const DESIGN_BYPASS_ENV_VAR = 'EPSX_DESIGN_BYPASS';
export const DESIGN_BYPASS_COOKIE = 'epsx_design_bypass';
export const DESIGN_BYPASS_QUERY_PARAM = '__design_bypass';
export const DESIGN_BYPASS_HEADER = 'x-epsx-design-bypass';
export const DESIGN_BYPASS_WALLET =
  '0xDe51gnBYP4550000000000000000000000000001';
export const DESIGN_BYPASS_EMAIL = 'design-mode@epsx.local';
export const DESIGN_BYPASS_NAME = 'EPSX Design Mode';

const FRONTEND_PERMISSIONS = [
  'epsx:analytics:view',
  'epsx:portfolio:view',
  'epsx:plans:view',
  'epsx:notifications:view',
  'epsx:developer:view',
  'epsx:profile:manage',
  'epsx:account:manage',
  'epsx:payments:view',
];

const ADMIN_PERMISSIONS = [
  'admin:*:*',
  'admin:analytics:view',
  'admin:wallets:manage',
  'admin:plans:manage',
  'admin:permissions:manage',
  'admin:developer:manage',
  'admin:notifications:manage',
  'admin:audit:view',
];

export function isDesignBypassTruthy(raw: string | null | undefined): boolean {
  return TRUTHY_VALUES.has(raw?.trim().toLowerCase() ?? '');
}

export function isDesignBypassEnabled(): boolean {
  if (process.env.NODE_ENV === 'production') {
    return false;
  }

  return isDesignBypassTruthy(process.env[DESIGN_BYPASS_ENV_VAR]);
}

export function isDesignBypassRequestEnabled(
  searchParams?: URLSearchParams,
  cookieValue?: string | null
): boolean {
  if (isDesignBypassEnabled()) {
    return true;
  }

  if (process.env.NODE_ENV === 'production') {
    return false;
  }

  return (
    isDesignBypassTruthy(searchParams?.get(DESIGN_BYPASS_QUERY_PARAM)) ||
    isDesignBypassTruthy(cookieValue)
  );
}

export async function isDesignBypassServerEnabled(): Promise<boolean> {
  if (isDesignBypassEnabled()) {
    return true;
  }

  if (process.env.NODE_ENV === 'production') {
    return false;
  }

  try {
    const { cookies, headers } = await import('next/headers');
    const headersList = await headers();
    if (isDesignBypassTruthy(headersList.get(DESIGN_BYPASS_HEADER))) {
      return true;
    }

    const cookieStore = await cookies();
    return isDesignBypassTruthy(cookieStore.get(DESIGN_BYPASS_COOKIE)?.value);
  } catch {
    return false;
  }
}

export function getDesignBypassPermissions(
  kind: 'frontend' | 'admin' | 'all' = 'all'
): string[] {
  if (kind === 'frontend') {
    return [...FRONTEND_PERMISSIONS];
  }

  if (kind === 'admin') {
    return [...FRONTEND_PERMISSIONS, ...ADMIN_PERMISSIONS];
  }

  return [...FRONTEND_PERMISSIONS, ...ADMIN_PERMISSIONS];
}

export function getDesignBypassUserInfo(
  kind: 'frontend' | 'admin' = 'frontend'
): UserInfoResponse {
  return {
    sub: DESIGN_BYPASS_WALLET,
    wallet_address: DESIGN_BYPASS_WALLET,
    tier_level: 'enterprise',
    auth_method: 'design_bypass',
    permissions: getDesignBypassPermissions(kind),
    email: DESIGN_BYPASS_EMAIL,
    access: 'design-bypass',
    packageTier: 'enterprise',
    group: kind === 'admin' ? 'admin' : 'design',
    is_admin: kind === 'admin',
  };
}

export function getDesignBypassFrontendUser(): User {
  return {
    id: DESIGN_BYPASS_WALLET,
    email: DESIGN_BYPASS_EMAIL,
    name: DESIGN_BYPASS_NAME,
    permissions: getDesignBypassPermissions('frontend'),
    plan: 'enterprise',
    platform_context: 'epsx',
    tier: 'enterprise',
    verified: true,
    enterpriseTier: 'enterprise',
    hasApiAccess: true,
    verifiedTokensUsd: 1_000_000,
    nftCollections: [],
    daoMemberships: [],
  };
}

export function getDesignBypassAdminPayload(): EPSXJWTPayload {
  const now = Math.floor(Date.now() / 1000);

  return {
    sub: DESIGN_BYPASS_WALLET,
    email: DESIGN_BYPASS_EMAIL,
    name: DESIGN_BYPASS_NAME,
    permissions: getDesignBypassPermissions('admin'),
    iat: now,
    exp: now + 60 * 60,
    id: DESIGN_BYPASS_WALLET,
    role: 'super_admin',
    package_tier: 'enterprise',
    platforms: ['epsx', 'admin'],
    primary_platform: 'admin',
    platform_context: 'admin',
  };
}
