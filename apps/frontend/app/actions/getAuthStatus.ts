// Server action
'use server';

import { cookies } from 'next/headers';

interface AuthStatus {
  isAuthenticated: boolean;
  email: string | null;
  role: string | null;
}

export async function getAuthStatus(): Promise<AuthStatus> {
  const cookieStore = await cookies();
  const session = cookieStore.get('__session')?.value;
  const email = cookieStore.get('email')?.value;
  const role = cookieStore.get('role')?.value;

  return {
    isAuthenticated: !!session,
    email: email || null,
    role: role || null
  };
}
