import { NextRequest, NextResponse } from 'next/server';
import { getBearerToken } from '@/lib/actions/server-auth';

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

export async function GET(request: NextRequest) {
  try {
    const token = await getBearerToken();
    
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };
    
    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    const response = await fetch(`${BACKEND_URL}/api/v1/admin/admin-modules`, {
      method: 'GET',
      headers,
    });

    if (!response.ok) {
      throw new Error(`Backend request failed: ${response.status} ${response.statusText}`);
    }

    const data = await response.json();
    return NextResponse.json(data);
    
  } catch (error) {
    console.error('Admin modules API error:', error);
    return NextResponse.json(
      { error: 'Failed to fetch admin modules' },
      { status: 500 }
    );
  }
}