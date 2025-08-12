import { NextRequest, NextResponse } from 'next/server';
import { getBearerToken } from '@/lib/actions/server-auth';

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

export async function GET() {
  try {
    const token = await getBearerToken();
    
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };
    
    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    const response = await fetch(`${BACKEND_URL}/api/v1/admin/stock-ranking/assignments`, {
      method: 'GET',
      headers,
    });

    if (!response.ok) {
      // If backend route doesn't exist, return empty data instead of error page
      if (response.status === 404) {
        return NextResponse.json({ 
          assignments: [],
          message: 'Stock ranking assignments not yet implemented' 
        });
      }
      
      const errorText = await response.text();
      return NextResponse.json(
        { error: `Backend error: ${errorText}` }, 
        { status: response.status }
      );
    }

    const data = await response.json();
    return NextResponse.json(data);
  } catch (error) {
    console.error('Stock ranking assignments API error:', error);
    return NextResponse.json(
      { 
        assignments: [],
        error: 'Failed to fetch assignments',
        message: 'Stock ranking service currently unavailable' 
      }, 
      { status: 500 }
    );
  }
}