'use server';

// Legacy compatibility functions if needed
import { redirect } from 'next/navigation';
import { adminLogin, logout, getCurrentUser, checkAdminPermission } from '@epsx/server-actions';

// Re-export as server actions
export const adminLoginAction = adminLogin;
export const adminLogoutAction = logout;
export { getCurrentUser, checkAdminPermission };

export async function adminLoginWithRedirect(formData: FormData) {
  const email = formData.get('email') as string;
  const password = formData.get('password') as string;

  const result = await adminLogin({ type: 'credentials', email, password });
  
  if (result.success) {
    redirect('/');
  }
  
  return result;
}