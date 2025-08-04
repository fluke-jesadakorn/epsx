// Re-export all utilities from shared package
export * from '@epsx/shared-utils';

// Legacy aliases for backward compatibility
export { fmtDate as formatDate } from '@epsx/shared-utils';
export { fmtCurrency as formatCurrency } from '@epsx/shared-utils';
export { generateId as genId } from '@epsx/shared-utils';
export { url } from '@epsx/shared-utils';

/**
 * Parse URL query to object (legacy compatibility)
 */
export function parseQuery(queryString: string): Record<string, string> {
  return url.parseQuery(queryString);
}

/**
 * Get file extension (legacy compatibility)
 */
export function getExt(filename: string): string {
  return url.getExtension(filename);
}