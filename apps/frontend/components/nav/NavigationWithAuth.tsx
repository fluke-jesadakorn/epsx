import { getCurrentUser } from '@/lib/server-actions';
import { NavigationClient } from './NavigationClient';

/**
 * Server component wrapper that fetches user data and passes to client Navigation
 */
export async function NavigationWithAuth() {
  try {
    const user = await getCurrentUser();
    
    return <NavigationClient />;
  } catch (error) {
    console.error('NavigationWithAuth error:', error);
    // Return navigation with no user data on error
    return <NavigationClient />;
  }
}