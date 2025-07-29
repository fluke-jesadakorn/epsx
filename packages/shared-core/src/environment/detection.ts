export const Environment = {
  isServer: () => typeof window === 'undefined',
  isClient: () => typeof window !== 'undefined',
  isDevelopment: () => process.env.NODE_ENV === 'development',
  isProduction: () => process.env.NODE_ENV === 'production',
  isTest: () => process.env.NODE_ENV === 'test',
} as const;

export function getApiBaseUrl(): string {
  if (Environment.isServer()) {
    return process.env.BACKEND_URL || process.env.API_URL || 'http://localhost:8080';
  }
  return process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';
}

export function getEnvironmentInfo() {
  return {
    isServer: Environment.isServer(),
    isClient: Environment.isClient(),
    isDevelopment: Environment.isDevelopment(),
    isProduction: Environment.isProduction(),
    isTest: Environment.isTest(),
    nodeEnv: process.env.NODE_ENV,
    apiBaseUrl: getApiBaseUrl(),
  };
}