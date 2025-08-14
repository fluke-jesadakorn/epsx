'use server';

import { auth } from '@/lib/auth';
import { redirect } from 'next/navigation';

/**
 * Ensure user is not authenticated (guest only pages)
 * Uses custom iron-session based authentication
 */
export async function requireGuest(): Promise<void> {
  const session = await auth();
  
  // If authenticated, redirect to dashboard
  if (session?.user) {
    redirect('/dashboard');
  }
}

/**
 * Request password reset action
 * Sends password reset email to the user
 */
export async function requestPasswordResetAction(email: string): Promise<{
  success: boolean;
  error?: string;
}> {
  try {
    // Validate email format
    if (!email || !email.includes('@')) {
      return {
        success: false,
        error: 'Please enter a valid email address'
      };
    }

    // Make request to backend API to initiate password reset
    const backendUrl = process.env.BACKEND_URL || 'http://localhost:8080';
    
    // Note: Password reset might need to be implemented as a separate OIDC extension
    // For now, keep using direct API call until backend implements it
    const response = await fetch(`${backendUrl}/oauth/password-reset`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ email }),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      
      if (response.status === 404) {
        // Don't reveal whether email exists for security
        return {
          success: true, // Always return success for password reset requests
        };
      }
      
      return {
        success: false,
        error: errorData.message || 'Failed to process password reset request'
      };
    }

    // Always return success for password reset requests for security
    // (Don't reveal whether the email exists in the system)
    return {
      success: true,
    };
  } catch (error) {
    console.error('Password reset request failed:', error);
    return {
      success: false,
      error: 'Network error. Please try again later.'
    };
  }
}