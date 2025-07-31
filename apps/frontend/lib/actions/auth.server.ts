'use server';

// Re-export from consolidated server actions package
export {
  login as loginAction,
  register as registerAction,
  logout as logoutAction,
  getCurrentUser,
  refreshToken,
  updateProfile,
  changePassword,
  resetPassword,
  checkFeatureAccess,
  getUserFeatures
} from '@epsx/server-actions/actions/auth';

// Legacy compatibility functions if needed
import { redirect } from 'next/navigation';
import { login, register } from '@epsx/server-actions/actions/auth';

export async function loginWithRedirect(formData: FormData) {
  const email = formData.get('email') as string;
  const password = formData.get('password') as string;

  const result = await login({ type: 'credentials', email, password });
  
  if (result.success) {
    redirect('/dashboard');
  }
  
  return result;
}

export async function registerWithRedirect(formData: FormData) {
  const email = formData.get('email') as string;
  const password = formData.get('password') as string;
  const name = formData.get('name') as string;
  const packageTier = formData.get('packageTier') as string;

  const result = await register({ email, password, name, package_tier: packageTier });
  
  if (result.success) {
    redirect('/dashboard');
  }
  
  return result;
}