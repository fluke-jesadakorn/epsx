'use server';

import { logoutOIDC } from './oidc-auth';

/**
 * Logout action that handles both OIDC and legacy auth cleanup
 */
export async function logoutAction(): Promise<void> {
  try {
    // Use OIDC logout which handles session cleanup and redirection
    await logoutOIDC();
  } catch (error) {
    console.error('Logout action failed:', error);
    throw error;
  }
}