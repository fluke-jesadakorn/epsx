/**
 * Frontend Sign Out API Route
 * Clears user session and redirects to login
 */
import { NextRequest, NextResponse } from 'next/server';
import { clearSession } from '@/lib/auth';

export async function POST(request: NextRequest) {
  try {
    console.log('🔄 Frontend: Processing sign out request');

    // Clear the user session
    await clearSession();

    console.log('✅ Frontend: User session cleared successfully');

    // Return success response
    return NextResponse.json({ success: true }, { status: 200 });

  } catch (error) {
    console.error('❌ Frontend: Sign out failed:', error);
    
    return NextResponse.json(
      { error: 'Sign out failed' }, 
      { status: 500 }
    );
  }
}

// Allow GET method for simple logout links
export async function GET(request: NextRequest) {
  try {
    console.log('🔄 Frontend: Processing sign out request (GET)');

    // Clear the user session
    await clearSession();

    console.log('✅ Frontend: User session cleared successfully');

    // Redirect to login page
    return NextResponse.redirect(new URL('/login', request.url));

  } catch (error) {
    console.error('❌ Frontend: Sign out failed:', error);
    
    // Redirect to login page anyway
    return NextResponse.redirect(new URL('/login', request.url));
  }
}