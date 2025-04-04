import { redirect } from 'next/navigation';
import { getCurrentUser } from '@/app/actions/auth';

export async function checkAuth() {
  const user = await getCurrentUser();

  if (!user) {
    redirect('/auth');
  }

  return user;
}

export async function checkGuest() {
  const user = await getCurrentUser();

  if (user) {
    redirect('/dashboard');
  }
}
