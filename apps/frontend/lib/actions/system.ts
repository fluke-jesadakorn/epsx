'use server';

import { cookies } from 'next/headers';
import { revalidatePath } from 'next/cache';
import { createApiClient, isApiError } from '@/lib/api-client';

// Get API client - will automatically use backend URL
const getApi = () => {
  return createApiClient();
};

export async function clearCacheAction(formData: FormData) {
  const symbol = formData.get('symbol') as string;
  const action = formData.get('action') as string;

  try {
    const api = getApi();
    const response = await api.clearSystemCache({ symbol, action });

    if (isApiError(response)) {
      throw new Error(response.error || 'Cache clear failed');
    }
    
    // Revalidate paths that might be affected by cache changes
    revalidatePath('/');
    revalidatePath('/dashboard');
    revalidatePath('/stocks');
    
    return { success: true, data: response.data };
  } catch (error) {
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Cache clear failed' 
    };
  }
}

export async function createAuditLogAction(formData: FormData) {
  const action = formData.get('action') as string;
  const resource = formData.get('resource') as string;
  const details = formData.get('details') as string;
  const metadata = formData.get('metadata') as string;

  try {
    const api = getApi();
    const logData = {
      action,
      resource,
      details,
      metadata: metadata ? JSON.parse(metadata) : undefined,
    };

    const response = await api.createAuditLog(logData);

    if (isApiError(response)) {
      throw new Error(response.error || 'Audit log creation failed');
    }

    return { success: true, data: response.data };
  } catch (error) {
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Audit log creation failed' 
    };
  }
}