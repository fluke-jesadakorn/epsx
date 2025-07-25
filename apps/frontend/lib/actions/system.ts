'use server';

import { cookies } from 'next/headers';
import { revalidatePath } from 'next/cache';

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

export async function clearCacheAction(formData: FormData) {
  const symbol = formData.get('symbol') as string;
  const action = formData.get('action') as string;

  try {
    const cookieStore = cookies();
    const cookieHeader = cookieStore.toString();

    const body: { symbol?: string; action?: string } = {};
    if (symbol) body.symbol = symbol;
    if (action) body.action = action;

    const response = await fetch(`${BACKEND_URL}/api/system/cache`, {
      method: 'DELETE',
      headers: { 
        'Content-Type': 'application/json',
        'Cookie': cookieHeader,
      },
      body: JSON.stringify(body),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error || 'Cache clear failed');
    }

    const result = await response.json();
    
    // Revalidate paths that might be affected by cache changes
    revalidatePath('/');
    revalidatePath('/dashboard');
    revalidatePath('/stocks');
    
    return { success: true, data: result };
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
    const cookieStore = cookies();
    const cookieHeader = cookieStore.toString();

    const logData = {
      action,
      resource,
      details,
      metadata: metadata ? JSON.parse(metadata) : undefined,
    };

    const response = await fetch(`${BACKEND_URL}/api/audit/logs`, {
      method: 'POST',
      headers: { 
        'Content-Type': 'application/json',
        'Cookie': cookieHeader,
      },
      body: JSON.stringify(logData),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error || 'Audit log creation failed');
    }

    const result = await response.json();
    return { success: true, data: result };
  } catch (error) {
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Audit log creation failed' 
    };
  }
}