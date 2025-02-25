import { NextResponse } from 'next/server';

export async function POST() {
  const response = NextResponse.json({ 
    success: true,
    redirectUrl: "/login"
  });

  response.cookies.delete('__session');
  
  return response;
}
