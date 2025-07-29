'use server';

import { requestPasswordReset, resetPassword } from '@epsx/server-actions';

// Re-export password reset functions for compatibility
export async function requestPasswordResetAction(email: string) {
  try {
    const result = await requestPasswordReset(email);
    return result;
  } catch (error) {
    console.error('Error in requestPasswordResetAction:', error);
    throw error;
  }
}

export async function resetPasswordAction(data: { token: string; newPassword: string }) {
  try {
    const result = await resetPassword(data);
    return result;
  } catch (error) {
    console.error('Error in resetPasswordAction:', error);
    throw error;
  }
}

// Note: This file provides compatibility layer for components
// that still reference @/app/actions/auth. New code should
// import directly from @epsx/server-actions.