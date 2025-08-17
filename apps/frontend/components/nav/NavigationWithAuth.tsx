import { getCurrentUser } from '@/lib/server-actions';
import { NavigationClient } from './NavigationClient';

/**
 * Server component wrapper that fetches user data and passes to client Navigation
 */
export async function NavigationWithAuth() {
  try {
    const result = await getCurrentUser({});
    
    // Extract user data from the server action result
    const user = result?.success ? result.data : null;
    
    return <NavigationClient user={user} />;
  } catch (error) {
    console.error('NavigationWithAuth error:', error);
    // Return navigation with no user data on error
    return <NavigationClient user={null} />;
  }
}