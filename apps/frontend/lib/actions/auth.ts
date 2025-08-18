'use server';

import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';

export async function handleSignOut() {
  // Clear JWT cookie
  const cookieStore = await cookies();
  cookieStore.delete('epsx_frontend_jwt');
  
  // Redirect to login page - NEXT_REDIRECT error is expected behavior
  redirect('/login');
}