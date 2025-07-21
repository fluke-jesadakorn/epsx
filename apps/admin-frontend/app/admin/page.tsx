import { redirect } from 'next/navigation';

/**
 * Admin route that redirects to the main admin dashboard
 * This ensures backward compatibility with existing admin links
 */
export default function AdminPage() {
  // Redirect to the main admin dashboard
  redirect('/');
}
